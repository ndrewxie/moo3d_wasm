all:
	clear
	cargo build --target wasm32-unknown-unknown --release

	rm -f ./client/moo3d_wasm.wasm
	rm -f ./client/moo3d_wasm.wasm.gzip
	cp ./target/wasm32-unknown-unknown/release/moo3d_wasm.wasm ./client/moo3d_wasm.wasm
	gzip -k ./client/moo3d_wasm.wasm

	rm -f ./client/images.bin
	rm -f ./client/images.bin.gzip
	cp ./moo3d_core/images.bin ./client/images.bin
	gzip -k ./client/images.bin

	npx http-server ./client/ --gzip

test:
	clear
	RUST_BACKTRACE=1 cargo test -p moo3d_core --bin moo3d_test -- --nocapture

profile:
	clear
	cargo build --release -p moo3d_core --bin moo3d_test
	cp `find ./target/release/deps/ -maxdepth 1 -name "*moo3d_test*" ! -name "*.*"` ./

callgrind:
	clear
	cargo build --release -p moo3d_core --bin moo3d_test
	valgrind --tool=callgrind --cache-sim=yes --simulate-wb=yes --cacheuse=yes `find ./target/release/deps/ -maxdepth 1 -name "*moo3d_test*" ! -name "*.*"`

sanitize:
	cargo fmt --all
	cargo clean