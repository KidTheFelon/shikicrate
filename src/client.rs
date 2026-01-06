use crate::error::{Result, ShikicrateError};
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

const API_BASE_URL: &str = "https://shikimori.one/api/graphql";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
];

pub struct ShikicrateClient {
    client: Client,
    base_url: String,
}

pub struct ShikicrateClientBuilder {
    base_url: Option<String>,
    timeout: Option<Duration>,
}

impl ShikicrateClientBuilder {
    pub fn new() -> Self {
        Self {
            base_url: None,
            timeout: None,
        }
    }

    pub fn base_url(mut self, url: String) -> Self {
        self.base_url = Some(url);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<ShikicrateClient> {
        let base_url = self.base_url.as_deref().unwrap_or(API_BASE_URL);
        let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);

        Ok(ShikicrateClient {
            client: ShikicrateClient::mk_client(timeout)?,
            base_url: base_url.to_string(),
        })
    }
}

impl Default for ShikicrateClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ShikicrateClient {
    pub fn new() -> Result<Self> {
        Self::with_timeout(DEFAULT_TIMEOUT)
    }

    fn mk_client(timeout: Duration) -> Result<Client> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Origin", "https://shikimori.one".parse().unwrap());
        headers.insert("Referer", "https://shikimori.one/".parse().unwrap());
        headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        Client::builder()
            .timeout(timeout)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .default_headers(headers)
            .build()
            .map_err(ShikicrateError::Http)
    }

    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        Ok(Self {
            client: Self::mk_client(timeout)?,
            base_url: API_BASE_URL.to_string(),
        })
    }

    pub fn with_base_url(base_url: String) -> Result<Self> {
        Ok(Self {
            client: Self::mk_client(DEFAULT_TIMEOUT)?,
            base_url,
        })
    }

    fn is_retryable(error: &ShikicrateError) -> bool {
        match error {
            ShikicrateError::Http(e) => e.is_timeout() || e.is_connect() || e.is_request(),
            ShikicrateError::RateLimit { .. } => true,
            _ => false,
        }
    }

    async fn exec_once<T>(&self, query: &str, variables: Option<serde_json::Value>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let body = json!({
            "query": query,
            "variables": variables.unwrap_or(json!({}))
        });

        println!(">>> [shikicrate] Запрос к {}", self.base_url);
        
        let response = self
            .client
            .post(&self.base_url)
            .header("Origin", "https://shikimori.one")
            .header("Referer", "https://shikimori.one/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        println!(">>> [shikicrate] Статус ответа: {}", status);
        
        let text = response.text().await?;

        if !status.is_success() {
            println!(">>> [shikicrate] Ошибка API: {} - {}", status, text);
            return Err(ShikicrateError::Api {
                status: status.as_u16(),
                message: format!("HTTP {}: {}", status, text),
            });
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;
        
        if let Some(errors) = json.get("errors") {
            println!(">>> [shikicrate] GraphQL Errors: {:?}", errors);
            return Err(ShikicrateError::GraphQL {
                message: "GraphQL error".to_string(),
                errors: Some(errors.clone()),
            });
        }

        let data = json.get("data").ok_or_else(|| ShikicrateError::GraphQL {
            message: "No data in response".to_string(),
            errors: None,
        })?;

        serde_json::from_value(data.clone()).map_err(ShikicrateError::from)
    }

    pub(crate) async fn execute_query<T>(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut last_error = match self.exec_once(query, variables.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) if !Self::is_retryable(&e) => return Err(e),
            Err(e) => e,
        };

        for delay in RETRY_DELAYS.iter() {
            tokio::time::sleep(*delay).await;
            match self.exec_once(query, variables.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if Self::is_retryable(&e) => last_error = e,
                Err(e) => return Err(e),
            }
        }

        Err(last_error)
    }

    pub(crate) fn to_arc(&self) -> Arc<Self> {
        Arc::new(Self {
            client: Self::mk_client(DEFAULT_TIMEOUT).unwrap(),
            base_url: self.base_url.clone(),
        })
    }
}

impl Default for ShikicrateClient {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
