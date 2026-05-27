use crate::api::invidious::InvidiousClient;
use crate::api::piped::PipedClient;
use crate::api::ytdlp::YtdlpWrapper;
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Provider not available: {0}")]
    NotAvailable(String),
    #[error("Video not found: {0}")]
    NotFound(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub trait StreamProvider: Send + Sync {
    fn get_stream_url<'a>(
        &'a self,
        video_id: &'a str,
    ) -> BoxFuture<'a, Result<String, ProviderError>>;
}

pub struct YtdlpProvider;

impl StreamProvider for YtdlpProvider {
    fn get_stream_url<'a>(&'a self, video_id: &'a str) -> BoxFuture<'a, Result<String, ProviderError>> {
        Box::pin(async move {
            YtdlpWrapper::get_stream_url(video_id).map_err(|e| match e {
                crate::api::ytdlp::YtdlpError::NotFound(s) => ProviderError::NotFound(s),
                _ => ProviderError::ApiError(e.to_string()),
            })
        })
    }
}

pub struct InvidiousProvider {
    pub client: InvidiousClient,
}

impl StreamProvider for InvidiousProvider {
    fn get_stream_url<'a>(&'a self, video_id: &'a str) -> BoxFuture<'a, Result<String, ProviderError>> {
        Box::pin(async move {
            self.client.get_stream_url(video_id).await.map_err(|e| match e {
                crate::api::invidious::InvidiousError::NotFound(s) => ProviderError::NotFound(s),
                _ => ProviderError::ApiError(e.to_string()),
            })
        })
    }
}

pub struct PipedProvider {
    pub client: PipedClient,
}

impl StreamProvider for PipedProvider {
    fn get_stream_url<'a>(&'a self, video_id: &'a str) -> BoxFuture<'a, Result<String, ProviderError>> {
        Box::pin(async move {
            let streams = self.client.get_streams(video_id).await.map_err(|e| match e {
                crate::api::piped::PipedError::NotFound(s) => ProviderError::NotFound(s),
                _ => ProviderError::ApiError(e.to_string()),
            })?;

            if let Some(video_stream) = streams.video_streams.first() {
                return Ok(video_stream.url.clone());
            }

            if let Some(hls) = streams.hls {
                return Ok(hls);
            }

            if let Some(dash) = streams.dash {
                return Ok(dash);
            }

            Err(ProviderError::ApiError("No suitable stream found".to_string()))
        })
    }
}
