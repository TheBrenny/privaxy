use std::sync::OnceLock;

use reqwest::header;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use web_sys;
use url::Url;

pub struct API {
    client: reqwest::Client,
    hostname: String,
    port: u16,
    endpoint: String,
}

impl API {
    pub fn api() -> &'static API {
        static ARRAY: OnceLock<API> = OnceLock::new();
        ARRAY.get_or_init(|| {
            let url = web_sys::window().unwrap().location().href().unwrap();
            let url = Url::parse(&url).expect("a valid URL");
            let href = url.host_str().expect("a valid host").to_string();
            let port = url.port_or_known_default().expect("a port or known prototype");

            API::new(href, port, String::from("api/"))
        })
    }

    fn new(hostname: String, port: u16, endpoint: String) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            client,
            hostname,
            port,
            endpoint,
        }
    }

    pub async fn get_statistics(&self) -> Result<Statistics, reqwest::Error> {
        (&self).call::<Statistics>("statistics/").await
    }

    pub async fn call<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, reqwest::Error> {
        (&self)
            .client
            .get(format!(
                "http://{}:{}/{}{}",
                &self.hostname, &&self.port, &self.endpoint, endpoint
            ))
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Statistics {
    pub proxied_requests: Option<u64>,
    pub blocked_requests: Option<u64>,
    pub modified_responses: Option<u64>,
    #[serde(with = "tuple_vec_map")]
    pub top_blocked_paths: Vec<(String, u64)>,
    #[serde(with = "tuple_vec_map")]
    pub top_clients: Vec<(String, u64)>,
}
