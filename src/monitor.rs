//! System monitoring code. This module contains the [`SysMonitor`] struct.

use crate::{CpuStats, Observation};
use sysinfo::System;
use tokio::spawn;
use tracing::{info_span, instrument, trace};

/// System monitor that takes observations at a fixed interval, and sends them
/// to a channel.
pub struct SysMonitor {
    system: System,
    interval: tokio::time::Duration,
    counter: u64,

    outbound: tokio::sync::mpsc::Sender<Observation>,
}

impl SysMonitor {
    /// Create a new system monitor that takes observations at the given
    /// interval.
    pub fn new(
        system: System,
        interval: tokio::time::Duration,
        outbound: tokio::sync::mpsc::Sender<Observation>,
    ) -> Self {
        Self {
            system,
            interval,
            counter: 0,
            outbound,
        }
    }

    /// Take a single observation of the system state.
    ///
    /// This is instrumented so that we can see when observations are taken.
    /// When using the `instrument` macro, the span created is the child of the
    /// current span. This means that if we call this function from within
    /// another span, the observation span will be a child of that span.
    ///
    /// We skip `self` so that the span does not include the debug
    /// representation of the `SysMonitor` struct, which would be noisy.
    ///
    /// See the tracing crate documentation for more details:
    /// <https://docs.rs/tracing/latest/tracing/attr.instrument.html>
    #[instrument(skip(self), name = "Taking observation")]
    fn take_observation(&mut self) -> Vec<CpuStats> {
        // We're going to emit an event when we create the observation
        self.system.refresh_cpu_all();

        trace!("Refreshed CPU information");

        let cpus = self
            .system
            .cpus()
            .iter()
            .map(|cpu| {
                let name = cpu.name().to_owned();
                CpuStats {
                    name,
                    usage: cpu.cpu_usage(),
                    frequency: cpu.frequency(),
                }
            })
            .collect();

        self.counter = self.counter.wrapping_add(1);

        cpus
    }

    /// Spawn the system monitor in a new task. This is the core task loop,
    /// which takes observations at the configured interval, and sends them to
    /// the outbound channel.
    pub(crate) fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        spawn(async move {
            let mut interval = tokio::time::interval(self.interval);

            loop {
                interval.tick().await;

                // We create a new span for each observation, so that we can see
                // when observations are taken, and how long they take.
                //
                // The observation ID is included as a field in the span, so
                // that we can correlate logs and traces.
                let span = info_span!("Observation", observation_id = self.counter);

                // In-scope runs the closure within the context of the
                // span. This ensures that the observation span is the
                // parent of any spans created within the closure, as well
                // as that the observation span is Entered and Exited
                // correctly.
                let stats = span.in_scope(|| {
                    trace!("Taking observation");
                    self.take_observation()
                });

                let obs = Observation::new(stats, span);

                if self.outbound.send(obs).await.is_err() {
                    trace!("SysStats receiver dropped, exiting");
                    break;
                }
            }
        })
    }
}
