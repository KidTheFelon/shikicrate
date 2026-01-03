# Shikicrate

[![crates.io](https://img.shields.io/crates/v/shikicrate.svg)](https://crates.io/crates/shikicrate)
[![docs.rs](https://docs.rs/shikicrate/badge.svg)](https://docs.rs/shikicrate)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/KidTheFelon/shikicrate/workflows/CI/badge.svg)](https://github.com/KidTheFelon/shikicrate/actions)

Rust клиент для Shikimori GraphQL API. Потому что парсить JSON руками — это прошлый век.

## Что это вообще такое?

Нужен доступ к аниме, манге, персонажам и прочей японской культуре через Shikimori? Не хочешь писать GraphQL запросы вручную? Добро пожаловать. Клиент сам разберется с retry-логикой, rate limiting и прочими радостями жизни.

## Установка

Кидай в `Cargo.toml`:

```toml
[dependencies]
shikicrate = "0.1.1"
```

Или для локальной разработки (если хочешь поковырять код):

```toml
[dependencies]
shikicrate = { path = "../shikicrate" }
```

## Быстрый старт

```rust
use shikicrate::{ShikicrateClient, queries::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ShikicrateClient::new()?;
    
    // Ищем Наруто (или что там тебе нужно)
    let params = AnimeSearchParams {
        search: Some("naruto".to_string()),
        limit: Some(10),
        kind: None,
    };
    
    let animes = client.animes(params).await?;
    
    for anime in animes {
        println!("{} (ID: {})", anime.name, anime.id);
    }
    
    Ok(())
}
```

## Что умеет

### Аниме

```rust
let params = AnimeSearchParams {
    limit: Some(10),
    search: Some("naruto".to_string()),
    kind: None,
};

let animes = client.animes(params).await?;
```

### Манга

```rust
let params = MangaSearchParams {
    limit: Some(5),
    search: Some("one piece".to_string()),
    kind: None,
};

let mangas = client.mangas(params).await?;
```

### Персонажи

Можно искать по странице, можно по ID — как удобнее:

```rust
// По странице
let params = CharacterSearchParams {
    page: Some(1),
    limit: Some(20),
    ids: None,
};

let characters = client.characters(params).await?;

// Или по ID (если знаешь, кого ищешь)
let params = CharacterSearchParams {
    page: None,
    limit: None,
    ids: Some(vec!["1".to_string(), "2".to_string()]),
};

let characters = client.characters(params).await?;
```

### Люди (режиссеры, сценаристы и прочие)

```rust
let params = PeopleSearchParams {
    limit: Some(10),
    search: Some("miyazaki".to_string()),
};

let people = client.people(params).await?;
```

### Пользовательские оценки

```rust
let params = UserRateSearchParams {
    page: Some(1),
    limit: Some(20),
    target_type: Some("Anime".to_string()),
    order_field: Some("updated_at".to_string()),
    order: Some("desc".to_string()),
};

let user_rates = client.user_rates(params).await?;
```

## Настройка клиента

### Builder (если любишь цепочки методов)

```rust
use shikicrate::ShikicrateClientBuilder;
use std::time::Duration;

let client = ShikicrateClientBuilder::new()
    .timeout(Duration::from_secs(60))
    .base_url("https://shikimori.one/api/graphql".to_string())
    .build()?;
```

### Прямое создание (если не любишь)

```rust
use shikicrate::ShikicrateClient;

// Дефолтные настройки (30 секунд таймаут)
let client = ShikicrateClient::new()?;

// С кастомным таймаутом
let client = ShikicrateClient::with_timeout(Duration::from_secs(60))?;

// С кастомным URL (если у тебя свой инстанс)
let client = ShikicrateClient::with_base_url("https://api.example.com/graphql".to_string())?;
```

## Обработка ошибок

Клиент сам разбирается с:
- **Rate limiting (429)**: ждет `Retry-After` и повторяет запрос
- **Сетевые ошибки**: ретраит до 3 раз с экспоненциальной задержкой (1s → 2s → 4s)
- **GraphQL ошибки**: возвращает все сообщения об ошибках
- **Валидация**: проверяет параметры до отправки (чтобы не тратить время зря)

```rust
use shikicrate::{ShikicrateError, Result};

match client.animes(params).await {
    Err(ShikicrateError::RateLimit { retry_after, .. }) => {
        println!("Rate limit, retry after: {:?} seconds", retry_after);
    }
    Err(ShikicrateError::Validation(msg)) => {
        println!("Validation error: {}", msg);
    }
    Ok(animes) => println!("Found {} animes", animes.len()),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Примеры

Запусти пример, чтобы посмотреть, как это работает:

```bash
cargo run --example test_client
```

Там показывается:
- Поиск аниме с фильтрами
- Поиск манги
- Поиск персонажей (по странице и по ID)
- Поиск людей
- Вывод детальной информации

## Структура проекта

- `src/client.rs` — HTTP клиент, который делает всю грязную работу
- `src/error.rs` — типы ошибок (чтобы знать, что пошло не так)
- `src/types.rs` — типы данных (Anime, Manga, Character, Person и т.д.)
- `src/queries.rs` — методы для выполнения запросов
- `animes`, `mangas`, `characters`, `people`, `userrates` — GraphQL запросы

## Тесты

```bash
cargo test
```

## Лицензия

MIT OR Apache-2.0 — используй как хочешь.
