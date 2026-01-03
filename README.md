# Shikicrate GraphQL Client

Rust клиент для работы с GraphQL API Shikimori.

## Установка

Добавь в `Cargo.toml`:

```toml
[dependencies]
shikicrate = "0.1.0"
```

Или для локальной разработки:

```toml
[dependencies]
shikicrate = { path = "../shikimori" }
```

## Использование

### Базовый пример

```rust
use shikicrate::{ShikicrateClient, queries::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ShikicrateClient::new()?;
    
    // Поиск аниме
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

### Поиск манги

```rust
let params = MangaSearchParams {
    limit: Some(5),
    search: Some("one piece".to_string()),
    kind: None,
};

let mangas = client.mangas(params).await?;
```

### Поиск персонажей

```rust
// Поиск по странице
let params = CharacterSearchParams {
    page: Some(1),
    limit: Some(20),
    ids: None,
};

let characters = client.characters(params).await?;

// Поиск по ID
let params = CharacterSearchParams {
    page: None,
    limit: None,
    ids: Some(vec!["1".to_string(), "2".to_string()]),
};

let characters = client.characters(params).await?;
```

### Поиск людей

```rust
let params = PeopleSearchParams {
    limit: Some(10),
    search: Some("miyazaki".to_string()),
};

let people = client.people(params).await?;
```

### Поиск пользовательских оценок

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

## Структура проекта

- `src/client.rs` - HTTP клиент для выполнения GraphQL запросов
- `src/error.rs` - Типы ошибок
- `src/types.rs` - Типы данных (Anime, Manga, Character, Person и т.д.)
- `src/queries.rs` - Методы для выполнения запросов
- `animes`, `mangas`, `characters`, `people`, `userrates` - GraphQL запросы

## Пример использования

Запусти пример приложения для проверки работоспособности:

```bash
cargo run --example test_client
```

Пример демонстрирует:
- Поиск аниме с фильтрами
- Поиск манги
- Поиск персонажей (по странице и по ID)
- Поиск людей
- Вывод детальной информации

## Тестирование

```bash
cargo test
```

## Лицензия

MIT OR Apache-2.0
