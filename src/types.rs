use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

fn deser_id<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct IdVisitor;

    impl<'de> serde::de::Visitor<'de> for IdVisitor {
        type Value = i64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer or a string containing an integer")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value as i64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse().map_err(serde::de::Error::custom)
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Err(serde::de::Error::custom("ID cannot be null"))
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Err(serde::de::Error::custom("ID cannot be null"))
        }
    }

    deserializer.deserialize_any(IdVisitor)
}

fn deser_opt_id<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionIdVisitor;

    impl<'de> serde::de::Visitor<'de> for OptionIdVisitor {
        type Value = Option<i64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer, a string containing an integer, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deser_id(deserializer).map(Some)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
            Ok(Some(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
            Ok(Some(value as i64))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse().map(Some).map_err(serde::de::Error::custom)
        }
    }

    deserializer.deserialize_option(OptionIdVisitor)
}

/// Дата с опциональными компонентами.
///
/// Используется для дат выхода аниме/манги, дат рождения людей и т.д.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Date {
    /// Год (например, 2024).
    pub year: Option<i32>,

    /// Месяц (1-12).
    pub month: Option<i32>,

    /// День месяца (1-31).
    pub day: Option<i32>,

    /// Полная дата в формате строки (ISO 8601 или другой формат API).
    pub date: Option<String>,
}

/// Постер (изображение) для аниме, манги, персонажа или человека.
///
/// Содержит ссылки на изображения разных размеров.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poster {
    /// ID постера в системе Shikimori.
    #[serde(deserialize_with = "deser_opt_id")]
    pub id: Option<i64>,

    /// URL оригинального изображения (полный размер).
    #[serde(rename = "originalUrl")]
    pub original_url: Option<String>,

    /// URL основного изображения (оптимизированный размер).
    #[serde(rename = "mainUrl")]
    pub main_url: Option<String>,
}

/// Жанр аниме или манги.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    /// ID жанра в системе Shikimori.
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,

    /// Название жанра на английском.
    pub name: String,

    /// Название жанра на русском.
    pub russian: Option<String>,

    /// Тип жанра (например, "anime", "manga").
    pub kind: Option<String>,
}

/// Студия аниме.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Studio {
    /// ID студии в системе Shikimori.
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,

    /// Название студии.
    pub name: String,

    /// URL логотипа студии.
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
}

