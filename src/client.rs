use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

#[cfg(test)]
use mockito;

use crate::models::{Challenges, Champions, Items, Runes, SummonerSpells, Translations};

#[derive(Error, Debug)]
pub enum DDragonClientError {
    #[error("Could not parse URL.")]
    UrlParse(#[from] url::ParseError),
    #[error("Could not complete request.")]
    Request(#[from] Box<ureq::Error>),
    #[error("Could not parse JSON data.")]
    Parse(#[from] std::io::Error),
    #[error("Could not parse JSON data.")]
    JSONParse(#[from] serde_json::Error),
    #[error("Could not find the latest API version.")]
    NoLatestVersion,
}

pub struct DDragonClient {
    agent: ureq::Agent,
    pub version: String,
    base_url: Url,
    cache_dir: Option<String>,
}

impl DDragonClient {
    fn create(
        agent: ureq::Agent,
        cache_dir: Option<String>,
        base_url: Url,
    ) -> Result<Self, DDragonClientError> {
        let version_list = agent
            .get(base_url.join("/api/versions.json")?.as_str())
            .call()
            .map_err(Box::new)?
            .into_json::<Vec<String>>()?;

        let latest_version = version_list
            .get(0)
            .ok_or(DDragonClientError::NoLatestVersion)?;

        Ok(DDragonClient {
            agent,
            version: latest_version.to_owned(),
            base_url,
            cache_dir,
        })
    }

    pub fn with_agent(
        agent: ureq::Agent,
        cache_dir: Option<String>,
    ) -> Result<Self, DDragonClientError> {
        #[cfg(not(test))]
        let base_url = "https://ddragon.leagueoflegends.com";

        #[cfg(test)]
        let base_url = mockito::server_url();

        Self::create(agent, cache_dir, Url::parse(&base_url)?)
    }

    pub fn with_cache(cache_dir: &str) -> Result<Self, DDragonClientError> {
        let agent = ureq::Agent::new();
        Self::with_agent(agent, Some(cache_dir.to_owned()))
    }

    pub fn new() -> Result<Self, DDragonClientError> {
        let agent = ureq::Agent::new();
        Self::with_agent(agent, None)
    }

    fn get_data_url(&self) -> Result<Url, url::ParseError> {
        self.base_url
            .join(&format!("/cdn/{}/data/en_US/", &self.version))
    }

    fn get_data<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, DDragonClientError> {
        let joined_url = self.get_data_url()?.join(endpoint)?;
        let request_url = joined_url.as_str();

        if let Some(dir) = &self.cache_dir {
            if let Ok(data) = cacache::read_sync(dir, request_url) {
                if let Ok(parsed) = serde_json::from_slice(&data) {
                    return Ok(parsed);
                }
            }
        }

        let response = self.agent.get(request_url).call().map_err(Box::new)?;
        let response_str = response.into_string()?;
        let response_json = serde_json::from_str(&response_str)?;

        if let Some(dir) = &self.cache_dir {
            let _ = cacache::write_sync(dir, request_url, response_str);
        }

        Ok(response_json)
    }

    pub fn challenges(&self) -> Result<Challenges, DDragonClientError> {
        self.get_data::<Challenges>("./challenges.json")
    }

    pub fn champions(&self) -> Result<Champions, DDragonClientError> {
        self.get_data::<Champions>("./champion.json")
    }

    pub fn items(&self) -> Result<Items, DDragonClientError> {
        self.get_data::<Items>("./item.json")
    }

    pub fn runes(&self) -> Result<Runes, DDragonClientError> {
        self.get_data::<Runes>("./runesReforged.json")
    }

    pub fn summoner_spells(&self) -> Result<SummonerSpells, DDragonClientError> {
        self.get_data::<SummonerSpells>("./summoner.json")
    }

    pub fn translations(&self) -> Result<Translations, DDragonClientError> {
        self.get_data::<Translations>("./language.json")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockito::mock;

    impl Default for DDragonClient {
        fn default() -> Self {
            Self {
                agent: ureq::Agent::new(),
                version: "0.0.0".to_owned(),
                base_url: Url::parse(&mockito::server_url()).unwrap(),
                cache_dir: None,
            }
        }
    }

    mod create {
        use super::*;

        #[test]
        fn result_ok_if_at_least_one_version() {
            let _mock = mock("GET", "/api/versions.json")
                .with_status(200)
                .with_header("Content-Type", "application/json")
                .with_body(r#"["0.0.0"]"#)
                .create();

            let maybe_client = DDragonClient::new();

            assert!(maybe_client.is_ok());
            assert_eq!(maybe_client.unwrap().version, "0.0.0");
        }

        #[test]
        fn result_ok_first_version_in_list() {
            let _mock = mock("GET", "/api/versions.json")
                .with_status(200)
                .with_header("Content-Type", "application/json")
                .with_body(r#"["0.0.0", "1.1.1", "2.2.2"]"#)
                .create();

            let maybe_client = DDragonClient::new();

            assert!(maybe_client.is_ok());
            assert_eq!(maybe_client.unwrap().version, "0.0.0");
        }

        #[test]
        fn result_err_server_unavailable() {
            assert!(DDragonClient::new().is_err());
        }

        #[test]
        fn result_err_no_versions_in_list() {
            let _mock = mock("GET", "/api/versions.json")
                .with_status(200)
                .with_header("Content-Type", "application/json")
                .with_body(r#"[]"#)
                .create();

            assert!(DDragonClient::new().is_err());
        }

        #[test]
        fn result_err_cannot_deserialize() {
            let _mock = mock("GET", "/api/versions.json")
                .with_status(200)
                .with_body(r#"some non-deserializable content"#)
                .create();

            assert!(DDragonClient::new().is_err());
        }
    }

    mod requests {
        use super::*;

        #[test]
        fn get_data_url_constructs_expected_baseurl() {
            let client = DDragonClient::default();
            assert_eq!(
                client.get_data_url().unwrap().as_str(),
                format!("{}/cdn/0.0.0/data/en_US/", mockito::server_url())
            );
        }

        #[test]
        fn get_data_err_if_server_unavailable() {
            let client = DDragonClient::default();
            assert!(client
                .get_data::<serde_json::Value>("/fake-endpoint")
                .is_err());
        }

        #[test]
        fn get_data_err_if_data_not_deserializable() {
            let _mock = mock("GET", "/cdn/0.0.0/data/en_US/data.json")
                .with_status(200)
                .with_header("Content-Type", "application/json")
                .with_body(r#"no chance to deserialize this"#)
                .create();

            let client = DDragonClient::default();
            assert!(client.get_data::<serde_json::Value>("./data.json").is_err());
        }

        #[test]
        fn get_data_ok_deserializes_to_type() {
            let _mock = mock("GET", "/cdn/0.0.0/data/en_US/data.json")
                .with_status(200)
                .with_header("Content-Type", "application/json")
                .with_body(r#"["value"]"#)
                .create();

            let client = DDragonClient::default();
            assert_eq!(
                client.get_data::<Vec<String>>("./data.json").unwrap(),
                vec!["value".to_owned()]
            );
        }
    }
}
