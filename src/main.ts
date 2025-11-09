const statusEl = document.getElementById("kind")!

function log(message: string) {
	console.log(message)
}

function updateStatus(status: string) {
	statusEl.textContent = status
	log(`Status: ${status}`)
}

// Register the Service Worker
async function registerServiceWorker() {
	if (!("serviceWorker" in navigator)) {
		updateStatus("Not supported")
		log("Service Workers are not supported in this browser")
		return
	}

	try {
		updateStatus("Registering...")

		// Register the service worker
		// Note: The service worker file is an ES module that imports WASM
		const swPath = import.meta.env.DEV ? "/src/sw.ts" : "/sw.js"
		const registration = await navigator.serviceWorker.register(swPath, {
			type: "module", // Important: ES module support
			scope: "/",
		})

		log("Service Worker registered successfully")
		log(`Scope: ${registration.scope}`)

		// Monitor Service Worker state
		if (registration.installing) {
			updateStatus("Installing...")
			trackWorker(registration.installing)
		} else if (registration.waiting) {
			updateStatus("Waiting")
			log("Service Worker is waiting to activate")
		} else if (registration.active) {
			updateStatus("Active")
			log("Service Worker is active and running")
		}

		// Listen for updates
		registration.addEventListener("updatefound", () => {
			log("Service Worker update found!")
			trackWorker(registration.installing)
		})
	} catch (error) {
		updateStatus("Registration failed")
		log(`Registration failed: ${error.message}`)
		console.error(error)
	}
}

function trackWorker(worker) {
	worker.addEventListener("statechange", () => {
		log(`Service Worker state: ${worker.state}`)
		updateStatus(worker.state)

		if (worker.state === "activated") {
			log("Service Worker activated! Page is now cached.")
		}
	})
}

// Unregister the Service Worker
async function unregisterServiceWorker() {
	if (!("serviceWorker" in navigator)) {
		return
	}

	try {
		const registrations = await navigator.serviceWorker.getRegistrations()

		for (const registration of registrations) {
			await registration.unregister()
			log("Service Worker unregistered")
		}

		updateStatus("Unregistered")

		// Clear cache
		const cacheNames = await caches.keys()
		for (const cacheName of cacheNames) {
			await caches.delete(cacheName)
			log(`Cache deleted: ${cacheName}`)
		}
	} catch (error) {
		log(`Unregister failed: ${error.message}`)
	}
}

// Test fetching a resource (will be intercepted by SW)
async function testFetch() {
	try {
		log("Testing fetch...")
		const response = await fetch("/index.html")

		if (response.ok) {
			log("✓ Fetch successful (check console to see if it came from cache)")
		} else {
			log(`✗ Fetch failed: ${response.status}`)
		}
	} catch (error) {
		log(`✗ Fetch error: ${error.message}`)
	}
}

// Event listeners
document.getElementById("register-btn")!.addEventListener("click", registerServiceWorker)
document.getElementById("unregister-btn")!.addEventListener("click", unregisterServiceWorker)
document.getElementById("test-fetch")!.addEventListener("click", testFetch)

// Check initial Service Worker status
async function checkServiceWorkerStatus() {
	if (!("serviceWorker" in navigator)) {
		updateStatus("Not supported")
		return
	}

	const registration = await navigator.serviceWorker.getRegistration()

	if (registration) {
		if (registration.active) {
			updateStatus("Active")
			log("Service Worker is already active")
		} else if (registration.installing) {
			updateStatus("Installing...")
		} else if (registration.waiting) {
			updateStatus("Waiting")
		}
	} else {
		updateStatus("Not registered")
	}
}

// Initialize
checkServiceWorkerStatus()
log('App loaded. Click "Register Service Worker" to start.')