/// Издательство манги.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    /// ID издательства в системе Shikimori.
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,

    /// Название издательства.
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalLink {
    #[serde(deserialize_with = "deser_opt_id")]
    pub id: Option<i64>,
    pub kind: String,
    pub url: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,
    pub name: String,
    pub poster: Option<Poster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonRole {
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,
    #[serde(rename = "rolesRu")]
    pub roles_ru: Option<Vec<String>>,
    #[serde(rename = "rolesEn")]
    pub roles_en: Option<Vec<String>>,
    pub person: Person,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,
    pub name: String,
    pub poster: Option<Poster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRole {
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,
    #[serde(rename = "rolesRu")]
    pub roles_ru: Option<Vec<String>>,
    #[serde(rename = "rolesEn")]
    pub roles_en: Option<Vec<String>>,
    pub character: Character,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedAnime {
    #[serde(deserialize_with = "deser_opt_id")]
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedManga {
    #[serde(deserialize_with = "deser_opt_id")]
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Related {
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,
    pub anime: Option<RelatedAnime>,
    pub manga: Option<RelatedManga>,
    #[serde(rename = "relationKind")]
    pub relation_kind: String,
    #[serde(rename = "relationText")]
    pub relation_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,
    pub url: Option<String>,
    pub name: Option<String>,
    pub kind: Option<String>,
    #[serde(rename = "playerUrl")]
    pub player_url: Option<String>,
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,
    #[serde(rename = "originalUrl")]
    pub original_url: Option<String>,
    #[serde(rename = "x166Url")]
    pub x166_url: Option<String>,
    #[serde(rename = "x332Url")]
    pub x332_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreStat {
    pub score: i32,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusStat {
    pub status: String,
    pub count: i32,
}

/// Полная информация об аниме.
///
/// Содержит все доступные данные об аниме: названия, оценки, студии, жанры,
/// персонажи, связанные произведения, видео, скриншоты и многое другое.
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::{ShikicrateClient, queries::*};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ShikicrateClient::new()?;
/// let animes = client.animes(AnimeSearchParams {
///     search: Some("naruto".to_string()),
///     limit: Some(1),
///     kind: None,
/// }).await?;
///
/// if let Some(anime) = animes.first() {
///     println!("Название: {}", anime.name);
///     if let Some(russian) = &anime.russian {
///         println!("Русское название: {}", russian);
///     }
///     if let Some(score) = anime.score {
///         println!("Оценка: {:.2}", score);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anime {
    /// ID аниме в системе Shikimori.
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,

    /// ID аниме в MyAnimeList (если есть).
    #[serde(rename = "malId", default, deserialize_with = "deser_opt_id")]
    pub mal_id: Option<i64>,

    /// Основное название аниме.
    pub name: String,

    /// Русское название (если есть).
    pub russian: Option<String>,

    /// Лицензионное русское название (если есть).
    #[serde(rename = "licenseNameRu")]
    pub license_name_ru: Option<String>,

    /// Английское название (если есть).
    pub english: Option<String>,

    /// Японское название (если есть).
    pub japanese: Option<String>,

    /// Синонимы и альтернативные названия.
    pub synonyms: Option<Vec<String>>,

    /// Тип аниме: `"tv"`, `"movie"`, `"ova"`, `"ona"`, `"special"`, `"music"`.
    pub kind: Option<String>,

    /// Возрастной рейтинг: `"g"`, `"pg"`, `"pg_13"`, `"r"`, `"r_plus"`, `"rx"`.
    pub rating: Option<String>,

    /// Средняя оценка пользователей (0.0 - 10.0).
    pub score: Option<f64>,

    /// Статус: `"anons"`, `"ongoing"`, `"released"`.
    pub status: Option<String>,

    /// Общее количество эпизодов (планируемое).
    pub episodes: Option<i32>,

    /// Количество вышедших эпизодов.
    #[serde(rename = "episodesAired")]
    pub episodes_aired: Option<i32>,

    /// Длительность одного эпизода в минутах.
    pub duration: Option<i32>,

    /// Дата начала показа.
    #[serde(rename = "airedOn")]
    pub aired_on: Option<Date>,

    /// Дата релиза.
    #[serde(rename = "releasedOn")]
    pub released_on: Option<Date>,

    /// URL страницы аниме на Shikimori.
    pub url: Option<String>,

    /// Сезон выхода: `"winter"`, `"spring"`, `"summer"`, `"fall"`.
    pub season: Option<String>,

    /// Постер аниме.
    pub poster: Option<Poster>,

    /// Список фансабберов (если есть).
    pub fansubbers: Option<Vec<String>>,

    /// Список фандабберов (если есть).
    pub fandubbers: Option<Vec<String>>,

    /// Список лицензиатов.
    pub licensors: Option<Vec<String>>,

    /// Дата создания записи в системе.
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,

    /// Дата последнего обновления.
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,

    /// Дата выхода следующего эпизода (для онгоингов).
    #[serde(rename = "nextEpisodeAt")]
    pub next_episode_at: Option<String>,

    /// Флаг цензуры.
    #[serde(rename = "isCensored")]
    pub is_censored: Option<bool>,

    /// Список жанров.
    pub genres: Option<Vec<Genre>>,

    /// Список студий.
    pub studios: Option<Vec<Studio>>,

    /// Внешние ссылки (официальные сайты, соцсети и т.д.).
    #[serde(rename = "externalLinks")]
    pub external_links: Option<Vec<ExternalLink>>,

    /// Роли людей (сейю, режиссеры, композиторы и т.д.).
    #[serde(rename = "personRoles")]
    pub person_roles: Option<Vec<PersonRole>>,

    /// Роли персонажей.
    #[serde(rename = "characterRoles")]
    pub character_roles: Option<Vec<CharacterRole>>,

    /// Связанные произведения (сиквелы, приквелы, спин-оффы и т.д.).
    pub related: Option<Vec<Related>>,

    /// Видео (трейлеры, опенинги, эндинги).
    pub videos: Option<Vec<Video>>,

    /// Скриншоты из аниме.
    pub screenshots: Option<Vec<Screenshot>>,

    /// Статистика оценок (распределение по баллам).
    #[serde(rename = "scoresStats")]
    pub scores_stats: Option<Vec<ScoreStat>>,

    /// Статистика статусов просмотра (сколько пользователей смотрит, дропнуло и т.д.).
    #[serde(rename = "statusesStats")]
    pub statuses_stats: Option<Vec<StatusStat>>,

    /// Описание аниме (текст).
    pub description: Option<String>,

    /// Описание аниме (HTML).
    #[serde(rename = "descriptionHtml")]
    pub description_html: Option<String>,

    /// Источник описания.
    #[serde(rename = "descriptionSource")]
    pub description_source: Option<String>,
}

/// Полная информация о манге.
///
/// Содержит все доступные данные о манге: названия, оценки, издательства, жанры,
/// персонажей, связанные произведения и многое другое.
///
/// Структура похожа на `Anime`, но содержит специфичные для манги поля
/// (например, `volumes`, `chapters`, `publishers` вместо `studios`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manga {
    /// ID манги в системе Shikimori.
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,

    /// ID манги в MyAnimeList (если есть).
    #[serde(rename = "malId", default, deserialize_with = "deser_opt_id")]
    pub mal_id: Option<i64>,

    /// Основное название манги.
    pub name: String,

    /// Русское название (если есть).
    pub russian: Option<String>,

    /// Лицензионное русское название (если есть).
    #[serde(rename = "licenseNameRu")]
    pub license_name_ru: Option<String>,

    /// Английское название (если есть).
    pub english: Option<String>,

    /// Японское название (если есть).
    pub japanese: Option<String>,

    /// Синонимы и альтернативные названия.
    pub synonyms: Option<Vec<String>>,

    /// Тип манги: `"manga"`, `"novel"`, `"one_shot"`, `"doujin"`, `"manhwa"`, `"manhua"`.
    pub kind: Option<String>,

    /// Средняя оценка пользователей (0.0 - 10.0).
    pub score: Option<f64>,

    /// Статус: `"anons"`, `"ongoing"`, `"released"`.
    pub status: Option<String>,

    /// Количество томов (планируемое).
    pub volumes: Option<i32>,

    /// Количество глав (планируемое).
    pub chapters: Option<i32>,

    /// Дата начала публикации.
    #[serde(rename = "airedOn")]
    pub aired_on: Option<Date>,

    /// Дата релиза.
    #[serde(rename = "releasedOn")]
    pub released_on: Option<Date>,

    /// URL страницы манги на Shikimori.
    pub url: Option<String>,

    /// Постер манги.
    pub poster: Option<Poster>,

    /// Список лицензиатов.
    pub licensors: Option<Vec<String>>,

    /// Дата создания записи в системе.
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,

    /// Дата последнего обновления.
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,

    /// Флаг цензуры.
    #[serde(rename = "isCensored")]
    pub is_censored: Option<bool>,

    /// Список жанров.
    pub genres: Option<Vec<Genre>>,

    /// Список издательств.
    pub publishers: Option<Vec<Publisher>>,

    /// Внешние ссылки (официальные сайты, соцсети и т.д.).
    #[serde(rename = "externalLinks")]
    pub external_links: Option<Vec<ExternalLink>>,

    /// Роли людей (авторы, иллюстраторы и т.д.).
    #[serde(rename = "personRoles")]
    pub person_roles: Option<Vec<PersonRole>>,

    /// Роли персонажей.
    #[serde(rename = "characterRoles")]
    pub character_roles: Option<Vec<CharacterRole>>,

    /// Связанные произведения (сиквелы, приквелы, спин-оффы и т.д.).
    pub related: Option<Vec<Related>>,

    /// Статистика оценок (распределение по баллам).
    #[serde(rename = "scoresStats")]
    pub scores_stats: Option<Vec<ScoreStat>>,

    /// Статистика статусов чтения (сколько пользователей читает, дропнуло и т.д.).
    #[serde(rename = "statusesStats")]
    pub statuses_stats: Option<Vec<StatusStat>>,

    /// Описание манги (текст).
    pub description: Option<String>,

    /// Описание манги (HTML).
    #[serde(rename = "descriptionHtml")]
    pub description_html: Option<String>,

    /// Источник описания.
    #[serde(rename = "descriptionSource")]
    pub description_source: Option<String>,
}

/// Полная информация о персонаже.
///
/// Содержит все доступные данные о персонаже: имена, описания, постеры,
/// флаги участия в аниме/манге/ранобэ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterFull {
    /// ID персонажа в системе Shikimori.
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,

    /// ID персонажа в MyAnimeList (если есть).
    #[serde(rename = "malId", default, deserialize_with = "deser_opt_id")]
    pub mal_id: Option<i64>,

    /// Основное имя персонажа.
    pub name: String,

    /// Русское имя (если есть).
    pub russian: Option<String>,

    /// Японское имя (если есть).
    pub japanese: Option<String>,

    /// Синонимы и альтернативные имена.
    pub synonyms: Option<Vec<String>>,

    /// URL страницы персонажа на Shikimori.
    pub url: Option<String>,

    /// Дата создания записи в системе.
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,

    /// Дата последнего обновления.
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,

    /// Флаг участия в аниме.
    #[serde(rename = "isAnime")]
    pub is_anime: Option<bool>,

    /// Флаг участия в манге.
    #[serde(rename = "isManga")]
    pub is_manga: Option<bool>,

    /// Флаг участия в ранобэ.
    #[serde(rename = "isRanobe")]
    pub is_ranobe: Option<bool>,

    /// Постер персонажа.
    pub poster: Option<Poster>,

    /// Описание персонажа (текст).
    pub description: Option<String>,

    /// Описание персонажа (HTML).
    #[serde(rename = "descriptionHtml")]
    pub description_html: Option<String>,

    /// Источник описания.
    #[serde(rename = "descriptionSource")]
    pub description_source: Option<String>,
}

/// Полная информация о человеке (сейю, мангака, продюсер и т.д.).
///
/// Содержит все доступные данные о человеке: имена, даты рождения/смерти,
/// роли (сейю, мангака, продюсер), постеры и другую информацию.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonFull {
    /// ID человека в системе Shikimori.
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,

    /// ID человека в MyAnimeList (если есть).
    #[serde(rename = "malId", default, deserialize_with = "deser_opt_id")]
    pub mal_id: Option<i64>,

    /// Основное имя человека.
    pub name: String,

    /// Русское имя (если есть).
    pub russian: Option<String>,

    /// Японское имя (если есть).
    pub japanese: Option<String>,

    /// Синонимы и альтернативные имена.
    pub synonyms: Option<Vec<String>>,

    /// URL страницы человека на Shikimori.
    pub url: Option<String>,

    /// Флаг: является ли сейю.
    #[serde(rename = "isSeyu")]
    pub is_seyu: Option<bool>,

    /// Флаг: является ли мангакой.
    #[serde(rename = "isMangaka")]
    pub is_mangaka: Option<bool>,

    /// Флаг: является ли продюсером.
    #[serde(rename = "isProducer")]
    pub is_producer: Option<bool>,

    /// Официальный сайт человека (если есть).
    pub website: Option<String>,

    /// Дата создания записи в системе.
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,

    /// Дата последнего обновления.
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,

    /// Дата рождения.
    #[serde(rename = "birthOn")]
    pub birth_on: Option<Date>,

    /// Дата смерти (если есть).
    #[serde(rename = "deceasedOn")]
    pub deceased_on: Option<Date>,

    /// Постер человека.
    pub poster: Option<Poster>,
}

/// Пользовательская оценка аниме или манги.
///
/// Содержит информацию об оценке пользователя и ссылку на оцениваемое произведение.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRate {
    /// ID оценки в системе Shikimori.
    #[serde(deserialize_with = "deser_id")]
    pub id: i64,

    /// Аниме (если оценка относится к аниме).
    pub anime: Option<Anime>,

    /// Манга (если оценка относится к манге).
    pub manga: Option<Manga>,

    /// Дата создания оценки.
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}
