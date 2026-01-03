use crate::error::{Result, ShikicrateError};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

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
/// Ошибки валидации, GraphQL ошибки и API ошибки (неуспешные HTTP статусы) не повторяются.
pub struct ShikicrateClient {
    client: Client,
    base_url: String,
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
    /// // Клиент для тестового окружения
    /// let client = ShikicrateClient::with_base_url(
    ///     "https://test.shikimori.one/api/graphql".to_string()
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_base_url(base_url: String) -> Result<Self> {
        Ok(Self {
            client: Self::mk_client(DEFAULT_TIMEOUT)?,
            base_url,
        })
    }

    /// Проверяет, является ли ошибка повторяемой (retryable).
    ///
    /// Повторяемыми считаются только сетевые ошибки:
    /// - Таймауты
    /// - Ошибки подключения
    /// - Ошибки запроса
    ///
    /// Ошибки валидации, GraphQL ошибки и API ошибки не повторяются.
    fn is_retryable(error: &ShikicrateError) -> bool {
        match error {
            ShikicrateError::Http(e) => {
                e.is_timeout() || e.is_connect() || e.is_request()
            }
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
        let text = response.text().await?;

        if !status.is_success() {
            return Err(ShikicrateError::Api {
                message: format!("HTTP {}: {}", status, text),
            });
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;

        if let Some(errors) = json.get("errors") {
            let error_msg = errors
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|err| err.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown GraphQL error")
                .to_string();

            return Err(ShikicrateError::GraphQL(error_msg));
        }

        let data = json
            .get("data")
            .ok_or_else(|| ShikicrateError::GraphQL("No data in response".to_string()))?;

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
    /// - Максимум 3 попытки
    /// - Retry только для сетевых ошибок (таймауты, ошибки подключения)
    /// - Задержки между попытками: 1 секунда, 2 секунды, 4 секунды
    /// - Ошибки валидации, GraphQL и API ошибки возвращаются немедленно без retry
    pub(crate) async fn execute_query<T>(&self, query: &str, variables: Option<serde_json::Value>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut last_error = None;

        for (attempt, delay) in RETRY_DELAYS.iter().enumerate() {
            match self.exec_once(query, variables.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if Self::is_retryable(&e) => {
                    last_error = Some(e);
                    if attempt < RETRY_DELAYS.len() - 1 {
                        tokio::time::sleep(*delay).await;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap())
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
