use clap::Parser;
use consul_kv_trigger::Watcher;
use eyre::Result;
use tokio::task::{block_in_place, spawn_blocking};
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to watch
    #[clap(value_parser)]
    path: String,
}

#[tokio::main]
async fn watch(watcher: Watcher) {
    watcher
        .run(|kv| async move {
            if let Some(pair) = kv {
                tracing::info!("{:?}", pair);
            }
        })
        .await;
}

fn main() -> Result<()> {
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatter = fmt::Layer::default();
    let subscriber = Registry::default().with(env_filter).with(formatter);
    set_global_default(subscriber).expect("Failed to set subscriber");

    let args = Args::parse();

    let watcher = Watcher::new(args.path).unwrap();

    watch(watcher);

    Ok(())
}
