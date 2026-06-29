//! Authentication context, login page, and route guard for the frontend.
//!
//! # Architecture
//!
//! Signals MUST be created in component bodies (Dioxus hook rule), not in
//! constructors. So `AuthContext` takes existing signals, not creates them.
//!
//! - `use_auth_provider()` — Hook that creates the AuthContext (call in App())
//! - `use_auth()` — Hook to access auth state from any child
//! - `AuthGuard` — Wraps pages, redirects unauthenticated users
//! - `LoginPage` — Username/password login form

use crate::api::ApiClient;
use crate::components::common::{Button, ButtonVariant, FormInput, InputType};
use crate::models::*;
use dioxus::prelude::*;

// ============================================================================
// Auth State
// ============================================================================

/// Authentication state shared across the app via Dioxus context.
///
/// IMPORTANT: Do NOT call `use_signal` inside any method — Dioxus hooks
/// must be called directly in the component body. Use `use_auth_provider()`
/// to create this context.
#[derive(Clone)]
pub struct AuthContext {
    /// Current user profile (None = not logged in).
    pub user: Signal<Option<UserProfile>>,
    /// Whether an auth check is in progress (e.g. verifying stored token).
    pub is_loading: Signal<bool>,
    /// The API client instance (shared across the app).
    pub api: Signal<ApiClient>,
}

impl AuthContext {
    /// Create auth context from existing signals (called from component body).
    pub fn from_signals(
        user: Signal<Option<UserProfile>>,
        is_loading: Signal<bool>,
        api: Signal<ApiClient>,
    ) -> Self {
        Self {
            user,
            is_loading,
            api,
        }
    }

    /// Log in with username and password. Returns Ok(()) on success.
    ///
    /// When `persist` is true, the token is persisted to localStorage ("remember me").
    /// When false, the token is kept only in memory for the current session.
    pub async fn login(&self, username: &str, password: &str, persist: bool) -> Result<(), String> {
        let req = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };
        let mut api = self.api.clone();
        let mut user = self.user.clone();
        let resp = api.read().login(&req).await?;

        // Store token (in-memory always, localStorage only if persist)
        api.write().set_token(Some(resp.token.clone()));
        if persist {
            store_token(&resp.token);
        }

        // Set user
        user.set(Some(resp.user));
        Ok(())
    }

    /// Log out — clears token and user state.
    pub async fn logout(&self) {
        let mut api = self.api.clone();
        let mut user = self.user.clone();
        let _ = api.read().logout().await;
        api.write().set_token(None);
        user.set(None);
        clear_stored_token();
    }
}

// ============================================================================
// Provider Hook (call from App() component)
// ============================================================================

/// Create the auth context provider signals and restore session.
/// Must be called inside a component (e.g. App() root).
pub fn use_auth_provider() -> AuthContext {
    let user = use_signal(|| None::<UserProfile>);
    let is_loading = use_signal(|| true);
    let api = use_signal(ApiClient::new);

    // Restore session from localStorage (runs once on mount)
    let restore_user = user.clone();
    let restore_loading = is_loading.clone();
    let restore_api = api.clone();
    use_effect(move || {
        let user = restore_user.clone();
        let loading = restore_loading.clone();
        let api = restore_api.clone();

        spawn(async move {
            let mut user = user.clone();
            let mut loading = loading.clone();
            let mut api = api.clone();
            // Check localStorage for stored token
            let stored = get_stored_token();
            if let Some(token) = stored {
                api.write().set_token(Some(token));
                // Verify token with /me endpoint
                let me_result = api.read().me().await;
                match me_result {
                    Ok(profile) => {
                        user.set(Some(profile));
                    }
                    Err(_) => {
                        // Token expired or invalid — clear it
                        api.write().set_token(None);
                        clear_stored_token();
                    }
                }
            }
            loading.set(false);
        });
    });

    AuthContext::from_signals(user, is_loading, api)
}

// ============================================================================
// localStorage helpers (using web-sys)
// ============================================================================

#[cfg(target_arch = "wasm32")]
fn get_stored_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("mini_erp_token").ok()?
}

#[cfg(not(target_arch = "wasm32"))]
fn get_stored_token() -> Option<String> {
    None
}

