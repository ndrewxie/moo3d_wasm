all:
	clear
	cargo build --target wasm32-unknown-unknown --release
	cp ./target/wasm32-unknown-unknown/release/moo3d_wasm.wasm ./client/moo3d_wasm.wasm
	python3 -m http.server -d ./client/	

test:
	clear
	RUST_BACKTRACE=1 cargo test --all -- --nocapture
	RUST_BACKTRACE=1 cargo test -p moo3d_core -- --nocapture

profile:
	clear
	cargo test --no-run -p moo3d_core
	cp `find ./target/debug/deps/ -maxdepth 1 -name "*moo3d_core*" ! -name "*.*"` ./

callgrind:
	clear
	cargo test --no-run -p moo3d_core
	valgrind --tool=callgrind --cache-sim=yes --simulate-wb=yes --cacheuse=yes `find ./target/debug/deps/ -maxdepth 1 -name "*moo3d_core*" ! -name "*.*"`
