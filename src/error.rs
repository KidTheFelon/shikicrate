use thiserror::Error;

/// Ошибки, которые могут возникнуть при работе с Shikimori GraphQL API.
///
/// Все ошибки реализуют `std::error::Error` и `std::fmt::Display`,
/// что позволяет легко логировать и обрабатывать их.
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::{ShikicrateClient, Result, queries::*};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = ShikicrateClient::new()?;
///     
///     // Ошибка валидации
///     let params = AnimeSearchParams {
///         search: None,
///         limit: Some(-1), // Невалидное значение
///         kind: None,
///     };
///     
///     match client.animes(params).await {
///         Err(shikicrate::ShikicrateError::Validation(msg)) => {
///             eprintln!("Ошибка валидации: {}", msg);
///         }
///         Ok(animes) => println!("Найдено {} аниме", animes.len()),
///         Err(e) => eprintln!("Другая ошибка: {}", e),
///     }
///     
///     Ok(())
/// }
/// ```
#[derive(Error, Debug)]
pub enum ShikicrateError {
    /// Ошибка HTTP-запроса.
    ///
    /// Возникает при проблемах с сетью, таймаутах, ошибках подключения
    /// или других проблемах на уровне HTTP-клиента.
    ///
    /// Автоматически конвертируется из `reqwest::Error`.
    ///
    /// # Примеры ситуаций
    /// - Таймаут запроса
    /// - Ошибка подключения к серверу
    /// - DNS-ошибка
    /// - SSL/TLS ошибка
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    /// Ошибка GraphQL запроса.
    ///
    /// Возникает когда GraphQL API возвращает ошибку в поле `errors`
    /// ответа, или когда в ответе отсутствует поле `data`.
    ///
    /// # Примеры ситуаций
    /// - Невалидный GraphQL запрос
    /// - Ошибка валидации на стороне сервера
    /// - Отсутствие данных в ответе
    #[error("GraphQL error: {0}")]
    GraphQL(String),
    
    /// Ошибка сериализации/десериализации JSON.
    ///
    /// Возникает при проблемах с преобразованием данных в/из JSON.
    ///
    /// Автоматически конвертируется из `serde_json::Error`.
    ///
    /// # Примеры ситуаций
    /// - Невалидный JSON в ответе
    /// - Несоответствие структуры данных ожидаемому типу
    /// - Ошибка парсинга числа или строки
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Ошибка API (неуспешный HTTP статус).
    ///
    /// Возникает когда сервер возвращает HTTP статус, отличный от 2xx.
    ///
    /// # Примеры ситуаций
    /// - 404 Not Found
    /// - 500 Internal Server Error
    /// - 429 Too Many Requests
    #[error("API error: {message}")]
    Api { message: String },
    
    /// Ошибка валидации параметров запроса.
    ///
    /// Возникает при попытке выполнить запрос с невалидными параметрами
    /// (например, отрицательный `limit`, `page` меньше 1, пустой `ids`).
    ///
    /// # Примеры ситуаций
    /// - `limit <= 0`
    /// - `page < 1`
    /// - Пустой вектор `ids`
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Тип-алиас для `Result<T, ShikicrateError>`.
///
/// Упрощает работу с результатами операций клиента.
///
/// # Пример
///
/// ```no_run
/// use shikicrate::Result;
///
/// async fn fetch_anime() -> Result<()> {
///     // ...
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, ShikicrateError>;
