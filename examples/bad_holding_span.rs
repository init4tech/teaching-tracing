//! An example of bad span hygiene.
//!
//! This examples shows how holding on to spans longer than necessary
//! can result in delays to exporting spans to the collector.
//!
//! To play with it, open up your `otel-desktop-viewer` and run this example.
//! You should see that spans are only exported when they are dropped,
//! which in this case is when the collection reaches its maximum size. This
//! causes a delay of about 50 seconds (10 observations at 5 seconds each). In
//! addition, the last few spans may never be exported.

use metrics_tracing_example::{init_metrics, init_tracing, run_observations};
use std::{collections::VecDeque, time::Duration};
use tokio::{select, sync::mpsc};
use tracing::info;

#[tokio::main]
async fn main() {
    // Set up the tracing.
    let _provider = init_tracing();
    // Set up a prometheus metrics exporter on port 9000
    init_metrics(None);

    // We want the observations to be sent to us over a channel.
    let (tx, mut rx) = mpsc::channel(2);

    // We'll run the observations every 5 seconds
    let jh = run_observations(Duration::from_secs(5), Some(tx));
    tokio::pin!(jh);

    // Why is this bad?
    //
    // By holding the `Observations` in a collection here, we hold on to their
    // spans. This keeps the span open longer than necessary, and means that
    // the batch exporter will not export them until they are dropped.
    //
    // This will result in significant delays to exporting spans to the
    // collector. About 50 seconds in this case, as we hold on to 10
    // observations, each created 5 seconds apart.
    //
    // Their children spans will also be closed and exported BEFORE the parent
    // span, which can be confusing when viewing in a trace viewer, and result
    // in missing data if the exporter closes before the parent span is dropped.
    let mut obs_collection = VecDeque::with_capacity(10);

    // The loop select here will run until the observation task exits.
    loop {
        select! {
            _ = &mut jh => {
                info!("Observation task exited");
                break;
            }
            Some(obs) = rx.recv() => {
                obs.span().in_scope(|| {
                    info!("Received observation in main");
                });
                obs_collection.push_back(obs);
                if obs_collection.len() > 10 {
                    obs_collection.pop_front();
                }
            },
        }
    }
}
