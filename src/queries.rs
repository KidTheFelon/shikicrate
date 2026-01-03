use crate::client::ShikicrateClient;
use crate::error::{Result, ShikicrateError};
use crate::types::*;
use serde_json::json;

const ANIMES_QUERY: &str = r#"
  query SearchAnimes($search: String, $limit: Int, $kind: AnimeKindString) {
    animes(search: $search, limit: $limit, kind: $kind) {
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
  query SearchMangas($search: String, $limit: Int, $kind: MangaKindString) {
    mangas(search: $search, limit: $limit, kind: $kind) {
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

/// Параметры поиска аниме.
///
/// Все поля опциональны. Если поле не указано, используется значение по умолчанию API.
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::queries::AnimeSearchParams;
///
/// // Поиск по названию
/// let params = AnimeSearchParams {
///     search: Some("naruto".to_string()),
///     limit: Some(10),
///     kind: None,
/// };
///
/// // Поиск с фильтром по типу (исключить спешлы)
/// let params = AnimeSearchParams {
///     search: Some("bakemono".to_string()),
///     limit: Some(5),
///     kind: Some("!special".to_string()),
/// };
/// ```
pub struct AnimeSearchParams {
    /// Поисковый запрос (название аниме).
    ///
    /// Ищет по названию на русском, английском и японском языках.
    pub search: Option<String>,

    /// Максимальное количество результатов.
    ///
    /// Должно быть больше 0. Если не указано, используется значение по умолчанию API.
    pub limit: Option<i32>,

    /// Фильтр по типу аниме.
    ///
    /// Поддерживаемые значения: `"tv"`, `"movie"`, `"ova"`, `"ona"`, `"special"`, `"music"`.
    /// Можно использовать префикс `!` для исключения типа (например, `"!special"`).
    pub kind: Option<String>,
}

/// Параметры поиска манги.
///
/// Все поля опциональны. Если поле не указано, используется значение по умолчанию API.
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::queries::MangaSearchParams;
///
/// // Поиск по названию
/// let params = MangaSearchParams {
///     search: Some("one piece".to_string()),
///     limit: Some(5),
///     kind: None,
/// };
/// ```
pub struct MangaSearchParams {
    /// Максимальное количество результатов.
    ///
    /// Должно быть больше 0. Если не указано, используется значение по умолчанию API.
    pub limit: Option<i32>,

    /// Поисковый запрос (название манги).
    ///
    /// Ищет по названию на русском, английском и японском языках.
    pub search: Option<String>,

    /// Фильтр по типу манги.
    ///
    /// Поддерживаемые значения: `"manga"`, `"novel"`, `"one_shot"`, `"doujin"`, `"manhwa"`, `"manhua"`.
    pub kind: Option<String>,
}

/// Параметры поиска людей (сейю, мангаки, продюсеры и т.д.).
///
/// Все поля опциональны. Если поле не указано, используется значение по умолчанию API.
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::queries::PeopleSearchParams;
///
/// // Поиск по имени
/// let params = PeopleSearchParams {
///     search: Some("miyazaki".to_string()),
///     limit: Some(10),
/// };
/// ```
pub struct PeopleSearchParams {
    /// Максимальное количество результатов.
    ///
    /// Должно быть больше 0. Если не указано, используется значение по умолчанию API.
    pub limit: Option<i32>,

    /// Поисковый запрос (имя человека).
    ///
    /// Ищет по имени на русском, английском и японском языках.
    pub search: Option<String>,
}

/// Параметры поиска персонажей.
///
/// Поддерживает два режима поиска:
/// - По странице и лимиту (если указаны `page` и/или `limit`)
/// - По списку ID (если указан `ids`)
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::queries::CharacterSearchParams;
///
/// // Поиск по странице
/// let params = CharacterSearchParams {
///     page: Some(1),
///     limit: Some(20),
///     ids: None,
/// };
///
/// // Поиск по ID
/// let params = CharacterSearchParams {
///     page: None,
///     limit: None,
///     ids: Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
/// };
/// ```
pub struct CharacterSearchParams {
    /// Номер страницы (начиная с 1).
    ///
    /// Должно быть >= 1. Используется только если `ids` не указан.
    pub page: Option<i32>,

    /// Максимальное количество результатов на странице.
    ///
    /// Должно быть больше 0. Используется только если `ids` не указан.
    pub limit: Option<i32>,

    /// Список ID персонажей для получения.
    ///
    /// Если указан, поиск выполняется по ID, игнорируя `page` и `limit`.
    /// Вектор не должен быть пустым.
    pub ids: Option<Vec<String>>,
}

/// Параметры поиска пользовательских оценок.
///
/// Все поля опциональны. Если поле не указано, используется значение по умолчанию API.
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::queries::UserRateSearchParams;
///
/// // Поиск оценок аниме, отсортированных по дате обновления
/// let params = UserRateSearchParams {
///     page: Some(1),
///     limit: Some(20),
///     target_type: Some("Anime".to_string()),
///     order_field: Some("updated_at".to_string()),
///     order: Some("desc".to_string()),
/// };
/// ```
pub struct UserRateSearchParams {
    /// Номер страницы (начиная с 1).
    ///
    /// Должно быть >= 1.
    pub page: Option<i32>,

    /// Максимальное количество результатов на странице.
    ///
    /// Должно быть больше 0.
    pub limit: Option<i32>,

    /// Тип контента для фильтрации.
    ///
    /// Поддерживаемые значения: `"Anime"`, `"Manga"`.
    pub target_type: Option<String>,

    /// Поле для сортировки.
    ///
    /// Поддерживаемые значения: `"updated_at"`, `"created_at"`, `"score"` и другие.
    pub order_field: Option<String>,

    /// Направление сортировки.
    ///
    /// Поддерживаемые значения: `"asc"`, `"desc"`.
    pub order: Option<String>,
}

impl ShikicrateClient {
    fn val_lim(limit: Option<i32>) -> Result<()> {
        if let Some(limit) = limit {
            if limit <= 0 {
                return Err(ShikicrateError::Validation(
                    "limit must be greater than 0".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn val_pg(page: Option<i32>) -> Result<()> {
        if let Some(page) = page {
            if page < 1 {
                return Err(ShikicrateError::Validation(
                    "page must be greater than or equal to 1".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn val_ids(ids: Option<&Vec<String>>) -> Result<()> {
        if let Some(ids) = ids {
            if ids.is_empty() {
                return Err(ShikicrateError::Validation(
                    "ids must not be empty".to_string(),
                ));
            }
        }
        Ok(())
    }

    async fn fetch<T, F>(
        &self,
        query: String,
        build_variables: F,
        response_key: &str,
    ) -> Result<Vec<T>>
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

    fn build_vars(
        search: Option<String>,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> serde_json::Value {
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

    /// Выполняет поиск аниме по заданным параметрам.
    ///
    /// Возвращает список аниме, соответствующих критериям поиска.
    /// Результаты включают полную информацию: названия, оценки, студии, жанры,
    /// персонажей, связанные произведения и многое другое.
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры поиска (`AnimeSearchParams`)
    ///
    /// # Возвращает
    ///
    /// `Result<Vec<Anime>>` - вектор найденных аниме или ошибка.
    ///
    /// # Ошибки
    ///
    /// - `ShikicrateError::Validation` - если `limit <= 0`
    /// - `ShikicrateError::Http` - ошибка сети или таймаут
    /// - `ShikicrateError::GraphQL` - ошибка GraphQL запроса
    /// - `ShikicrateError::Api` - неуспешный HTTP статус
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::{ShikicrateClient, queries::*};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClient::new()?;
    ///
    /// // Поиск аниме по названию
    /// let animes = client.animes(AnimeSearchParams {
    ///     search: Some("naruto".to_string()),
    ///     limit: Some(10),
    ///     kind: None,
    /// }).await?;
    ///
    /// for anime in animes {
    ///     println!("{} (ID: {})", anime.name, anime.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn animes(&self, params: AnimeSearchParams) -> Result<Vec<Anime>> {
        Self::val_lim(params.limit)?;

        self.fetch(
            ANIMES_QUERY.to_string(),
            || {
                let mut vars = Self::build_vars(params.search.clone(), None, params.limit);
                if let Some(kind) = &params.kind {
                    vars["kind"] = json!(kind);
                }
                vars
            },
            "animes",
        )
        .await
    }

    /// Выполняет поиск манги по заданным параметрам.
    ///
    /// Возвращает список манги, соответствующих критериям поиска.
    /// Результаты включают полную информацию: названия, оценки, издательства, жанры,
    /// персонажей, связанные произведения и многое другое.
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры поиска (`MangaSearchParams`)
    ///
    /// # Возвращает
    ///
    /// `Result<Vec<Manga>>` - вектор найденной манги или ошибка.
    ///
    /// # Ошибки
    ///
    /// - `ShikicrateError::Validation` - если `limit <= 0`
    /// - `ShikicrateError::Http` - ошибка сети или таймаут
    /// - `ShikicrateError::GraphQL` - ошибка GraphQL запроса
    /// - `ShikicrateError::Api` - неуспешный HTTP статус
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::{ShikicrateClient, queries::*};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClient::new()?;
    ///
    /// // Поиск манги по названию
    /// let mangas = client.mangas(MangaSearchParams {
    ///     search: Some("one piece".to_string()),
    ///     limit: Some(5),
    ///     kind: None,
    /// }).await?;
    ///
    /// for manga in mangas {
    ///     println!("{} (ID: {})", manga.name, manga.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mangas(&self, params: MangaSearchParams) -> Result<Vec<Manga>> {
        Self::val_lim(params.limit)?;

        let mut query = MANGAS_QUERY.to_string();
        let mut variables = Self::build_vars(params.search.clone(), None, params.limit);

        // Если kind не указан, нужно убрать его из запроса
        if params.kind.is_none() {
            query = query.replace("$kind: MangaKindString", "");
            query = query.replace(", kind: $kind", "");
        } else {
            variables["kind"] = json!(params.kind);
        }

        self.fetch(query, || variables.clone(), "mangas").await
    }

    /// Выполняет поиск людей (сейю, мангаки, продюсеры и т.д.) по заданным параметрам.
    ///
    /// Возвращает список людей, соответствующих критериям поиска.
    /// Результаты включают полную информацию: имена, даты рождения/смерти,
    /// роли (сейю, мангака, продюсер), постеры и другую информацию.
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры поиска (`PeopleSearchParams`)
    ///
    /// # Возвращает
    ///
    /// `Result<Vec<PersonFull>>` - вектор найденных людей или ошибка.
    ///
    /// # Ошибки
    ///
    /// - `ShikicrateError::Validation` - если `limit <= 0`
    /// - `ShikicrateError::Http` - ошибка сети или таймаут
    /// - `ShikicrateError::GraphQL` - ошибка GraphQL запроса
    /// - `ShikicrateError::Api` - неуспешный HTTP статус
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::{ShikicrateClient, queries::*};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClient::new()?;
    ///
    /// // Поиск людей по имени
    /// let people = client.people(PeopleSearchParams {
    ///     search: Some("miyazaki".to_string()),
    ///     limit: Some(10),
    /// }).await?;
    ///
    /// for person in people {
    ///     println!("{} (ID: {})", person.name, person.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn people(&self, params: PeopleSearchParams) -> Result<Vec<PersonFull>> {
        Self::val_lim(params.limit)?;

        self.fetch(
            PEOPLE_QUERY.to_string(),
            || Self::build_vars(params.search.clone(), None, params.limit),
            "people",
        )
        .await
    }

    /// Выполняет поиск персонажей по заданным параметрам.
    ///
    /// Поддерживает два режима поиска:
    /// - По странице и лимиту (если указаны `page` и/или `limit`)
    /// - По списку ID (если указан `ids`)
    ///
    /// Возвращает список персонажей с полной информацией: имена, описания,
    /// постеры, флаги участия в аниме/манге/ранобэ.
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры поиска (`CharacterSearchParams`)
    ///
    /// # Возвращает
    ///
    /// `Result<Vec<CharacterFull>>` - вектор найденных персонажей или ошибка.
    ///
    /// # Ошибки
    ///
    /// - `ShikicrateError::Validation` - если `page < 1`, `limit <= 0` или `ids` пустой
    /// - `ShikicrateError::Http` - ошибка сети или таймаут
    /// - `ShikicrateError::GraphQL` - ошибка GraphQL запроса
    /// - `ShikicrateError::Api` - неуспешный HTTP статус
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::{ShikicrateClient, queries::*};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClient::new()?;
    ///
    /// // Поиск по странице
    /// let characters = client.characters(CharacterSearchParams {
    ///     page: Some(1),
    ///     limit: Some(20),
    ///     ids: None,
    /// }).await?;
    ///
    /// // Поиск по ID
    /// let characters = client.characters(CharacterSearchParams {
    ///     page: None,
    ///     limit: None,
    ///     ids: Some(vec!["1".to_string(), "2".to_string()]),
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn characters(&self, params: CharacterSearchParams) -> Result<Vec<CharacterFull>> {
        Self::val_pg(params.page)?;
        Self::val_lim(params.limit)?;
        Self::val_ids(params.ids.as_ref())?;

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

    /// Выполняет поиск пользовательских оценок по заданным параметрам.
    ///
    /// Возвращает список оценок пользователя с информацией об аниме или манге.
    /// Поддерживает фильтрацию по типу контента и сортировку.
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры поиска (`UserRateSearchParams`)
    ///
    /// # Возвращает
    ///
    /// `Result<Vec<UserRate>>` - вектор найденных оценок или ошибка.
    ///
    /// # Ошибки
    ///
    /// - `ShikicrateError::Validation` - если `page < 1` или `limit <= 0`
    /// - `ShikicrateError::Http` - ошибка сети или таймаут
    /// - `ShikicrateError::GraphQL` - ошибка GraphQL запроса
    /// - `ShikicrateError::Api` - неуспешный HTTP статус
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::{ShikicrateClient, queries::*};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClient::new()?;
    ///
    /// // Поиск оценок аниме, отсортированных по дате обновления
    /// let user_rates = client.user_rates(UserRateSearchParams {
    ///     page: Some(1),
    ///     limit: Some(20),
    ///     target_type: Some("Anime".to_string()),
    ///     order_field: Some("updated_at".to_string()),
    ///     order: Some("desc".to_string()),
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
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
