default:
    just --list

# build wasm module and copy to assets
build-wasm:
    rm -rf ./wasm-module/pkg
    cd wasm-module; wasm-pack build --target web --no-typescript
    cp ./wasm-module/pkg/wasm_module.js ./http-server/assets/js/wasm/
    cp ./wasm-module/pkg/wasm_module_bg.wasm ./http-server/assets/js/wasm

# watch changes in wasm-module and run build and copy
watch-wasm:
    cd wasm-module; watchexec --exts rs -- just build-wasm

# spin up web-server
spin-up:
    cd http-server; spin up

# watch changes and update web-server
spin-watch:
    cd http-server; spin watch

