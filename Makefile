.PHONY: all clean

all:
	RUSTFLAGS='-C target-cpu=native'  cargo build --release --all-targets

clean:
	cargo clean