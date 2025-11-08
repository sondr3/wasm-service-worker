// Service Worker entry point that initializes the WASM module
import init from "./pkg/wasm_service_worker.js"

console.log("Service worker script loaded, initializing WASM...")

// Event handlers MUST be registered synchronously during initial script evaluation
self.addEventListener("install", (event) => {
	console.log("Service worker installing...")
	// @ts-ignore
	self.skipWaiting()
})

self.addEventListener("activate", (event) => {
	console.log("Service worker activating...")
	// @ts-ignore
	event.waitUntil(self.clients.claim())
})

// Initialize the WASM module
// @ts-ignore
init()
	.then(() => {
		console.log("Service worker WASM initialized successfully")
	})
	.catch((err: unknown) => {
		console.error("Failed to initialize service worker WASM:", err)
	})
