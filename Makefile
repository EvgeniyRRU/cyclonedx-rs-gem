install:
	cargo install --path $(shell pwd) --profile release --force

lint:
	cargo clippy

test:
	cargo test