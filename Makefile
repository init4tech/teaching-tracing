all: build

build:
	@cargo build --all-targets

set-env-vars:
	@export RUST_LOG=info
	@export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
	@export OTEL_TRACES_EXPORTER="otlp"
	@export OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf"