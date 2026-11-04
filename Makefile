export CARGO_TARGET_DIR:=./target
export LIBRARY_NAME=thisshoudlbelinked

build:
	ln -sf Cargo1.toml Cargo.toml
	cargo build
	ld -r -b binary -o $(LIBRARY_NAME).o $(CARGO_TARGET_DIR)/debug/librust_bear_lib.so
	ar rcs $(LIBRARY_NAME).a $(LIBRARY_NAME).o
	ln -sf Cargo2.toml Cargo.toml
	cargo build

clean:
	ln -sf Cargo1.toml Cargo.toml
	cargo clean
	ln -sf Cargo2.toml Cargo.toml
	cargo clean