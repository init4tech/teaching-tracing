//! Metrics collection and exporting. Check the docs for out [`init_metrics`].

use crate::CpuStats;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::LazyLock;

const OBSERVATIONS_MADE: &str = "my_cute_app.observations_made";
const OBSERVATIONS_MADE_DESC: &str = "The total number of observations made";

const OBSERVATIONS_LIVE: &str = "my_cute_app.observations_live";
const OBSERVATIONS_LIVE_DESC: &str = "The number of observations currently held in memory";

const CPU_USAGE_HISTOGRAM: &str = "my_cute_app.cpu_usage";
const CPU_USAGE_HISTOGRAM_DESC: &str = "The CPU usage percentage";

const CPU_FREQUENCY_HISTOGRAM: &str = "my_cute_app.cpu_frequency_mhz";
const CPU_FREQUENCY_HISTOGRAM_DESC: &str = "The CPU frequency in MHz";

static DESCRIBE: LazyLock<()> = LazyLock::new(|| {
    metrics::describe_counter!(OBSERVATIONS_MADE, OBSERVATIONS_MADE_DESC);
    metrics::describe_gauge!(OBSERVATIONS_LIVE, OBSERVATIONS_LIVE_DESC);
    metrics::describe_histogram!(
        CPU_USAGE_HISTOGRAM,
        metrics::Unit::Percent,
        CPU_USAGE_HISTOGRAM_DESC
    );
    metrics::describe_histogram!(CPU_FREQUENCY_HISTOGRAM, CPU_FREQUENCY_HISTOGRAM_DESC);
});

pub(crate) fn record_observation(obs: &[CpuStats]) {
    counter!(OBSERVATIONS_MADE).increment(1);
    gauge!(OBSERVATIONS_LIVE).increment(1);

    for cpu in obs.iter() {
        histogram!(CPU_USAGE_HISTOGRAM, "name" => cpu.name.clone()).record(cpu.usage as f64);
        histogram!(CPU_FREQUENCY_HISTOGRAM, "name" => cpu.name.clone())
            .record(cpu.frequency as f64);
    }
}

/// Initialize a prometheus metrics exporter on the given port, or 9000 if
/// `None`.
///
/// ## What are metrics?
///
/// While tracing records program execution information, metrics record
/// numerical data about a system. They are typically aggregated over time, and
/// can be used to generate graphs and alerts. Metrics are typically
/// exported to a time-series database, such as Prometheus, and can be
/// visualized using tools like Grafana.
///
/// The goal of metrics is to provide a high-level overview of the system's
/// health and performance with automatic aggregation, downsampling, and
/// alerting. This is in contrast to tracing, which provides detailed
/// information about individual requests and operations.
///
/// E.g. while tracing might tell us that a specific request to an API
/// endpoint took 500ms, metrics would tell us that the average latency for
/// that endpoint over the last 5 minutes was 200ms, with a 95th percentile of
/// 400ms.
///
/// ## What is an exporter?
///
/// The [`metrics`] crate provides a generic interface for recording metrics,
/// but does not provide any way to aggregate or export them. An exporter
/// implements the backend for the metrics crate, and is responsible for
/// aggregating and exporting the metrics to a specific system, such as
/// Prometheus.
///
/// In general, libraries should not depend on a specific exporter, as this
/// would limit their usability. Instead, they should depend on the generic
/// [`metrics`] crate, and allow the binary developer to choose the exporter.
/// This is similar to how libraries should depend on the [`tracing`] crate,
/// and allow the binary developer to choose the tracing subscriber(s).
///
/// ## Metrics in this program
///
/// This program records the following metrics:
/// - `my_cute_app.observations_made` (counter): The total number of
///   observations made while the program has been running.
/// - `my_cute_app.observations_live` (gauge): The number of observations
///   currently held in memory.
/// - `my_cute_app.cpu_usage` (histogram): The CPU usage percentage,
///   labeled by CPU name.
/// - `my_cute_app.cpu_frequency_mhz` (histogram): The CPU frequency in MHz,
///   labeled by CPU name.
///
/// Collecting usage and frequency allows metrics aggregators to monitor the
/// CPU over time, and to alert if the CPU usage is too high or the frequency
/// is too low for an extended period. This could allow us to detect CPU
/// throttling, overheating, or other issues.
///
/// Collecting the number of observations made and live allows us to monitor the
/// health of the application itself, and to alert if it is not making
/// observations as expected, or if it is holding too many observations in
/// memory (a memory leak).
///
/// ## Interacting with metrics
///
/// Usually metrics are scraped by a Prometheus server (ask your DevOps friend
/// about these, it'll make them like you more). However, you can also
/// interact with the metrics endpoint directly. If you run this program
/// locally, you can visit `http://localhost:9000/` in your web browser to see
/// the raw metrics data. You can also use `curl`:
/// ```sh
/// curl http://localhost:9000/
/// ```
///
/// This will return a plaintext response with the metrics in the
/// [Prometheus exposition format].
///
/// [Prometheus exposition format]: https://prometheus.io/docs/instrumenting/exposition_formats/
pub fn init_metrics(port: Option<u16>) -> u16 {
    LazyLock::force(&DESCRIBE);
    let port = port.unwrap_or(9000);
    PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], port))
        .install()
        .expect("failed to install prometheus exporter");
    port
}
