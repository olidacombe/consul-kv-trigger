use core::future::Future;
use rs_consul::{
    types::{ReadKeyRequest, ReadKeyResponse},
    Config, Consul, ConsulError,
};
use thiserror::Error;
use tokio::time::{sleep, Duration};

// TODO make configurable
const MIN_ERROR_BACKOFF_MS: u64 = 1000;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error(transparent)]
    Consul(#[from] ConsulError),
}

pub struct Watcher {
    client: Consul,
    path: String,
}

impl Watcher {
    pub fn new(path: String) -> Self {
        Self {
            client: Consul::new(Config::from_env()),
            path,
        }
    }
    pub async fn run<F, Fut>(&self, callback: F)
    where
        F: Fn(Vec<ReadKeyResponse>) -> Fut,
        Fut: Future<Output = ()>,
    {
        let mut query = ReadKeyRequest::default();

        let backoff = Duration::from_millis(MIN_ERROR_BACKOFF_MS);

        loop {
            query.key = &self.path;
            match self.client.read_key(query.clone()).await {
                Ok(responses) => {
                    if let Some(response) = responses.first() {
                        // this should be the largest for the entire
                        // prefix or a recursive query acconding to
                        // documentation, so no need to take a max
                        // over the vector
                        query.index = response.modify_index.try_into().ok();
                    }
                    callback(responses).await;
                }
                Err(e) => {
                    tracing::error!("{:?}", e);
                    sleep(backoff).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
