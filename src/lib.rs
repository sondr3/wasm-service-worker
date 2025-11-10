use std::sync::{Arc, LazyLock, Mutex};

use askama::Template;
use axum::{
    Form, Router,
    body::{Body, to_bytes},
    extract::{Path, State},
    response::{Html, Response},
    routing::post,
};
use http::{Request, StatusCode, request::Builder};
use js_sys::JsString;
use serde::Deserialize;
use tower_service::Service;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Headers, Request as JsRequest, Response as JsResponse, ResponseInit, ServiceWorkerGlobalScope,
    console,
};

#[wasm_bindgen]
pub async fn handle_fetch(request: JsRequest) -> Result<JsResponse, JsValue> {
    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    fetch_handler(&global, &request).await
}

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<usize>>,
}

static ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/form", post(accept_form))
        .route("/{name}/clicked", post(index))
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

async fn index(Path((name,)): Path<(String,)>, state: State<AppState>) -> Html<String> {
    let mut counter = state.counter.lock().expect("mutex was poisoned");
    *counter += 1;

    #[derive(Debug, Template)]
    #[template(path = "hello.html")]
    struct Template {
        name: String,
        counter: usize,
    }

    let temp = Template {
        name,
        counter: *counter,
    };

    Html(temp.render().unwrap())
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

async fn fetch_from_network(
    global: &ServiceWorkerGlobalScope,
    request: &JsRequest,
) -> Result<JsResponse, JsValue> {
    let response_promise = global.fetch_with_request(request);
    let response = JsFuture::from(response_promise).await?;
    Ok(response.into())
}
