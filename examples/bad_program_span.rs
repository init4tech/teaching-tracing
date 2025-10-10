use metrics_tracing_example::{init_metrics, init_tracing, run_observations};
use std::time::Duration;
use tokio::{select, sync::mpsc};
use tracing::{info, info_span};

#[tokio::main]
async fn main() {
    // Set up the tracing.
    let _provider = init_tracing();
    // Set up a prometheus metrics exporter on port 9000
    init_metrics(None);

    // Why is this bad?
    // Because this span is never exited, and so every observation taken
    // will be a child of this span. This means that if you have a long-running
    // span, and you take many observations, your trace tree will grow
    // indefinitely, which can lead to performance issues and make it
    // difficult to understand the trace.
    //
    // In addition, the Otel batch exporter will only attempt to export spans
    // that ARE closed. So if you have a long-running span that is never
    // closed, it will not be exported, and its child spans will be orphaned
    // in the collector.
    let _my_forever_span = info_span!("my_forever_span").entered();

    // We want the observations to be sent to us over a channel.
    let (tx, mut rx) = mpsc::channel(2);

    // We'll run the observations every 5 seconds
    let jh = run_observations(Duration::from_secs(5), Some(tx));
    tokio::pin!(jh);

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
            },
        }
    }
}
