//! POS Terminal Page — Simplified point-of-sale terminal with item search,
//! cart management, number pad, and checkout.

use crate::components::common::{
    Button, ButtonVariant, ButtonSize, FormInput, InputType, StatCard, StatCardVariant,
};
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
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
}

#[derive(Clone, Debug)]
struct QuickItem {
    code: String,
    name: String,
    price: f64,
}

fn quick_items() -> Vec<QuickItem> {
    vec![
        QuickItem { code: "ITM-0001".to_string(), name: "Widget Alpha".to_string(), price: 29.99 },
        QuickItem { code: "ITM-0002".to_string(), name: "Bolt M12".to_string(), price: 0.45 },
        QuickItem { code: "ITM-0005".to_string(), name: "Gasket Set".to_string(), price: 8.99 },
        QuickItem { code: "ITM-0007".to_string(), name: "LED Panel 24W".to_string(), price: 18.50 },
        QuickItem { code: "ITM-0008".to_string(), name: "Box 40x30x20".to_string(), price: 1.20 },
        QuickItem { code: "ITM-0009".to_string(), name: "Safety Helmet".to_string(), price: 12.00 },
        QuickItem { code: "ITM-0006".to_string(), name: "Copper Wire".to_string(), price: 45.00 },
        QuickItem { code: "ITM-0003".to_string(), name: "Steel Rod 12mm".to_string(), price: 15.75 },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn PosTerminalPage() -> Element {
    let toast = crate::components::common::use_toast();
    let navigator = use_navigator();

    let cart = use_signal(Vec::<PosCartItem>::new);
    let search_query = use_signal(String::new);
    let customer_name = use_signal(String::new);

    let next_id = use_signal(|| 1u64);

    let add_item = {
        let mut c = cart.clone();
        let mut nid = next_id.clone();
        move |code: &str, name: &str, price: f64| {
            let id = *nid.read();
            *nid.write() += 1;

            // Check if already in cart, increment qty
            let mut w = c.write();
            if let Some(existing) = w.iter_mut().find(|i| i.item_code == code) {
                existing.quantity += 1.0;
            } else {
                w.push(PosCartItem {
                    id,
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

    // Computed totals
    let subtotal: f64 = cart.read().iter().map(|i| i.quantity * i.unit_price).sum();
    let tax_amount = subtotal * DEFAULT_TAX_RATE / 100.0;
    let grand_total = subtotal + tax_amount;
    let item_count: usize = cart.read().len();

    let on_checkout = {
        let mut t = toast.clone();
        let mut c = cart.clone();
        move |_| {
            if c.read().is_empty() {
                t.error("Cart Empty", "Add items to cart before checkout.");
                return;
            }
            let count = c.read().len();
            t.success("Checkout Complete", &format!("Sale completed — {} item(s). Generating invoice…", count));
        }
    };

    let on_search = {
        let mut q = search_query.clone();
        move |v: String| q.set(v)
    };

    let search_lowered = search_query.read().to_lowercase();
    let all_items = quick_items();
    let filtered_items: Vec<&QuickItem> = if search_lowered.is_empty() {
        Vec::new()
    } else {
        all_items.iter().filter(|qi| qi.name.to_lowercase().contains(&search_lowered) || qi.code.to_lowercase().contains(&search_lowered)).collect()
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page pos-page",
            h1 { "🛒 POS Terminal" }

            div { class: "pos-layout",

                // ── Left Panel ──
                div { class: "pos-left",

                    // Item Search
                    div { class: "pos-search-section",
                        h2 { "Search Items" }
                        FormInput {
                            value: search_query.read().clone(),
                            oninput: on_search,
                            r#type: InputType::Text,
                            placeholder: Some("Type item name or code…".to_string()),
                        }
                        if !filtered_items.is_empty() {
                            div { style: "margin-top: 8px; display: flex; flex-direction: column; gap: 4px;",
                                {filtered_items.iter().map(|qi| {
                                    let code = qi.code.clone();
                                    let name = qi.name.clone();
                                    let price = qi.price;
                                    let on_select = {
                                        let mut a = add_item.clone();
                                        move |_| a(&code, &name, price)
                                    };
                                    rsx! {
                                        button {
                                            key: "{qi.code}",
                                            r#type: "button",
                                            style: "display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; background: #fff; cursor: pointer; font-size: 13px; text-align: left;",
                                            onclick: on_select,
                                            span { span { style: "font-weight: 600;", "{qi.name}" } span { style: "color: var(--text-secondary); margin-left: 8px;", "{qi.code}" } }
                                            span { style: "font-weight: 600; color: var(--accent);", "PKR {qi.price:.2}" }
                                        }
                                    }
                                })}
                            }
                        }
                    }

                    // Cart
                    div { class: "pos-cart-section",
                        h2 {
                            span { "Cart" }
                            span { style: "font-size: 12px; color: var(--text-secondary); font-weight: 400;", "{item_count} item(s)" }
                        }
                        if cart.read().is_empty() {
                            div { class: "pos-cart-empty", "🛒 Cart is empty. Search or tap quick items to add." }
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
                                                    td { button { class: "pos-remove-btn", r#type: "button", onclick: on_rem, "×" } }
                                                }
                                            }
                                        })}
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Right Panel ──
                div { class: "pos-right",

                    // Totals
                    div { class: "pos-totals-section",
                        h2 { "Sale Summary" }
                        div { class: "pos-totals-grid",
                            StatCard { title: "Subtotal".to_string(), value: format!("PKR {:.2}", subtotal), variant: StatCardVariant::Default }
                            StatCard { title: format!("Tax ({}%)", DEFAULT_TAX_RATE), value: format!("PKR {:.2}", tax_amount), variant: StatCardVariant::Default }
                            StatCard { title: "Grand Total".to_string(), value: format!("PKR {:.2}", grand_total), variant: StatCardVariant::Primary }
                        }
                    }

                    // Number Pad / Qty Quick Buttons
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

                    // Quick Items
                    div { class: "pos-numpad-section",
                        h2 { "Quick Items" }
                        div { class: "pos-quick-buttons",
                            {quick_items().iter().map(|qi| {
                                let code = qi.code.clone();
                                let name = qi.name.clone();
                                let price = qi.price;
                                let on_add = {
                                    let mut a = add_item.clone();
                                    move |_| a(&code, &name, price)
                                };
                                rsx! {
                                    button { key: "{qi.code}", class: "pos-quick-btn", r#type: "button", onclick: on_add,
                                        div { style: "font-size: 11px; font-weight: 600;", "{qi.name}" }
                                        div { style: "font-size: 10px; color: var(--accent);", "PKR {qi.price:.2}" }
                                    }
                                }
                            })}
                        }
                    }

                    // Customer Lookup
                    div { class: "pos-customer-section",
                        h2 { "Customer (optional)" }
                        FormInput {
                            value: customer_name.read().clone(),
                            oninput: { let mut c = customer_name.clone(); move |v| c.set(v) },
                            r#type: InputType::Text,
                            placeholder: Some("Search customer…".to_string()),
                        }
                    }

                    // Action Buttons
                    div { class: "pos-actions",
                        Button {
                            variant: ButtonVariant::Danger,
                            onclick: clear_cart,
                            icon: Some("🗑️".to_string()),
                            "Clear Cart"
                        }
                        Button {
                            variant: ButtonVariant::Success,
                            onclick: on_checkout,
                            icon: Some("✓".to_string()),
                            "Checkout"
                        }
                    }
                }
            }
        }
    }
}
