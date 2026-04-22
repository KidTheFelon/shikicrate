use shikicrate::types::*;
use ts_rs::TS;

fn main() {
    let cfg = ts_rs::Config::new().with_large_int("bigint");

    let types: Vec<String> = vec![
        Date::export_to_string(&cfg).unwrap(),
        Poster::export_to_string(&cfg).unwrap(),
        Genre::export_to_string(&cfg).unwrap(),
        Studio::export_to_string(&cfg).unwrap(),
        Publisher::export_to_string(&cfg).unwrap(),
        ExternalLink::export_to_string(&cfg).unwrap(),
        Person::export_to_string(&cfg).unwrap(),
        PersonRole::export_to_string(&cfg).unwrap(),
        Character::export_to_string(&cfg).unwrap(),
        CharacterRole::export_to_string(&cfg).unwrap(),
        RelatedAnime::export_to_string(&cfg).unwrap(),
        RelatedManga::export_to_string(&cfg).unwrap(),
        SimilarAnime::export_to_string(&cfg).unwrap(),
        SimilarAnimeImage::export_to_string(&cfg).unwrap(),
        Related::export_to_string(&cfg).unwrap(),
        Video::export_to_string(&cfg).unwrap(),
        Screenshot::export_to_string(&cfg).unwrap(),
        ScoreStat::export_to_string(&cfg).unwrap(),
        StatusStat::export_to_string(&cfg).unwrap(),
        Anime::export_to_string(&cfg).unwrap(),
        Manga::export_to_string(&cfg).unwrap(),
        CharacterFull::export_to_string(&cfg).unwrap(),
        PersonFull::export_to_string(&cfg).unwrap(),
    ];

    let output = types.join("\n\n");

    let out_path = "../src/types/generated.ts";
    std::fs::write(out_path, output).expect("Failed to write TypeScript types");

    println!("TypeScript types generated to {}", out_path);
}