#[cfg(target_arch = "wasm32")]
fn store_token(token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("mini_erp_token", token);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn store_token(_token: &str) {}

#[cfg(target_arch = "wasm32")]
fn clear_stored_token() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("mini_erp_token");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn clear_stored_token() {}

// ============================================================================
// use_auth Hook
// ============================================================================

/// Hook to access the auth context from any child component.
pub fn use_auth() -> AuthContext {
    consume_context::<AuthContext>()
}

// ============================================================================
// Login Page CSS
// ============================================================================

/// CSS styles for the login page — dark gradient background, centered card,
/// smooth animations, responsive layout.
pub const LOGIN_CSS: &str = r##"
/* ── Login Page ── */
.login-page {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    min-height: 100dvh;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
    padding: 20px;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, sans-serif;
}

.login-container {
    width: 100%;
    max-width: 400px;
    background: #ffffff;
    border-radius: 16px;
    box-shadow: 0 25px 80px rgba(0, 0, 0, 0.35), 0 0 0 1px rgba(255, 255, 255, 0.05);
    padding: 40px 36px 36px;
    animation: login-card-in 0.5s ease-out;
    position: relative;
    overflow: hidden;
}

.login-container::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 4px;
    background: linear-gradient(90deg, #4a90d9, #50c878, #4a90d9);
    background-size: 200% 100%;
    animation: login-shimmer 3s ease-in-out infinite;
}

@keyframes login-shimmer {
    0%, 100% { background-position: 0% 0%; }
    50% { background-position: 100% 0%; }
}

@keyframes login-card-in {
    from {
        opacity: 0;
        transform: translateY(24px) scale(0.96);
    }
    to {
        opacity: 1;
        transform: translateY(0) scale(1);
    }
}

.login-header {
    text-align: center;
    margin-bottom: 32px;
}

.login-logo {
    font-size: 32px;
    font-weight: 800;
    background: linear-gradient(135deg, #4a90d9, #50c878);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    letter-spacing: -0.5px;
    margin-bottom: 6px;
}

.login-subtitle {
    font-size: 14px;
    color: #6c757d;
    font-weight: 400;
}

.login-error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: #fff0f0;
    border: 1px solid #ffc0c0;
    border-radius: 8px;
    color: #dc3545;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 20px;
    animation: login-shake 0.4s ease;
}

@keyframes login-shake {
    0%, 100% { transform: translateX(0); }
    20% { transform: translateX(-6px); }
    40% { transform: translateX(6px); }
    60% { transform: translateX(-4px); }
    80% { transform: translateX(4px); }
}

.login-form {
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.login-form .cb-input-group {
    gap: 6px;
}

.login-form .cb-input-label {
    font-size: 13px;
    font-weight: 600;
    color: #374151;
}

.login-form .cb-input {
    padding: 10px 14px;
    border-radius: 8px;
    border: 1.5px solid #e5e7eb;
    transition: all 0.15s ease;
}

.login-form .cb-input:hover {
    border-color: #d1d5db;
}

.login-form .cb-input:focus-within {
    border-color: #4a90d9;
    box-shadow: 0 0 0 3px rgba(74, 144, 217, 0.15);
}

.login-form .cb-input input {
    font-size: 14px;
    color: #1f2937;
}

.login-form .cb-input input::placeholder {
    color: #9ca3af;
}

/* Password field wrapper for toggle button */
.login-password-wrapper {
    position: relative;
}

.login-password-toggle {
    position: absolute;
    right: 2px;
    top: 2px;
    bottom: 2px;
    display: flex;
    align-items: center;
    padding: 0 10px;
    border: none;
    background: transparent;
    cursor: pointer;
    color: #9ca3af;
    font-size: 14px;
    border-radius: 6px;
    transition: all 0.15s ease;
}

.login-password-toggle:hover {
    color: #4a90d9;
    background: rgba(74, 144, 217, 0.08);
}

.login-options {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: -4px;
}

.login-remember {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    color: #6c757d;
    user-select: none;
}

.login-remember input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: #4a90d9;
    cursor: pointer;
    border-radius: 3px;
}

.login-remember:hover {
    color: #374151;
}

.login-submit-btn {
    width: 100%;
    padding: 12px !important;
    font-size: 15px !important;
    font-weight: 600 !important;
    border-radius: 8px !important;
    margin-top: 4px;
    letter-spacing: 0.02em;
}

.login-submit-btn:not(:disabled):hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(74, 144, 217, 0.3);
}

.login-submit-btn:not(:disabled):active {
    transform: translateY(0);
}

.login-footer {
    margin-top: 24px;
    padding-top: 20px;
    border-top: 1px solid #f0f0f0;
    text-align: center;
}

.login-footer p {
    margin: 0;
    font-size: 12px;
    color: #adb5bd;
    line-height: 1.6;
}

.login-footer .login-demo-label {
    display: inline-block;
    padding: 2px 8px;
    background: #e8f0fe;
    color: #4a90d9;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    margin-bottom: 6px;
}

/* Loading state */
.login-page-loader {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    min-height: 100dvh;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
}

.login-page-loader .login-container {
    text-align: center;
    padding: 48px 36px;
}

.login-loader-spinner {
    width: 36px;
    height: 36px;
    border: 3px solid #e5e7eb;
    border-top-color: #4a90d9;
    border-radius: 50%;
    animation: login-spin 0.7s linear infinite;
    margin: 16px auto;
}

@keyframes login-spin {
    to { transform: rotate(360deg); }
}

.login-loader-text {
    font-size: 14px;
    color: #6c757d;
}

/* Responsive */
@media (max-width: 480px) {
    .login-page {
        padding: 0;
        align-items: flex-end;
    }

    .login-container {
        max-width: 100%;
        border-radius: 16px 16px 0 0;
        padding: 32px 24px 28px;
        animation: login-card-up 0.4s ease-out;
    }

    @keyframes login-card-up {
        from { transform: translateY(100%); }
        to { transform: translateY(0); }
    }

    .login-logo {
        font-size: 28px;
    }
}
"##;

