SHADER_PATH = ./assets/shaders
BINARY_PATH = ./assets/compiled_shaders

all: build

build: shaders
	cargo build

run: shaders 
	RUST_LOG=info cargo run

run_release: shaders
	RUST_LOG=info cargo run --release

shaders: quad comp

quad: shaders_dir
	glslc -fshader-stage=vert -O $(SHADER_PATH)/$@.vert -o $(BINARY_PATH)/$@.vert.spv
	glslc -fshader-stage=frag -O $(SHADER_PATH)/$@.frag -o $(BINARY_PATH)/$@.frag.spv

comp: shaders_dir
	glslc -fshader-stage=comp -O $(SHADER_PATH)/main.glsl -o $(BINARY_PATH)/main.comp.spv

shaders_dir:
	mkdir -p assets/compiled_shaders 

.PHONY: clean

clean:
	rm $(BINARY_PATH)/*.spv
	cargo clean
