use std::sync::OnceLock;

use reqwest::{header, Method, Body};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use serde_json::json;
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

            API::new(href, port, String::from("api"))
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
        (&self).get::<Statistics>("/statistics").await
    }
    pub async fn set_blocking(&self, blocking_state: &BlockingState) -> Result<BlockingState, reqwest::Error> {
        (&self).post::<BlockingState, BlockingState>("/blocking", &blocking_state).await
    }
    pub async fn get_blocking(&self) -> Result<BlockingState, reqwest::Error> {
        (&self).get::<BlockingState>("/blocking").await
    }

    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, reqwest::Error> {
        (&self)
            .client.request(Method::GET, format!(
                "http://{}:{}/{}{}",
                &self.hostname, &&self.port, &self.endpoint, endpoint
            ))
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, endpoint: &str, body: &B) -> Result<T, reqwest::Error> {
        (&self)
            .client.request(Method::POST, format!(
                "http://{}:{}/{}{}",
                &self.hostname, &&self.port, &self.endpoint, endpoint
            ))
            .body(Body::from(json!(*body).to_string()))
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag="state")]
pub enum BlockingState {
    Enabled,
    Disabled,
}