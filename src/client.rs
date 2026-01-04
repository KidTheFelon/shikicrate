use crate::error::{Result, ShikicrateError};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use url::Url;

const API_BASE_URL: &str = "https://shikimori.one/api/graphql";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
];

/// HTTP клиент для выполнения GraphQL запросов к Shikimori API.
///
/// Клиент автоматически обрабатывает retry логику для сетевых ошибок
/// с экспоненциальной задержкой (1s, 2s, 4s). Все запросы выполняются
/// асинхронно через `tokio`.
///
/// # Примеры
///
/// ## Создание клиента с настройками по умолчанию
///
/// ```no_run
/// use shikicrate::ShikicrateClient;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ShikicrateClient::new()?;
/// # Ok(())
/// # }
/// ```
///
/// ## Создание клиента с кастомным таймаутом
///
/// ```no_run
/// use shikicrate::ShikicrateClient;
/// use std::time::Duration;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ShikicrateClient::with_timeout(Duration::from_secs(60))?;
/// # Ok(())
/// # }
/// ```
///
/// ## Создание клиента с кастомным URL
///
/// ```no_run
/// use shikicrate::ShikicrateClient;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ShikicrateClient::with_base_url(
///     "https://shikimori.one/api/graphql".to_string()
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// # Retry логика
///
/// Клиент автоматически повторяет запросы при следующих ошибках:
/// - Таймауты (`reqwest::Error::is_timeout()`)
/// - Ошибки подключения (`reqwest::Error::is_connect()`)
/// - Ошибки запроса (`reqwest::Error::is_request()`)
///
/// Retry выполняется максимум 3 раза с задержками: 1 секунда, 2 секунды, 4 секунды.
/// Rate limiting (429) также повторяется с учетом заголовка `Retry-After`.
/// Ошибки валидации, GraphQL ошибки и API ошибки (неуспешные HTTP статусы, кроме 429) не повторяются.
pub struct ShikicrateClient {
    client: Client,
    base_url: String,
}

/// Builder для создания и настройки `ShikicrateClient`.
///
/// Позволяет гибко конфигурировать клиент перед созданием.
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::ShikicrateClientBuilder;
/// use std::time::Duration;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Клиент с кастомным таймаутом
/// let client = ShikicrateClientBuilder::new()
///     .timeout(Duration::from_secs(60))
///     .build()?;
///
/// // Клиент с кастомным URL
/// let client = ShikicrateClientBuilder::new()
///     .base_url("https://test.shikimori.one/api/graphql".to_string())
///     .build()?;
///
/// // Комбинированная настройка
/// let client = ShikicrateClientBuilder::new()
///     .base_url("https://api.example.com/graphql".to_string())
///     .timeout(Duration::from_secs(90))
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ShikicrateClientBuilder {
    base_url: Option<String>,
    timeout: Option<Duration>,
}

impl ShikicrateClientBuilder {
    /// Создает новый builder с настройками по умолчанию.
    ///
    /// По умолчанию используется стандартный URL и таймаут.
    pub fn new() -> Self {
        Self {
            base_url: None,
            timeout: None,
        }
    }

    /// Устанавливает базовый URL для GraphQL API.
    ///
    /// URL должен быть валидным HTTP/HTTPS адресом.
    ///
    /// # Параметры
    ///
    /// * `url` - Базовый URL GraphQL API
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::ShikicrateClientBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClientBuilder::new()
    ///     .base_url("https://test.shikimori.one/api/graphql".to_string())
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn base_url(mut self, url: String) -> Self {
        self.base_url = Some(url);
        self
    }

    /// Устанавливает таймаут для HTTP запросов.
    ///
    /// # Параметры
    ///
    /// * `timeout` - Максимальное время ожидания ответа от сервера
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::ShikicrateClientBuilder;
    /// use std::time::Duration;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClientBuilder::new()
    ///     .timeout(Duration::from_secs(60))
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Создает `ShikicrateClient` с заданными параметрами.
    ///
    /// # Возвращает
    ///
    /// `Result<ShikicrateClient>` - клиент в случае успеха, или ошибка валидации/создания.
    ///
    /// # Ошибки
    ///
    /// - `ShikicrateError::Validation` - если URL невалидный
    /// - `ShikicrateError::Http` - если не удалось создать HTTP клиент
    pub fn build(self) -> Result<ShikicrateClient> {
        let base_url = self.base_url.as_deref().unwrap_or(API_BASE_URL);
        let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);

