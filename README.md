# `wasm-service-worker`

This is a thought experiment that I wanted to see if I could figure out. The 
idea came from trying to make some simple PWA's in regular SPA frameworks like 
React and Vue, but also wanting to try HTMX. Thus, this idea was spawned.

## How?

Pain and trial and error. This also only workers in Chrome since Firefox does 
not support `es` module ServiceWorkers, which is very awkward. The ServiceWorker 
gets registered and initializes some hooks that intercept the `fetch` requests and 
then passes it through an `Axum` router, and if it's a hit it returns that or lets 
the request fall through.

## Requirements

1. [`wasm-bindgen-cli`](https://github.com/wasm-bindgen/wasm-bindgen/)
2. WASM for rustup: `rustup target add wasm32-unknown-unknown`
3. [`mkcert`](https://github.com/FiloSottile/mkcert) for local HTTPS certificates
4. Optionally, [`just`](https://github.com/casey/just) as a task runner

## Installation

1. `pnpm install`
2. `just`
3. `pnpm dev`
4. ???
5. Profit or pain

## Inspiration

- https://github.com/richardanaya/wasm-service
- https://github.com/justinrubek/wasm-bindgen-service-worker

# License 

MIT.
