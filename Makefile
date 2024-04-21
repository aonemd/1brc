build:
	cargo build --profile profiling

run:
	cargo run --bin 1brc

time: build
	time ./target/profiling/1brc

fine: build
	hyperfine ./target/profiling/1brc

profile: build
	samply record ./target/profiling/1brc

.PHONY: all build run time fine profile
