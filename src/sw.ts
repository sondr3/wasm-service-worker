// Service Worker entry point that initializes the WASM module
import init from "./wasm_service_worker.js"

console.log("Service worker script loaded, initializing WASM...")

// Initialize the WASM module
// @ts-ignore
init()
	.then(() => {
		console.log("Service worker WASM initialized successfully")
	})
	.catch((err: unknown) => {
		console.error("Failed to initialize service worker WASM:", err)
	})
