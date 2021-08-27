all:
	clear
	cargo build --target wasm32-unknown-unknown --release
	cp ./target/wasm32-unknown-unknown/release/moo3d_wasm.wasm ./client/moo3d_wasm.wasm

test:
	clear
	cargo test --all -- --nocapture
	cargo test -p moo3d_core -- --nocapture