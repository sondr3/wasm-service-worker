import init, { handle_install, handle_activate, handle_fetch, handle_message } from "./pkg/wasm_service_worker.js"
let wasmReady = init()

declare let self: ServiceWorkerGlobalScope
self.addEventListener("install", (event) => {
	console.log("Install event triggered")
	event.waitUntil(wasmReady.then(() => handle_install()))
	self.skipWaiting()
})

self.addEventListener("activate", (event) => {
	console.log("Activate event triggered")
	event.waitUntil(wasmReady.then(() => handle_activate()).then(() => self.clients.claim()))
})

self.addEventListener("fetch", (event) => {
	event.respondWith(wasmReady.then(() => handle_fetch(event)))
})

self.addEventListener("message", (event) => {
	console.log("Message event triggered")
	event.waitUntil(wasmReady.then(() => handle_message(event)))
})

console.log("Service Worker script loaded")
