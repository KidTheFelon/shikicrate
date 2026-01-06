use crate::client::ShikicrateClient;
use crate::error::{Result, ShikicrateError};
use crate::types::*;
use serde_json::json;

const ANIMES_QUERY: &str = r#"
  query SearchAnimes($search: String, $ids: String, $limit: Int, $page: Int, $kind: AnimeKindString) {
    animes(search: $search, ids: $ids, limit: $limit, page: $page, kind: $kind) {
      id
      malId
      name
      russian
      licenseNameRu
      english
      japanese
      synonyms
      kind
      rating
      score
      status
      episodes
      episodesAired
      duration
      airedOn {
        year
        month
        day
        date
      }
      releasedOn {
        year
        month
        day
        date
      }
      url
      season
      poster {
        id
        originalUrl
        mainUrl
      }
      fansubbers
      fandubbers
      licensors
      createdAt
      updatedAt
      nextEpisodeAt
      isCensored
      genres {
        id
        name
        russian
        kind
      }
      studios {
        id
        name
        imageUrl
      }
      externalLinks {
        id
        kind
        url
        createdAt
        updatedAt
      }
      personRoles {
        id
        rolesRu
        rolesEn
        person {
          id
          name
          poster {
            id
          }
        }
      }
      characterRoles {
        id
        rolesRu
        rolesEn
        character {
          id
          name
          poster {
            id
          }
        }
      }
      related {
        id
        anime {
          id
          name
        }
        manga {
          id
          name
        }
        relationKind
        relationText
      }
      videos {
        id
        url
        name
        kind
        playerUrl
        imageUrl
      }
      screenshots {
        id
        originalUrl
        x166Url
        x332Url
      }
      scoresStats {
        score
        count
      }
      statusesStats {
        status
        count
      }
      description
      descriptionHtml
      descriptionSource
    }
  }
"#;

const MANGAS_QUERY: &str = r#"
  query SearchMangas($search: String, $ids: String, $limit: Int, $page: Int) {
    mangas(search: $search, ids: $ids, limit: $limit, page: $page) {
      id
      malId
      name
      russian
      licenseNameRu
      english
      japanese
      synonyms
      kind
      score
      status
      volumes
      chapters
      airedOn {
        year
        month
        day
        date
      }
      releasedOn {
        year
        month
        day
        date
      }
      url
      poster {
        id
        originalUrl
        mainUrl
      }
      licensors
      createdAt
      updatedAt
      isCensored
      genres {
        id
        name
        russian
        kind
      }
      publishers {
        id
        name
      }
      externalLinks {
        id
        kind
        url
        createdAt
        updatedAt
      }
      personRoles {
        id
        rolesRu
        rolesEn
        person {
          id
          name
          poster {
            id
          }
        }
      }
      characterRoles {
        id
        rolesRu
        rolesEn
        character {
          id
          name
          poster {
            id
          }
        }
      }
      related {
        id
        anime {
          id
          name
        }
        manga {
          id
          name
        }
        relationKind
        relationText
      }
      scoresStats {
        score
        count
      }
      statusesStats {
        status
        count
      }
      description
      descriptionHtml
      descriptionSource
    }
  }
"#;

const MANGAS_WITH_KIND_QUERY: &str = r#"
  query SearchMangas($search: String, $ids: String, $limit: Int, $page: Int, $kind: MangaKindString) {
    mangas(search: $search, ids: $ids, limit: $limit, page: $page, kind: $kind) {
      id
      malId
      name
      russian
      licenseNameRu
      english
      japanese
      synonyms
      kind
      score
      status
      volumes
      chapters
      airedOn {
        year
        month
        day
        date
      }
      releasedOn {
        year
        month
        day
        date
      }
      url
      poster {
        id
        originalUrl
        mainUrl
      }
      licensors
      createdAt
      updatedAt
      isCensored
      genres {
        id
        name
        russian
        kind
      }
      publishers {
        id
        name
      }
      externalLinks {
        id
        kind
        url
        createdAt
        updatedAt
      }
      personRoles {
        id
        rolesRu
        rolesEn
        person {
          id
          name
          poster {
            id
          }
        }
      }
      characterRoles {
        id
        rolesRu
        rolesEn
        character {
          id
          name
          poster {
            id
          }
        }
      }
      related {
        id
        anime {
          id
          name
        }
        manga {
          id
          name
        }
        relationKind
        relationText
      }
      scoresStats {
        score
        count
      }
      statusesStats {
        status
        count
      }
      description
      descriptionHtml
      descriptionSource
    }
  }
"#;

