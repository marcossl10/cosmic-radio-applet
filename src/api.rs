use serde::{Deserialize, Deserializer, Serialize};
use reqwest::Error;

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Station {
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub stationuuid: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub url: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub url_resolved: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub homepage: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub favicon: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub tags: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub country: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub language: String,
}

pub async fn search_stations(query: String) -> Result<Vec<Station>, Error> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Lista de servidores espelho para redundância
    let servers = [
        "https://all.api.radio-browser.info",
        "https://de1.api.radio-browser.info",
        "https://fr1.api.radio-browser.info",
        "https://at1.api.radio-browser.info",
        "https://nl1.api.radio-browser.info",
        "https://us1.api.radio-browser.info",
        "https://es1.api.radio-browser.info",
    ];
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let mut last_result: Result<Vec<Station>, Error> = Ok(Vec::new());

    for server in servers {
        let url = format!("{}/json/stations/search", server);
        let params = [("name", query.as_str()), ("limit", "20")];
        
        let response_attempt = client.get(&url)
            .query(&params)
            .send()
            .await;

        match response_attempt {
            Ok(response) => {
                match response.error_for_status() {
                    Ok(valid_response) => {
                        match valid_response.json::<Vec<Station>>().await {
                            Ok(stations) => return Ok(stations), // Sucesso! Retorna imediatamente
                            Err(e) => last_result = Err(e),      // Erro no JSON, tenta próximo
                        }
                    },
                    Err(e) => last_result = Err(e), // Erro HTTP (ex: 502), tenta próximo
                }
            },
            Err(e) => last_result = Err(e), // Erro de conexão, tenta próximo
        }
    }
    
    // Se chegou aqui, todos os servidores falharam. Retorna o erro da última tentativa.
    last_result
}
