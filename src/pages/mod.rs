//! Page components for MiniERP routes.
//!
//! Each module provides the root component for a route, using shared UI
//! components from `crate::components` (DataGrid, layout, common, etc.).
//!
//! # Current Pages
//!
//! | Module | Route | Status |
//! |--------|-------|--------|
//! | `dashboard` | `/` | ✅ Real page |
//! | `item_list` | `/inventory/items` | ✅ Real DataGrid |
//! | `item_create` | `/inventory/items/new` | ✅ Real form |
//! | `item_detail` | `/inventory/items/:id` | ✅ Real detail |
//! | `warehouse_list` | `/inventory/warehouses` | ✅ Real DataGrid |
//! | `stock_movement_list` | `/inventory/stock-movements` | ✅ Real DataGrid |
//! | `physical_count_list` | `/inventory/physical-counts` | ✅ Real DataGrid |
//! | `inventory_dashboard` | `/inventory` | ✅ Real dashboard |
//! | `invoice_list` | `/sales/invoices` | ✅ Real DataGrid |
//! | `invoice_create` | `/sales/invoices/new` | ✅ Real form |
//! | `customer_list` | `/customers` | ✅ Real DataGrid |
//! | `customer_detail` | `/customers/:id` | ✅ Real detail |
//! | `sales_dashboard` | `/sales` | ✅ Real dashboard |
//! | `invoice_detail` | `/sales/invoices/:id` | ✅ Real detail |
//! | `invoice_print` | `/sales/invoices/:id/print` | ✅ Real print view |
//! | `quotation_list` | `/sales/quotations` | ✅ Real DataGrid |
//! | `quotation_create` | `/sales/quotations/new` | ✅ Real form |
//! | `quotation_detail` | `/sales/quotations/:id` | ✅ Real detail |
//! | `quotation_print` | `/sales/quotations/:id/print` | ✅ Real print view |
//! | `sales_order_list` | `/sales/orders` | ✅ Real DataGrid |
//! | `sales_order_create` | `/sales/orders/new` | ✅ Real form |
//! | `sales_order_detail` | `/sales/orders/:id` | ✅ Real detail |
//! | `sales_return_list` | `/sales/returns` | ✅ Real DataGrid |
//! | `pos_terminal` | `/sales/pos` | ✅ Real page |
//! | `manufacturing_dashboard` | `/manufacturing` | ✅ Real dashboard |
//! | `bom_list` | `/manufacturing/boms` | ✅ Real DataGrid |
//! | `bom_create` | `/manufacturing/boms/new` | ✅ Real form |
//! | `bom_detail` | `/manufacturing/boms/:id` | ✅ Real detail |
//! | `production_list` | `/manufacturing/production` | ✅ Real DataGrid |
//! | `production_create` | `/manufacturing/production/new` | ✅ Real form |
//! | `production_detail` | `/manufacturing/production/:id` | ✅ Real detail |

pub mod bom_create;
pub mod bom_detail;
pub mod bom_list;
pub mod customer_create;
pub mod customer_detail;
pub mod customer_list;
pub mod accounting_dashboard;
pub mod accounting_periods;
pub mod chart_of_accounts;
pub mod dashboard;
pub mod employee_create;
pub mod employee_detail;
pub mod employee_list;
pub mod expense_category_list;
pub mod expense_create;
pub mod expense_list;
pub mod inventory_dashboard;
pub mod invoice_create;
pub mod invoice_list;
pub mod item_create;
pub mod item_detail;
pub mod item_list;
pub mod direct_purchase_create;
pub mod direct_purchase_detail;
pub mod direct_purchase_list;
pub mod goods_receipt_list;
pub mod physical_count_create;
pub mod physical_count_detail;
pub mod physical_count_list;
pub mod purchase_order_create;
pub mod purchase_order_detail;
pub mod purchase_order_list;
pub mod purchase_order_print;
pub mod purchase_return_list;
pub mod purchases_dashboard;
pub mod stock_ledger;
pub mod stock_movement_create;
pub mod stock_movement_list;
pub mod invoice_detail;
pub mod invoice_print;
pub mod print_shared;
pub mod manufacturing_dashboard;
pub mod production_create;
pub mod production_detail;
pub mod production_list;
pub mod supplier_create;
pub mod supplier_detail;
pub mod supplier_list;
pub mod warehouse_create;
pub mod warehouse_detail;
pub mod pos_terminal;
pub mod quotation_create;
pub mod quotation_detail;
pub mod quotation_list;
pub mod quotation_print;
pub mod sales_dashboard;
pub mod sales_order_create;
pub mod sales_order_detail;
pub mod sales_order_list;
pub mod sales_return_list;
pub mod warehouse_list;
pub mod settings;
pub mod integrations;
pub mod user_create;
pub mod user_edit;
pub mod user_list;
pub mod user_detail;
pub mod user_profile;
pub mod role_list;
pub mod role_detail;
pub mod activity_log;

// Reports module
pub mod reports_dashboard;
pub mod ar_aging;
pub mod customer_statements;
pub mod sales_report;
pub mod inventory_report;
pub mod financial_report;
pub mod custom_report_builder;
pub mod tax_summary;

// Forecasts module
pub mod forecasts_dashboard;
pub mod demand_forecast;
pub mod trend_analysis;
pub mod forecast_accuracy;
pub mod forecast_model_config;
pub mod seasonal_events;