// ============================================================================
// LoginPage Component
// ============================================================================

/// Polished login page with username/password form, error display, loading
/// state, password visibility toggle, and "remember me" checkbox.
#[component]
pub fn LoginPage() -> Element {
    let auth = use_auth();
    let navigator = use_navigator();
    let is_loading_auth = *auth.is_loading.read();

    // Show the auth loading state while restoring session
    if is_loading_auth {
        return rsx! {
            div { class: "login-page-loader",
                div { class: "login-container",
                    div { class: "login-logo", "MiniERP" }
                    div { class: "login-loader-spinner" }
                    p { class: "login-loader-text", "Checking session…" }
                }
            }
        };
    }

    // If already logged in, redirect via sub-component
    if auth.user.read().is_some() {
        return rsx! {
            RedirectDashboard { }
        };
    }

    // ── Form state ──
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| Option::<String>::None);
    let mut is_submitting = use_signal(|| false);
    let mut show_password = use_signal(|| false);
    let mut remember_me = use_signal(|| true);

    // ── Submit handler ──
    let on_submit = move |e: Event<FormData>| {
        e.prevent_default();
        let auth = auth.clone();
        let u = username.read().clone();
        let p = password.read().clone();
        let mut e = error.clone();
        let mut s = is_submitting.clone();
        let nav = navigator.clone();

        // Validate
        if u.trim().is_empty() && p.is_empty() {
            e.set(Some("Please enter your username and password.".to_string()));
            return;
        }
        if u.trim().is_empty() {
            e.set(Some("Please enter your username.".to_string()));
            return;
        }
        if p.is_empty() {
            e.set(Some("Please enter your password.".to_string()));
            return;
        }

        s.set(true);
        e.set(None);

        let persist = *remember_me.read();

        spawn(async move {
            match auth.login(&u, &p, persist).await {
                Ok(()) => {
                    s.set(false);
                    nav.push("/");
                }
                Err(msg) => {
                    s.set(false);
                    e.set(Some(msg));
                }
            }
        });
    };

    let password_type = if *show_password.read() {
        InputType::Text
    } else {
        InputType::Password
    };

    rsx! {
        div { class: "login-page",
            div { class: "login-container",
                // Header / Brand
                div { class: "login-header",
                    div { class: "login-logo", "MiniERP" }
                    div { class: "login-subtitle", "Enterprise Resource Planning" }
                }

                // Error message with shake animation
                if let Some(msg) = error.read().as_ref() {
                    div { class: "login-error",
                        span { "⚠" }
                        span { "{msg}" }
                    }
                }

                // Login form
                form {
                    class: "login-form",
                    onsubmit: on_submit,

                    // Username field
                    FormInput {
                        label: Some("Username".to_string()),
                        value: username.read().clone(),
                        oninput: move |v| username.set(v),
                        r#type: InputType::Text,
                        placeholder: Some("Enter your username".to_string()),
                        disabled: *is_submitting.read(),
                        autocomplete: Some("username".to_string()),
                        icon: Some("👤".to_string()),
                    }

                    // Password field with show/hide toggle
                    div { class: "login-password-wrapper",
                        FormInput {
                            label: Some("Password".to_string()),
                            value: password.read().clone(),
                            oninput: move |v| password.set(v),
                            r#type: password_type.clone(),
                            placeholder: Some("Enter your password".to_string()),
                            disabled: *is_submitting.read(),
                            autocomplete: Some("current-password".to_string()),
                            icon: Some("🔒".to_string()),
                        }
                        button {
                            class: "login-password-toggle",
                            r#type: "button",
                            onclick: move |_| {
                                let val = *show_password.read();
                                show_password.set(!val);
                            },
                            tabindex: "-1",
                            if *show_password.read() {
                                "🙈"
                            } else {
                                "👁"
                            }
                        }
                    }

                    // Remember me + Forgot password
                    div { class: "login-options",
                        label { class: "login-remember",
                            input {
                                r#type: "checkbox",
                                checked: *remember_me.read(),
                                oninput: move |e| remember_me.set(e.checked()),
                            }
                            span { "Remember me" }
                        }
                    }

                    // Submit button
                    Button {
                        variant: ButtonVariant::Primary,
                        r#type: "submit".to_string(),
                        loading: *is_submitting.read(),
                        disabled: *is_submitting.read(),
                        block: true,
                        class: Some("login-submit-btn".to_string()),
                        if *is_submitting.read() {
                            "Signing in…"
                        } else {
                            "Sign In"
                        }
                    }
                }

                // Footer with demo credentials
                div { class: "login-footer",
                    span { class: "login-demo-label", "Demo Credentials" }
                    p { "Username: admin  •  Password: admin123" }
                }
            }
        }
    }
}

/// Helper: redirect to dashboard via side-effect.
#[component]
fn RedirectDashboard() -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.push("/");
    });
    rsx! { div { } }
}
