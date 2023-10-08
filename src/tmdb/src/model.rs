#![allow(unused)]

use std::fmt::{Display, Formatter};

use chrono::Utc;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use serde_with::serde_as;
use serde_with::NoneAsEmptyString;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct TMDBId(u32);

impl PartialEq for TMDBId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Into<TMDBId> for u32 {
    fn into(self) -> TMDBId {
        TMDBId(self)
    }
}

impl From<TMDBId> for u32 {
    fn from(value: TMDBId) -> Self {
        value.0
    }
}

impl Display for TMDBId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct MovieSearchResult {
    pub id: TMDBId,
    pub title: String,

    #[serde_as(as = "NoneAsEmptyString")]
    pub release_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct MovieSearchResponse {
    pub results: Vec<MovieSearchResult>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Movie {
    pub id: TMDBId,
    pub imdb_id: String,

    release_dates: MovieReleaseDatesResponse,

    pub title: String,
    pub runtime: u32,
}

impl Movie {
    pub fn release_dates(&self) -> &Vec<MovieRegionReleaseDates> {
        &self.release_dates.results
    }
}

#[derive(Debug, Deserialize)]
pub struct MovieReleaseDatesResponse {
    pub results: Vec<MovieRegionReleaseDates>,
}

#[derive(Debug, Deserialize)]
pub struct MovieRegionReleaseDates {
    pub iso_3166_1: heapless::String<2>,
    // country code
    pub release_dates: Vec<MovieReleaseDate>,
}

#[derive(Debug, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum ReleaseType {
    Premiere = 1,
    TheatricalLimited = 2,
    Theatrical = 3,
    Digital = 4,
    Physical = 5,
    TV = 6,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct MovieReleaseDate {
    #[serde(rename = "type")]
    pub release_type: ReleaseType,

    #[serde_as(as = "NoneAsEmptyString")]
    pub release_date: Option<chrono::DateTime<Utc>>,
}
