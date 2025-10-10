# Metrics and Tracing Examples

This repo contains a simple example of:

- How to instrument your rust programs with [`tracing`]
- How to integrate [`tracing-opentelemetry`] to export spans to an [OpenTelemetry] collector.
- How to instrument your program with the [`metrics` crate]
- How to create a prometheus metrics endpoint via the
  [`metrics-exporter-prometheus` crate]

As well as negative examples of:

- Holding open spans by adding them to a collection.
- Long-lived spans that never end.

## Setup

These examples use the tracing [`EnvFilter`] to configure tracing at the
following env vars:

- `RUST_LOG` - set the filter for console logging
- `OTEL_FILTER` - set the sampling strategy for OpenTelemetry

Some recommended values:

```
$ export RUST_LOG="debug,hyper_util=off,reqwest=off"
$ export OTEL_FILTER="trace"
```

We recommend that you install [`otel-desktop-viewer`]:

With `otel-desktop-viewer` running, you can export spans to the viewer by
running the following commands in a terminal. This configuration is
automatically picked up by the OTEL crates :)

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
export OTEL_TRACES_EXPORTER="otlp"
export OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf"
```

## Where to start with this repo?

1. Build and read the docs! They have a lot of discussion

   `cargo doc --document-private-items --open`

1. Read the example code! It's in the `examples/` directory. It has useful
   information and commentary. The `images/` directory shows roughly what the
   `otel-desktop-viewer` should look like for each example.

1. Run some examples!

   ```bash
   # First, open the otel-desktop-viewer in some other terminal
   # It'll pop open a web browser for you.
   otel-desktop-viewer

   # Then, in this terminal, run some examples!
   # You can use ctrl-c to stop them.
   cargo run --example good_tracing

   # The bad examples
   cargo run --example bad_holding_spans
   cargo run --example bad_program_span
   ```

1. Read the `BEST_PRACTICES.md` doc. It has our opinions on how to use
   tracing effectively.

1. Read the `src/` code! Check out how we use the [`tracing_subscriber`] and
   [`metrics_exporter_prometheus`] crates to set up tracing and metrics. Think about how to adapt this to your own binaries.

[OpenTelemetry]: https://opentelemetry.io/
[`tracing`]: https://docs.rs/tracing/latest/tracing/
[`tracing-opentelemetry`]: https://docs.rs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/
[`EnvFilter`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
[`metrics` crate]: https://docs.rs/metrics/latest/metrics/
[`metrics-exporter-prometheus` crate]: https://docs.rs/metrics-exporter-prometheus/latest/metrics_exporter_prometheus/
[`otel-desktop-viewer`]: https://github.com/CtrlSpice/otel-desktop-viewer
