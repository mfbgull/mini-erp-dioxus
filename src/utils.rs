use std::time::Duration;

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(_duration: Duration) {
    // No-op on native — these are just fake loading delays for mock data.
    // gloo_timers and tokio::time::sleep both require async runtimes
    // that Dioxus desktop doesn't provide.
}
