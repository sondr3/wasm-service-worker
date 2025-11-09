// Service Worker entry point that initializes the WASM module
// // Import the WASM module
import init, { handle_install, handle_activate, handle_fetch, handle_message } from "./pkg/wasm_service_worker.js"

// Initialize WASM module
let wasmReady = init()

self.addEventListener("install", (event) => {
	console.log("Install event triggered")
	console.dir(event)
	event.waitUntil(wasmReady.then(() => handle_install()))
})

self.addEventListener("activate", (event) => {
	console.log("Activate event triggered")
	event.waitUntil(wasmReady.then(() => handle_activate()))
})

self.addEventListener("fetch", (event) => {
	event.respondWith(wasmReady.then(() => handle_fetch(event)))
})

self.addEventListener("message", (event) => {
	console.log("Message event triggered")
	event.waitUntil(wasmReady.then(() => handle_message(event)))
})

console.log("Service Worker script loaded")
