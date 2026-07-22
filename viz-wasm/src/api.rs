//! The crate's one wire call: tracing runs through the ORDINARY `/api/run` (no new
//! endpoint), so this is a minimal same-origin POST with its own bearer seam. The Astro app
//! calls [`entry`](crate::entry)'s `viz_install_token` with the auth island's provider; until
//! it does, the default stays anonymous.

use std::cell::RefCell;

use serde::de::DeserializeOwned;
use synapse_shared::api::ApiError;
use synapse_shared::execution::{RunRequest, RunResult};

thread_local! {
    // Boxed, not a plain `fn` pointer: the provider is a closure over a `js_sys::Function`
    // handed across the wasm boundary — it captures, so it needs an owning box.
    static TOKEN_PROVIDER: RefCell<Box<dyn Fn() -> Option<String>>> =
        RefCell::new(Box::new(|| None));
}

/// Install the bearer provider — called once by the host's auth island (via `entry`).
pub fn set_token_provider(provider: impl Fn() -> Option<String> + 'static) {
    TOKEN_PROVIDER.with_borrow_mut(|p| *p = Box::new(provider));
}

fn bearer() -> Option<String> {
    TOKEN_PROVIDER.with_borrow(|p| p())
}

/// Run one snippet in the sandbox — a badly-running program is an `Ok(RunResult)`, exactly as
/// the server promises.
pub async fn run(request: &RunRequest) -> Result<RunResult, String> {
    crate::log::debug("POST /api/run (trace)");
    let mut req = gloo_net::http::Request::post("/api/run");
    if let Some(token) = bearer() {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }
    let response = req
        .json(request)
        .map_err(|error| error.to_string())?
        .send()
        .await
        .map_err(|error| error.to_string())?;
    decode(response).await
}

/// Non-2xx → the `ApiError` envelope's message when the server sent one, `HTTP n` otherwise.
async fn decode<T: DeserializeOwned>(response: gloo_net::http::Response) -> Result<T, String> {
    if !response.ok() {
        let fallback = format!("HTTP {}", response.status());
        return Err(match response.json::<ApiError>().await {
            Ok(envelope) => envelope
                .detail
                .map_or(envelope.error.clone(), |d| format!("{}: {d}", envelope.error)),
            Err(_) => fallback,
        });
    }
    response.json().await.map_err(|error| error.to_string())
}
