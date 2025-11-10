use std::sync::LazyLock;

use askama::Template;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Form,
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
    Headers, Request as JsRequest, Response as JsResponse, ResponseInit, ServiceWorkerGlobalScope,
    console,
};

#[wasm_bindgen]
pub async fn handle_fetch(request: JsRequest) -> Result<JsResponse, JsValue> {
    let global = js_sys::global().unchecked_into::<ServiceWorkerGlobalScope>();
    fetch_handler(&global, &request).await
}

static ROUTER: LazyLock<Router> = LazyLock::new(|| {
    Router::new()
        .route("/form", get(form))
        .route("/form", post(accept_form))
        .route("/hello", get(hello))
        .route("/hello", post(hello_partial))
});

#[allow(clippy::let_and_return)]
async fn app(request: Request<Body>) -> Response {
    let response = ROUTER.clone().call(request).await.unwrap();
    response
}

async fn form() -> Html<String> {
    #[derive(Debug, Template)]
    #[template(path = "form.html")]
    struct Template {}

    let temp = Template {};

    Html(temp.render().unwrap())
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

async fn hello() -> Html<String> {
    #[derive(Debug, Template)]
    #[template(path = "hello.html")]
    struct Template {}

    let temp = Template {};

    Html(temp.render().unwrap())
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct HelloInput {
    name: String,
}

async fn hello_partial(Form(input): Form<HelloInput>) -> Html<String> {
    #[derive(Debug, Template)]
    #[template(path = "hello_partial.html")]
    struct Template {
        name: String,
    }

    let temp = Template { name: input.name };

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
