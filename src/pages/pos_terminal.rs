//! POS Terminal Page — Simplified point-of-sale terminal with item search,
//! cart management, number pad, and checkout.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, StatCard, StatCardVariant,
};
use crate::models::Item;
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const DEFAULT_TAX_RATE: f64 = 16.0;

const PAGE_CSS: &str = r##"
.pos-page { max-width: 1200px; margin: 0 auto; }
.pos-page h1 { font-size: 22px; font-weight: 700; margin: 0 0 16px 0; color: var(--text-primary); }

.pos-layout { display: grid; grid-template-columns: 1fr 400px; gap: 16px; align-items: start; }

/* ── Left Panel: Item Search & Cart ── */
.pos-left { display: flex; flex-direction: column; gap: 16px; }

.pos-search-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; }
.pos-search-section h2 { font-size: 14px; font-weight: 600; margin: 0 0 12px 0; color: var(--text-primary); }

.pos-cart-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; flex: 1; }
.pos-cart-section h2 { font-size: 14px; font-weight: 600; margin: 0 0 12px 0; color: var(--text-primary); display: flex; justify-content: space-between; align-items: center; }

.pos-cart-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.pos-cart-table thead th { text-align: left; padding: 6px 8px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); }
.pos-cart-table thead th.text-right { text-align: right; }
.pos-cart-table tbody td { padding: 6px 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.pos-cart-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.pos-cart-table tbody tr:last-child td { border-bottom: none; }
.pos-cart-empty { text-align: center; padding: 40px 20px; color: var(--text-secondary); font-size: 14px; }

.pos-remove-btn { border: none; background: transparent; cursor: pointer; color: var(--danger, #dc3545); font-size: 14px; padding: 2px 6px; border-radius: 4px; transition: background 0.15s; line-height: 1; }
.pos-remove-btn:hover { background: rgba(220, 53, 69, 0.1); }

/* ── Right Panel: Totals & Controls ── */
.pos-right { display: flex; flex-direction: column; gap: 16px; }

.pos-totals-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; }
.pos-totals-section h2 { font-size: 14px; font-weight: 600; margin: 0 0 12px 0; color: var(--text-primary); }

.pos-totals-grid { display: flex; flex-direction: column; gap: 8px; }

.pos-numpad-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; }
.pos-numpad-section h2 { font-size: 14px; font-weight: 600; margin: 0 0 12px 0; color: var(--text-primary); }

.pos-numpad { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
.pos-numpad-btn { padding: 14px 0; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius-sm, 4px); background: #fff; font-size: 16px; font-weight: 600; cursor: pointer; transition: all 0.1s; text-align: center; color: var(--text-primary); }
.pos-numpad-btn:hover { background: var(--bg-muted, #f5f5f5); border-color: var(--accent); }
.pos-numpad-btn:active { transform: scale(0.96); }
.pos-numpad-btn.pos-numpad-clear { background: rgba(220, 53, 69, 0.08); color: #dc3545; border-color: #dc3545; }
.pos-numpad-btn.pos-numpad-enter { background: var(--accent, #4a90d9); color: #fff; border-color: var(--accent, #4a90d9); }

.pos-item-quick { display: flex; flex-direction: column; gap: 8px; }
.pos-quick-buttons { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; }
.pos-quick-btn { padding: 10px 6px; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius-sm, 4px); background: var(--bg-muted, #f5f5f5); font-size: 12px; font-weight: 500; cursor: pointer; text-align: center; color: var(--text-primary); transition: all 0.1s; }
.pos-quick-btn:hover { background: #fff; border-color: var(--accent); }

.pos-actions { display: flex; gap: 8px; }
.pos-actions > * { flex: 1; }

.pos-customer-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; }
.pos-customer-section h2 { font-size: 14px; font-weight: 600; margin: 0 0 12px 0; color: var(--text-primary); }

.pos-txn-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; }
.pos-txn-section h2 { font-size: 14px; font-weight: 600; margin: 0 0 12px 0; color: var(--text-primary); }
.pos-txn-table { width: 100%; border-collapse: collapse; font-size: 12px; }
.pos-txn-table thead th { text-align: left; padding: 4px 6px; font-weight: 600; font-size: 10px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 1px solid var(--border-color, #e0e0e0); }
.pos-txn-table tbody td { padding: 4px 6px; color: var(--text-primary); }

@media (max-width: 900px) {
    .pos-layout { grid-template-columns: 1fr; }
    .pos-cart-table { font-size: 12px; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Debug)]
struct PosCartItem {
    id: u64,
    item_id: i64,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn PosTerminalPage() -> Element {
    let toast = crate::components::common::use_toast();

    let cart = use_signal(Vec::<PosCartItem>::new);
    let search_query = use_signal(String::new);
    let customer_name = use_signal(String::new);
    let payment_method = use_signal(|| "Cash".to_string());

    let next_id = use_signal(|| 1u64);

    let api = use_auth().api;

    let catalog_items = use_signal(Vec::<Item>::new);
    let search_results = use_signal(Vec::<serde_json::Value>::new);
    let is_searching = use_signal(|| false);
    let recent_transactions = use_signal(Vec::<serde_json::Value>::new);

    {
        let api = api.clone();
        let catalog_items = catalog_items.clone();
        let recent_transactions = recent_transactions.clone();
        use_effect(move || {
            let api = api.clone();
            let mut catalog_items = catalog_items.clone();
            let mut recent_transactions = recent_transactions.clone();
            spawn(async move {
                let client = api.read().clone();
                if let Ok(items) = client.list_items_catalog().await {
                    catalog_items.set(items);
                }
                if let Ok(txns) = client.list_pos_transactions().await {
                    recent_transactions.set(txns);
                }
            });
        });
    }

    let on_search = {
        let mut search_query = search_query.clone();
        let mut search_results = search_results.clone();
        let is_searching = is_searching.clone();
        let api = api.clone();
        move |v: String| {
            search_query.set(v.clone());
            if v.len() < 2 {
                search_results.set(vec![]);
                return;
            }
            let api = api.clone();
            let mut search_results = search_results.clone();
            let mut is_searching = is_searching.clone();
            is_searching.set(true);
            let query = v.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.search_items(&query).await {
                    Ok(results) => { search_results.set(results); }
                    Err(_) => { search_results.set(vec![]); }
                }
                is_searching.set(false);
            });
        }
    };

    let add_item = {
        let mut c = cart.clone();
        let mut nid = next_id.clone();
        move |item_id: i64, code: &str, name: &str, price: f64| {
            let id = *nid.read();
            *nid.write() += 1;

            let mut w = c.write();
            if let Some(existing) = w.iter_mut().find(|i| i.item_code == code) {
                existing.quantity += 1.0;
            } else {
                w.push(PosCartItem {
                    id,
                    item_id,
                    item_code: code.to_string(),
                    item_name: name.to_string(),
                    quantity: 1.0,
                    unit_price: price,
                });
            }
        }
    };

    let remove_item = {
        let mut c = cart.clone();
        move |id: u64| {
            c.write().retain(|i| i.id != id);
        }
    };

    let clear_cart = {
        let mut c = cart.clone();
        move |_| {
            c.write().clear();
        }
    };

    let subtotal: f64 = cart.read().iter().map(|i| i.quantity * i.unit_price).sum();
    let tax_amount = subtotal * DEFAULT_TAX_RATE / 100.0;
    let grand_total = subtotal + tax_amount;
    let item_count: usize = cart.read().len();

    let on_checkout = {
        let mut toast = toast.clone();
        let cart = cart.clone();
        let api = api.clone();
        let customer_name = customer_name.clone();
        let payment_method = payment_method.clone();
        let recent_transactions = recent_transactions.clone();
        move |_| {
            if cart.read().is_empty() {
                toast.error("Cart Empty", "Add items to cart before checkout.");
                return;
            }
            let cart_items: Vec<PosCartItem> = cart.read().iter().cloned().collect();
            let cust = customer_name.read().clone();
            let pm = payment_method.read().clone();
            let total = grand_total;
            let api = api.clone();
            let mut toast = toast.clone();
            let mut cart = cart.clone();
            let mut recent_transactions = recent_transactions.clone();
            spawn(async move {
                let items_json: Vec<serde_json::Value> = cart_items.iter().map(|ci| {
                    serde_json::json!({
                        "item_id": ci.item_id,
                        "quantity": ci.quantity,
                        "unit_price": ci.unit_price,
                    })
                }).collect();
                let form = serde_json::json!({
                    "customer_name": if cust.is_empty() { "Walk-in Customer".to_string() } else { cust },
                    "items": items_json,
                    "payment_method": pm,
                    "paid_amount": total,
                });
                let client = api.read().clone();
                match client.create_pos_sale(&form).await {
                    Ok(_result) => {
                        let count = cart.read().len();
                        toast.success("Checkout Complete", &format!("Sale completed — {} item(s).", count));
                        cart.write().clear();
                        if let Ok(txns) = client.list_pos_transactions().await {
                            recent_transactions.set(txns);
                        }
                    }
                    Err(e) => {
                        toast.error("Checkout Failed", &e);
                    }
                }
            });
        }
    };

    let quick_items_list: Vec<Item> = catalog_items.read().iter()
        .filter(|i| i.is_active)
        .take(8)
        .cloned()
        .collect();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page pos-page",
            h1 { "POS Terminal" }

            div { class: "pos-layout",

                div { class: "pos-left",

                    div { class: "pos-search-section",
                        h2 { "Search Items" }
                        FormInput {
                            value: search_query.read().clone(),
                            oninput: on_search,
                            r#type: InputType::Text,
                            placeholder: Some("Type item name or code…".to_string()),
                        }
                        if !search_results.read().is_empty() {
                            div { style: "margin-top: 8px; display: flex; flex-direction: column; gap: 4px;",
                                {search_results.read().iter().map(|sr| {
                                    let item_id = sr["id"].as_i64().unwrap_or(0);
                                    let code = sr["item_code"].as_str().unwrap_or("").to_string();
                                    let name = sr["item_name"].as_str().unwrap_or("").to_string();
                                    let price = sr["selling_price"].as_f64().unwrap_or(0.0);
                                    let code2 = code.clone();
                                    let name2 = name.clone();
                                    let on_select = {
                                        let mut a = add_item.clone();
                                        move |_| a(item_id, &code2, &name2, price)
                                    };
                                    rsx! {
                                        button {
                                            key: "{code}",
                                            r#type: "button",
                                            style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; background: #fff; cursor: pointer; font-size: 13px; text-align: left;",
                                            onclick: on_select,
                                            span { span { style: "font-weight: 600;", "{name}" } span { style: "color: var(--text-secondary); margin-left: 8px;", "{code}" } }
                                            span { style: "font-weight: 600; color: var(--accent);", "PKR {price:.2}" }
                                        }
                                    }
                                })}
                            }
                        } else if search_query.read().len() >= 2 && *is_searching.read() {
                            div { style: "margin-top: 8px; color: var(--text-secondary); font-size: 13px;", "Searching…" }
                        }
                    }

                    div { class: "pos-cart-section",
                        h2 {
                            span { "Cart" }
                            span { style: "font-size: 12px; color: var(--text-secondary); font-weight: 400;", "{item_count} item(s)" }
                        }
                        if cart.read().is_empty() {
                            div { class: "pos-cart-empty", "Cart is empty. Search or tap quick items to add." }
                        } else {
                            div { style: "overflow-x: auto;",
                                table { class: "pos-cart-table",
                                    thead { tr {
                                        th { "Item" } th { class: "text-right", "Qty" }
                                        th { class: "text-right", "Price" } th { class: "text-right", "Total" }
                                        th { style: "width: 30px;" }
                                    }}
                                    tbody {
                                        {cart.read().iter().map(|ci| {
                                            let total = ci.quantity * ci.unit_price;
                                            let rid = ci.id;
                                            let on_rem = {
                                                let mut r = remove_item.clone();
                                                move |_| r(rid)
                                            };
                                            rsx! {
                                                tr {
                                                    td { "{ci.item_name}" }
                                                    td { class: "text-right", "{ci.quantity}" }
                                                    td { class: "text-right", "PKR {ci.unit_price:.2}" }
                                                    td { class: "text-right", "PKR {total:.2}" }
                                                    td { button { class: "pos-remove-btn", r#type: "button", onclick: on_rem, "x" } }
                                                }
                                            }
                                        })}
                                    }
                                }
                            }
                        }
                    }

                    div { class: "pos-txn-section",
                        h2 { "Recent Transactions" }
                        if recent_transactions.read().is_empty() {
                            div { style: "color: var(--text-secondary); font-size: 12px;", "No recent transactions." }
                        } else {
                            table { class: "pos-txn-table",
                                thead { tr {
                                    th { "Invoice" } th { "Date" }
                                    th { style: "text-align: right;", "Amount" } th { "Status" }
                                }}
                                tbody {
                                    {recent_transactions.read().iter().take(5).map(|txn| {
                                        let inv_no = txn["invoice_no"].as_str().unwrap_or("");
                                        let inv_date = txn["invoice_date"].as_str().unwrap_or("");
                                        let total = txn["total_amount"].as_f64().unwrap_or(0.0);
                                        let status = txn["status"].as_str().unwrap_or("");
                                        rsx! {
                                            tr {
                                                td { "{inv_no}" }
                                                td { "{inv_date}" }
                                                td { style: "text-align: right; font-family: monospace;", "PKR {total:.2}" }
                                                td { "{status}" }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }

                div { class: "pos-right",

                    div { class: "pos-totals-section",
                        h2 { "Sale Summary" }
                        div { class: "pos-totals-grid",
                            StatCard { title: "Subtotal".to_string(), value: format!("PKR {:.2}", subtotal), variant: StatCardVariant::Default }
                            StatCard { title: format!("Tax ({}%)", DEFAULT_TAX_RATE), value: format!("PKR {:.2}", tax_amount), variant: StatCardVariant::Default }
                            StatCard { title: "Grand Total".to_string(), value: format!("PKR {:.2}", grand_total), variant: StatCardVariant::Primary }
                        }
                    }

                    div { class: "pos-numpad-section",
                        h2 { "Quick Qty / Actions" }
                        div { class: "pos-item-quick",
                            div { class: "pos-quick-buttons",
                                {[("1", 1u32), ("2", 2), ("3", 3), ("5", 5), ("10", 10), ("15", 15), ("20", 20), ("50", 50)].iter().map(|(label, _qty)| {
                                    let lbl = *label;
                                    rsx! {
                                        button { class: "pos-quick-btn", r#type: "button", "x{lbl}" }
                                    }
                                })}
                            }
                        }
                    }

                    div { class: "pos-numpad-section",
                        h2 { "Quick Items" }
                        if quick_items_list.is_empty() {
                            div { style: "color: var(--text-secondary); font-size: 12px;", "Loading items…" }
                        } else {
                            div { class: "pos-quick-buttons",
                                {quick_items_list.iter().map(|item| {
                                    let item_id = item.id;
                                    let code = item.item_code.clone();
                                    let name = item.item_name.clone();
                                    let price = item.selling_price;
                                    let code2 = code.clone();
                                    let name2 = name.clone();
                                    let on_add = {
                                        let mut a = add_item.clone();
                                        move |_| a(item_id, &code2, &name2, price)
                                    };
                                    rsx! {
                                        button { key: "{code}", class: "pos-quick-btn", r#type: "button", onclick: on_add,
                                            div { style: "font-size: 11px; font-weight: 600;", "{name}" }
                                            div { style: "font-size: 10px; color: var(--accent);", "PKR {price:.2}" }
                                        }
                                    }
                                })}
                            }
                        }
                    }

                    div { class: "pos-customer-section",
                        h2 { "Customer (optional)" }
                        FormInput {
                            value: customer_name.read().clone(),
                            oninput: { let mut c = customer_name.clone(); move |v| c.set(v) },
                            r#type: InputType::Text,
                            placeholder: Some("Search customer…".to_string()),
                        }
                    }

                    div { class: "pos-customer-section",
                        h2 { "Payment Method" }
                        div { style: "display: flex; gap: 8px;",
                            {["Cash", "Card", "Bank Transfer"].iter().map(|method| {
                                let is_active = *payment_method.read() == **method;
                                let on_click = {
                                    let mut pm = payment_method.clone();
                                    let m = method.to_string();
                                    move |_| pm.set(m.clone())
                                };
                                let style = if is_active {
                                    "padding: 8px 16px; border: 2px solid var(--accent); border-radius: 4px; background: var(--accent); color: #fff; cursor: pointer; font-size: 13px; font-weight: 600;"
                                } else {
                                    "padding: 8px 16px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; background: #fff; cursor: pointer; font-size: 13px;"
                                };
                                rsx! {
                                    button { r#type: "button", style, onclick: on_click, "{method}" }
                                }
                            })}
                        }
                    }

                    div { class: "pos-actions",
                        Button {
                            variant: ButtonVariant::Danger,
                            onclick: clear_cart,
                            "Clear Cart"
                        }
                        Button {
                            variant: ButtonVariant::Success,
                            onclick: on_checkout,
                            "Checkout"
                        }
                    }
                }
            }
        }
    }
}
