use crate::error::{Result, ShikicrateError};
use reqwest::Client;
use serde_json::json;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use lru::LruCache;

const API_BASE_URL: &str = "https://shikimori.io/api/graphql";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
];
// Rate limit: 0.33 requests per second (3000ms between requests)
const RATE_LIMIT_DELAY: Duration = Duration::from_millis(3000);

// Cache TTL: 5 minutes for search results, 1 hour for details
const CACHE_TTL_SEARCH: Duration = Duration::from_secs(300);
const CACHE_TTL_USER_RATES: Duration = Duration::from_secs(60); // 1 minute for user rates (they change frequently)
const CACHE_TTL_DETAILS: Duration = Duration::from_secs(3600);
const CACHE_TTL_STATIC: Duration = Duration::from_secs(86400); // 24 hours for genres/studios

#[derive(Clone)]
struct CacheKey {
    query: String,
    variables: String,
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.query.hash(state);
        self.variables.hash(state);
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.query == other.query && self.variables == other.variables
    }
}

impl Eq for CacheKey {}

struct CacheEntry {
    data: serde_json::Value,
    expires_at: Instant,
}

impl CacheEntry {
    fn new(data: serde_json::Value, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

pub struct ShikicrateClient {
    client: Client,
    base_url: String,
    last_request: Arc<Mutex<Instant>>,
    cache: Arc<Mutex<LruCache<CacheKey, CacheEntry>>>,
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
            last_request: Arc::new(Mutex::new(Instant::now() - RATE_LIMIT_DELAY)),
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(500).unwrap()))), // Cache up to 500 entries
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
        use reqwest::header::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();

        headers.insert("Origin", HeaderValue::from_static("https://shikimori.io"));
        headers.insert("Referer", HeaderValue::from_static("https://shikimori.io/"));
        headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

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
            last_request: Arc::new(Mutex::new(Instant::now() - RATE_LIMIT_DELAY)),
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(500).unwrap()))),
        })
    }

    pub fn with_base_url(base_url: String) -> Result<Self> {
        Ok(Self {
            client: Self::mk_client(DEFAULT_TIMEOUT)?,
            base_url,
            last_request: Arc::new(Mutex::new(Instant::now() - RATE_LIMIT_DELAY)),
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(500).unwrap()))),
        })
    }

    async fn wait_for_rate_limit(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();
        if elapsed < RATE_LIMIT_DELAY {
            let delay = RATE_LIMIT_DELAY - elapsed;
            drop(last);
            tokio::time::sleep(delay).await;
            let mut last = self.last_request.lock().await;
            *last = Instant::now();
        } else {
            *last = Instant::now();
        }
    }

    fn get_cache_key(&self, query: &str, variables: &Option<serde_json::Value>) -> CacheKey {
        CacheKey {
            query: query.to_string(),
            variables: variables.as_ref().map_or(String::new(), |v| v.to_string()),
        }
    }

    async fn get_from_cache(&self, key: &CacheKey) -> Option<serde_json::Value> {
        let mut cache = self.cache.lock().await;
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            } else {
                cache.pop(key);
            }
        }
        None
    }

    async fn put_to_cache(&self, key: CacheKey, data: serde_json::Value, ttl: Duration) {
        let mut cache = self.cache.lock().await;
        cache.put(key, CacheEntry::new(data, ttl));
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
        let cache_key = self.get_cache_key(query, &variables);

        // Try cache first
        if let Some(cached_data) = self.get_from_cache(&cache_key).await {
            let data = cached_data.get("data").ok_or_else(|| ShikicrateError::GraphQL {
                message: "No data in cached response".to_string(),
                errors: None,
            })?;
            return serde_json::from_value(data.clone()).map_err(ShikicrateError::from);
        }

        self.wait_for_rate_limit().await;

        let body = json!({
            "query": query,
            "variables": variables.unwrap_or(json!({}))
        });

        let response = self
            .client
            .post(&self.base_url)
            .header("Origin", "https://shikimori.io")
            .header("Referer", "https://shikimori.io/")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            // Extract Retry-After header for rate limiting before consuming response
            let retry_after = response.headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            let text = response.text().await?;

            if status.as_u16() == 429 {
                return Err(ShikicrateError::RateLimit {
                    message: format!("Too Many Requests: {}", text),
                    retry_after: retry_after.or(Some(60)), // Default to 60 seconds if not provided
                });
            }

            return Err(ShikicrateError::Api {
                status: status.as_u16(),
                message: format!("HTTP {}: {}", status, text),
            });
        }

        let text = response.text().await?;

        let json: serde_json::Value = serde_json::from_str(&text)?;

        if let Some(errors) = json.get("errors") {
            return Err(ShikicrateError::GraphQL {
                message: "GraphQL error".to_string(),
                errors: Some(errors.clone()),
            });
        }

        let data = json.get("data").ok_or_else(|| ShikicrateError::GraphQL {
            message: "No data in response".to_string(),
            errors: None,
        })?;

        // Cache successful response
        let ttl = if query.contains("userRates") {
            CACHE_TTL_USER_RATES
        } else if query.contains("GetAnimeDetails") || query.contains("GetMangaDetails") {
            CACHE_TTL_DETAILS
        } else if query.contains("genres") || query.contains("studios") || query.contains("publishers") {
            CACHE_TTL_STATIC
        } else {
            CACHE_TTL_SEARCH
        };
        self.put_to_cache(cache_key, json.clone(), ttl).await;

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

    pub async fn get_rest<T, Q>(&self, path: &str, query: Option<Q>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        Q: serde::Serialize,
    {
        let url = format!("https://shikimori.io/api/{}", path);
        let query_str = query.as_ref().map_or(String::new(), |q| serde_json::to_string(q).unwrap_or_default());
        let cache_key = CacheKey {
            query: format!("REST:{}", path),
            variables: query_str,
        };

        // Try cache first for static data
        if path == "genres" || path == "studios" || path == "publishers" {
            if let Some(cached_data) = self.get_from_cache(&cache_key).await {
                return serde_json::from_value(cached_data).map_err(ShikicrateError::Serialization);
            }
        }

        self.wait_for_rate_limit().await;

        let mut req = self.client.get(&url);

        if let Some(q) = query {
            req = req.query(&q);
        }

        let response = req.send().await?;
        let status = response.status();

        if !status.is_success() {
            // Extract Retry-After header for rate limiting before consuming response
            let retry_after = response.headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            let text = response.text().await?;

            if status.as_u16() == 429 {
                return Err(ShikicrateError::RateLimit {
                    message: format!("Too Many Requests: {}", text),
                    retry_after: retry_after.or(Some(60)), // Default to 60 seconds if not provided
                });
            }

            return Err(ShikicrateError::Api {
                status: status.as_u16(),
                message: format!("REST HTTP {}: {}", status, text),
            });
        }

        let text = response.text().await.map_err(ShikicrateError::Http)?;
        let data: serde_json::Value = serde_json::from_str(&text).map_err(ShikicrateError::Serialization)?;

        // Cache static data
        if path == "genres" || path == "studios" || path == "publishers" {
            self.put_to_cache(cache_key, data.clone(), CACHE_TTL_STATIC).await;
        }

        serde_json::from_value(data).map_err(ShikicrateError::Serialization)
    }

    pub(crate) fn to_arc(&self) -> Arc<Self> {
        Arc::new(Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            last_request: Arc::clone(&self.last_request),
            cache: Arc::clone(&self.cache),
        })
    }
}

impl Clone for ShikicrateClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            last_request: Arc::clone(&self.last_request),
            cache: Arc::clone(&self.cache),
        }
    }
}

impl Default for ShikicrateClient {
    fn default() -> Self {
        Self::new().expect("Failed to create ShikicrateClient with default settings")
    }
}
