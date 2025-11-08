use wasm_bindgen::prelude::*;
use web_sys::{ServiceWorkerGlobalScope, console};

#[wasm_bindgen(start)]
pub fn start() -> std::result::Result<(), JsValue> {
    let global = js_sys::global();
    console::log_1(&JsValue::from_str("whut"));

    if let Ok(true) = js_sys::Reflect::has(&global, &JsValue::from_str("ServiceWorkerGlobalScope"))
    {
        console::log_1(&JsValue::from_str("in service worker"));
        // we're in a service worker, so we can cast the global to a ServiceWorkerGlobalScope
        let global = global.unchecked_into::<ServiceWorkerGlobalScope>();

        // Note: install/activate handlers are registered in sw.ts to avoid async timing issues
        // They must be registered synchronously during initial script evaluation

        // register message callback
        let on_message = on_message(&global)?;
        global.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        // Ensure that the closure is not dropped before the service worker is terminated
        on_message.forget();
    } else {
        console::log_1(&JsValue::from_str("not in service worker"));
        return Err("ohno".into());
    }

    Ok(())
}

/// Displays a message in the console when a message is received from the client
fn on_message(
    _global: &ServiceWorkerGlobalScope,
) -> std::result::Result<Closure<dyn FnMut(web_sys::ExtendableMessageEvent)>, JsValue> {
    Ok(Closure::wrap(
        Box::new(move |event: web_sys::ExtendableMessageEvent| {
            console::log_2(&JsValue::from_str("sw msg:"), &event.data());
        }) as Box<dyn FnMut(_)>,
    ))
}
