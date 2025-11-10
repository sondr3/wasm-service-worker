import init, { handle_fetch } from "./pkg/wasm_service_worker.js"
let wasmReady = init()

const CACHE_NAME = "wasm-service-worker-v1"
const STATIC_ASSETS = [
	"/",
	"/index.html",
	"/styles.css",
	"/app.js",
	"/offline.html",
	"/htmx.min.js",
	"/htmx.js",
	"/pico.min.css",
]

declare let self: ServiceWorkerGlobalScope
self.addEventListener("install", (event) => {
	console.log("Installing service worker...")
	event.waitUntil(
		self.caches.open(CACHE_NAME).then((cache) => {
			return cache.addAll(STATIC_ASSETS)
		}),
	)
	self.skipWaiting()
})

self.addEventListener("activate", (event) => {
	event.waitUntil(self.clients.claim())
})

self.addEventListener("fetch", (event) => {
	event.respondWith(
		caches.match(event.request).then((response) => {
			return response || wasmReady.then(() => handle_fetch(event.request))
		}),
	)
})

self.addEventListener("message", (event) => {
	console.log("SW message: ", event)
})

console.log("Service Worker script loaded")
