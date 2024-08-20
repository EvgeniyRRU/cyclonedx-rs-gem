install:
	cargo install --path $(shell pwd) --profile release --force

build:
	cargo build

fix:
	cargo clippy --all-targets --allow-dirty --fix && cargo fmt

check:
	cargo check --all-targets

check-ci:
	cargo check --all-targets --verbose

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets -- -D warnings

lint:
	@make check
	@make fmt
	@make clippy

audit:
	cargo audit

deny:
	cargo deny check advisories bans sources

audit-full:
	@make audit
	@make deny

test:
	cargo test

test-ci:
	cargo test --verbose

full-check:
	@make lint
	@make audit-full
	@make test