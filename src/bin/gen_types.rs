use shikicrate::types::*;
use ts_rs::TS;

fn main() {
    let types: Vec<String> = vec![
        Date::export_to_string().unwrap(),
        Poster::export_to_string().unwrap(),
        Genre::export_to_string().unwrap(),
        Studio::export_to_string().unwrap(),
        Publisher::export_to_string().unwrap(),
        ExternalLink::export_to_string().unwrap(),
        Person::export_to_string().unwrap(),
        PersonRole::export_to_string().unwrap(),
        Character::export_to_string().unwrap(),
        CharacterRole::export_to_string().unwrap(),
        RelatedAnime::export_to_string().unwrap(),
        RelatedManga::export_to_string().unwrap(),
        SimilarAnime::export_to_string().unwrap(),
        SimilarAnimeImage::export_to_string().unwrap(),
        Related::export_to_string().unwrap(),
        Video::export_to_string().unwrap(),
        Screenshot::export_to_string().unwrap(),
        ScoreStat::export_to_string().unwrap(),
        StatusStat::export_to_string().unwrap(),
        Anime::export_to_string().unwrap(),
        Manga::export_to_string().unwrap(),
        CharacterFull::export_to_string().unwrap(),
        PersonFull::export_to_string().unwrap(),
    ];

    let output = types.join("\n\n");
    
    let out_path = "../src/types/generated.ts";
    std::fs::write(out_path, output).expect("Failed to write TypeScript types");
    
    println!("TypeScript types generated to {}", out_path);
}
