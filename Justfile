default: build

build:
  cargo br
  wasm-bindgen ./target/wasm32-unknown-unknown/release/wasm_service_worker.wasm --out-dir ./src --typescript --target web

[working-directory: 'pkg']
dev:
  pnpm vite
