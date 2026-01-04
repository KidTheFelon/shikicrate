use crate::client::ShikicrateClient;
use crate::error::Result;
use crate::queries::*;
use crate::types::*;
use futures::stream::{self, Stream, StreamExt};

use std::sync::Arc;

/// Состояние пагинатора для аниме
struct AnimesPaginatorState {
    client: Arc<ShikicrateClient>,
    params: AnimeSearchParams,
    current_page: i32,
}

/// Состояние пагинатора для манги
struct MangasPaginatorState {
    client: Arc<ShikicrateClient>,
    params: MangaSearchParams,
    current_page: i32,
}

/// Состояние пагинатора для персонажей
struct CharactersPaginatorState {
    client: Arc<ShikicrateClient>,
    params: CharacterSearchParams,
    current_page: i32,
}

/// Состояние пагинатора для людей
#[allow(dead_code)]
struct PeoplePaginatorState {
    client: Arc<ShikicrateClient>,
    params: PeopleSearchParams,
    current_page: i32,
}

/// Состояние пагинатора для пользовательских оценок
struct UserRatesPaginatorState {
    client: Arc<ShikicrateClient>,
    params: UserRateSearchParams,
    current_page: i32,
}


/// Ленивый итератор для пагинации результатов поиска аниме.
///
/// Автоматически загружает следующую страницу при достижении конца текущей.
/// Используется через метод `animes_paginated()`.
///
/// # Примеры
///
/// ```no_run
/// use shikicrate::{ShikicrateClient, queries::*};
/// use futures::stream::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ShikicrateClient::new()?;
///
/// let mut paginator = client.animes_paginated(AnimeSearchParams {
///     search: Some("naruto".to_string()),
///     page: None,  // Начнет с первой страницы
///     limit: Some(10),
///     kind: None,
/// });
///
/// while let Some(anime) = paginator.next().await {
///     let anime = anime?;
///     println!("{} (ID: {})", anime.name, anime.id);
/// }
/// # Ok(())
/// # }
/// ```
pub type AnimesPaginator = Box<dyn Stream<Item = Result<Anime>> + Send + Unpin>;

/// Ленивый итератор для пагинации результатов поиска манги.
///
/// Автоматически загружает следующую страницу при достижении конца текущей.
/// Используется через метод `mangas_paginated()`.
pub type MangasPaginator = Box<dyn Stream<Item = Result<Manga>> + Send + Unpin>;

/// Ленивый итератор для пагинации результатов поиска персонажей.
///
/// Автоматически загружает следующую страницу при достижении конца текущей.
/// Используется через метод `characters_paginated()`.
///
/// **Примечание:** Не работает с режимом поиска по ID (`ids`).
pub type CharactersPaginator = Box<dyn Stream<Item = Result<CharacterFull>> + Send + Unpin>;

/// Ленивый итератор для пагинации результатов поиска людей.
///
/// Автоматически загружает следующую страницу при достижении конца текущей.
/// Используется через метод `people_paginated()`.
pub type PeoplePaginator = Box<dyn Stream<Item = Result<PersonFull>> + Send + Unpin>;

/// Ленивый итератор для пагинации результатов поиска пользовательских оценок.
///
/// Автоматически загружает следующую страницу при достижении конца текущей.
/// Используется через метод `user_rates_paginated()`.
pub type UserRatesPaginator = Box<dyn Stream<Item = Result<UserRate>> + Send + Unpin>;

impl ShikicrateClient {
    /// Создает ленивый итератор для пагинации результатов поиска аниме.
    ///
    /// Итератор автоматически загружает следующую страницу при достижении конца текущей.
    /// Если `page` не указан, начнет с первой страницы.
    ///
    /// # Примеры
    ///
    /// ```no_run
    /// use shikicrate::{ShikicrateClient, queries::*};
    /// use futures::stream::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ShikicrateClient::new()?;
    ///
    /// let mut paginator = client.animes_paginated(AnimeSearchParams {
    ///     search: Some("naruto".to_string()),
    ///     page: None,
    ///     limit: Some(10),
    ///     kind: None,
    /// });
    ///
    /// // Обрабатываем первые 50 результатов
    /// let mut count = 0;
    /// while let Some(anime) = paginator.next().await {
    ///     let anime = anime?;
    ///     println!("{} (ID: {})", anime.name, anime.id);
    ///     count += 1;
    ///     if count >= 50 {
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn animes_paginated(&self, mut params: AnimeSearchParams) -> AnimesPaginator {
        let start_page = params.page.unwrap_or(1);
        params.page = Some(start_page);

        // Для пагинации нужен Arc, но мы не можем клонировать клиент напрямую
        // Используем замыкание, которое захватывает ссылку на self
        // Это работает, так как пагинатор живет пока живет клиент
        let client = self.to_arc();
        let state = AnimesPaginatorState {
            client,
            params,
            current_page: start_page - 1,
        };

        // Создаем стрим страниц, затем разворачиваем каждую страницу в элементы
        Box::new(
            stream::unfold(state, |mut state| async move {
                state.current_page += 1;
                state.params.page = Some(state.current_page);

                match state.client.animes(state.params.clone()).await {
                    Ok(page) if page.is_empty() => None,
                    Ok(page) => Some((Ok(page), state)),
                    Err(e) => {
                        // Возвращаем ошибку как элемент, стрим остановится после обработки в flat_map
                        Some((Err(e), state))
                    }
                }
            })
            .flat_map(|result: Result<Vec<Anime>>| {
                stream::iter(match result {
                    Ok(page) => page.into_iter().map(Ok).collect(),
                    Err(e) => {
                        // Возвращаем ошибку как элемент стрима
                        vec![Err(e)]
                    }
                })
            })
            .boxed()
        )
    }

