use std::time::{SystemTime, UNIX_EPOCH};

use prost::Message;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::IGDBClientError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Token {
    ValidToken {
        access_token: String,
        expires_in: u64,
        token_type: String,
    },
    InvalidToken,
}

impl Token {
    pub fn is_expired_at(&self, timestamp: u64) -> bool {
        match &self {
            Token::InvalidToken => true,
            Token::ValidToken { expires_in, .. } => timestamp >= *expires_in,
        }
    }

    pub fn is_expired(&self) -> bool {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time less than Unix epoch")
            .as_secs();
        self.is_expired_at(timestamp)
    }
}

pub struct IGDBClient {
    client_id: String,
    client_secret: String,
    token: Token,
    client: reqwest::Client,
}

impl IGDBClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            token: Token::InvalidToken,
            client: reqwest::Client::new(),
        }
    }

    pub async fn query_endpoint<T>(
        &mut self,
        endpoint: String,
        query: String,
    ) -> Result<T, IGDBClientError>
    where
        T: Message + Default,
    {
        self.token = self.get_valid_token().await?;
        let access_token = match &self.token {
            Token::ValidToken { access_token, .. } => access_token,
            Token::InvalidToken => panic!("Should only have valid tokens at this point"),
        };

        let url = format!("https://api.igdb.com/v4/{}.pb", endpoint);

        let response = self
            .client
            .post(url)
            .header("Client-ID", &self.client_id)
            .bearer_auth(access_token)
            .body(query)
            .send()
            .await?
            .bytes()
            .await?;

        match T::decode(response) {
            Ok(result) => Ok(result),
            Err(e) => Err(IGDBClientError::ResponseDecodeError(e)),
        }
    }

    async fn get_valid_token(&self) -> Result<Token, IGDBClientError> {
        if self.token.is_expired() {
            let query = vec![
                ("client_id", &self.client_id[..]),
                ("client_secret", &self.client_secret[..]),
                ("grant_type", "client_credentials"),
            ];

            let url = "https://id.twitch.tv/oauth2/token";

            let new_token = self
                .client
                .post(url)
                .query(&query)
                .send()
                .await?
                .json::<Token>()
                .await?;

            return if let Token::InvalidToken = new_token {
                Err(IGDBClientError::TokenError)
            } else {
                Ok(new_token)
            };
        }

        Ok(self.token.clone())
    }
}

#[cfg(test)]
mod tests {
    use dotenvy::dotenv;

    use crate::api::GameResult;

    use super::IGDBClient;
    use super::Token;

    #[tokio::test]
    async fn can_get_token() {
        dotenv().ok();
        dotenvy::from_filename("../../.env").ok();

        let client_id = std::env::var("TWITCH_CLIENT_ID").expect("couldn't find Twitch client id");
        let client_secret =
            std::env::var("TWITCH_CLIENT_SECRET").expect("couldn't find Twitch client secret");

        let client = IGDBClient::new(client_id, client_secret);

        assert!(matches!(client.token, Token::InvalidToken));

        let token = client.get_valid_token().await.unwrap();
        if let Token::ValidToken { access_token, .. } = token {
            assert_ne!(access_token, "");
        } else {
            panic!("Got an invalid token");
        }
    }

    #[tokio::test]
    async fn can_get_game() {
        dotenv().ok();

        let client_id = std::env::var("TWITCH_CLIENT_ID").expect("couldn't find Twitch client id");
        let client_secret =
            std::env::var("TWITCH_CLIENT_SECRET").expect("couldn't find Twitch client secret");

        let mut client = IGDBClient::new(client_id, client_secret);
        let result = client
            .query_endpoint::<GameResult>(
                "games".to_owned(),
                "fields *; where id = 427;".to_owned(),
            )
            .await
            .expect("Should have gotten Final Fantasy VII");

        let game = result.games.first().expect("empty result");
        assert_eq!("Final Fantasy VII", game.name);

        let release_timestamp = game
            .first_release_date
            .as_ref()
            .expect("game not released")
            .seconds;
        assert_eq!(854668800, release_timestamp)
    }
}
