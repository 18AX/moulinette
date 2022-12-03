all: release

release:
	cargo build --release

debug:
	cargo build

run-release: release
	sudo RUST_LOG=info target/release/moulinette -I library/alpine:latest /bin/sh

run-debug: debug
	sudo RUST_LOG=info target/debug/moulinette -I library/alpine:latest /bin/sh

clean:
	cargo clean
