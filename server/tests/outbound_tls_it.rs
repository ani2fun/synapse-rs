//! The outbound HTTPS guard — network-free, and it exists because its absence reached production.
//!
//! `reqwest` is declared with `default-features = false`, which drops the TLS backend unless one is
//! asked for explicitly. Nothing in the test suite or the dev loop noticed: go-judge, Ollama and the
//! dev Keycloak are all plain `http://`, so the ONLY `https://` caller is the JWKS fetch against the
//! production issuer. Sign-in returned 503 on the live site — "error sending request for url
//! (…/protocol/openid-connect/certs)" — while every test stayed green.
//!
//! The tell is three levels down the error chain: without a TLS backend hyper's connector rejects
//! the scheme outright with `invalid URL, scheme is not http`. With one, the same request gets far
//! enough to fail at the TLS handshake instead. That distinction needs no network and no
//! certificates — just a listener that accepts TCP and then says nothing.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::error::Error;

/// Accepts TCP and speaks nothing. The connect therefore SUCCEEDS, so any failure comes from the
/// TLS layer — or from its absence, which is the thing under test.
async fn silent_tcp_listener() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let _ = listener.accept().await;
        }
    });
    port
}

#[tokio::test]
async fn the_http_client_can_speak_https() {
    let port = silent_tcp_listener().await;

    let error = reqwest::Client::new()
        .get(format!("https://127.0.0.1:{port}/"))
        .send()
        .await
        .expect_err("a silent listener cannot complete a TLS handshake");

    let mut chain = vec![error.to_string()];
    let mut source: Option<&dyn Error> = error.source();
    while let Some(current) = source {
        chain.push(current.to_string());
        source = current.source();
    }
    let chain = chain.join(" | ");

    assert!(
        !chain.contains("scheme is not http"),
        "reqwest has NO TLS backend compiled in — every https:// call will fail, including the \
         JWKS fetch that gates sign-in. Add a TLS feature (rustls-tls) in the workspace Cargo.toml.\n\
         error chain: {chain}"
    );
}
