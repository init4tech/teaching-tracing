use metrics_tracing_example::{init_metrics, init_tracing, run_observations};
use std::time::Duration;
use tokio::{select, sync::mpsc};
use tracing::info;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Set up the tracing.
    let provider = init_tracing();
    // Set up a prometheus metrics exporter on port 9000
    init_metrics(None);

    // We want the observations to be sent to us over a channel.
    let (tx, mut rx) = mpsc::channel(2);

    // We'll run the observations every 5 seconds
    let jh = run_observations(Duration::from_secs(5), Some(tx));
    tokio::pin!(jh);

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    // The loop select here will run until the observation task exits.
    loop {
        select! {
            _ = &mut ctrl_c => {
                info!("Received Ctrl-C, shutting down");
                break;
            }
            _ = &mut jh => {
                info!("Observation task exited");
                break;
            }
            Some(obs) = rx.recv() => {
                obs.span().in_scope(|| {
                    info!("Received observation in main");
                });
            },
        }
    }

    // Ensure the provider has a chance to shut down cleanly.
    // This allows it a chance to flush any remaining spans to the collector.
    provider.shutdown().map_err(Into::into)
}
