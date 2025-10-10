//! Just the [`Observation`] struct.

use metrics::gauge;
use std::ops::{Deref, DerefMut};
use tracing::trace;

/// CPU statistics at a point in time.
#[derive(Debug, Clone)]
pub struct CpuStats {
    /// CPU name
    pub name: String,

    /// CPU usage percentage
    pub usage: f32,

    /// CPU frequency in MHz
    pub frequency: u64,
}

/// An observation of CPU stats at a point in time, along with the tracing span
/// associated with it.
///
/// The core pattern here is to associate the span with the data directly.
/// [`Span`]s are not invisible background things. they are part of your data!
/// When designing your application, a root [`Span`] should be created _when
/// data is created_, and should be closed when the data is dropped.
///
/// The `Observation` is the basic "unit of work" for this application, and is
/// sent over channels between the monitor and stats processor, and optionally
/// out for subsequent processing. The `Observation` struct contains the CPU
/// stats as well as a [`Span`] that is used to trace the processing of this
/// observation. Whenever the `Observation` is processed, the span _should_ be
/// entered.
///
/// For sync code, this can be done with the [`tracing::Span::in_scope`]
/// method. For  async code, syou can use the [`tracing::Instrument`] trait
/// from the `tracing` crate.
///
/// ```rust
/// use tracing::Instrument;
/// use metrics_tracing_example::{Observation, CpuStats};
///
/// // Instrument an async function with the observation's span using the
/// async fn obs_processor(obs: Observation)
/// {
///     async fn obs_processor_inner(obs: &[CpuStats]) {
///         // Do something with the observation
///     }
///
///     let obs_span = obs.span().clone();
///     obs_processor_inner(&obs).instrument(obs_span).await;
/// }
///
/// fn obs_processor_sync(obs: Observation) {
///     fn obs_processor_inner(obs: &[CpuStats]) {
///         // Do something with the observation
///     }
///    obs.span().in_scope(|| obs_processor_inner);
/// }
/// ```
///
/// [`Span`]: tracing::Span
#[derive(Debug)]
pub struct Observation {
    cpus: Vec<CpuStats>,

    span: tracing::Span,
}

impl Deref for Observation {
    type Target = Vec<CpuStats>;

    fn deref(&self) -> &Self::Target {
        &self.cpus
    }
}

impl DerefMut for Observation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cpus
    }
}

impl Observation {
    /// Create a new Observation. The `Observation` is our core unit of work
    /// for this program. It contains the CPU statistics at a point in time, as
    /// well as a span for use when accessing the observation.
    ///
    /// The `span` here is the tracing span associated with this Observation.
    pub fn new(cpus: Vec<CpuStats>, span: tracing::Span) -> Self {
        crate::metrics::record_observation(&cpus);
        Self { cpus, span }
    }

    /// Run a function within the scope of this observation's span.
    pub fn in_scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[CpuStats]) -> R,
    {
        self.span().in_scope(|| f(&self.cpus))
    }

    /// Get the tracing span associated with this observation
    pub fn span(&self) -> &tracing::Span {
        &self.span
    }
}

impl Drop for Observation {
    fn drop(&mut self) {
        self.span().in_scope(|| {
            trace!("Dropping observation");
        });
        gauge!("my_cute_app.observations_live").decrement(1);
    }
}
