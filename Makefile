.PHONY: check build test fmt-check fmt lint clean

check:
	cargo check --all

build:
	cargo build --all

test:
	cargo test --all

fmt-check:
	taplo fmt --check
	cargo +nightly fmt --all -- --check

fmt:
	taplo fmt
	cargo +nightly fmt --all

lint:
	cargo +nightly fmt
	cargo clippy --all --all-targets -- -D warnings

clean:
	cargo clean
