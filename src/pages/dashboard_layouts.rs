//! Dashboard Layouts Page — Manage saved dashboard layouts.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, Modal, ModalSize, use_toast};
use dioxus::prelude::*;

const PAGE_CSS: &str = r#"
.dl-page { max-width: 900px; margin: 0 auto; padding: 20px; }
.dl-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.dl-header h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.dl-table-container { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; overflow: hidden; }
.dl-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.dl-table thead th { text-align: left; padding: 10px 12px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); background: var(--bg-muted, #f8f9fa); border-bottom: 2px solid var(--border-color); }
.dl-table tbody td { padding: 10px 12px; border-bottom: 1px solid var(--border-color); color: var(--text-primary); }
.dl-table tbody tr:last-child td { border-bottom: none; }
.dl-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.dl-active-badge { display: inline-flex; align-items: center; padding: 2px 8px; border-radius: 10px; font-size: 11px; font-weight: 600; }
.dl-active-yes { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.dl-active-no { background: rgba(108, 117, 125, 0.1); color: #6c757d; }
.dl-empty { text-align: center; padding: 40px 20px; color: var(--text-secondary); font-size: 14px; }
.dl-actions { display: flex; gap: 6px; }
.dl-form-modal { display: flex; flex-direction: column; gap: 14px; }
.dl-field { display: flex; flex-direction: column; gap: 4px; }
.dl-field label { font-size: 12px; font-weight: 600; color: var(--text-secondary); }
.dl-field input, .dl-field textarea { padding: 8px 10px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; font-size: 13px; background: #fff; color: var(--text-primary); font-family: monospace; }
.dl-field textarea { min-height: 100px; resize: vertical; }
"#;

#[derive(Clone, Debug)]
struct LayoutItem {
    id: i64,
    name: String,
    blocks: String,
    is_active: bool,
    created_at: String,
}

#[component]
pub fn DashboardLayoutsPage() -> Element {
    let navigator = use_navigator();
    let mut toast = use_toast();
    let api = use_auth().api;
    let counter = use_signal(|| 0u32);

    let mut show_create_modal = use_signal(|| false);
    let mut show_delete_modal = use_signal(|| false);
    let mut delete_id = use_signal(|| 0i64);
    let mut new_name = use_signal(|| String::new());
    let mut new_blocks = use_signal(|| "{}".to_string());

    let resource = use_resource(move || {
        let api = api.clone();
        let _ = *counter.read();
        async move {
            let client = api.read().clone();
            match client.list_dashboard_layouts().await {
                Ok(layouts) => layouts.into_iter().map(|l| LayoutItem {
                    id: l.id,
                    name: l.layout_name,
                    blocks: l.blocks,
                    is_active: l.is_active,
                    created_at: l.created_at,
                }).collect(),
                Err(_) => vec![],
            }
        }
    });

    let is_loading = resource.read().is_none();
    let layouts = resource.read().cloned().unwrap_or_default();

    let create_layout = {
        let mut toast = toast.clone();
        move |_| {
            let name = new_name.read().clone();
            let blocks = new_blocks.read().clone();
            if name.is_empty() {
                toast.error("Error", "Layout name is required.");
                return;
            }
            let api = api.clone();
            let mut toast = toast.clone();
            let mut counter = counter.clone();
            let mut show = show_create_modal.clone();
            let mut name_sig = new_name.clone();
            let mut blocks_sig = new_blocks.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.create_dashboard_layout(&name, &blocks).await {
                    Ok(_) => {
                        toast.success("Created", "Layout created.");
                        show.set(false);
                        name_sig.set(String::new());
                        blocks_sig.set("{}".to_string());
                        let current = *counter.read();
                        counter.set(current + 1);
                    }
                    Err(e) => toast.error("Error", &e),
                }
            });
        }
    };

    let confirm_delete = move |_| {
        let id = *delete_id.read();
        let api = api.clone();
        let mut toast = toast.clone();
        let mut counter = counter.clone();
        let mut show = show_delete_modal.clone();
        spawn(async move {
            let client = api.read().clone();
            match client.delete_dashboard_layout(id).await {
                Ok(_) => {
                    toast.success("Deleted", "Layout deleted.");
                    show.set(false);
                    let current = *counter.read();
                    counter.set(current + 1);
                }
                Err(e) => toast.error("Error", &e),
            }
        });
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "dl-page",
            div { class: "dl-header",
                h1 { "Dashboard Layouts" }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| show_create_modal.set(true),
                    "＋ New Layout"
                }
            }

            if is_loading {
                div { class: "dl-empty", "Loading..." }
            } else if layouts.is_empty() {
                div { class: "dl-empty", "No dashboard layouts found. Create one to customize your dashboard." }
            } else {
                div { class: "dl-table-container",
                    table { class: "dl-table",
                        thead { tr {
                            th { "Name" }
                            th { "Active" }
                            th { "Created" }
                            th { "Blocks Preview" }
                            th { style: "text-align: right;", "Actions" }
                        }}
                        tbody {
                            for layout in layouts.iter() {
                                {let layout_clone = layout.clone(); rsx! {
                                    tr {
                                        td { style: "font-weight: 600;", "{layout.name}" }
                                        td {
                                            if layout.is_active {
                                                span { class: "dl-active-badge dl-active-yes", "✓ Active" }
                                            } else {
                                                span { class: "dl-active-badge dl-active-no", "Inactive" }
                                            }
                                        }
                                        td { style: "font-size: 12px; color: var(--text-secondary);", "{layout.created_at}" }
                                        td { style: "font-family: monospace; font-size: 11px; color: var(--text-secondary); max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                            span { "JSON config" }
                                        }
                                        td {
                                            div { class: "dl-actions",
                                                Button {
                                                    variant: ButtonVariant::Ghost,
                                                    onclick: { let mut nav = navigator.clone(); let eid = layout.id; move |_| { nav.push(format!("/dashboard?layout={}", eid)); } },
                                                    "View"
                                                }
                                                Button {
                                                    variant: ButtonVariant::Ghost,
                                                    onclick: { let mut did = delete_id.clone(); let lid = layout.id; move |_| { did.set(lid); show_delete_modal.set(true); } },
                                                    "Delete"
                                                }
                                            }
                                        }
                                    }
                                }}
                            }
                        }
                    }
                }
            }

            Modal {
                is_open: show_create_modal,
                title: Some("Create Dashboard Layout".to_string()),
                size: ModalSize::Md,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| show_create_modal.set(false), "Cancel" }
                    Button { variant: ButtonVariant::Primary, onclick: create_layout, "Create Layout" }
                },
                div { class: "dl-form-modal",
                    div { class: "dl-field",
                        label { "Layout Name" }
                        input {
                            r#type: "text",
                            placeholder: "e.g. Sales Dashboard",
                            value: "{new_name}",
                            onchange: move |e| new_name.set(e.value()),
                        }
                    }
                    div { class: "dl-field",
                        label { "Blocks Configuration (JSON)" }
                        textarea {
                            placeholder: "Enter JSON configuration",
                            value: "{new_blocks}",
                            onchange: move |e| new_blocks.set(e.value()),
                        }
                    }
                }
            }

            Modal {
                is_open: show_delete_modal,
                title: Some("Delete Layout".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| show_delete_modal.set(false), "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete" }
                },
                div {
                    p { style: "margin: 0; color: var(--text-primary); font-size: 14px;", "Are you sure you want to delete this layout?" }
                    p { style: "margin: 8px 0 0; color: var(--text-secondary); font-size: 13px;", "This action cannot be undone." }
                }
            }
        }
    }
}
