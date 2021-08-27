all:
	clear
	cargo build --target wasm32-unknown-unknown --release
	cp ./target/wasm32-unknown-unknown/release/moo3d_wasm.wasm ./client/moo3d_wasm.wasm

test:
	clear
	RUST_BACKTRACE=1 cargo test --all -- --nocapture
	RUST_BACKTRACE=1 cargo test -p moo3d_core -- --nocapture

profile:
	clear
	cargo test --no-run -p moo3d_core
	cp `find ./target/debug/deps/ -maxdepth 1 -name "*moo3d_core*" ! -name "*.*"` ./