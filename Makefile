.PHONY: all test profile callgrind sanitize

all:
	clear
	cargo build --target wasm32-unknown-unknown --release

	rm -f ./client/m3d_wasm.wasm
	rm -f ./client/m3d_wasm.wasm.gz
	cp ./target/wasm32-unknown-unknown/release/m3d_wasm.wasm ./client/m3d_wasm.wasm
	gzip -9 -k ./client/m3d_wasm.wasm

	rm -f ./client/images.txt
	rm -f ./client/images.txt.gz
	cp ./m3d_core/images.txt ./client/images.txt
	gzip -9 -k ./client/images.txt

	#npx http-server ./client/ --gzip -c-1
	http-server ./client/ --gzip -c-1

test:
	clear
	RUST_BACKTRACE=1 cargo test -p m3d_core --bin m3d_test -- --nocapture

profile:
	clear
	cargo build --release -p m3d_core --bin m3d_test

	rm -rf ./profiling
	mkdir -p ./profiling
	cp `find ./target/release/deps/ -maxdepth 1 -name "*m3d_test*" ! -name "*.*"` ./profiling/profile_target
	cp ./m3d_core/images.txt ./profiling/

callgrind:
	clear
	cargo build --release -p m3d_core --bin m3d_test --features m3d_core/callgrind

	rm -rf ./profiling
	mkdir -p ./profiling
	cp `find ./target/release/deps/ -maxdepth 1 -name "*m3d_test*" ! -name "*.*"` ./profiling/profile_target
	cp ./m3d_core/images.txt ./profiling/
	cd ./profiling && valgrind --tool=callgrind --branch-sim=yes --cache-sim=yes --simulate-wb=yes ./profile_target 

sanitize:
	cargo fmt --all
	cargo clean
	rm -rf ./profiling
