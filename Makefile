SHADER_PATH = ./assets/shaders
BINARY_PATH = ./assets/compiled_shaders

all: build

build: shaders_dir
	cargo build

run:  
	WINIT_UNIX_BACKEND=x11 RUST_LOG=info cargo run

run_release:
	WINIT_UNIX_BACKEND=x11 RUST_LOG=info cargo run --release

shaders_dir:
	mkdir -p assets/compiled_shaders 

.PHONY: clean

clean:
	rm $(BINARY_PATH)/*.spv
	cargo clean
