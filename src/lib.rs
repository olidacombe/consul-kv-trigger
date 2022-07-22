use consul::kv::{KVPair, KV};
use consul::{Client, Config, QueryOptions};
use core::future::Future;
use thiserror::Error;
use tokio::task::spawn_blocking;
use tokio::time::{sleep, Duration};

// TODO make configurable
const MIN_ERROR_BACKOFF_MS: u64 = 1000;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error(transparent)]
    Consul(#[from] consul::errors::Error),
}

pub struct Watcher {
    client: Client,
    path: String,
}

impl Watcher {
    pub fn new(path: String) -> Result<Self, WatcherError> {
        Ok(Self {
            client: Client::new(Config::new_from_env()?),
            path,
        })
    }
    pub async fn run<F, Fut>(&self, callback: F)
    where
        F: Fn(Option<KVPair>) -> Fut,
        Fut: Future<Output = ()>,
    {
        let mut opts = QueryOptions {
            datacenter: None,
            wait_index: None,
            wait_time: None,
        };

        let backoff = Duration::from_millis(MIN_ERROR_BACKOFF_MS);

        loop {
            let result = spawn_blocking(|| self.client.get(&self.path, Some(&opts)))
                .await
                // TODO fix usage of async in here
                .map(|result| match result {
                    Ok((kv, meta)) => {
                        opts.wait_index = meta.last_index;
                        //callback(kv).await;
                    }
                    Err(e) => {
                        tracing::error!("{:?}", e);
                        sleep(backoff).await;
                    }
                });
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
