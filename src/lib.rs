use std::sync::{Arc, LazyLock, Mutex};

use axum::{
    Form, Router,
    body::{Body, to_bytes},
    extract::State,
    response::{Html, Response},
    routing::{get, post},
};
use http::{Request, StatusCode, request::Builder};
use js_sys::JsString;
use serde::Deserialize;
use tower_service::Service;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Cache, CacheStorage, ExtendableMessageEvent, Headers, Request as JsRequest,
    Response as JsResponse, ResponseInit, ServiceWorkerGlobalScope, console,
};

const CACHE_NAME: &str = "my-app-cache-v1";

// Files to cache during installation
const STATIC_ASSETS: &[&str] = &[
    "/",
    "/index.html",
    "/styles.css",
    "/app.js",
    "/offline.html",
    "/htmx.min.js",
    "/htmx.js",
    "/pico.min.css",
];

#[wasm_bindgen]
pub async fn handle_install() -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();
    console::log_1(&"Service Worker installing...".into());

    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    install_handler(&global).await?;

    Ok(JsValue::NULL)
}

#[wasm_bindgen]
pub async fn handle_activate() -> Result<JsValue, JsValue> {
    console::log_1(&"Service Worker activating...".into());

    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    activate_handler(&global).await?;

    Ok(JsValue::NULL)
}

#[wasm_bindgen]
pub async fn handle_fetch(request: JsRequest) -> Result<JsResponse, JsValue> {
    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    fetch_handler(&global, &request).await
}

#[wasm_bindgen]
pub async fn handle_message(event: ExtendableMessageEvent) -> Result<JsValue, JsValue> {
    let data = event.data();
    console::log_2(&"Service Worker received message:".into(), &data);
    Ok(JsValue::NULL)
}

async fn install_handler(global: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
    let cache_storage = global.caches()?;
    let cache = open_cache(&cache_storage, CACHE_NAME).await?;

    // Cache all static assets
    for asset in STATIC_ASSETS {
        let request = JsRequest::new_with_str(asset)?;

        match fetch_and_cache(&cache, &request).await {
            Ok(_) => console::log_1(&format!("Cached: {}", asset).into()),
            Err(e) => console::error_1(&format!("Failed to cache {}: {:?}", asset, e).into()),
        }
    }

    console::log_1(&"Service Worker installed!".into());
    Ok(())
}

async fn activate_handler(global: &ServiceWorkerGlobalScope) -> Result<(), JsValue> {
    console::log_1(&"Service Worker activating...".into());

    let cache_storage = global.caches()?;
    let cache_keys = JsFuture::from(cache_storage.keys()).await?;
    let cache_keys: js_sys::Array = cache_keys.into();

    // Delete old caches
    for key in cache_keys.iter() {
        let cache_name = key.as_string().unwrap_or_default();
        if cache_name != CACHE_NAME {
            console::log_1(&format!("Deleting old cache: {}", cache_name).into());
            JsFuture::from(cache_storage.delete(&cache_name)).await?;
        }
    }

    console::log_1(&"Service Worker activated!".into());
    Ok(())
}

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<usize>>,
}

static ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/hello", get(index))
        .route("/form", post(accept_form))
        .route("/clicked", post(index))
        .with_state(AppState {
            counter: Arc::new(Mutex::new(0)),
        })
});

#[allow(clippy::let_and_return)]
async fn app(request: Request<Body>) -> Response {
    let response = ROUTER.clone().call(request).await.unwrap();
    response
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Input {
    name: String,
    email: String,
}

async fn accept_form(Form(input): Form<Input>) -> Html<String> {
    dbg!(&input);
    Html(format!(
        "email='{}'\nname='{}'\n",
        &input.email, &input.name
    ))
}

async fn index(state: State<AppState>) -> Html<String> {
    let mut counter = state.counter.lock().expect("mutex was poisoned");
    *counter += 1;

    let res = format!(
        "<div><h1>Hello, World!</h1>\n<p>Counter: {}</p></div>",
        counter
    );
    Html(res)
}

async fn fetch_handler(
    global: &ServiceWorkerGlobalScope,
    request: &JsRequest,
) -> Result<JsResponse, JsValue> {
    let body = JsFuture::from(request.text()?).await?;
    let body = Body::new(body.as_string().unwrap());
    let mut req = Builder::new()
        .uri(request.url())
        .method(request.method().as_str());

    for header in request.headers().entries() {
        let header = header?;
        let header = header.dyn_ref::<js_sys::Array>().unwrap();
        if let (Some(name), Some(val)) = (
            header.get(0).dyn_ref::<JsString>(),
            header.get(1).dyn_ref::<JsString>(),
        ) {
            req = req.header::<String, String>(name.into(), val.into());
        }
    }

    let req = req.body(body).unwrap();
    let resp = app(req).await;

    if resp.status() == StatusCode::NOT_FOUND {
        console::log_1(&"router did not match".into());
        return match fetch_from_network(global, request).await {
            Ok(response) => Ok(response),
            Err(e) => {
                console::error_1(&format!("Network fetch failed: {:?}", e).into());
                Err(e)
            }
        };
    }

    let init = ResponseInit::new();
    init.set_status(resp.status().as_u16());

    let headers = Headers::new()?;
    for (name, value) in resp.headers() {
        headers.append(name.as_str(), value.to_str().unwrap())?;
    }

    init.set_headers(&headers);
    let mut body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    let js_resp = JsResponse::new_with_opt_u8_array_and_init(Some(&mut *body), &init)?;
    Ok(js_resp)
}

async fn open_cache(cache_storage: &CacheStorage, name: &str) -> Result<Cache, JsValue> {
    let cache_promise = cache_storage.open(name);
    let cache = JsFuture::from(cache_promise).await?;
    Ok(cache.into())
}

async fn _get_from_cache(
    cache: &Cache,
    request: &JsRequest,
) -> Result<Option<JsResponse>, JsValue> {
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
    request: &JsRequest,
) -> Result<JsResponse, JsValue> {
    let response_promise = global.fetch_with_request(request);
    let response = JsFuture::from(response_promise).await?;
    Ok(response.into())
}

async fn fetch_and_cache(cache: &Cache, request: &JsRequest) -> Result<(), JsValue> {
    // We need the global scope to fetch
    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    let response = fetch_from_network(&global, request).await?;

    if response.ok() {
        let cache_promise = cache.put_with_request(request, &response);
        JsFuture::from(cache_promise).await?;
    }

    Ok(())
}
