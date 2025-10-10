//! Read [`SysStats`] instead, it's more interesting.

use crate::{CpuStats, Observation};
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument};

/// A simple stats processor.
pub struct SysStats {
    inbound: mpsc::Receiver<Observation>,
    outbound: Option<mpsc::Sender<Observation>>,

    /// NB: An easy mistake to make here would be to store the [`Observation`]
    /// structs directly. This would result in the `Span` being held in the
    /// `SysStats` struct, which would delay its closure until it's removed from
    /// the collection.
    ///
    /// If you see unknown spans in your tracing output, you're likely holding
    /// them somewhere like this.
    previous_obs: VecDeque<Vec<CpuStats>>,
}

impl SysStats {
    /// Create a new `SysStats` processor
    pub fn new(
        inbound: mpsc::Receiver<Observation>,
        outbound: Option<mpsc::Sender<Observation>>,
    ) -> Self {
        Self {
            inbound,
            outbound,
            previous_obs: VecDeque::with_capacity(10),
        }
    }

    /// Compute stats over previous observations and emit a tracing event.
    #[instrument(skip(self), name = "Computing stats")]
    fn run_stats(&self) {
        let iter = self.previous_obs.iter().flat_map(|obs| obs.iter());

        let count = iter.clone().count() as f64;

        let total_usage: f64 = iter.clone().map(|cpu| cpu.usage as f64).sum();
        let total_freq: f64 = iter.map(|cpu| cpu.frequency as f64).sum();

        let average_usage = total_usage / count;
        let average_freq_mhz = total_freq / count;

        // Attaching fields puts structured data into your tracing
        // event, which may then be automatically parsed by your collector or
        // backend. `tracing` also supports string formatted messages, but
        // these cannot be automatically parsed.
        //
        // It is ALWAYS better to use fields than to use formatted strings.
        //
        // ```
        // // avoid this! It is not structured, and is hard to parse!
        // info!(
        //     "{} observations, {} CPUs: avg usage {:.2}%, avg freq {:.2}MHz",
        //     self.previous_obs.len(),
        // ```
        info!(
            count = self.previous_obs.len(),
            cpus = count / self.previous_obs.len() as f64,
            average_usage,
            average_freq_mhz,
            "finished cpu stats"
        );
    }

    /// Spawn the stats processor task.
    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(obs) = self.inbound.recv().await {
                obs.span().in_scope(|| {
                    if self.previous_obs.len() == 10 {
                        self.previous_obs.pop_front();
                    }
                    self.previous_obs.push_back((*obs).clone());

                    self.run_stats();
                });

                if let Some(outbound) = &mut self.outbound
                    && outbound.send(obs).await.is_err()
                {
                    debug!("Outbound receiver dropped, stopping forwarding");
                    break;
                }
            }
        })
    }
}
