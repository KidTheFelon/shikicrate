use shikicrate::{Result, ShikicrateClient, queries::*};

#[tokio::test]
async fn test_search_animes() -> Result<()> {
    let client = ShikicrateClient::new()?;

    let params = AnimeSearchParams {
        search: Some("bakemono".to_string()),
        limit: Some(1),
        kind: Some("!special".to_string()),
        page: None,
    };

    let animes = client.animes(params).await?;

    assert!(!animes.is_empty());
    println!("Found {} anime(s)", animes.len());

    if let Some(anime) = animes.first() {
        println!("First result: {} (ID: {})", anime.name, anime.id);
        if let Some(russian) = &anime.russian {
            println!("Russian name: {}", russian);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_search_mangas() -> Result<()> {
    let client = ShikicrateClient::new()?;

    let params = MangaSearchParams {
        limit: Some(5),
        search: None,
        kind: None,
        page: None,
    };

    let mangas = client.mangas(params).await?;

    println!("Found {} manga(s)", mangas.len());

    Ok(())
}

#[tokio::test]
async fn test_search_people() -> Result<()> {
    let client = ShikicrateClient::new()?;

    let params = PeopleSearchParams {
        limit: Some(1),
        search: None,
    };

    let people = client.people(params).await?;

    println!("Found {} people", people.len());

    Ok(())
}

#[tokio::test]
async fn test_search_characters() -> Result<()> {
    let client = ShikicrateClient::new()?;

    let params = CharacterSearchParams {
        page: Some(1),
        limit: Some(1),
        ids: None,
    };

    let characters = client.characters(params).await?;

    println!("Found {} character(s)", characters.len());

    Ok(())
}

#[tokio::test]
async fn test_search_characters_by_ids() -> Result<()> {
    let client = ShikicrateClient::new()?;

    let params = CharacterSearchParams {
        page: None,
        limit: None,
        ids: Some(vec!["1".to_string(), "2".to_string()]),
    };

    let characters = client.characters(params).await?;

    println!("Found {} character(s) by IDs", characters.len());

    Ok(())
}
