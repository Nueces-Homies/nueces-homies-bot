use color_eyre::Result;
use std::sync::Arc;

use azure_identity::DefaultAzureCredential;
use azure_security_keyvault::SecretClient;

pub struct Azure {
    secrets_client: SecretClient,
}

impl Azure {
    pub fn new(vault_name: &str) -> Result<Self> {
        let credentials = Arc::new(DefaultAzureCredential::default());
        let vault_url = format!("https://{}.vault.azure.net", vault_name);
        Ok(Self {
            secrets_client: SecretClient::new(&vault_url, credentials.clone())?,
        })
    }

    pub async fn get_secret(&self, name: &str) -> Result<String> {
        let value = self.secrets_client.get(name).await?;
        Ok(value.value)
    }
}
