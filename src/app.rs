use crate::api::{self, Station};
use crate::audio::AudioManager;
use crate::config::Config;
use crate::fl;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::event::{self, Event};
use cosmic::iced::keyboard::{key::Named, Key};
use cosmic::iced::widget::text_input;
use cosmic::iced::{window::Id, Alignment, Length, Subscription, Task};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget::{self, icon, slider};
use tracing::{debug, error, info, warn};

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

    // Volume control
    VolumeChanged(f32),
    VolumeUp,
    VolumeDown,

    // Keyboard shortcuts
    TogglePlayPause,
    KeyboardEvent(Event),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.marcos.RadioApplet";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config_handler = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .expect("Failed to create config handler");

        let config = match Config::get_entry(&config_handler) {
            Ok(c) => {
                info!("Config loaded successfully");
                c
            }
            Err((errs, c)) => {
                warn!("Errors loading config: {:?}. Using defaults.", errs);
                if let Err(e) = c.write_entry(&config_handler) {
                    error!("Failed to write initial config: {:?}", e);
                }
                c
            }
        };

        debug!(
            "Running Config v{}. Favorites: {}",
            Config::VERSION,
            config.favorites.len()
        );

        let audio = AudioManager::new();
        audio.set_volume(config.volume as f32);

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

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Subscribe to keyboard events when popup is open
        if self.popup.is_some() {
            event::listen().map(Message::KeyboardEvent)
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        widget::container(
            cosmic::widget::button::custom(icon::from_name("multimedia-player-symbolic").size(16))
                .on_press(Message::TogglePopup)
                .class(cosmic::theme::Button::Icon),
        )
        .height(Length::Fill)
        .center_y(Length::Fill)
        .center_x(Length::Fill)
        .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let title = widget::text(fl!("window-title")).size(24);

        // Now Playing section (if playing)
        let now_playing: Element<'_, Message> = if let Some(station) = &self.current_station {
            let status_icon = if self.is_playing {
                "media-playback-pause-symbolic"
            } else {
                "media-playback-stop-symbolic"
            };

            widget::column()
                .spacing(8)
                .push(
                    widget::row()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(icon::from_name(status_icon).size(20))
                        .push(
                            widget::text(&station.name)
                                .size(16)
                                .width(Length::Fill),
                        )
                        .push(
                            cosmic::iced::widget::button(icon::from_name(
                                "media-playback-stop-symbolic",
                            ))
                            .on_press(Message::TogglePlayPause),
                        ),
                )
                .into()
        } else {
            widget::text(fl!("not-playing")).size(14).into()
        };

        // Volume control section
        let volume_section = {
            let volume_label = format!("{} {}%", fl!("volume"), self.config.volume);
            let volume_icon = if self.config.volume == 0 {
                "audio-volume-muted-symbolic"
            } else if self.config.volume < 33 {
                "audio-volume-low-symbolic"
            } else if self.config.volume < 66 {
                "audio-volume-medium-symbolic"
            } else {
                "audio-volume-high-symbolic"
            };

            widget::row()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(icon::from_name(volume_icon).size(20))
                .push(
                    slider(0.0..=100.0, self.config.volume as f32, Message::VolumeChanged)
                        .width(Length::Fill),
                )
                .push(widget::text(volume_label).size(14).width(Length::Shrink))
        };

        // Search Bar
        let search_input = text_input(&fl!("search-placeholder"), &self.search_query)
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::PerformSearch)
            .padding(10);

        let search_btn = cosmic::iced::widget::button(widget::text(fl!("search-button")))
            .on_press(Message::PerformSearch);

        let search_row = widget::row()
            .spacing(10)
            .push(search_input)
            .push(search_btn);

        // Results List
        let mut stations_list = widget::column().spacing(5);

        if self.is_searching {
            stations_list = stations_list.push(widget::text(fl!("searching-status")));
        } else if let Some(err) = &self.error_message {
            stations_list =
                stations_list.push(widget::text(format!("{} {}", fl!("error-message"), err)));
        } else if self.search_query.is_empty() && self.search_results.is_empty() {
            stations_list = stations_list.push(widget::text(fl!("favorites-header")).size(18));
            if self.config.favorites.is_empty() {
                stations_list = stations_list.push(widget::text(fl!("no-favorites")));
            }
            for station in &self.config.favorites {
                stations_list = stations_list.push(self.view_station_row(station, true));
            }
        } else {
            let back_btn = cosmic::iced::widget::button(widget::text(fl!("back-to-favorites")))
                .on_press(Message::ClearSearch);

            stations_list = stations_list.push(back_btn);
            stations_list = stations_list.push(widget::text(fl!("search-results-header")).size(18));
            for station in &self.search_results {
                let is_fav = self
                    .config
                    .favorites
                    .iter()
                    .any(|s| s.stationuuid == station.stationuuid);
                stations_list = stations_list.push(self.view_station_row(station, is_fav));
            }
        }

        // Keyboard shortcuts hint
        let shortcuts_hint = widget::text(fl!("shortcuts-hint"))
            .size(11)
            .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(
                0.5, 0.5, 0.5,
            )));

        let content = widget::column()
            .padding(20)
            .spacing(12)
            .push(title)
            .push(widget::divider::horizontal::light())
            .push(now_playing)
            .push(volume_section)
            .push(widget::divider::horizontal::light())
            .push(search_row)
            .push(widget::scrollable(stations_list).height(250))
            .push(shortcuts_hint);

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
                    if let Some(main_id) = self.core.main_window_id() {
                        let popup_settings =
                            self.core
                                .applet
                                .get_popup_settings(main_id, new_id, None, None, None);
                        get_popup(popup_settings)
                    } else {
                        warn!("No main window ID available");
                        Task::none()
                    }
                };
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
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
                        api::search_stations(query)
                            .await
                            .map_err(|e: reqwest::Error| e.to_string())
                    },
                    Message::SearchCompleted,
                )
                .map(Into::into);
            }
            Message::SearchCompleted(res) => {
                self.is_searching = false;
                match res {
                    Ok(stations) => {
                        debug!("Search completed: {} stations found", stations.len());
                        self.search_results = stations;
                    }
                    Err(e) => {
                        error!("Search failed: {}", e);
                        self.error_message = Some(e);
                    }
                }
            }
            Message::PlayStation(station) => {
                let is_same = self
                    .current_station
                    .as_ref()
                    .map(|s| s.stationuuid == station.stationuuid)
                    .unwrap_or(false);

                if self.is_playing && is_same {
                    self.audio.stop();
                    self.is_playing = false;
                    debug!("Stopped playback");
                } else {
                    self.current_station = Some(station.clone());
                    self.is_playing = true;
                    self.audio
                        .play(station.url_resolved.clone(), self.config.volume);
                    debug!("Playing: {}", station.name);
                }
            }
            Message::ClearSearch => {
                self.search_query.clear();
                self.search_results.clear();
                self.error_message = None;
            }
            Message::ToggleFavorite(station) => {
                if let Some(pos) = self
                    .config
                    .favorites
                    .iter()
                    .position(|s| s.stationuuid == station.stationuuid)
                {
                    self.config.favorites.remove(pos);
                    debug!("Removed from favorites: {}", station.name);
                } else {
                    self.config.favorites.push(station.clone());
                    debug!("Added to favorites: {}", station.name);
                }
                self.save_config();
            }
            Message::VolumeChanged(vol) => {
                let volume = vol.round() as u8;
                self.config.volume = volume;
                self.audio.set_volume(volume as f32);
                debug!("Volume changed to {}%", volume);
                self.save_config();
            }
            Message::VolumeUp => {
                let new_vol = (self.config.volume as i16 + 5).min(100) as u8;
                self.config.volume = new_vol;
                self.audio.set_volume(new_vol as f32);
                debug!("Volume up to {}%", new_vol);
                self.save_config();
            }
            Message::VolumeDown => {
                let new_vol = (self.config.volume as i16 - 5).max(0) as u8;
                self.config.volume = new_vol;
                self.audio.set_volume(new_vol as f32);
                debug!("Volume down to {}%", new_vol);
                self.save_config();
            }
            Message::TogglePlayPause => {
                if self.is_playing {
                    self.audio.stop();
                    self.is_playing = false;
                    debug!("Paused playback via shortcut");
                } else if let Some(station) = &self.current_station {
                    self.audio
                        .play(station.url_resolved.clone(), self.config.volume);
                    self.is_playing = true;
                    debug!("Resumed playback via shortcut: {}", station.name);
                }
            }
            Message::KeyboardEvent(event) => {
                if let Event::Keyboard(keyboard_event) = event {
                    if let cosmic::iced::keyboard::Event::KeyPressed { key, .. } = keyboard_event {
                        match key {
                            Key::Named(Named::Space) => {
                                return self.update(Message::TogglePlayPause);
                            }
                            Key::Named(Named::ArrowUp) => {
                                return self.update(Message::VolumeUp);
                            }
                            Key::Named(Named::ArrowDown) => {
                                return self.update(Message::VolumeDown);
                            }
                            Key::Named(Named::Escape) => {
                                if let Some(p) = self.popup.take() {
                                    return destroy_popup(p);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Task::none()
    }
}

impl AppModel {
    fn view_station_row<'a>(&self, station: &'a Station, is_fav: bool) -> Element<'a, Message> {
        let play_icon = if self.is_playing
            && self
                .current_station
                .as_ref()
                .map(|s| s.stationuuid == station.stationuuid)
                .unwrap_or(false)
        {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        };

        let fav_icon = if is_fav {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        };

        widget::row()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(
                cosmic::iced::widget::button(icon::from_name(play_icon))
                    .on_press(Message::PlayStation(station.clone())),
            )
            .push(widget::text(&station.name).width(cosmic::iced::Length::Fill))
            .push(
                cosmic::iced::widget::button(icon::from_name(fav_icon))
                    .on_press(Message::ToggleFavorite(station.clone())),
            )
            .into()
    }

    fn save_config(&self) {
        if let Err(e) = self.config.write_entry(&self.config_handler) {
            error!("Failed to save config: {:?}", e);
        } else {
            debug!("Config saved");
        }
    }
}