const PEOPLE_QUERY: &str = r#"
  query SearchPeople($search: String, $limit: Int) {
    people(search: $search, limit: $limit) {
      id
      malId
      name
      russian
      japanese
      synonyms
      url
      isSeyu
      isMangaka
      isProducer
      website
      createdAt
      updatedAt
      birthOn {
        year
        month
        day
        date
      }
      deceasedOn {
        year
        month
        day
        date
      }
      poster {
        id
        originalUrl
        mainUrl
      }
    }
  }
"#;

const CHARACTERS_QUERY: &str = r#"
  query SearchCharacters($page: Int, $limit: Int) {
    characters(page: $page, limit: $limit) {
      id
      malId
      name
      russian
      japanese
      synonyms
      url
      createdAt
      updatedAt
      isAnime
      isManga
      isRanobe
      poster {
        id
        originalUrl
        mainUrl
      }
      description
      descriptionHtml
      descriptionSource
    }
  }
"#;

const CHARACTERS_BY_IDS_QUERY: &str = r#"
  query GetCharactersByIds($ids: [ID!]) {
    characters(ids: $ids) {
      id
      name
    }
  }
"#;

const USER_RATES_QUERY: &str = r#"
  query SearchUserRates($page: Int, $limit: Int, $targetType: TargetType, $order: UserRateOrder) {
    userRates(page: $page, limit: $limit, targetType: $targetType, order: $order) {
      id
      anime {
        id
        name
      }
      manga {
        id
        name
      }
      createdAt
    }
  }
"#;

#[derive(Clone)]
pub struct AnimeSearchParams {
    pub search: Option<String>,
    pub ids: Option<String>,
    pub limit: Option<i32>,
    pub kind: Option<String>,
    pub page: Option<i32>,
}

#[derive(Clone)]
pub struct MangaSearchParams {
    pub limit: Option<i32>,
    pub search: Option<String>,
    pub ids: Option<String>,
    pub kind: Option<String>,
    pub page: Option<i32>,
}

pub struct PeopleSearchParams {
    pub limit: Option<i32>,
    pub search: Option<String>,
}

#[derive(Clone)]
pub struct CharacterSearchParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub ids: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct UserRateSearchParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub target_type: Option<String>,
    pub order_field: Option<String>,
    pub order: Option<String>,
}

impl ShikicrateClient {
    fn val_lim(limit: Option<i32>) -> Result<()> {
        if let Some(limit) = limit {
            if limit <= 0 {
                return Err(ShikicrateError::Validation("limit must be greater than 0".to_string()));
            }
        }
        Ok(())
    }

    fn val_pg(page: Option<i32>) -> Result<()> {
        if let Some(page) = page {
            if page < 1 {
                return Err(ShikicrateError::Validation("page must be greater than or equal to 1".to_string()));
            }
        }
        Ok(())
    }

    fn val_ids(ids: Option<&Vec<String>>) -> Result<()> {
        if let Some(ids) = ids {
            if ids.is_empty() {
                return Err(ShikicrateError::Validation("ids must not be empty".to_string()));
            }
        }
        Ok(())
    }

    async fn fetch<T, F>(&self, query: String, build_variables: F, response_key: &str) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
        F: FnOnce() -> serde_json::Value,
    {
        let variables = build_variables();
        let response: serde_json::Value = self.execute_query(&query, Some(variables)).await?;

        let items = response
            .get(response_key)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        serde_json::from_value(json!(items)).map_err(crate::error::ShikicrateError::Serialization)
    }

    fn build_vars(search: Option<String>, page: Option<i32>, limit: Option<i32>) -> serde_json::Value {
        let mut variables = json!({});
        if let Some(search) = search {
            variables["search"] = json!(search);
        }
        if let Some(page) = page {
            variables["page"] = json!(page);
        }
        if let Some(limit) = limit {
            variables["limit"] = json!(limit);
        }
        variables
    }

    pub async fn animes(&self, params: AnimeSearchParams) -> Result<Vec<Anime>> {
        Self::val_lim(params.limit)?;
        Self::val_pg(params.page)?;

        self.fetch(
            ANIMES_QUERY.to_string(),
            || {
                let mut vars = Self::build_vars(params.search.clone(), params.page, params.limit);
                if let Some(kind) = &params.kind {
                    vars["kind"] = json!(kind);
                }
                if let Some(ids) = &params.ids {
                    vars["ids"] = json!(ids);
                }
                vars
            },
            "animes",
        )
        .await
    }