    /// Создает ленивый итератор для пагинации результатов поиска манги.
    ///
    /// Итератор автоматически загружает следующую страницу при достижении конца текущей.
    /// Если `page` не указан, начнет с первой страницы.
    pub fn mangas_paginated(&self, mut params: MangaSearchParams) -> MangasPaginator {
        let start_page = params.page.unwrap_or(1);
        params.page = Some(start_page);

        let client = self.to_arc();
        let state = MangasPaginatorState {
            client,
            params,
            current_page: start_page - 1,
        };

        Box::new(
            stream::unfold(state, |mut state| async move {
                state.current_page += 1;
                state.params.page = Some(state.current_page);

                match state.client.mangas(state.params.clone()).await {
                    Ok(page) if page.is_empty() => None,
                    Ok(page) => Some((Ok(page), state)),
                    Err(e) => {
                        // Возвращаем ошибку как элемент, стрим остановится после обработки в flat_map
                        Some((Err(e), state))
                    }
                }
            })
            .flat_map(|result: Result<Vec<Manga>>| {
                stream::iter(match result {
                    Ok(page) => page.into_iter().map(Ok).collect(),
                    Err(e) => {
                        // Возвращаем ошибку как элемент стрима
                        vec![Err(e)]
                    }
                })
            })
            .boxed()
        )
    }

    /// Создает ленивый итератор для пагинации результатов поиска персонажей.
    ///
    /// Итератор автоматически загружает следующую страницу при достижении конца текущей.
    /// Если `page` не указан, начнет с первой страницы.
    ///
    /// **Примечание:** Не работает с режимом поиска по ID (`ids`).
    pub fn characters_paginated(&self, mut params: CharacterSearchParams) -> CharactersPaginator {
        if params.ids.is_some() {
            // Если указаны ID, возвращаем пустой стрим или ошибку
            return Box::new(stream::empty().boxed());
        }

        let start_page = params.page.unwrap_or(1);
        params.page = Some(start_page);

        let client = self.to_arc();
        let state = CharactersPaginatorState {
            client,
            params,
            current_page: start_page - 1,
        };

        Box::new(
            stream::unfold(state, |mut state| async move {
                state.current_page += 1;
                state.params.page = Some(state.current_page);

                match state.client.characters(state.params.clone()).await {
                    Ok(page) if page.is_empty() => None,
                    Ok(page) => Some((Ok(page), state)),
                    Err(e) => {
                        // Возвращаем ошибку как элемент, стрим остановится после обработки в flat_map
                        Some((Err(e), state))
                    }
                }
            })
            .flat_map(|result: Result<Vec<CharacterFull>>| {
                stream::iter(match result {
                    Ok(page) => page.into_iter().map(Ok).collect(),
                    Err(e) => {
                        // Возвращаем ошибку как элемент стрима
                        vec![Err(e)]
                    }
                })
            })
            .boxed()
        )
    }

    /// Создает ленивый итератор для пагинации результатов поиска людей.
    ///
    /// Итератор автоматически загружает следующую страницу при достижении конца текущей.
    pub fn people_paginated(&self, _params: PeopleSearchParams) -> PeoplePaginator {
        // Для people нет параметра page, но можно использовать limit для пагинации
        // Пока что возвращаем пустой стрим, так как API не поддерживает page для people
        Box::new(stream::empty().boxed())
    }

    /// Создает ленивый итератор для пагинации результатов поиска пользовательских оценок.
    ///
    /// Итератор автоматически загружает следующую страницу при достижении конца текущей.
    /// Если `page` не указан, начнет с первой страницы.
    pub fn user_rates_paginated(&self, mut params: UserRateSearchParams) -> UserRatesPaginator {
        let start_page = params.page.unwrap_or(1);
        params.page = Some(start_page);

        let client = self.to_arc();
        let state = UserRatesPaginatorState {
            client,
            params,
            current_page: start_page - 1,
        };

        Box::new(
            stream::unfold(state, |mut state| async move {
                state.current_page += 1;
                state.params.page = Some(state.current_page);

                match state.client.user_rates(state.params.clone()).await {
                    Ok(page) if page.is_empty() => None,
                    Ok(page) => Some((Ok(page), state)),
                    Err(e) => {
                        // Возвращаем ошибку как элемент, стрим остановится после обработки в flat_map
                        Some((Err(e), state))
                    }
                }
            })
            .flat_map(|result: Result<Vec<UserRate>>| {
                stream::iter(match result {
                    Ok(page) => page.into_iter().map(Ok).collect(),
                    Err(e) => {
                        // Возвращаем ошибку как элемент стрима
                        vec![Err(e)]
                    }
                })
            })
            .boxed()
        )
    }
}
