//! Invoice Edit Page — Edit an existing invoice with pre-filled form data,
//! customer credit info, and existing payments management.

use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, StatCard,
    StatCardVariant, use_toast,
};
use crate::auth::use_auth;
use crate::models::{Customer, Item, Payment, InvoiceForm, InvoiceItemForm};
use dioxus::prelude::*;
use std::collections::HashMap;

const DEFAULT_TAX_RATE: f64 = 16.0;

#[derive(Clone, PartialEq)]
struct EditLineItem {
    id: u64,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    tax_rate: f64,
    discount_value: f64,
}

impl Default for EditLineItem {
    fn default() -> Self {
        Self {
            id: 0,
            item_code: String::new(),
            item_name: String::new(),
            quantity: 1.0,
            unit_price: 0.0,
            tax_rate: DEFAULT_TAX_RATE,
            discount_value: 0.0,
        }
    }
}

#[component]
pub fn InvoiceEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let auth_api = use_auth().api;

    let parsed_id = id.parse::<i64>().unwrap_or(0);

    // Load invoice
    let invoice_resource = use_resource(move || {
        let api = auth_api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_invoice(parsed_id).await.ok()
        }
    });

    let inv_data = invoice_resource.read().as_ref().cloned().flatten();

    // Load customer credit info
    let customer_credit = use_signal(|| -> Option<(f64, f64, f64)> { None }); // (credit_limit, credit_balance, current_balance)

    // Load existing payments
    let existing_payments = use_signal(|| -> Vec<Payment> { Vec::new() });
    let deleted_payment_ids = use_signal(|| -> Vec<i64> { Vec::new() });

    // Load customers and items
    let customer_map = use_signal(HashMap::<String, Customer>::new);
    let item_map = use_signal(HashMap::<String, Item>::new);
    let customer_options = use_signal(Vec::<SelectOption>::new);
    let item_options = use_signal(Vec::<SelectOption>::new);

    {
        let api = auth_api.clone();
        let mut cmap = customer_map.clone();
        let mut copts = customer_options.clone();
        let mut imap = item_map.clone();
        let mut iopts = item_options.clone();
        spawn(async move {
            let client = api.with(|c| c.clone());
            if let Ok(custs) = client.list_customers().await {
                let mut map = HashMap::new();
                let mut opts = Vec::new();
                for c in &custs {
                    map.insert(c.customer_code.clone(), c.clone());
                    opts.push(SelectOption {
                        value: c.customer_code.clone(),
                        label: format!("{} — {}", c.customer_code, c.customer_name),
                    });
                }
                cmap.set(map);
                copts.set(opts);
            }
            if let Ok(items) = client.list_items().await {
                let mut map = HashMap::new();
                let mut opts = Vec::new();
                for i in &items {
                    map.insert(i.item_code.clone(), i.clone());
                    opts.push(SelectOption {
                        value: i.item_code.clone(),
                        label: format!("{} — {}", i.item_code, i.item_name),
                    });
                }
                imap.set(map);
                iopts.set(opts);
            }
        });
    }

    // Load customer credit and payments when invoice is available
    {
        let api = auth_api.clone();
        let mut credit = customer_credit.clone();
        let mut payments = existing_payments.clone();
        spawn(async move {
            if let Some(inv) = invoice_resource.read().as_ref().and_then(|v| v.as_ref()) {
                let cust_id = inv.get("customer_id").and_then(|v| v.as_i64()).unwrap_or(0);
                let inv_id = inv.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                let client = api.with(|c| c.clone());

                // Fetch customer credit info
                if let Ok(cust) = client.get_customer(cust_id).await {
                    credit.set(Some((cust.credit_limit, cust.credit_balance, cust.current_balance)));
                }

                // Fetch existing payments
                if let Ok(pay_list) = client.get_invoice_payments(inv_id).await {
                    payments.set(pay_list);
                }
            }
        });
    }

    let is_loading = invoice_resource.read().is_none();
    let is_saving = use_signal(|| false);
    let is_dirty = use_signal(|| false);

    if is_loading {
        return rsx! {
            div { class: "page", style: "max-width: 1000px; margin: 0 auto;",
                div { style: "text-align: center; padding: 60px 0; color: var(--text-secondary);",
                    "Loading invoice for editing…"
                }
            }
        };
    }

    if inv_data.is_none() {
        return rsx! {
            div { class: "page", style: "max-width: 1000px; margin: 0 auto;",
                div { style: "text-align: center; padding: 60px 0;",
                    h2 { style: "color: var(--text-primary);", "Invoice Not Found" }
                    p { style: "color: var(--text-secondary);", "No invoice with ID \"{id}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: move |_| { navigator.push("/sales/invoices"); }, "← Back to Invoices" }
                }
            }
        };
    }

    let inv = inv_data.as_ref().unwrap();

    let customer_code = use_signal(|| String::new());
    let customer_name = use_signal(|| inv.get("customer_name").and_then(|v| v.as_str()).unwrap_or("").to_string());
    let source_type = use_signal(|| inv.get("source_type").and_then(|v| v.as_str()).unwrap_or("Direct").to_string());
    let invoice_date = use_signal(|| inv.get("invoice_date").and_then(|v| v.as_str()).and_then(|s| s.parse::<chrono::NaiveDate>().ok()));
    let due_date = use_signal(|| inv.get("due_date").and_then(|v| v.as_str()).and_then(|s| s.parse::<chrono::NaiveDate>().ok()));
    let discount_pct = use_signal(|| format!("{}", inv.get("discount_value").and_then(|v| v.as_f64()).unwrap_or(0.0)));
    let tax_rate_str = use_signal(|| format!("{}", inv.get("tax_rate").and_then(|v| v.as_f64()).unwrap_or(DEFAULT_TAX_RATE)));
    let notes = use_signal(|| inv.get("notes").and_then(|v| v.as_str()).unwrap_or("").to_string());

    let items = use_signal(|| -> Vec<EditLineItem> { Vec::new() });

    {
        let api = auth_api.clone();
        let inv_id = inv.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let mut its = items.clone();
        spawn(async move {
            let client = api.with(|c| c.clone());
            if let Ok(body) = client.get_invoice(inv_id).await {
                if let Some(data) = body.get("data") {
                    if let Some(items_arr) = data.get("items").and_then(|v| v.as_array()) {
                        let mut result = Vec::new();
                        for (i, item) in items_arr.iter().enumerate() {
                            let item_code = item.get("item_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let item_name = item.get("item_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let quantity = item.get("quantity").and_then(|v| v.as_f64()).unwrap_or(1.0);
                            let unit_price = item.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let tax_rate = item.get("tax_rate").and_then(|v| v.as_f64()).unwrap_or(DEFAULT_TAX_RATE);
                            let discount_value = item.get("discount_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            result.push(EditLineItem { id: (i + 1) as u64, item_code, item_name, quantity, unit_price, tax_rate, discount_value });
                        }
                        if result.is_empty() {
                            result.push(EditLineItem::default());
                            result.push(EditLineItem::default());
                            result.push(EditLineItem::default());
                        }
                        its.set(result);
                    }
                }
            }
        });
    }

    let tax_rate_val: f64 = tax_rate_str.read().parse().unwrap_or(DEFAULT_TAX_RATE);
    let disc_val: f64 = discount_pct.read().parse().unwrap_or(0.0);
    let (subtotal, discount_amount, tax_amount, total) = {
        let its = items.read();
        let sub: f64 = its.iter().map(|li| {
            let base = li.quantity * li.unit_price;
            let item_disc = base * li.discount_value / 100.0;
            base - item_disc
        }).sum();
        let disc = sub * disc_val / 100.0;
        let after_disc = sub - disc;
        let tax = after_disc * tax_rate_val / 100.0;
        let tot = after_disc + tax;
        (sub, disc, tax, tot)
    };

    let on_customer_change = {
        let mut code = customer_code.clone();
        let mut name = customer_name.clone();
        let mut dirty = is_dirty.clone();
        let cust_map = customer_map.clone();
        move |value: String| {
            code.set(value.clone());
            let label = cust_map.read().get(&value).map(|c| c.customer_name.clone()).unwrap_or_default();
            name.set(label);
            dirty.set(true);
        }
    };

    let add_item = {
        let mut its = items.clone();
        let mut dirty = is_dirty.clone();
        move |_| { its.write().push(EditLineItem::default()); dirty.set(true); }
    };

    let remove_item = {
        let mut its = items.clone();
        let mut dirty = is_dirty.clone();
        move |id: u64| { its.write().retain(|li| li.id != id); dirty.set(true); }
    };

    let delete_payment = {
        let mut payments = existing_payments.clone();
        let mut deleted = deleted_payment_ids.clone();
        let mut dirty = is_dirty.clone();
        move |pid: i64| {
            payments.write().retain(|p| p.id != pid);
            deleted.write().push(pid);
            dirty.set(true);
        }
    };

    let save_invoice = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut c_code = customer_code.clone();
        let c_name = customer_name.clone();
        let mut its = items.clone();
        let navigator = navigator.clone();
        let source_type = source_type.clone();
        let inv_date = invoice_date.clone();
        let d_date = due_date.clone();
        let nts = notes.clone();
        let disc_pct = discount_pct.clone();
        let tax_str = tax_rate_str.clone();
        let api = auth_api.clone();
        let cust_map = customer_map.clone();
        let it_map = item_map.clone();
        let inv_id = inv.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let deleted = deleted_payment_ids.clone();

        move |_| {
            if c_code.read().is_empty() {
                toast.error("Validation Error", "Please select a customer.");
                return;
            }
            let filled = its.read().iter().filter(|li| !li.item_code.is_empty()).count();
            if filled == 0 {
                toast.error("Validation Error", "Please add at least one item.");
                return;
            }

            let cust_id = cust_map.read().get(&c_code.read().clone()).map(|c| c.id).unwrap_or(0);
            let form_items: Vec<InvoiceItemForm> = its.read().iter()
                .filter(|li| !li.item_code.is_empty())
                .map(|li| {
                    let item_id = it_map.read().get(&li.item_code).map(|i| i.id).unwrap_or(0);
                    InvoiceItemForm {
                        item_id,
                        description: Some(li.item_name.clone()),
                        quantity: li.quantity,
                        unit_price: li.unit_price,
                        tax_rate: Some(li.tax_rate),
                        discount_type: if li.discount_value > 0.0 { Some("percentage".to_string()) } else { None },
                        discount_value: if li.discount_value > 0.0 { Some(li.discount_value) } else { None },
                    }
                }).collect();

            let deleted_ids = deleted.read().clone();
            let form = InvoiceForm {
                customer_id: cust_id,
                invoice_date: inv_date.read().as_ref().map(|d| d.to_string()).unwrap_or_default(),
                due_date: d_date.read().as_ref().map(|d| d.to_string()),
                source_type: Some(source_type.read().clone()),
                warehouse_id: None,
                discount_scope: Some("BeforeTax".to_string()),
                discount_type: Some("percentage".to_string()),
                discount_value: Some(disc_pct.read().parse::<f64>().unwrap_or(0.0)),
                tax_rate: Some(tax_str.read().parse::<f64>().unwrap_or(0.0)),
                notes: Some(nts.read().clone()),
                items: form_items,
                record_payment: None,
                payment_amount: None,
                payment_method: None,
                deleted_payment_ids: if deleted_ids.is_empty() { None } else { Some(deleted_ids) },
            };

            saving.set(true);
            let mut toast = toast.clone();
            let nav = navigator.clone();
            let customer = c_name.read().clone();
            let client = api.with(|c| c.clone());

            spawn(async move {
                match client.update_invoice(inv_id, &form).await {
                    Ok(_) => {
                        toast.success("Invoice Updated", &format!("Invoice for {} updated.", customer));
                        saving.set(false);
                        nav.push(format!("/sales/invoices/{}", inv_id));
                    }
                    Err(e) => {
                        toast.error("Failed", &format!("Could not update invoice: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };

    let open_discard = move |_| { navigator.push("/sales/invoices"); };
    let inv_no = inv.get("invoice_no").and_then(|v| v.as_str()).unwrap_or("");

    // Compute paid amount from non-deleted payments
    let deleted_ids = deleted_payment_ids.read();
    let payments = existing_payments.read();
    let paid_amount: f64 = payments.iter()
        .filter(|p| !deleted_ids.contains(&p.id))
        .map(|p| p.amount)
        .sum();
    let balance = total - paid_amount;

    rsx! {
        div { class: "page", style: "max-width: 1000px; margin: 0 auto;",
            div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px;",
                div {
                    Button { variant: ButtonVariant::Ghost, onclick: open_discard, "← Back" }
                    h1 { style: "font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0;", "Edit Invoice {inv_no}" }
                }
            }

            // Customer & Dates
            div { class: "invoice-section",
                h2 { "Customer & Dates" }
                div { style: "display: flex; gap: 16px; flex-wrap: wrap;",
                    div { style: "flex: 1; min-width: 200px;",
                        SearchableSelect {
                            options: customer_options.read().clone(),
                            selected_value: Some(customer_code.read().clone()),
                            on_select: on_customer_change,
                            placeholder: "Select customer…",
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                    div { style: "flex: 1; min-width: 200px;",
                        label { style: "display: block; font-size: 12px; font-weight: 600; color: var(--text-secondary); margin-bottom: 4px;", "Invoice Date" }
                        input { r#type: "date", value: invoice_date.read().as_ref().map(|d| d.to_string()).unwrap_or_default(),
                            onchange: { let mut d = invoice_date.clone(); let mut dirty = is_dirty.clone(); move |e| { d.set(e.value().parse::<chrono::NaiveDate>().ok()); dirty.set(true); } },
                        }
                    }
                    div { style: "flex: 1; min-width: 200px;",
                        label { style: "display: block; font-size: 12px; font-weight: 600; color: var(--text-secondary); margin-bottom: 4px;", "Due Date" }
                        input { r#type: "date", value: due_date.read().as_ref().map(|d| d.to_string()).unwrap_or_default(),
                            onchange: { let mut d = due_date.clone(); let mut dirty = is_dirty.clone(); move |e| { d.set(e.value().parse::<chrono::NaiveDate>().ok()); dirty.set(true); } },
                        }
                    }
                }
            }

            // Customer Credit Info
            if let Some((limit, credit_bal, current_bal)) = *customer_credit.read() {
                div { class: "invoice-section",
                    h2 { "Customer Credit" }
                    div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px;",
                        StatCard { title: "Credit Limit".to_string(), value: format!("PKR {:.0}", limit), variant: StatCardVariant::Default, icon: Some("💳".to_string()) }
                        StatCard { title: "Credit Balance".to_string(), value: format!("PKR {:.0}", credit_bal), variant: if credit_bal > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Success }, icon: Some("📊".to_string()) }
                        StatCard { title: "Current Balance".to_string(), value: format!("PKR {:.0}", current_bal), variant: if current_bal > limit && limit > 0.0 { StatCardVariant::Danger } else { StatCardVariant::Default }, icon: Some("📋".to_string()) }
                    }
                }
            }

            // Line Items
            div { class: "invoice-section",
                h2 { "Line Items" }
                table { style: "width: 100%; border-collapse: collapse; font-size: 13px;",
                    thead { tr {
                        th { style: "text-align: left; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Item" }
                        th { style: "text-align: right; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Qty" }
                        th { style: "text-align: right; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Rate" }
                        th { style: "text-align: right; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Disc %" }
                        th { style: "text-align: right; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Tax %" }
                        th { style: "text-align: right; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Amount" }
                        th { style: "width: 40px;", "" }
                    } }
                    tbody {
                        {items.read().iter().map(|li| {
                            let base = li.quantity * li.unit_price;
                            let item_disc = base * li.discount_value / 100.0;
                            let net = base - item_disc;
                            let tax = net * li.tax_rate / 100.0;
                            let amount = net + tax;
                            rsx! {
                                tr {
                                    td { style: "padding: 6px;",
                                        SearchableSelect { options: item_options.read().clone(), selected_value: Some(li.item_code.clone()),
                                            on_select: { let mut its = items.clone(); let it_map = item_map.clone(); let id = li.id; let mut dirty = is_dirty.clone();
                                                move |v: String| { if let Some(item) = it_map.read().get(&v).cloned() { if let Some(line) = its.write().iter_mut().find(|l| l.id == id) { line.item_code = v; line.item_name = item.item_name.clone(); line.unit_price = item.selling_price; } } dirty.set(true); }
                                            },
                                            placeholder: "Select item…", class: Some("cb-input-group".to_string()),
                                        }
                                    }
                                    td { style: "padding: 6px; text-align: right;",
                                        input { r#type: "number", value: "{li.quantity}", min: "0", step: "0.01", style: "width: 60px; text-align: right; padding: 4px 6px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px;",
                                            oninput: { let mut its = items.clone(); let id = li.id; let mut dirty = is_dirty.clone(); move |e| { if let Some(line) = its.write().iter_mut().find(|l| l.id == id) { line.quantity = e.value().parse().unwrap_or(1.0); } dirty.set(true); } },
                                        }
                                    }
                                    td { style: "padding: 6px; text-align: right;",
                                        input { r#type: "number", value: "{li.unit_price}", min: "0", step: "0.01", style: "width: 80px; text-align: right; padding: 4px 6px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px;",
                                            oninput: { let mut its = items.clone(); let id = li.id; let mut dirty = is_dirty.clone(); move |e| { if let Some(line) = its.write().iter_mut().find(|l| l.id == id) { line.unit_price = e.value().parse().unwrap_or(0.0); } dirty.set(true); } },
                                        }
                                    }
                                    td { style: "padding: 6px; text-align: right;",
                                        input { r#type: "number", value: "{li.discount_value}", min: "0", max: "100", step: "0.5", style: "width: 60px; text-align: right; padding: 4px 6px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px;",
                                            oninput: { let mut its = items.clone(); let id = li.id; let mut dirty = is_dirty.clone(); move |e| { if let Some(line) = its.write().iter_mut().find(|l| l.id == id) { line.discount_value = e.value().parse().unwrap_or(0.0); } dirty.set(true); } },
                                        }
                                    }
                                    td { style: "padding: 6px; text-align: right;",
                                        input { r#type: "number", value: "{li.tax_rate}", min: "0", step: "0.5", style: "width: 60px; text-align: right; padding: 4px 6px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px;",
                                            oninput: { let mut its = items.clone(); let id = li.id; let mut dirty = is_dirty.clone(); move |e| { if let Some(line) = its.write().iter_mut().find(|l| l.id == id) { line.tax_rate = e.value().parse().unwrap_or(DEFAULT_TAX_RATE); } dirty.set(true); } },
                                        }
                                    }
                                    td { style: "padding: 6px; text-align: right; font-weight: 600; font-family: monospace; font-size: 12px;", "{amount:.2}" }
                                    td { style: "padding: 6px; text-align: center;",
                                        button { style: "border: none; background: transparent; cursor: pointer; color: var(--danger, #dc3545); font-size: 16px; padding: 4px;",
                                            onclick: { let mut remove = remove_item.clone(); let id = li.id; move |_| remove(id) }, "×"
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
                Button { variant: ButtonVariant::Ghost, onclick: add_item, "+ Add Item" }
            }

            // Discount & Tax
            div { class: "invoice-section",
                h2 { "Discount & Tax" }
                div { style: "display: flex; gap: 16px; align-items: flex-end; flex-wrap: wrap;",
                    FormInput { label: "Header Discount (%)".to_string(), value: discount_pct.read().clone(),
                        oninput: { let mut d = discount_pct.clone(); let mut dirty = is_dirty.clone(); move |v| { d.set(v); dirty.set(true); } },
                        r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(0.5),
                    }
                    FormInput { label: "Tax Rate (%)".to_string(), value: tax_rate_str.read().clone(),
                        oninput: { let mut t = tax_rate_str.clone(); let mut dirty = is_dirty.clone(); move |v| { t.set(v); dirty.set(true); } },
                        r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(0.5),
                    }
                }
            }

            // Totals
            div { class: "invoice-section",
                h2 { "Totals" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px;",
                    StatCard { title: "Subtotal".to_string(), value: format!("PKR {:.0}", subtotal), variant: StatCardVariant::Default }
                    StatCard { title: "Discount".to_string(), value: format!("PKR {:.0}", discount_amount), variant: if discount_amount > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default } }
                    StatCard { title: format!("Tax ({:.0}%)", tax_rate_val), value: format!("PKR {:.0}", tax_amount), variant: StatCardVariant::Default }
                    StatCard { title: "Grand Total".to_string(), value: format!("PKR {:.0}", total), variant: StatCardVariant::Primary }
                    StatCard { title: "Paid".to_string(), value: format!("PKR {:.0}", paid_amount), variant: if paid_amount > 0.0 { StatCardVariant::Success } else { StatCardVariant::Default } }
                    StatCard { title: "Balance".to_string(), value: format!("PKR {:.0}", balance), variant: if balance > 0.0 { StatCardVariant::Danger } else { StatCardVariant::Success } }
                }
            }

            // Existing Payments
            { let del_ids = deleted_payment_ids.read();
            let pay_list = existing_payments.read();
            let active_payments: Vec<&Payment> = pay_list.iter().filter(|p| !del_ids.contains(&p.id)).collect();
            if !active_payments.is_empty() {
                rsx! {
                    div { class: "invoice-section",
                        h2 { "Existing Payments " span { style: "font-size: 11px; font-weight: 400; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 10px;", "{active_payments.len()} payment(s)" } }
                        table { style: "width: 100%; border-collapse: collapse; font-size: 13px;",
                            thead { tr {
                                th { style: "text-align: left; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Payment #" }
                                th { style: "text-align: left; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Date" }
                                th { style: "text-align: right; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Amount" }
                                th { style: "text-align: left; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Method" }
                                th { style: "text-align: left; padding: 8px 6px; font-weight: 600; font-size: 11px; text-transform: uppercase; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0);", "Reference" }
                                th { style: "width: 40px;", "" }
                            } }
                            tbody {
                                {active_payments.into_iter().map(|p| {
                                    rsx! {
                                        tr {
                                            td { style: "padding: 8px 6px; font-weight: 500;", "{p.payment_no}" }
                                            td { style: "padding: 8px 6px; color: var(--text-secondary);", "{p.payment_date}" }
                                            td { style: "padding: 8px 6px; text-align: right; font-weight: 600; font-family: monospace;", "PKR {p.amount:.0}" }
                                            td { style: "padding: 8px 6px;", "{p.payment_method}" }
                                            td { style: "padding: 8px 6px; color: var(--text-secondary);",
                                                {let ref_str = p.reference.as_deref().unwrap_or("-"); "{ref_str}"}
                                            }
                                            td { style: "padding: 8px 6px; text-align: center;",
                                                button {
                                                    style: "border: none; background: transparent; cursor: pointer; color: var(--danger, #dc3545); font-size: 14px; padding: 4px;",
                                                    title: "Remove this payment",
                                                    onclick: { let mut del = delete_payment.clone(); let pid = p.id; move |_| del(pid) }, "🗑"
                                                }
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                        p { style: "margin: 8px 0 0 0; font-size: 12px; color: var(--text-secondary);", "Removed payments will be deleted when you save." }
                    }
                }
            } else {
                rsx! {}
            } }

            // Notes
            div { class: "invoice-section",
                h2 { "Notes" }
                FormInput { value: notes.read().clone(),
                    oninput: { let mut n = notes.clone(); let mut dirty = is_dirty.clone(); move |v| { n.set(v); dirty.set(true); } },
                    r#type: InputType::TextArea, placeholder: Some("Optional notes…".to_string()),
                }
            }

            // Action Bar
            div { style: "display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0);",
                Button { variant: ButtonVariant::Secondary, onclick: open_discard, "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: save_invoice, loading: *is_saving.read(), icon: Some("✓".to_string()), "Save Changes" }
            }
        }
    }
}
