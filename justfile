default:
    just --list

build-wasm:
    rm -rf ./wasm-module/pkg
    cd wasm-module; wasm-pack build --target web --no-typescript
    cp ./wasm-module/pkg/wasm_module.js ./http-server/assets/
    cp ./wasm-module/pkg/wasm_module_bg.wasm ./http-server/assets/

watch-wasm:
    cd wasm-module; watchexec --exts rs -- just build-wasm

spin-up:
    cd http-server; spin up

spin-watch:
    cd http-server; spin watch

