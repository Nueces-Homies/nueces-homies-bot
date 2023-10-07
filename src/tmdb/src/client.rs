#![allow(unused)]

use async_trait::async_trait;
use reqwest::header;
use serde::de::DeserializeOwned;

use crate::errors::TMDBClientError;
use crate::model::*;

const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";

type Result<T> = std::result::Result<T, TMDBClientError>;

#[async_trait]
pub trait MovieClient {
    async fn search_movie(&self, query: &str) -> Result<Vec<MovieSearchResult>>;
    async fn get_movie(&self, id: TMDBId) -> Result<Movie>;
}

pub struct TMDBClient {
    client: reqwest::Client,
}

impl TMDBClient {
    pub fn new(api_token: &str) -> Result<Self> {
        let mut token = header::HeaderValue::from_str(&format!("Bearer {}", api_token))?;
        token.set_sensitive(true);
        let mut default_headers = header::HeaderMap::new();
        default_headers.insert(header::AUTHORIZATION, token);

        Ok(Self {
            client: reqwest::Client::builder()
                .default_headers(default_headers)
                .build()?,
        })
    }

    async fn make_request<'a, T>(&self, endpoint: &str, params: &Vec<(&str, &str)>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/{}", TMDB_BASE_URL, endpoint);
        let response = self
            .client
            .get(url)
            .query(params)
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str::<T>(&response)?)
    }
}

#[async_trait]
impl MovieClient for TMDBClient {
    async fn search_movie(&self, query: &str) -> Result<Vec<MovieSearchResult>> {
        let params = vec![("include_adult", "false"), ("query", query)];
        let result = self
            .make_request::<MovieSearchResponse>("search/movie", &params)
            .await;
        Ok(result?.results)
    }

    async fn get_movie(&self, id: TMDBId) -> Result<Movie> {
        let params = vec![("append_to_response", "release_dates")];
        let endpoint = format!("movie/{}", id);
        let result = self.make_request::<Movie>(&endpoint, &params).await;
        result
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::str::FromStr;

    use chrono::DateTime;
    use chrono::{NaiveDateTime, Utc};
    use dotenvy::dotenv;

    use crate::client::{MovieClient, TMDBClient};
    use crate::errors::TMDBClientError;
    use crate::model::Movie;

    #[tokio::test]
    async fn it_can_search_for_movies() {
        dotenv().ok();

        let api_token = env::var("TMDB_TOKEN").expect("couldn't find TMDB token");
        let client = TMDBClient::new(&api_token).expect("couldn't create client");

        let movies = client
            .search_movie("avengers")
            .await
            .expect("No Avenger movies");
        let first_avengers_movie = movies
            .iter()
            .find(|m| m.title == "The Avengers")
            .expect("Couldn't find the first Avengers movie");
        assert_eq!(first_avengers_movie.id, 24428.into());
        assert_eq!(
            first_avengers_movie.release_date,
            chrono::NaiveDate::from_ymd_opt(2012, 4, 25)
        );
    }

    #[tokio::test]
    async fn it_can_search_for_movies_without_release_date() {
        dotenv().ok();

        let api_token = env::var("TMDB_TOKEN").expect("couldn't find TMDB token");
        let client = TMDBClient::new(&api_token).expect("couldn't create client");

        // TODO: This test will break when Armor Wars gets a release date...
        let movies = client
            .search_movie("Armor Wars")
            .await
            .expect("No Armor Wars");

        let armor_wars_movie = movies.iter().find(|m| m.title == "Armor Wars").unwrap();
        assert_eq!(armor_wars_movie.release_date, None);
    }

    #[tokio::test]
    async fn it_can_lookup_a_movie() {
        dotenv().ok();

        let api_token = env::var("TMDB_TOKEN").expect("couldn't find TMDB token");
        let client = TMDBClient::new(&api_token).expect("couldn't create client");

        let movie = match client.get_movie(24428.into()).await {
            Ok(m) => m,
            Err(e) => match e {
                TMDBClientError::ClientDeserializationError(e) => {
                    println!("{}:{} {:?}", e.line(), e.column(), e.classify());
                    panic!("Boo")
                }
                _ => panic!("boo"),
            },
        };

        let british_release_date = movie
            .release_dates()
            .iter()
            .find(|country_date| country_date.iso_3166_1 == "GB")
            .expect("no british releases")
            .release_dates
            .first()
            .expect("no expected british releases")
            .release_date
            .expect("not dated");

        assert_eq!(
            british_release_date,
            DateTime::<Utc>::from_str(&"2012-04-26T00:00:00.000Z").unwrap()
        );
    }
}
