use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;
use tracing::{error, info};

use crate::github::download_and_extract_github_artifact;
use crate::AppState;

pub struct DownloadQueue {
    pub app_state: Arc<AppState>,
    pub queue_rx: Receiver<String>,
}

impl DownloadQueue {
    pub async fn process_queue(&mut self) {
        info!("Started download queue processor");
        while let Some(url) = self.queue_rx.recv().await {
            for i in 1..10 {
                info!("Attempt #{} to download artifacts from {}", i, url);
                if let Err(e) = download_and_extract_github_artifact(
                    &self.app_state.azure,
                    &url,
                    &self.app_state.args.extraction_directory,
                )
                .await
                {
                    error!("Error downloading from {}", e);
                } else {
                    info!(
                        "Successfully downloaded {} to {}",
                        &url, &self.app_state.args.extraction_directory
                    );
                    break;
                }
                sleep(Duration::from_secs(2u64.pow(i))).await;
            }
        }
        info!("Finished download queue processor");
    }
}
