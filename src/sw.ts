import init, { handle_fetch } from "./pkg/wasm_service_worker.js"
let wasmReady = init()

const CACHE_NAME = "wasm-service-worker-v1"
const STATIC_ASSETS = [
	"/",
	"/index.html",
	"/main.js",
	"/sw.js",
	"/htmx.min.js",
	"/htmx.js",
	"/pico.min.css",
	"wasm_service_worker_bg.wasm",
]

declare let self: ServiceWorkerGlobalScope
self.addEventListener("install", (event) => {
	console.log("Installing service worker...")
	event.waitUntil(
		caches
			.open(CACHE_NAME)
			.then((cache) => cache.addAll(STATIC_ASSETS))
			.then(() => console.log("Succesfully fetched and cached"))
			.catch((err) => console.error(err)),
	)
	self.skipWaiting()
})

const activate = async () => {
	if (self.registration.navigationPreload) {
		await self.registration.navigationPreload.enable()
	}

	self.clients
		.claim()
		.then(() => console.log("Client claimed"))
		.catch((err) => console.error(err))
}

self.addEventListener("activate", (event) => {
	event.waitUntil(activate())
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
