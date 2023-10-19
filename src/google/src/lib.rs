use std::io;

use google_books1::api::{Volume, VolumeVolumeInfo};
use google_books1::Books;
use google_calendar3::api::Event;
use google_calendar3::chrono::{DateTime, Utc};
use google_calendar3::{hyper, hyper_rustls, CalendarHub};
use thiserror::Error;
use yup_oauth2::hyper::client::HttpConnector;
use yup_oauth2::hyper_rustls::HttpsConnector;
use yup_oauth2::ServiceAccountAuthenticator;

type GoogleCalendar = CalendarHub<HttpsConnector<HttpConnector>>;
type GoogleBooks = Books<HttpsConnector<HttpConnector>>;

pub struct Google {
    calendar_hub: GoogleCalendar,
    books_hub: GoogleBooks,
}

#[derive(Error, Debug)]
pub enum GoogleError {
    #[error("unable to parse service account key")]
    ServiceAccountKeyParseError(#[from] io::Error),

    #[error("error with google api")]
    GoogleApiError(#[from] google_calendar3::Error),

    #[error("expected data for field '{0}'")]
    MissingDataError(&'static str),
}

impl Google {
    pub async fn new(service_account_key: &str) -> Result<Self, GoogleError> {
        let creds = yup_oauth2::parse_service_account_key(service_account_key)?;
        let auth = ServiceAccountAuthenticator::builder(creds).build().await?;

        let calendar_hub = CalendarHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth.clone(),
        );

        let books_hub = Books::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth.clone(),
        );

        Ok(Self {
            calendar_hub,
            books_hub,
        })
    }

    pub async fn get_events(
        &self,
        calendar_id: &str,
        min_time: DateTime<Utc>,
        max_time: DateTime<Utc>,
    ) -> Result<Option<Vec<Event>>, GoogleError> {
        let (_, events) = self
            .calendar_hub
            .events()
            .list(calendar_id)
            .single_events(true)
            .show_deleted(true)
            .order_by("startTime")
            .time_min(min_time)
            .time_max(max_time)
            .doit()
            .await?;

        Ok(events.items)
    }

    pub async fn books_search(&self, query: &str) -> Result<Option<Vec<Volume>>, GoogleError> {
        let (_, books) = self.books_hub.volumes().list(query).doit().await?;
        Ok(books.items)
    }

    pub async fn isbn_lookup(&self, isbn: &str) -> Result<Option<VolumeVolumeInfo>, GoogleError> {
        let query = format!("isbn:{}", isbn.replace("-", ""));
        let (_, volumes) = self.books_hub.volumes().list(&query).doit().await?;
        if 0 == volumes
            .total_items
            .ok_or_else(|| GoogleError::MissingDataError("totalItems"))?
        {
            return Ok(None);
        }

        let items = volumes
            .items
            .ok_or_else(|| GoogleError::MissingDataError("items"))?;
        let result = items
            .first()
            .ok_or_else(|| GoogleError::MissingDataError("items[0]"))?;
        let info = result
            .volume_info
            .as_ref()
            .ok_or_else(|| GoogleError::MissingDataError("items[0]['volumeInfo']"))?;

        Ok(Some(info.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    use dotenvy::dotenv;
    use google_calendar3::chrono::{Duration, Utc};

    use crate::Google;

    #[tokio::test]
    async fn google_calendar_test() {
        dotenv().ok();
        let service_key = read_service_key_from_env();
        let google = Google::new(&service_key).await.unwrap();

        let summer_game_fest_calendar =
            "s71id26u0afr69leltrq0us0b97jp35k@import.calendar.google.com";
        let events = google
            .get_events(
                summer_game_fest_calendar,
                Utc::now() - Duration::days(365),
                Utc::now() + Duration::days(365),
            )
            .await
            .unwrap();

        assert_ne!(events.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn google_books_search_test() {
        dotenv().unwrap();
        let service_key = read_service_key_from_env();
        let google = Google::new(&service_key).await.unwrap();
        let books = google.books_search("Jade City").await.unwrap();

        assert_ne!(books.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn google_books_isbn_test() {
        dotenv().ok();
        let service_key = read_service_key_from_env();
        let google = Google::new(&service_key).await.unwrap();
        let book = google.isbn_lookup("9780316440882").await.unwrap().unwrap();

        assert_eq!("Jade City", book.title.unwrap());
    }

    fn read_service_key_from_env() -> String {
        let service_account_key_64 =
            env::var("GOOGLE_CREDENTIALS").expect("GOOGLE_CREDENTIALS not found in envvar");

        let decoded = BASE64_STANDARD
            .decode(service_account_key_64)
            .expect("Unable to decode service account key");

        String::from_utf8(decoded).unwrap()
    }
}
