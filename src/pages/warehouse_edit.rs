//! Warehouse Edit Page

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, FormInput, InputType, use_toast};
use crate::models::WarehouseForm;
use dioxus::prelude::*;

const EDIT_CSS: &str = r#"
.wh-edit-page { max-width: 800px; margin: 0 auto; }
.wh-edit-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.wh-edit-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.wh-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.wh-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.wh-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.wh-form-row > * { flex: 1; min-width: 180px; }
.wh-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.wh-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.wh-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .wh-form-row { flex-direction: column; } .wh-form-row > * { min-width: 100%; } }
"#;

#[component]
pub fn WarehouseEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;
    let parsed_id = id.parse::<i64>().unwrap_or(0);

    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_warehouse(parsed_id).await.ok()
        }
    });

    let data = resource.read().clone().flatten();

    let warehouse_code = use_signal(String::new);
    let warehouse_name = use_signal(String::new);
    let location = use_signal(String::new);
    let is_active = use_signal(|| true);
    let saving = use_signal(|| false);
    let loaded = use_signal(|| false);

    {
        let item = data.clone();
        let mut wc = warehouse_code.clone();
        let mut wn = warehouse_name.clone();
        let mut loc = location.clone();
        let mut act = is_active.clone();
        let mut ld = loaded.clone();
        use_effect(move || {
            if let Some(ref w) = item {
                if !*ld.read() {
                    wc.set(w.warehouse_code.clone());
                    wn.set(w.warehouse_name.clone());
                    loc.set(w.location.clone());
                    act.set(w.is_active);
                    ld.set(true);
                }
            }
        });
    }

    if resource.read().is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "wh-edit-page", div { class: "wh-loading", div { class: "loading-spinner" }, span { "Loading warehouse..." } } }
        };
    }
    if data.is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "wh-edit-page", div { class: "wh-loading", h2 { "Warehouse Not Found" }, Button { variant: ButtonVariant::Primary, onclick: move |_| { let _ = navigator.push("/inventory/warehouses"); }, "\u{2190} Back" } } }
        };
    }

    let save = {
        let api = api.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut saving = saving.clone();
        let wc = warehouse_code.clone();
        let wn = warehouse_name.clone();
        let loc = location.clone();
        move |_| {
            saving.set(true);
            let loc_str = loc.read().clone();
            let form = WarehouseForm {
                warehouse_code: wc.read().clone(),
                warehouse_name: wn.read().clone(),
                location: if loc_str.is_empty() { None } else { Some(loc_str) },
            };
            let api = api.clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let wn_display = wn.read().clone();
            let mut saving = saving.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                match client.update_warehouse(parsed_id, &form).await {
                    Ok(_) => {
                        toast.success("Warehouse Updated", &format!("{} updated.", wn_display));
                        nav.push(format!("/inventory/warehouses/{}", parsed_id));
                    }
                    Err(e) => { toast.error("Error", &e); saving.set(false); }
                }
            });
        }
    };

    rsx! {
        style { "{EDIT_CSS}" }
        div { class: "page wh-edit-page",
            div { class: "wh-edit-header", h1 { "Edit Warehouse" } }
            div { class: "wh-section",
                h2 { "Warehouse Information" }
                div { class: "wh-form-row",
                    FormInput { label: Some("Warehouse Code *".to_string()), value: "{warehouse_code}", placeholder: "e.g. WH-MAIN", r#type: InputType::Text, oninput: { let mut s = warehouse_code.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Warehouse Name *".to_string()), value: "{warehouse_name}", placeholder: "e.g. Main Warehouse", r#type: InputType::Text, oninput: { let mut s = warehouse_name.clone(); move |v| { s.set(v); } } }
                }
                div { class: "wh-form-row",
                    FormInput { label: Some("Location".to_string()), value: "{location}", placeholder: "e.g. Building A, Floor 2", r#type: InputType::Text, oninput: { let mut s = location.clone(); move |v| { s.set(v); } } }
                }
            }
            div { class: "wh-actions",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { let _ = navigator.push(format!("/inventory/warehouses/{}", parsed_id)); }, disabled: *saving.read(), "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *saving.read(), "Save Changes" }
            }
        }
    }
}
