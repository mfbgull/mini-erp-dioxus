//! Purchase Return List Page — DataGrid-backed list view for purchase returns.

use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug)]
pub struct PurchaseReturn {
    pub id: i64,
    pub pr_no: String,
    pub supplier_name: String,
    pub return_date: String,
    pub purchase_ref: String,
    pub status: String,
    pub total_amount: f64,
    pub reason: String,
}



#[component]
pub fn PurchaseReturnListPage() -> Element {
    let navigator = use_navigator();
    let refresh_counter = use_signal(|| 0u32);
    // ponytail: no returns list endpoint -- add when server exposes one
    let resource = use_resource(move || async move {
        let _ = *refresh_counter.read();
        Vec::new()
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    let is_loading = resource.read().is_none();
    let items: Vec<PurchaseReturn> = resource.read().cloned().unwrap_or_default();

    let columns: Vec<ColumnDef<PurchaseReturn>> = vec![
        ColumnDef::text("pr_no", "Return #", |r: &PurchaseReturn| r.pr_no.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Text),
        ColumnDef::text("supplier", "Supplier", |r: &PurchaseReturn| r.supplier_name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("date", "Return Date", |r: &PurchaseReturn| r.return_date.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("purchase_ref", "Purchase Ref", |r: &PurchaseReturn| r.purchase_ref.clone())
            .with_width(ColumnWidth::Px(120)),
        ColumnDef::text("status", "Status", |r: &PurchaseReturn| r.status.clone())
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Draft", BadgeColor::Gray),
                    ("Approved", BadgeColor::Blue),
                    ("Processed", BadgeColor::Green),
                    ("Rejected", BadgeColor::Red),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["Draft".to_string(), "Approved".to_string(), "Processed".to_string(), "Rejected".to_string()],
            }),
        ColumnDef::text("amount", "Amount", |r: &PurchaseReturn| r.total_amount.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(140))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
        ColumnDef::text("reason", "Reason", |r: &PurchaseReturn| r.reason.clone())
            .with_width(ColumnWidth::Px(180)),
    ];

    let total_amount: f64 = items.iter().map(|r| r.total_amount).sum();
    let processed = items.iter().filter(|r| r.status == "Processed").count();
    let pending = items.iter().filter(|r| r.status == "Draft" || r.status == "Approved").count();

    let on_row_click = move |(_idx, r): (usize, PurchaseReturn)| {
        tracing::info!("Clicked return: {}", r.pr_no);
    };

    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| counter += 1
    };

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Purchase Returns" }
                    p { class: "page-subtitle", "Manage returns to suppliers for defective or incorrect goods." }
                }
            }

            div { class: "invoice-summary-bar",
                if !is_loading {
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Returns" }
                        span { class: "summary-value", "{items.len()}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Amount" }
                        span { class: "summary-value summary-amount", "PKR {total_amount:.0}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Processed" }
                        span { class: "summary-value", "{processed}" }
                    }
                    div { class: "summary-item summary-warning",
                        span { class: "summary-label", "Pending" }
                        span { class: "summary-value", "{pending}" }
                    }
                }
            }

            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                }
            }

            DataGrid {
                columns: columns.clone(),
                rows: items.clone(),
                pagination: PaginationMode::Client { page_size: 15 },
                selection_mode: SelectionMode::Multi,
                striped: true,
                hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                on_row_click: on_row_click,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 8,
            }
        }
    }
}
