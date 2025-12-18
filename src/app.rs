use crate::api::{self, Station};
use crate::audio::AudioManager;
use crate::config::Config;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::{window::Id, Alignment, Length, Task};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget::{self, icon};

use cosmic::iced::widget::text_input;

pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    config_handler: cosmic::cosmic_config::Config,
    audio: AudioManager,
    
    // UI State
    search_query: String,
    search_results: Vec<Station>,
    is_searching: bool,
    current_station: Option<Station>,
    is_playing: bool,
    error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    
    // Search
    SearchInputChanged(String),
    PerformSearch,
    SearchCompleted(Result<Vec<Station>, String>),
    
    // Stations
    PlayStation(Station),
    ToggleFavorite(Station),
    ClearSearch,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.marcos.RadioApplet";

    fn core(&self) -> &cosmic::Core { &self.core }
    fn core_mut(&mut self) -> &mut cosmic::Core { &mut self.core }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config_handler = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION).unwrap();
        let config = match Config::get_entry(&config_handler) {
            Ok(c) => c,
            Err((_errs, c)) => {
                eprintln!("Errors loading config: {:?}", _errs);
                c
            }
        };

        let audio = AudioManager::new();
        audio.set_volume(config.volume as f32 / 100.0);

        let app = AppModel {
            core,
            popup: None,
            config,
            config_handler,
            audio,
            search_query: String::new(),
            search_results: Vec::new(),
            is_searching: false,
            current_station: None,
            is_playing: false,
            error_message: None,
        };
        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> { Some(Message::PopupClosed(id)) }

    fn view(&self) -> Element<'_, Self::Message> {
        widget::container(
            cosmic::widget::button::custom(icon::from_name("multimedia-player-symbolic").size(16))
                .on_press(Message::TogglePopup)
                .class(cosmic::theme::Button::Icon)
        )
        .height(Length::Fill)
        .center_y(Length::Fill)
        .center_x(Length::Fill)
        .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let title = widget::text("COSMIC Radio").size(24);
        
        // Search Bar
        let search_input = text_input("Buscar estações (ex: Jazz)...", &self.search_query)
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::PerformSearch)
            .padding(10);
            
        let search_btn = cosmic::iced::widget::button("Buscar")
            .on_press(Message::PerformSearch);
            
        let search_row = widget::row()
            .spacing(10)
            .push(search_input)
            .push(search_btn);

        // Results List
        let mut stations_list = widget::column().spacing(5);
        
        if self.is_searching {
             stations_list = stations_list.push(widget::text("Buscando via Satélite... 📡"));
        } else if let Some(err) = &self.error_message {
             stations_list = stations_list.push(widget::text(format!("Erro: {}", err)));
        } else {
            // Show favorites if search is empty and no results yet? Or always favorites first?
            if self.search_query.is_empty() && self.search_results.is_empty() {
                 stations_list = stations_list.push(widget::text("Meus Favoritos:").size(18));
                 if self.config.favorites.is_empty() {
                     stations_list = stations_list.push(widget::text("Nenhum favorito salvo."));
                 }
                 for station in &self.config.favorites {
                     stations_list = stations_list.push(self.view_station_row(station, true));
                 }
            } else {
                 let back_btn = cosmic::iced::widget::button("← Voltar para Favoritos")
                     .on_press(Message::ClearSearch);
                 
                 stations_list = stations_list.push(back_btn);
                 stations_list = stations_list.push(widget::text("Resultados da Busca:").size(18));
                 for station in &self.search_results {
                     let is_fav = self.config.favorites.iter().any(|s| s.stationuuid == station.stationuuid);
                     stations_list = stations_list.push(self.view_station_row(station, is_fav));
                 }
            }
        }
        
        let content = widget::column()
            .padding(20)
            .spacing(15)
            .push(title)
            .push(search_row)
            .push(widget::scrollable(stations_list).height(300));

        self.core.applet.popup_container(content).into()
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => { if self.popup == Some(id) { self.popup = None; } }
            Message::SearchInputChanged(val) => {
                self.search_query = val;
            }
            Message::PerformSearch => {
                self.is_searching = true;
                self.error_message = None;
                self.search_results.clear();
                let query = self.search_query.clone();
                return Task::perform(
                    async move {
                         api::search_stations(query).await.map_err(|e: reqwest::Error| e.to_string())
                    },
                    Message::SearchCompleted
                ).map(Into::into);
            }
            Message::SearchCompleted(res) => {
                self.is_searching = false;
                match res {
                    Ok(stations) => self.search_results = stations,
                    Err(e) => self.error_message = Some(e),
                }
            }
            Message::PlayStation(station) => {
                let is_same = self.current_station.as_ref().map(|s| s.stationuuid == station.stationuuid).unwrap_or(false);
                
                if self.is_playing && is_same {
                    self.audio.stop();
                    self.is_playing = false;
                } else {
                    self.current_station = Some(station.clone());
                    self.is_playing = true;
                    self.audio.play(station.url_resolved.clone(), self.config.volume);
                }
            }
            Message::ClearSearch => {
                self.search_query.clear();
                self.search_results.clear();
                self.error_message = None;
            }
            Message::ToggleFavorite(station) => {
                if let Some(pos) = self.config.favorites.iter().position(|s| s.stationuuid == station.stationuuid) {
                    self.config.favorites.remove(pos);
                } else {
                    self.config.favorites.push(station);
                }
                let _ = self.config.write_entry(&self.config_handler);
            }
        }
        Task::none()
    }
}

impl AppModel {
    fn view_station_row<'a>(&self, station: &'a Station, is_fav: bool) -> Element<'a, Message> {
        let play_icon = if self.is_playing && self.current_station.as_ref().map(|s| s.stationuuid == station.stationuuid).unwrap_or(false) {
             "media-playback-pause-symbolic"
        } else {
             "media-playback-start-symbolic"
        };
        
        let fav_icon = if is_fav { "starred-symbolic" } else { "non-starred-symbolic" }; // Check correct names
        
        widget::row()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(
                cosmic::iced::widget::button(icon::from_name(play_icon))
                    .on_press(Message::PlayStation(station.clone()))
            )
            .push(widget::text(&station.name).width(cosmic::iced::Length::Fill))
            .push(
                cosmic::iced::widget::button(icon::from_name(fav_icon))
                    .on_press(Message::ToggleFavorite(station.clone()))
            )
            .into()
    }
}
