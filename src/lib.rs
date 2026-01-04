//! # Shikicrate
//!
//! Rust клиент для работы с GraphQL API Shikimori.
//!
//! ## Основные возможности
//!
//! - Поиск аниме с фильтрацией по типу и названию
//! - Поиск манги с фильтрацией по типу и названию
//! - Поиск персонажей (по странице или по ID)
//! - Поиск людей (сейю, мангаки, продюсеры)
//! - Поиск пользовательских оценок
//! - Автоматический retry для сетевых ошибок с экспоненциальной задержкой
//! - Валидация параметров запросов
//!
//! ## Быстрый старт
//!
//! ```no_run
//! use shikicrate::{ShikicrateClient, queries::*};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Создание клиента с настройками по умолчанию
//!     let client = ShikicrateClient::new()?;
//!
//!     // Поиск аниме
//!     let animes = client.animes(AnimeSearchParams {
//!         search: Some("naruto".to_string()),
//!         limit: Some(10),
//!         kind: None,
//!         page: None,
//!     }).await?;
//!
//!     for anime in animes {
//!         println!("{} (ID: {})", anime.name, anime.id);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Настройка клиента через Builder
//!
//! ```no_run
//! use shikicrate::{ShikicrateClientBuilder, queries::*};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Создание клиента с кастомными настройками
//!     let client = ShikicrateClientBuilder::new()
//!         .timeout(Duration::from_secs(60))
//!         .base_url("https://shikimori.one/api/graphql".to_string())
//!         .build()?;
//!
//!     // Использование клиента...
//!     Ok(())
//! }
//! ```
//!
//! ## Модули
//!
//! - [`client`] - HTTP клиент для выполнения GraphQL запросов
//! - [`error`] - Типы ошибок
//! - [`types`] - Типы данных (Anime, Manga, Character, Person и т.д.)
//! - [`queries`] - Методы для выполнения запросов и параметры поиска
//!
//! ## Retry логика
//!
//! Клиент автоматически повторяет запросы при следующих ошибках:
//! - Сетевые ошибки (таймауты, ошибки подключения, ошибки запроса)
//! - Rate limiting (429 Too Many Requests) - с учетом заголовка `Retry-After`
//!
//! Retry выполняется до 3 раз с экспоненциальной задержкой: 1 секунда, 2 секунды, 4 секунды.
//! Для rate limiting используется значение из заголовка `Retry-After`, если оно указано.
//!
//! Ошибки валидации, GraphQL ошибки и другие API ошибки (неуспешные HTTP статусы, кроме 429) не повторяются.
//!
//! ## Валидация параметров
//!
//! Все методы автоматически валидируют параметры запроса:
//! - `limit` должен быть > 0
//! - `page` должен быть >= 1
//! - `ids` не должен быть пустым вектором
//!
//! При невалидных параметрах возвращается `ShikicrateError::Validation`.
//!
//! ## Примеры
//!
//! Смотрите примеры использования в модулях:
//! - [`ShikicrateClient`] - создание и настройка клиента
//! - [`queries`] - методы поиска и параметры
//! - [`types`] - структуры данных

pub mod client;
pub mod error;
pub mod pagination;
pub mod queries;
pub mod types;

pub use client::{ShikicrateClient, ShikicrateClientBuilder};
pub use error::{Result, ShikicrateError};
pub use queries::*;
pub use types::*;
