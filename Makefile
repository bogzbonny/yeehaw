
build:
	cargo build

lint:
	cargo clippy

test:
	cargo test

.PHONY: build lint test
