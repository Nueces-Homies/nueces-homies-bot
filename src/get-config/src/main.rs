use std::sync::Arc;

use azure_core::auth::TokenCredential;
use azure_identity::DefaultAzureCredential;
use azure_security_keyvault::SecretClient;
use azure_svc_appconfiguration::package_2019_07::models::KeyValue;
use clap::{arg, Parser};
use color_eyre::eyre::{eyre, WrapErr};
use color_eyre::Result;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use url::{Position, Url};

#[derive(Parser, Debug)]
struct Args {
    config_store_name: String,

    #[arg(short, long, default_value = "dev")]
    env: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config_url = format!("https://{}.azconfig.io", args.config_store_name);

    let creds = Arc::new(DefaultAzureCredential::default());
    let appconfig_client =
        azure_svc_appconfiguration::package_2019_07::Client::builder(creds.clone())
            .endpoint(&config_url)
            .scopes(&[&config_url])
            .build();

    let mut kv_stream = appconfig_client
        .get_key_values()
        .label(format!("{},\0", &args.env))
        .into_stream();

    while let Some(config) = kv_stream.next().await {
        let items = config
            .wrap_err("failed to get list of key/values from azure appconfig")?
            .items;
        for key_value in items {
            let key = key_value
                .key
                .as_ref()
                .ok_or_else(|| eyre!("item {:?} has no key", key_value))?;
            let value = get_config_value(&key_value, creds.clone()).await?;
            println!("{}={}", key, value);
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct KeyVaultReference {
    uri: Url,
}

async fn get_config_value(
    item: &KeyValue,
    credentials: Arc<dyn TokenCredential>,
) -> Result<String> {
    let value = item
        .value
        .as_ref()
        .ok_or_else(|| eyre!("Item {:?} doesn't have a value", item))?;
    if let Some(content_type) = &item.content_type {
        if content_type.contains("keyvaultref") {
            let secret_reference = serde_json::from_str::<KeyVaultReference>(value)
                .wrap_err("failed to deserialize KeyVaultReference config value")?;
            let secret = get_secret_by_id(&secret_reference.uri, credentials.clone()).await?;
            return Ok(secret);
        }
    }

    return Ok(value.to_owned());
}

fn get_secret_name(secret_id: &Url) -> Result<String> {
    Ok(secret_id
        .path_segments()
        .ok_or_else(|| eyre!("secret_id {} should be a path", secret_id))?
        .last()
        .ok_or_else(|| eyre!("secret_id is empty"))?
        .to_owned())
}

async fn get_secret_by_id(
    secret_id: &Url,
    credentials: Arc<dyn TokenCredential>,
) -> Result<String> {
    let vault_url = &secret_id[..Position::BeforePath];
    let secrets_client = SecretClient::new(vault_url, credentials.clone())?;

    let name = get_secret_name(secret_id)?;
    let value = secrets_client.get(name).await?;
    Ok(value.value)
}
