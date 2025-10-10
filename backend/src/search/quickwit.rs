use crate::config::Config;
use ntex::http::client::{Client, Connector};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct Quickwit {
    pub base_url: String,
    pub index: String,
    pub client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    pub hits: serde_json::Value,
}

impl Quickwit {
    pub fn from_config(cfg: &Config) -> Option<Self> {
        let base = cfg.quickwit_url.clone();

        if base.is_empty() {
            return None;
        }
        let index = if cfg.quickwit_index.is_empty() {
            "questions".to_string()
        } else {
            cfg.quickwit_index.clone()
        };

        Some(Self {
            base_url: base.trim_end_matches('/').to_string(),
            index,
            client: Client::new(),
        })
    }

    pub async fn index<T: Serialize>(&self, doc: &[T]) -> Result<(), String> {
        let url = format!("{}/api/v1/{}/index", self.base_url, self.index);
        let body = serde_json::to_string(&docs).map_err(|e| e.to_string())?;

        let mut response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .send_body(body)
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            let text = response.body().await.map_err(|e| e.to_string())?;
            Err(format!(
                "Quickwit ingest failed: {} {}",
                response.status(),
                String::from_utf8_lossy(&text)
            ))
        }
    }

    pub async fn search(&self, q: &str, limit: usize) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/v1/{}/search", self.base_url, self.index);
        let req = self
            .client
            .get(url)
            .query(&[("q", q), ("max_hits", &limit.to_string())])
            .map_err(|e| e.to_string())?;
        let mut response = req.send().await.map_err(|e| e.to_string())?;


        if response.status().is_success() {
            let body = response.body().await.map_err(|e| e.to_string())?;
            let val: serde_json::Value =
                serde_json::from_slice(&body).map_err(|e| e.to_string())?;
            Ok(val)
        } else {
            Err(format!("Quickwit search failed: {}", response.status()))
        }
    }
}
