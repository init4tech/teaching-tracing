//! Simple tracing and metrics example :)
//!
//! This example uses the `sysinfo` crate to take periodic observations of
//! CPU usage and frequency, and sends them over a channel to a stats
//! processor. The stats processor computes average CPU usage and frequency
//! over a sliding window, and emits tracing events with the computed stats.
//!
//! This crate is structured as a super-simple actor model, using
//! [`mpsc`] channels to communicate between actors. The main
//! actors are the [`SysMonitor`], which takes periodic observations of system
//! CPU stats, and the [`SysStats`], which processes observations and emits
//! tracing events with the computed statistics.
//!
//! The [`run_observations`] function starts the observation and stats
//! processing tasks, and returns a [`JoinHandle`] that will resolve if the
//! tasks panic or exit. The tasks will run indefinitely until the program
//! exits or are cancelled using the [`JoinHandle`]. The [`run_observations`]
//! function also takes an optional outbound channel, which can be used to
//! add your own actors to further process the observations.
//!
//! The library also provides sample code for initializing tracing subscribers
//! in [`init_tracing`], and a metrics exporter in [`init_metrics`]. Typically
//! these functions do not belong in library code, but are included here for
//! education.
//!
//! This crate is a teaching tool. Browse the source code, read the comments
//! and documentation, check out the examples! File issues if you have
//! questions, comments, concerns, worries, doubts, fears, or just need someone
//! to talk to :)

pub(crate) mod metrics;
pub use metrics::init_metrics;

mod monitor;
pub use monitor::SysMonitor;

mod obs;
pub use obs::{CpuStats, Observation};

mod stats;
pub use stats::SysStats;

mod trace;
pub use trace::init_tracing;

use std::time::Duration;
use tokio::{sync::mpsc, task::JoinHandle};

/// Start taking observations repeatedly, with an interval of
/// `duration`. If an outbound channel is provided, send observations to it
/// after processing them.
pub fn run_observations(
    every: Duration,
    outbound: Option<mpsc::Sender<Observation>>,
) -> JoinHandle<()> {
    let (tx, rx) = mpsc::channel(2);

    let monitor = SysMonitor::new(sysinfo::System::new_all(), every, tx);

    let stats = SysStats::new(rx, outbound);

    let monitor_handle = monitor.spawn();
    let stats_handle = stats.spawn();

    tokio::spawn(async move {
        tokio::select! {
            _ = monitor_handle => {
                tracing::debug!("Monitor task exited");
            }
            _ = stats_handle => {
                tracing::debug!("Stats task exited");
            }
        }
    })
}