    pub async fn mangas(&self, params: MangaSearchParams) -> Result<Vec<Manga>> {
        Self::val_lim(params.limit)?;
        Self::val_pg(params.page)?;

        let (query, mut variables) = if let Some(kind) = params.kind {
            let mut vars = Self::build_vars(params.search.clone(), params.page, params.limit);
            vars["kind"] = json!(kind);
            (MANGAS_WITH_KIND_QUERY.to_string(), vars)
        } else {
            (
                MANGAS_QUERY.to_string(),
                Self::build_vars(params.search.clone(), params.page, params.limit),
            )
        };

        if let Some(ids) = params.ids {
            variables["ids"] = json!(ids);
        }

        self.fetch(query, || variables.clone(), "mangas").await
    }

    pub async fn people(&self, params: PeopleSearchParams) -> Result<Vec<PersonFull>> {
        Self::val_lim(params.limit)?;

        self.fetch(
            PEOPLE_QUERY.to_string(),
            || Self::build_vars(params.search.clone(), None, params.limit),
            "people",
        )
        .await
    }

    pub async fn characters(&self, params: CharacterSearchParams) -> Result<Vec<CharacterFull>> {
        if params.ids.is_some() {
            Self::val_ids(params.ids.as_ref())?;
        } else {
            Self::val_pg(params.page)?;
            Self::val_lim(params.limit)?;
        }

        let query = if params.ids.is_some() {
            CHARACTERS_BY_IDS_QUERY.to_string()
        } else {
            CHARACTERS_QUERY.to_string()
        };

        self.fetch(
            query,
            || {
                if let Some(ids) = params.ids {
                    json!({ "ids": ids })
                } else {
                    Self::build_vars(None, params.page, params.limit)
                }
            },
            "characters",
        )
        .await
    }

    pub async fn user_rates(&self, params: UserRateSearchParams) -> Result<Vec<UserRate>> {
        Self::val_pg(params.page)?;
        Self::val_lim(params.limit)?;

        self.fetch(
            USER_RATES_QUERY.to_string(),
            || {
                let mut variables = Self::build_vars(None, params.page, params.limit);
                if let Some(target_type) = params.target_type {
                    variables["targetType"] = json!(target_type);
                }
                if let Some(order_field) = params.order_field {
                    variables["order"] = json!({
                        "field": order_field,
                        "order": params.order.unwrap_or_else(|| "desc".to_string())
                    });
                }
                variables
            },
            "userRates",
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ShikicrateError;

    #[test]
    fn test_val_lim_valid() {
        assert!(ShikicrateClient::val_lim(None).is_ok());
        assert!(ShikicrateClient::val_lim(Some(1)).is_ok());
        assert!(ShikicrateClient::val_lim(Some(100)).is_ok());
    }

    #[test]
    fn test_val_lim_invalid() {
        assert!(matches!(
            ShikicrateClient::val_lim(Some(0)),
            Err(ShikicrateError::Validation(_))
        ));
        assert!(matches!(
            ShikicrateClient::val_lim(Some(-1)),
            Err(ShikicrateError::Validation(_))
        ));
    }

    #[test]
    fn test_val_pg_valid() {
        assert!(ShikicrateClient::val_pg(None).is_ok());
        assert!(ShikicrateClient::val_pg(Some(1)).is_ok());
        assert!(ShikicrateClient::val_pg(Some(100)).is_ok());
    }

    #[test]
    fn test_val_pg_invalid() {
        assert!(matches!(
            ShikicrateClient::val_pg(Some(0)),
            Err(ShikicrateError::Validation(_))
        ));
        assert!(matches!(
            ShikicrateClient::val_pg(Some(-1)),
            Err(ShikicrateError::Validation(_))
        ));
    }

    #[test]
    fn test_val_ids_valid() {
        assert!(ShikicrateClient::val_ids(None).is_ok());
        let ids = vec!["1".to_string(), "2".to_string()];
        assert!(ShikicrateClient::val_ids(Some(&ids)).is_ok());
    }

    #[test]
    fn test_val_ids_invalid() {
        let empty_ids = vec![];
        assert!(matches!(
            ShikicrateClient::val_ids(Some(&empty_ids)),
            Err(ShikicrateError::Validation(_))
        ));
    }

    #[test]
    fn test_build_vars() {
        let vars = ShikicrateClient::build_vars(None, None, None);
        assert!(vars.as_object().unwrap().is_empty());

        let vars = ShikicrateClient::build_vars(Some("test".to_string()), Some(2), Some(10));
        assert_eq!(vars["search"], "test");
        assert_eq!(vars["page"], 2);
        assert_eq!(vars["limit"], 10);
    }
}
