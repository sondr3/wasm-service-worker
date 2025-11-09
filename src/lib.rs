use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Cache, CacheStorage, ExtendableEvent, ExtendableMessageEvent, FetchEvent, Request, Response,
    ServiceWorkerGlobalScope, console,
};

const CACHE_NAME: &str = "my-app-cache-v1";

// Files to cache during installation
const STATIC_ASSETS: &[&str] = &[
    "/",
    "/index.html",
    "/styles.css",
    "/app.js",
    "/offline.html",
];

#[wasm_bindgen]
pub async fn handle_install() -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Service Worker installing...".into());

    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    install_handler(&global).await?;

    Ok(JsValue::NULL)
}

#[wasm_bindgen]
pub async fn handle_activate() -> Result<JsValue, JsValue> {
    web_sys::console::log_1(&"Service Worker activating...".into());

    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    activate_handler(&global).await?;

    Ok(JsValue::NULL)
}

#[wasm_bindgen]
pub async fn handle_fetch(event: FetchEvent) -> Result<Response, JsValue> {
    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    let request = event.request();

    fetch_handler(&global, &request).await
}

#[wasm_bindgen]
pub async fn handle_message(event: ExtendableMessageEvent) -> Result<JsValue, JsValue> {
    let data = event.data();
    web_sys::console::log_2(&"Service Worker received message:".into(), &data);
    Ok(JsValue::NULL)
}

async fn install_handler(global: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
    let cache_storage = global.caches()?;
    let cache = open_cache(&cache_storage, CACHE_NAME).await?;

    // Cache all static assets
    for asset in STATIC_ASSETS {
        let request = Request::new_with_str(asset)?;

        match fetch_and_cache(&cache, &request).await {
            Ok(_) => web_sys::console::log_1(&format!("Cached: {}", asset).into()),
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to cache {}: {:?}", asset, e).into())
            }
        }
    }

    web_sys::console::log_1(&"Service Worker installed!".into());
    Ok(())
}

async fn activate_handler(global: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
    web_sys::console::log_1(&"Service Worker activating...".into());

    let cache_storage = global.caches()?;
    let cache_keys = JsFuture::from(cache_storage.keys()).await?;
    let cache_keys: js_sys::Array = cache_keys.into();

    // Delete old caches
    for key in cache_keys.iter() {
        let cache_name = key.as_string().unwrap_or_default();
        if cache_name != CACHE_NAME {
            web_sys::console::log_1(&format!("Deleting old cache: {}", cache_name).into());
            JsFuture::from(cache_storage.delete(&cache_name)).await?;
        }
    }

    web_sys::console::log_1(&"Service Worker activated!".into());
    Ok(())
}

async fn fetch_handler(
    global: &ServiceWorkerGlobalScope,
    request: &Request,
) -> Result<Response, JsValue> {
    let url = request.url();
    let cache_storage = global.caches()?;
    let cache = open_cache(&cache_storage, CACHE_NAME).await?;

    // Try cache first
    if let Ok(Some(cached_response)) = get_from_cache(&cache, request).await {
        web_sys::console::log_1(&format!("Cache hit: {}", url).into());
        return Ok(cached_response);
    }

    // Fallback to network
    web_sys::console::log_1(&format!("Cache miss, fetching: {}", url).into());

    match fetch_from_network(global, request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            web_sys::console::error_1(&format!("Network fetch failed: {:?}", e).into());

            // Return offline page if we have it
            let offline_request = Request::new_with_str("/offline.html")?;
            if let Ok(Some(offline_response)) = get_from_cache(&cache, &offline_request).await {
                return Ok(offline_response);
            }

            Err(e)
        }
    }
}

async fn open_cache(cache_storage: &CacheStorage, name: &str) -> Result<Cache, JsValue> {
    let cache_promise = cache_storage.open(name);
    let cache = JsFuture::from(cache_promise).await?;
    Ok(cache.into())
}

async fn get_from_cache(cache: &Cache, request: &Request) -> Result<Option<Response>, JsValue> {
    let response_promise = cache.match_with_request(request);
    let response = JsFuture::from(response_promise).await?;

    if response.is_undefined() || response.is_null() {
        Ok(None)
    } else {
        Ok(Some(response.into()))
    }
}

async fn fetch_from_network(
    global: &ServiceWorkerGlobalScope,
    request: &Request,
) -> Result<Response, JsValue> {
    let response_promise = global.fetch_with_request(request);
    let response = JsFuture::from(response_promise).await?;
    Ok(response.into())
}

async fn fetch_and_cache(cache: &Cache, request: &Request) -> Result<(), JsValue> {
    // We need the global scope to fetch
    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    let response = fetch_from_network(&global, request).await?;

    if response.ok() {
        let cache_promise = cache.put_with_request(request, &response);
        JsFuture::from(cache_promise).await?;
    }

    Ok(())
}
