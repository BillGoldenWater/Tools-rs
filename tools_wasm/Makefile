TARGET_DIR := $(strip $(shell cargo metadata --format-version 1 | jq -r .target_directory))
WASM_OUT := $(TARGET_DIR)/wasm32-unknown-unknown/release/tools_wasm.wasm

SOURCES := $(shell find src -type f) ./Cargo.toml
.PHONY: $(WASM_OUT)

build: $(WASM_OUT)
	wasm-bindgen --browser --out-dir pkg $(WASM_OUT)

$(WASM_OUT): $(SOURCES)
	cargo build --release --target wasm32-unknown-unknown
