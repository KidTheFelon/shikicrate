use shikicrate::{ShikicrateClient, queries::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Тестирование Shikicrate GraphQL клиента\n");

    let client = ShikicrateClient::new()?;

    // Тест 1: Поиск аниме
    println!("📺 Тест 1: Поиск аниме");
    println!("Поиск: 'bakemono', лимит: 3, исключить спешлы\n");

    let animes = client
        .animes(AnimeSearchParams {
            search: Some("bakemono".to_string()),
            limit: Some(3),
            kind: Some("!special".to_string()),
        })
        .await?;

    println!("Найдено аниме: {}\n", animes.len());

    for (i, anime) in animes.iter().enumerate() {
        println!("  {}. {} (ID: {})", i + 1, anime.name, anime.id);
        if let Some(russian) = &anime.russian {
            println!("     Русское название: {}", russian);
        }
        if let Some(score) = anime.score {
            println!("     Оценка: {:.2}", score);
        }
        if let Some(status) = &anime.status {
            println!("     Статус: {}", status);
        }
        println!();
    }

    // Тест 2: Поиск манги
    println!("📚 Тест 2: Поиск манги");
    println!("Лимит: 5\n");

    let mangas = client
        .mangas(MangaSearchParams {
            limit: Some(5),
            search: None,
            kind: None,
        })
        .await?;

    println!("Найдено манги: {}\n", mangas.len());

    for (i, manga) in mangas.iter().take(3).enumerate() {
        println!("  {}. {} (ID: {})", i + 1, manga.name, manga.id);
        if let Some(russian) = &manga.russian {
            println!("     Русское название: {}", russian);
        }
        if let Some(chapters) = manga.chapters {
            println!("     Глав: {}", chapters);
        }
        println!();
    }

    // Тест 3: Поиск персонажей
    println!("👤 Тест 3: Поиск персонажей");
    println!("Страница: 1, лимит: 5\n");

    let characters = client
        .characters(CharacterSearchParams {
            page: Some(1),
            limit: Some(5),
            ids: None,
        })
        .await?;

    println!("Найдено персонажей: {}\n", characters.len());

    for (i, character) in characters.iter().take(3).enumerate() {
        println!("  {}. {} (ID: {})", i + 1, character.name, character.id);
        if let Some(russian) = &character.russian {
            println!("     Русское имя: {}", russian);
        }
        println!();
    }

    // Тест 4: Поиск персонажей по ID
    println!("🔍 Тест 4: Поиск персонажей по ID");
    println!("ID: [1, 2, 3]\n");

    let characters_by_ids = client
        .characters(CharacterSearchParams {
            page: None,
            limit: None,
            ids: Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
        })
        .await?;

    println!("Найдено персонажей: {}\n", characters_by_ids.len());

    for character in &characters_by_ids {
        println!("  - {} (ID: {})", character.name, character.id);
    }
    println!();

    // Тест 5: Поиск людей
    println!("👥 Тест 5: Поиск людей");
    println!("Лимит: 3\n");

    let people = client
        .people(PeopleSearchParams {
            limit: Some(3),
            search: None,
        })
        .await?;

    println!("Найдено людей: {}\n", people.len());

    for (i, person) in people.iter().enumerate() {
        println!("  {}. {} (ID: {})", i + 1, person.name, person.id);
        if let Some(russian) = &person.russian {
            println!("     Русское имя: {}", russian);
        }
        if let Some(is_seyu) = person.is_seyu {
            if is_seyu {
                println!("     Сейю");
            }
        }
        if let Some(is_mangaka) = person.is_mangaka {
            if is_mangaka {
                println!("     Мангака");
            }
        }
        println!();
    }

    // Тест 6: Детальная информация об аниме
    if let Some(first_anime) = animes.first() {
        println!("📋 Тест 6: Детальная информация об аниме");
        println!("Аниме: {}\n", first_anime.name);

        if let Some(genres) = &first_anime.genres {
            if !genres.is_empty() {
                println!("  Жанры:");
                for genre in genres.iter().take(5) {
                    println!("    - {}", genre.name);
                }
                println!();
            }
        }

        if let Some(studios) = &first_anime.studios {
            if !studios.is_empty() {
                println!("  Студии:");
                for studio in studios.iter() {
                    println!("    - {}", studio.name);
                }
                println!();
            }
        }

        if let Some(aired_on) = &first_anime.aired_on {
            if let Some(date) = &aired_on.date {
                println!("  Дата выхода: {}", date);
            }
        }

        if let Some(description) = &first_anime.description {
            let desc_short = if description.len() > 200 {
                &description[..200]
            } else {
                description
            };
            println!("\n  Описание: {}...", desc_short);
        }
    }

    println!("\n✅ Все тесты пройдены успешно!");

    Ok(())
}