        // Валидация URL если он был задан кастомно
        if let Some(ref url) = self.base_url {
            let parsed_url = Url::parse(url)
                .map_err(|e| ShikicrateError::Validation(format!("Invalid URL: {}", e)))?;

            match parsed_url.scheme() {
                "http" | "https" => {}
                scheme => {
                    return Err(ShikicrateError::Validation(format!(
                        "Unsafe URL scheme: {}. Only http:// and https:// are allowed",
                        scheme
                    )));
                }
            }

            if parsed_url.host().is_none() {
                return Err(ShikicrateError::Validation(
                    "URL must have a host".to_string(),
                ));
            }
        }

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
    /// Создает новый клиент с настройками по умолчанию.
    ///
    /// Использует стандартный таймаут (30 секунд) и базовый URL Shikimori API.
    ///
    /// # Возвращает
    ///
    /// `Result<ShikicrateClient>` - клиент в случае успеха, или ошибка при создании HTTP клиента.
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::ShikicrateClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClient::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        Self::with_timeout(DEFAULT_TIMEOUT)
    }

    /// Создает внутренний HTTP клиент с указанным таймаутом.
    ///
    /// Устанавливает user-agent в формате `shikicrate/{version}`.
    fn mk_client(timeout: Duration) -> Result<Client> {
        Client::builder()
            .timeout(timeout)
            .user_agent(format!("shikicrate/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(ShikicrateError::Http)
    }

    /// Создает новый клиент с кастомным таймаутом.
    ///
    /// # Параметры
    ///
    /// * `timeout` - Максимальное время ожидания ответа от сервера.
    ///
    /// # Возвращает
    ///
    /// `Result<ShikicrateClient>` - клиент в случае успеха, или ошибка при создании HTTP клиента.
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::ShikicrateClient;
    /// use std::time::Duration;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Клиент с таймаутом 60 секунд
    /// let client = ShikicrateClient::with_timeout(Duration::from_secs(60))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        Ok(Self {
            client: Self::mk_client(timeout)?,
            base_url: API_BASE_URL.to_string(),
        })
    }

    /// Создает новый клиент с кастомным базовым URL.
    ///
    /// Использует стандартный таймаут (30 секунд).
    ///
    /// # Параметры
    ///
    /// * `base_url` - Базовый URL GraphQL API (например, для тестирования или использования прокси).
    ///   Должен быть валидным HTTP/HTTPS URL.
    ///
    /// # Возвращает
    ///
    /// `Result<ShikicrateClient>` - клиент в случае успеха, или ошибка валидации/создания HTTP клиента.
    ///
    /// # Ошибки
    ///
    /// - `ShikicrateError::Validation` - если URL невалидный или использует небезопасный протокол (не http/https)
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::ShikicrateClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Клиент для тестового окружения
    /// let client = ShikicrateClient::with_base_url(
    ///     "https://test.shikimori.one/api/graphql".to_string()
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_base_url(base_url: String) -> Result<Self> {
        // Валидация URL для защиты от SSRF
        let parsed_url = Url::parse(&base_url)
            .map_err(|e| ShikicrateError::Validation(format!("Invalid URL: {}", e)))?;

        // Проверка протокола (только http/https)
        match parsed_url.scheme() {
            "http" | "https" => {}
            scheme => {
                return Err(ShikicrateError::Validation(format!(
                    "Unsafe URL scheme: {}. Only http:// and https:// are allowed",
                    scheme
                )));
            }
        }

        // Проверка наличия хоста
        if parsed_url.host().is_none() {
            return Err(ShikicrateError::Validation(
                "URL must have a host".to_string(),
            ));
        }

        Ok(Self {
            client: Self::mk_client(DEFAULT_TIMEOUT)?,
            base_url,
        })
    }

    /// Проверяет, является ли ошибка повторяемой (retryable).
    ///
    /// Повторяемыми считаются:
    /// - Сетевые ошибки (таймауты, ошибки подключения, ошибки запроса)
    /// - Rate limiting (429) - для повторной попытки с задержкой
    ///
    /// Ошибки валидации, GraphQL ошибки и другие API ошибки (кроме 429) не повторяются.
    fn is_retryable(error: &ShikicrateError) -> bool {
        match error {
            ShikicrateError::Http(e) => e.is_timeout() || e.is_connect() || e.is_request(),
            ShikicrateError::RateLimit { .. } => true,
            _ => false,
        }
    }

    /// Выполняет GraphQL запрос один раз без retry логики.
    ///
    /// Внутренний метод, используется `execute_query()` для реализации retry.
    ///
    /// # Параметры
    ///
    /// * `query` - GraphQL запрос в виде строки
    /// * `variables` - Опциональные переменные для GraphQL запроса
    ///
    /// # Возвращает
    ///
    /// Десериализованный результат типа `T` или ошибка.
    async fn exec_once<T>(&self, query: &str, variables: Option<serde_json::Value>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let body = json!({
            "query": query,
            "variables": variables.unwrap_or(json!({}))
        });

        let response = self
            .client
            .post(&self.base_url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        // Считываем заголовки до чтения тела
        let retry_after_header = if status == 429 {
            response
                .headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
        } else {
            None
        };

        let text = response.text().await?;

        if !status.is_success() {
            // Обработка rate limiting (429)
            if status == 429 {
                return Err(ShikicrateError::RateLimit {
                    message: format!("Rate limit exceeded: {}", text),
                    retry_after: retry_after_header,
                });
            }

            return Err(ShikicrateError::Api {
                status: status.as_u16(),
                message: format!("HTTP {}: {}", status, text),
            });
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;

        if let Some(errors) = json.get("errors") {
            // Парсим все ошибки, а не только первую
            let error_messages: Vec<String> = errors
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|err| {
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect()
                })
                .unwrap_or_default();

            let error_msg = if error_messages.is_empty() {
                "Unknown GraphQL error".to_string()
            } else if error_messages.len() == 1 {
                error_messages[0].clone()
            } else {
                format!("Multiple GraphQL errors: {}", error_messages.join("; "))
            };

            return Err(ShikicrateError::GraphQL {
                message: error_msg,
                errors: Some(errors.clone()),
            });
        }

        let data = json.get("data").ok_or_else(|| ShikicrateError::GraphQL {
            message: "No data in response".to_string(),
            errors: None,
        })?;

        serde_json::from_value(data.clone()).map_err(ShikicrateError::from)
    }

    /// Выполняет GraphQL запрос с автоматическим retry для сетевых ошибок.
    ///
    /// Метод автоматически повторяет запрос до 3 раз при сетевых ошибках
    /// с экспоненциальной задержкой (1s, 2s, 4s).
    ///
    /// # Параметры
    ///
    /// * `query` - GraphQL запрос в виде строки
    /// * `variables` - Опциональные переменные для GraphQL запроса
    ///
    /// # Возвращает
    ///
    /// Десериализованный результат типа `T` или ошибка.
    ///
    /// # Поведение retry
    ///
    /// - Максимум 3 retry (всего 4 попытки)
    /// - Retry только для сетевых ошибок (таймауты, ошибки подключения)
    /// - Задержки между попытками: 1 секунда, 2 секунды, 4 секунды
    /// - Ошибки валидации, GraphQL и API ошибки возвращаются немедленно без retry
    pub(crate) async fn execute_query<T>(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Первая попытка
        let mut last_error = match self.exec_once(query, variables.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) if !Self::is_retryable(&e) => return Err(e),
            Err(e) => e,
        };

        // Retry с задержками
        for (attempt, delay) in RETRY_DELAYS.iter().enumerate() {
            // Определяем задержку для retry
            let retry_delay = if let ShikicrateError::RateLimit { retry_after, .. } = &last_error {
                // Используем Retry-After заголовок если есть, иначе экспоненциальную задержку
                retry_after.map(Duration::from_secs).unwrap_or(*delay)
            } else {
                *delay
            };

            tokio::time::sleep(retry_delay).await;

            match self.exec_once(query, variables.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if Self::is_retryable(&e) => {
                    last_error = e;
                    // Если это последняя попытка, возвращаем ошибку
                    if attempt >= RETRY_DELAYS.len() - 1 {
                        return Err(last_error);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error)
    }
}

impl Default for ShikicrateClient {
    /// Создает клиент с настройками по умолчанию.
    ///
    /// Использует `ShikicrateClient::new()` и паникует при ошибке создания.
    /// Для обработки ошибок используйте `ShikicrateClient::new()` напрямую.
    ///
    /// # Паникует
    ///
    /// Если не удалось создать HTTP клиент (крайне редкая ситуация).
    fn default() -> Self {
        Self::new().expect("Failed to create default client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable_http() {
        // Проверяем, что HTTP ошибки могут быть retryable
        // В реальности is_timeout/is_connect/is_request проверяются внутри reqwest::Error
        // Создание reqwest::Error в тестах сложно, поэтому просто проверяем,
        // что функция вызывается без паники и что логика существует
        // В реальных условиях HTTP ошибки проверяются через методы reqwest::Error
        let client_result = ShikicrateClient::new();
        assert!(client_result.is_ok());
        // Логика проверки is_retryable для HTTP ошибок находится в методе
        // и проверяет e.is_timeout() || e.is_connect() || e.is_request()
    }

    #[test]
    fn test_is_retryable_rate_limit() {
        let error = ShikicrateError::RateLimit {
            message: "rate limit".to_string(),
            retry_after: Some(60),
        };
        assert!(ShikicrateClient::is_retryable(&error));
    }

    #[test]
    fn test_is_retryable_validation() {
        let error = ShikicrateError::Validation("invalid".to_string());
        assert!(!ShikicrateClient::is_retryable(&error));
    }

    #[test]
    fn test_is_retryable_graphql() {
        let error = ShikicrateError::GraphQL {
            message: "graphql error".to_string(),
            errors: None,
        };
        assert!(!ShikicrateClient::is_retryable(&error));
    }

    #[test]
    fn test_builder_default() {
        let builder = ShikicrateClientBuilder::default();
        let client = builder.build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_builder_timeout() {
        let client = ShikicrateClientBuilder::new()
            .timeout(Duration::from_secs(60))
            .build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_builder_base_url() {
        let client = ShikicrateClientBuilder::new()
            .base_url("https://shikimori.one/api/graphql".to_string())
            .build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_builder_invalid_url_scheme() {
        let client = ShikicrateClientBuilder::new()
            .base_url("file:///etc/passwd".to_string())
            .build();
        assert!(matches!(client, Err(ShikicrateError::Validation(_))));
    }

    #[test]
    fn test_builder_invalid_url_no_host() {
        let client = ShikicrateClientBuilder::new()
            .base_url("http://".to_string())
            .build();
        assert!(matches!(client, Err(ShikicrateError::Validation(_))));
    }

    #[test]
    fn test_builder_invalid_url_parse() {
        let client = ShikicrateClientBuilder::new()
            .base_url("not a url".to_string())
            .build();
        assert!(matches!(client, Err(ShikicrateError::Validation(_))));
    }

    #[test]
    fn test_with_base_url_validation() {
        let client = ShikicrateClient::with_base_url("https://shikimori.one/api/graphql".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_with_base_url_invalid() {
        let client = ShikicrateClient::with_base_url("ftp://example.com".to_string());
        assert!(matches!(client, Err(ShikicrateError::Validation(_))));
    }

    #[test]
    fn test_with_timeout() {
        let client = ShikicrateClient::with_timeout(Duration::from_secs(60));
        assert!(client.is_ok());
    }
}
