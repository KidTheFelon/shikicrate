use reqwest::header::{ORIGIN, REFERER, ACCEPT, CONTENT_TYPE};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let body = json!({
        "query": "{ animes(search: \"Naruto\", limit: 1) { id name } }"
    });

    println!("Отправка запроса...");
    
    let response = client
        .post("https://shikimori.one/api/graphql")
        .header(ORIGIN, "https://shikimori.one")
        .header(REFERER, "https://shikimori.one/")
        .header("X-Requested-With", "XMLHttpRequest")
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    println!("Статус: {}", status);
    
    let text = response.text().await?;
    println!("Ответ: {}", text);

    Ok(())
}
