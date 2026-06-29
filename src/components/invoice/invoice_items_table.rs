use dioxus::prelude::*;

const INVOICE_ITEMS_CSS: &str = r#"
.invoice-items-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; margin-bottom: 16px; overflow: hidden; }
.invoice-items-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 16px; border-bottom: 1px solid var(--border-color); }
.invoice-items-header h3 { font-size: 14px; font-weight: 600; color: var(--text-primary); margin: 0; }
.invoice-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.invoice-items-table th { text-align: left; padding: 10px 12px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); background: var(--bg-muted, #f8f9fa); border-bottom: 2px solid var(--border-color); }
.invoice-items-table th.text-right { text-align: right; }
.invoice-items-table td { padding: 8px 12px; border-bottom: 1px solid var(--border-color); vertical-align: middle; }
.invoice-items-table td.text-right { text-align: right; font-family: monospace; }
.invoice-items-table tr:last-child td { border-bottom: none; }
.invoice-items-table input, .invoice-items-table select { width: 100%; padding: 6px 8px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px; background: #fff; }
.invoice-items-table input:focus, .invoice-items-table select:focus { outline: none; border-color: var(--accent, #4a90d9); box-shadow: 0 0 0 2px rgba(74,144,217,0.15); }
.invoice-items-table .qty-input { width: 70px; text-align: right; }
.invoice-items-table .price-input { width: 100px; text-align: right; }
.invoice-items-table .tax-input { width: 60px; text-align: right; }
.invoice-items-table .amount-cell { font-weight: 600; color: var(--text-primary); min-width: 90px; }
.invoice-items-table .remove-btn { padding: 4px 8px; border: none; background: transparent; color: #dc3545; cursor: pointer; font-size: 16px; border-radius: 4px; }
.invoice-items-table .remove-btn:hover { background: rgba(220,53,69,0.1); }
.invoice-items-table .row-num { color: var(--text-secondary); font-weight: 500; width: 30px; }
.invoice-items-footer { display: flex; align-items: center; justify-content: space-between; padding: 12px 16px; border-top: 1px solid var(--border-color); }
.invoice-add-item-btn { display: inline-flex; align-items: center; gap: 4px; padding: 6px 12px; border: 1px dashed var(--border-color); border-radius: 6px; background: transparent; color: var(--accent, #4a90d9); font-size: 13px; cursor: pointer; }
.invoice-add-item-btn:hover { background: rgba(74,144,217,0.05); border-color: var(--accent); }
.invoice-totals { display: flex; flex-direction: column; gap: 8px; min-width: 240px; }
.invoice-total-row { display: flex; justify-content: space-between; font-size: 13px; color: var(--text-primary); padding: 2px 0; }
.invoice-total-row.grand-total { font-size: 16px; font-weight: 700; padding-top: 8px; border-top: 2px solid var(--text-primary); }
"#;

#[derive(Clone, Debug, PartialEq)]
pub struct LineItem {
    pub id: i64,
    pub item_id: Option<i64>,
    pub item_code: String,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount: f64,
    pub tax_rate: f64,
}

impl LineItem {
    pub fn amount(&self) -> f64 {
        let sub = self.quantity * self.unit_price - self.discount;
        let tax = sub * (self.tax_rate / 100.0);
        sub + tax
    }
    pub fn subtotal(&self) -> f64 { self.quantity * self.unit_price }
    pub fn tax_amount(&self) -> f64 { (self.subtotal() - self.discount) * (self.tax_rate / 100.0) }
}

#[derive(Props, Clone, PartialEq)]
pub struct InvoiceItemsTableProps {
    pub items: Vec<LineItem>,
    pub on_update: EventHandler<(usize, LineItem)>,
    pub on_remove: EventHandler<usize>,
    pub on_add: EventHandler<MouseEvent>,
    pub on_item_search: Option<EventHandler<usize>>,
}

#[component]
pub fn InvoiceItemsTable(props: InvoiceItemsTableProps) -> Element {
    let subtotal: f64 = props.items.iter().map(|i| i.subtotal()).sum();
    let total_discount: f64 = props.items.iter().map(|i| i.discount).sum();
    let total_tax: f64 = props.items.iter().map(|i| i.tax_amount()).sum();
    let grand_total = subtotal - total_discount + total_tax;

    rsx! {
        style { "{INVOICE_ITEMS_CSS}" }
        div { class: "invoice-items-section",
            div { class: "invoice-items-header",
                h3 { "Line Items ({props.items.len()})" }
            }
            div { style: "overflow-x: auto;",
                table { class: "invoice-items-table",
                    thead { tr {
                        th { class: "row-num", "#" }
                        th { "Item / Description" }
                        th { class: "text-right", "Qty" }
                        th { class: "text-right", "Unit Price" }
                        th { class: "text-right", "Disc." }
                        th { class: "text-right", "Tax %" }
                        th { class: "text-right", "Amount" }
                        th { style: "width: 40px;" }
                    }}
                    tbody {
                        for (idx, item) in props.items.iter().enumerate() {
                            tr {
                                td { class: "row-num", "{idx + 1}" }
                                td {
                                    input {
                                        r#type: "text",
                                        value: "{item.description}",
                                        placeholder: "Item description",
                                        oninput: {
                                            let items = props.items.clone();
                                            let on_update = props.on_update;
                                            move |e| {
                                                let mut item = items[idx].clone();
                                                item.description = e.value();
                                                on_update.call((idx, item));
                                            }
                                        },
                                    }
                                }
                                td {
                                    input {
                                        r#type: "number",
                                        class: "qty-input",
                                        value: "{item.quantity}",
                                        min: "0",
                                        step: "0.01",
                                        oninput: {
                                            let items = props.items.clone();
                                            let on_update = props.on_update;
                                            move |e| {
                                                let mut item = items[idx].clone();
                                                item.quantity = e.value().parse().unwrap_or(0.0);
                                                on_update.call((idx, item));
                                            }
                                        },
                                    }
                                }
                                td {
                                    input {
                                        r#type: "number",
                                        class: "price-input",
                                        value: "{item.unit_price}",
                                        min: "0",
                                        step: "0.01",
                                        oninput: {
                                            let items = props.items.clone();
                                            let on_update = props.on_update;
                                            move |e| {
                                                let mut item = items[idx].clone();
                                                item.unit_price = e.value().parse().unwrap_or(0.0);
                                                on_update.call((idx, item));
                                            }
                                        },
                                    }
                                }
                                td {
                                    input {
                                        r#type: "number",
                                        class: "price-input",
                                        value: "{item.discount}",
                                        min: "0",
                                        step: "0.01",
                                        oninput: {
                                            let items = props.items.clone();
                                            let on_update = props.on_update;
                                            move |e| {
                                                let mut item = items[idx].clone();
                                                item.discount = e.value().parse().unwrap_or(0.0);
                                                on_update.call((idx, item));
                                            }
                                        },
                                    }
                                }
                                td {
                                    input {
                                        r#type: "number",
                                        class: "tax-input",
                                        value: "{item.tax_rate}",
                                        min: "0",
                                        max: "100",
                                        step: "0.01",
                                        oninput: {
                                            let items = props.items.clone();
                                            let on_update = props.on_update;
                                            move |e| {
                                                let mut item = items[idx].clone();
                                                item.tax_rate = e.value().parse().unwrap_or(0.0);
                                                on_update.call((idx, item));
                                            }
                                        },
                                    }
                                }
                                td { class: "amount-cell text-right",
                                    {format!("Rs. {:.2}", item.amount())}
                                }
                                td {
                                    button {
                                        class: "remove-btn",
                                        title: "Remove item",
                                        onclick: {
                                            let on_remove = props.on_remove;
                                            move |_| on_remove.call(idx)
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "invoice-items-footer",
                button {
                    class: "invoice-add-item-btn",
                    onclick: move |e| props.on_add.call(e),
                    "＋ Add Item"
                }
                div { class: "invoice-totals",
                    div { class: "invoice-total-row",
                        span { "Subtotal" }
                        span { {format!("Rs. {:.2}", subtotal)} }
                    }
                    if total_discount > 0.0 {
                        div { class: "invoice-total-row", style: "color: #28a745;",
                            span { "Discount" }
                            span { {format!("- Rs. {:.2}", total_discount)} }
                        }
                    }
                    if total_tax > 0.0 {
                        div { class: "invoice-total-row",
                            span { "Tax" }
                            span { {format!("Rs. {:.2}", total_tax)} }
                        }
                    }
                    div { class: "invoice-total-row grand-total",
                        span { "Total" }
                        span { {format!("Rs. {:.2}", grand_total)} }
                    }
                }
            }
        }
    }
}
