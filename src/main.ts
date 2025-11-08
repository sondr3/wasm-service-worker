if ("serviceWorker" in navigator) {
  navigator.serviceWorker
    .register("/sw.js", {
      scope: "/",
      type: "module",
    })
    .then((registration) => {
      let serviceWorker
      if (registration.installing) {
        serviceWorker = registration.installing
        document.querySelector("#kind")!.textContent = "installing"
      } else if (registration.waiting) {
        serviceWorker = registration.waiting
        document.querySelector("#kind")!.textContent = "waiting"
      } else if (registration.active) {
        serviceWorker = registration.active
        document.querySelector("#kind")!.textContent = "active"
      }
      if (serviceWorker) {
        console.log(serviceWorker.state)
        serviceWorker.addEventListener("statechange", (e) => {
          console.log((e as any).target.state)
        })
      }
    })
    .catch((error) => {
      // Something went wrong during registration. The service-worker.js file
      // might be unavailable or contain a syntax error.
      console.error(error)
    })
} else {
  // The current browser doesn't support service workers.
  // Perhaps it is too old or we are not in a Secure Context.
  console.warn("would not initialize service worker")
}
