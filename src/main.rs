//! MiniERP — Dioxus Application Entry Point
//!
//! Contains the full route map (~60 routes from PRD §6) and page components.
//! The sidebar navigation has been extracted into `components::layout::Sidebar`.

// Allow the dead_code lint in early development phases
#![allow(dead_code)]

// Use the library crate for all modules
use mini_erp::auth::*;
use mini_erp::components;
use mini_erp::pages;

use components::common::ToastProvider;
use components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType,
    PaginationMode, RowHeight, SelectionMode, TextAlign,
};
use components::layout::{Sidebar, SIDEBAR_CSS};
use components::rbac::{RbacContext, use_rbac, Can, Cannot, ProtectedRoute};

/// Helper macro to wrap a page component with a permission check.
/// Usage: protected_page!("inventory:read", pages::item_list::ItemListPage())
/// expands to: rsx! { ProtectedRoute { permission: "inventory:read".to_string(), pages::item_list::ItemListPage() } }
macro_rules! protected_page {
    ($perm:expr, $page:expr) => {
        rsx! {
            ProtectedRoute { permission: $perm.to_string(), { $page } }
        }
    };
}
use components::shortcuts::ShortcutsHelp;
use mini_erp::i18n::use_i18n;
use dioxus::prelude::*;

use pages::dashboard::DASHBOARD_CSS;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ============================================================================
// Routes
// ============================================================================

/// All application routes — the complete map from PRD §6 (~60 routes).
#[derive(Routable, Clone, PartialEq)]
enum Route {
    /// Login page (no layout wrapper)
    #[route("/login")]
    LoginPage {},

    /// All authenticated pages share the main layout shell
    #[layout(MainLayout)]

        // ── Dashboard ──
        #[route("/")]
        DashboardPage {},

        // ── Profile ──
        #[route("/profile")]
        UserProfilePage {},

        // ── Inventory ──
        #[route("/inventory")]
        InventoryDashboardPage {},
        #[route("/inventory/items")]
        ItemListPage {},
        #[route("/inventory/items/new")]
        ItemCreatePage {},
        #[route("/inventory/items/:id")]
        ItemDetailPage { id: String },
        #[route("/inventory/warehouses")]
        WarehouseListPage {},
        #[route("/inventory/warehouses/new")]
        WarehouseCreatePage {},
        #[route("/inventory/warehouses/:id")]
        WarehouseDetailPage { id: String },
        #[route("/inventory/stock-movements")]
        StockMovementListPage {},
        #[route("/inventory/stock-movements/new")]
        StockMovementCreatePage {},
        #[route("/inventory/stock-ledger/:item_id")]
        StockLedgerPage { item_id: String },
        #[route("/inventory/physical-counts")]
        PhysicalCountListPage {},
        #[route("/inventory/physical-counts/new")]
        PhysicalCountCreatePage {},
        #[route("/inventory/physical-counts/:id")]
        PhysicalCountDetailPage { id: String },

        // ── Sales ──
        #[route("/sales")]
        SalesDashboardPage {},
        #[route("/sales/invoices")]
        InvoiceListPage {},
        #[route("/sales/invoices/new")]
        InvoiceCreatePage {},
        #[route("/sales/invoices/:id")]
        InvoiceDetailPage { id: String },
        #[route("/sales/invoices/:id/print")]
        InvoicePrintPage { id: String },
        #[route("/sales/quotations")]
        QuotationListPage {},
        #[route("/sales/quotations/new")]
        QuotationCreatePage {},
        #[route("/sales/quotations/:id")]
        QuotationDetailPage { id: String },
        #[route("/sales/quotations/:id/print")]
        QuotationPrintPage { id: String },
        #[route("/sales/orders")]
        SalesOrderListPage {},
        #[route("/sales/orders/new")]
        SalesOrderCreatePage {},
        #[route("/sales/orders/:id")]
        SalesOrderDetailPage { id: String },
        #[route("/sales/returns")]
        SalesReturnListPage {},
        #[route("/pos")]
        PosTerminalPage {},

        // ── Purchasing ──
        #[route("/purchases")]
        PurchasesDashboardPage {},
        #[route("/purchases/direct")]
        DirectPurchaseListPage {},
        #[route("/purchases/direct/new")]
        DirectPurchaseCreatePage {},
        #[route("/purchases/direct/:id")]
        DirectPurchaseDetailPage { id: String },
        #[route("/purchases/orders")]
        PurchaseOrderListPage {},
        #[route("/purchases/orders/new")]
        PurchaseOrderCreatePage {},
        #[route("/purchases/orders/:id")]
        PurchaseOrderDetailPage { id: String },
        #[route("/purchases/orders/:id/print")]
        PurchaseOrderPrintPage { id: String },
        #[route("/purchases/receipts")]
        GoodsReceiptListPage {},
        #[route("/purchases/returns")]
        PurchaseReturnListPage {},

        // ── Manufacturing ──
        #[route("/manufacturing")]
        ManufacturingDashboardPage {},
        #[route("/manufacturing/bom")]
        BomListPage {},
        #[route("/manufacturing/bom/new")]
        BomCreatePage {},
        #[route("/manufacturing/bom/:id")]
        BomDetailPage { id: String },
        #[route("/manufacturing/production")]
        ProductionListPage {},
        #[route("/manufacturing/production/new")]
        ProductionCreatePage {},
        #[route("/manufacturing/production/:id")]
        ProductionDetailPage { id: String },

        // ── Customers ──
        #[route("/customers")]
        CustomerListPage {},
        #[route("/customers/new")]
        CustomerCreatePage {},
        #[route("/customers/:id")]
        CustomerDetailPage { id: String },

        // ── Suppliers ──
        #[route("/suppliers")]
        SupplierListPage {},
        #[route("/suppliers/new")]
        SupplierCreatePage {},
        #[route("/suppliers/:id")]
        SupplierDetailPage { id: String },

        // ── Employees ──
        #[route("/employees")]
        EmployeeListPage {},
        #[route("/employees/new")]
        EmployeeCreatePage {},
        #[route("/employees/:id")]
        EmployeeDetailPage { id: String },

        // ── Expenses ──
        #[route("/expenses")]
        ExpenseListPage {},
        #[route("/expenses/new")]
        ExpenseCreatePage {},
        #[route("/expenses/categories")]
        ExpenseCategoryListPage {},

        // ── Accounting ──
        #[route("/accounting")]
        AccountingDashboardPage {},
        #[route("/accounting/chart-of-accounts")]
        ChartOfAccountsPage {},
        #[route("/accounting/periods")]
        AccountingPeriodsPage {},

        // ── Reports ──
        #[route("/reports")]
        ReportsDashboardPage {},
        #[route("/reports/ar-aging")]
        ArAgingReportPage {},
        #[route("/reports/customer-statements")]
        CustomerStatementsPage {},
        #[route("/reports/sales")]
        SalesReportPage {},
        #[route("/reports/inventory")]
        InventoryReportPage {},
        #[route("/reports/financial")]
        FinancialReportPage {},
        #[route("/reports/custom")]
        CustomReportBuilderPage {},
        #[route("/reports/tax")]
        TaxSummaryPage {},

        // ── Forecasts ──
        #[route("/forecasts")]
        ForecastsDashboardPage {},
        #[route("/forecasts/demand")]
        DemandForecastPage {},
        #[route("/forecasts/trends")]
        TrendAnalysisPage {},
        #[route("/forecasts/accuracy")]
        ForecastAccuracyPage {},
        #[route("/forecasts/model-config")]
        ForecastModelConfigPage {},
        #[route("/forecasts/seasonal-events")]
        SeasonalEventsPage {},

        // ── Settings & Admin ──
        #[route("/settings")]
        SettingsPage {},
        #[route("/settings/integrations")]
        IntegrationsPage {},
        #[route("/users")]
        UserListPage {},
        #[route("/users/new")]
        UserCreatePage {},
        #[route("/users/:id/edit")]
        UserEditPage { id: String },
        #[route("/users/:id")]
        UserDetailPage { id: String },
        #[route("/roles")]
        RoleListPage {},
        #[route("/roles/:id")]
        RoleDetailPage { id: String },
        #[route("/activity-log")]
        ActivityLogPage {},

        // ── Demo ──
        #[route("/demo/data-grid")]
        DataGridDemoPage {},

    /// Catch-all 404
    #[route("/:..route")]
    NotFoundPage { route: Vec<String> },
}

// ============================================================================
// Entry Point
// ============================================================================

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .with_target(false)
        .init();
    dioxus::launch(App);
}

// ============================================================================
// App Root
// ============================================================================

static MAIN_CSS: &str = include_str!("../assets/main.css");

fn App() -> Element {
    let auth = use_auth_provider();
    use_context_provider(|| auth.clone());

    let rbac = RbacContext::new("");
    use_context_provider(|| rbac);

    let lang_signal = use_signal(|| mini_erp::i18n::Lang::En);
    use_context_provider(|| lang_signal);

    let mut shortcuts_show = use_signal(|| false);
    use_context_provider(|| shortcuts_show);

    rsx! {
        style { {MAIN_CSS} {components::data_grid::styles::DATA_GRID_CSS} {components::common::COMMON_CSS} "{SIDEBAR_CSS}" "{LOGIN_CSS}" "{DASHBOARD_CSS}" }
        ShortcutsHelp {}
        ToastProvider {
            Router::<Route> {}
        }
    }
}

// ============================================================================
// Layout Shell
// ============================================================================

#[component]
fn MainLayout() -> Element {
    let auth = use_auth();
    let navigator = use_navigator();

    // Show loading while restoring session
    if *auth.is_loading.read() {
        return rsx! {
            div { class: "page-loader",
                div { class: "loading-spinner" }
                p { "Authenticating…" }
            }
        };
    }

    // Redirect to login if not authenticated
    if auth.user.read().is_none() {
        return rsx! {
            div { class: "page-loader",
                p { "Redirecting to login…" }
                RedirectToLogin { }
            }
        };
    }

    let user = auth.user.read().clone();

    // Sync RBAC with user's actual role
    {
        let rbac = use_rbac();
        if let Some(ref u) = user {
            let current = rbac.permissions.read().role.clone();
            if current != u.role {
                rbac.set_role(&u.role);
            }
        }
    }

    // Logout handler
    let on_logout = {
        let auth = auth.clone();
        let navigator = navigator.clone();
        move |_| {
            let auth = auth.clone();
            let nav = navigator.clone();
            spawn(async move {
                auth.logout().await;
                nav.push("/login");
            });
        }
    };

    // ── Current route for active-highlighting ──
    let current_route = use_route::<Route>();
    let current_path = current_route.to_string();

    // ── Render ──

    let mut shortcuts_show = use_context::<Signal<bool>>();

    rsx! {
        div {
            class: "app-shell",
            tabindex: "0",
            onkeydown: move |e: Event<KeyboardData>| {
                use dioxus::prelude::keyboard_types::Key;
                let key = e.key();
                let mods = e.data().modifiers();
                let ctrl = mods.ctrl() || mods.meta();
                let is_q = matches!(&key, Key::Character(s) if s == "?");
                let is_k = matches!(&key, Key::Character(s) if s == "k");
                let is_esc = matches!(&key, Key::Escape);
                if is_q || (ctrl && is_k) {
                    let current = *shortcuts_show.read();
                    shortcuts_show.set(!current);
                } else if is_esc {
                    shortcuts_show.set(false);
                }
            },

            Sidebar { current_path: current_path.clone() }

            // ── Main Content ──
            main { class: "app-content",
                div { class: "app-topbar",
                    span { "MiniERP" }
                    div { class: "topbar-right",
                        if let Some(u) = user.as_ref() {
                            span { class: "topbar-user", "{u.full_name}" }
                        }
                        button {
                            class: "topbar-logout-btn",
                            r#type: "button",
                            onclick: on_logout,
                            "Logout"
                        }
                    }
                }
                div { class: "app-page",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

/// Helper component that redirects to login via side-effect.
#[component]
fn RedirectToLogin() -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.push("/login");
    });
    rsx! { div { } }
}

// ============================================================================
// Page Components
// ============================================================================
// Real pages delegate to the library crate.
// Stub pages use a simple placeholder.

// ── Dashboard ──

#[component]
fn DashboardPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "dashboard:read".to_string(),
            pages::dashboard::DashboardPage {}
        }
    }
}

#[component]
fn UserProfilePage() -> Element {
    pages::user_profile::UserProfilePage()
}

// ── Inventory ──

#[component]
fn InventoryDashboardPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::inventory_dashboard::InventoryDashboardPage {}
        }
    }
}

#[component]
fn ItemListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::item_list::ItemListPage {}
        }
    }
}

#[component]
fn ItemCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:create".to_string(),
            pages::item_create::ItemCreatePage {}
        }
    }
}

#[component]
fn ItemDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::item_detail::ItemDetailPage { id }
        }
    }
}

#[component]
fn WarehouseListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::warehouse_list::WarehouseListPage {}
        }
    }
}

#[component]
fn WarehouseCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:create".to_string(),
            pages::warehouse_create::WarehouseCreatePage {}
        }
    }
}

#[component]
fn WarehouseDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::warehouse_detail::WarehouseDetailPage { id }
        }
    }
}

#[component]
fn StockMovementListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::stock_movement_list::StockMovementListPage {}
        }
    }
}

#[component]
fn StockMovementCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:create".to_string(),
            pages::stock_movement_create::StockMovementCreatePage {}
        }
    }
}

#[component]
fn StockLedgerPage(item_id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::stock_ledger::StockLedgerPage { item_id }
        }
    }
}

#[component]
fn PhysicalCountListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::physical_count_list::PhysicalCountListPage {}
        }
    }
}

#[component]
fn PhysicalCountCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:create".to_string(),
            pages::physical_count_create::PhysicalCountCreatePage {}
        }
    }
}

#[component]
fn PhysicalCountDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "inventory:read".to_string(),
            pages::physical_count_detail::PhysicalCountDetailPage { id }
        }
    }
}

// ── Sales ──

#[component]
fn SalesDashboardPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "dashboard:read".to_string(),
            pages::sales_dashboard::SalesDashboardPage {}
        }
    }
}

#[component]
fn InvoiceListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "invoices:read".to_string(),
            pages::invoice_list::InvoiceListPage {}
        }
    }
}

#[component]
fn InvoiceCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "invoices:create".to_string(),
            pages::invoice_create::InvoiceCreatePage {}
        }
    }
}

#[component]
fn InvoiceDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "invoices:read".to_string(),
            pages::invoice_detail::InvoiceDetailPage { id }
        }
    }
}

#[component]
fn InvoicePrintPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "invoices:read".to_string(),
            pages::invoice_print::InvoicePrintPage { id }
        }
    }
}

#[component]
fn QuotationListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "quotations:read".to_string(),
            pages::quotation_list::QuotationListPage {}
        }
    }
}

#[component]
fn QuotationCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "quotations:create".to_string(),
            pages::quotation_create::QuotationCreatePage {}
        }
    }
}

#[component]
fn QuotationDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "quotations:read".to_string(),
            pages::quotation_detail::QuotationDetailPage { id }
        }
    }
}

#[component]
fn QuotationPrintPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "quotations:read".to_string(),
            pages::quotation_print::QuotationPrintPage { id }
        }
    }
}

#[component]
fn SalesOrderListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "sales_orders:read".to_string(),
            pages::sales_order_list::SalesOrderListPage {}
        }
    }
}

#[component]
fn SalesOrderCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "sales_orders:create".to_string(),
            pages::sales_order_create::SalesOrderCreatePage {}
        }
    }
}

#[component]
fn SalesOrderDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "sales_orders:read".to_string(),
            pages::sales_order_detail::SalesOrderDetailPage { id }
        }
    }
}

#[component]
fn SalesReturnListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "invoices:read".to_string(),
            pages::sales_return_list::SalesReturnListPage {}
        }
    }
}

#[component]
fn PosTerminalPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "invoices:create".to_string(),
            pages::pos_terminal::PosTerminalPage {}
        }
    }
}

// ── Purchasing ──

#[component]
fn PurchasesDashboardPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "dashboard:read".to_string(),
            pages::purchases_dashboard::PurchasesDashboardPage {}
        }
    }
}

#[component]
fn DirectPurchaseListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "purchases:read".to_string(),
            pages::direct_purchase_list::DirectPurchaseListPage {}
        }
    }
}

#[component]
fn DirectPurchaseCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "purchases:create".to_string(),
            pages::direct_purchase_create::DirectPurchaseCreatePage {}
        }
    }
}

#[component]
fn DirectPurchaseDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "purchases:read".to_string(),
            pages::direct_purchase_detail::DirectPurchaseDetailPage { id }
        }
    }
}

#[component]
fn PurchaseOrderListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "purchase_orders:read".to_string(),
            pages::purchase_order_list::PurchaseOrderListPage {}
        }
    }
}

#[component]
fn PurchaseOrderCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "purchase_orders:create".to_string(),
            pages::purchase_order_create::PurchaseOrderCreatePage {}
        }
    }
}

#[component]
fn PurchaseOrderDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "purchase_orders:read".to_string(),
            pages::purchase_order_detail::PurchaseOrderDetailPage { id }
        }
    }
}

#[component]
fn PurchaseOrderPrintPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "purchase_orders:read".to_string(),
            pages::purchase_order_print::PurchaseOrderPrintPage { id }
        }
    }
}

#[component]
fn GoodsReceiptListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "purchase_orders:read".to_string(),
            pages::goods_receipt_list::GoodsReceiptListPage {}
        }
    }
}

#[component]
fn PurchaseReturnListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "purchase_orders:read".to_string(),
            pages::purchase_return_list::PurchaseReturnListPage {}
        }
    }
}

// ── Manufacturing ──

#[component]
fn ManufacturingDashboardPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "dashboard:read".to_string(),
            pages::manufacturing_dashboard::ManufacturingDashboardPage {}
        }
    }
}

#[component]
fn BomListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "bom:read".to_string(),
            pages::bom_list::BomListPage {}
        }
    }
}

#[component]
fn BomCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "bom:create".to_string(),
            pages::bom_create::BomCreatePage {}
        }
    }
}

#[component]
fn BomDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "bom:read".to_string(),
            pages::bom_detail::BomDetailPage { id }
        }
    }
}

#[component]
fn ProductionListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "production:read".to_string(),
            pages::production_list::ProductionListPage {}
        }
    }
}

#[component]
fn ProductionCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "production:create".to_string(),
            pages::production_create::ProductionCreatePage {}
        }
    }
}

#[component]
fn ProductionDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "production:read".to_string(),
            pages::production_detail::ProductionDetailPage { id }
        }
    }
}

// ── Customers ──

#[component]
fn CustomerListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "customers:read".to_string(),
            pages::customer_list::CustomerListPage {}
        }
    }
}

#[component]
fn CustomerCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "customers:create".to_string(),
            pages::customer_create::CustomerCreatePage {}
        }
    }
}

#[component]
fn CustomerDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "customers:read".to_string(),
            pages::customer_detail::CustomerDetailPage { id }
        }
    }
}

// ── Suppliers ──

#[component]
fn SupplierListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "suppliers:read".to_string(),
            pages::supplier_list::SupplierListPage {}
        }
    }
}

#[component]
fn SupplierCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "suppliers:create".to_string(),
            pages::supplier_create::SupplierCreatePage {}
        }
    }
}

#[component]
fn SupplierDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "suppliers:read".to_string(),
            pages::supplier_detail::SupplierDetailPage { id }
        }
    }
}

// ── Employees ──

#[component]
fn EmployeeListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "employees:read".to_string(),
            pages::employee_list::EmployeeListPage {}
        }
    }
}

#[component]
fn EmployeeCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "employees:create".to_string(),
            pages::employee_create::EmployeeCreatePage {}
        }
    }
}

#[component]
fn EmployeeDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "employees:read".to_string(),
            pages::employee_detail::EmployeeDetailPage { id }
        }
    }
}

// ── Expenses ──

#[component]
fn ExpenseListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "expenses:read".to_string(),
            pages::expense_list::ExpenseListPage {}
        }
    }
}

#[component]
fn ExpenseCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "expenses:create".to_string(),
            pages::expense_create::ExpenseCreatePage {}
        }
    }
}

#[component]
fn ExpenseCategoryListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "expenses:read".to_string(),
            pages::expense_category_list::ExpenseCategoryListPage {}
        }
    }
}

// ── Accounting ──

#[component]
fn AccountingDashboardPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "dashboard:read".to_string(),
            pages::accounting_dashboard::AccountingDashboardPage {}
        }
    }
}

#[component]
fn ChartOfAccountsPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "accounting:read".to_string(),
            pages::chart_of_accounts::ChartOfAccountsPage {}
        }
    }
}

#[component]
fn AccountingPeriodsPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "accounting:read".to_string(),
            pages::accounting_periods::AccountingPeriodsPage {}
        }
    }
}

// ── Reports ──

#[component]
fn ReportsDashboardPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "reports:read".to_string(),
            pages::reports_dashboard::ReportsDashboardPage {}
        }
    }
}

#[component]
fn ArAgingReportPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "reports:read".to_string(),
            pages::ar_aging::ArAgingReportPage {}
        }
    }
}

#[component]
fn CustomerStatementsPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "customers:read".to_string(),
            pages::customer_statements::CustomerStatementsPage {}
        }
    }
}

#[component]
fn SalesReportPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "reports:read".to_string(),
            pages::sales_report::SalesReportPage {}
        }
    }
}

#[component]
fn InventoryReportPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "reports:read".to_string(),
            pages::inventory_report::InventoryReportPage {}
        }
    }
}

#[component]
fn FinancialReportPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "reports:read".to_string(),
            pages::financial_report::FinancialReportPage {}
        }
    }
}

#[component]
fn CustomReportBuilderPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "reports:create".to_string(),
            pages::custom_report_builder::CustomReportBuilderPage {}
        }
    }
}

#[component]
fn TaxSummaryPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "reports:read".to_string(),
            pages::tax_summary::TaxSummaryPage {}
        }
    }
}

// ── Forecasts ──

#[component]
fn ForecastsDashboardPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "forecasts:read".to_string(),
            pages::forecasts_dashboard::ForecastsDashboardPage {}
        }
    }
}

#[component]
fn DemandForecastPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "forecasts:read".to_string(),
            pages::demand_forecast::DemandForecastPage {}
        }
    }
}

#[component]
fn TrendAnalysisPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "forecasts:read".to_string(),
            pages::trend_analysis::TrendAnalysisPage {}
        }
    }
}

#[component]
fn ForecastAccuracyPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "forecasts:read".to_string(),
            pages::forecast_accuracy::ForecastAccuracyPage {}
        }
    }
}

#[component]
fn ForecastModelConfigPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "forecasts:update".to_string(),
            pages::forecast_model_config::ForecastModelConfigPage {}
        }
    }
}

#[component]
fn SeasonalEventsPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "forecasts:read".to_string(),
            pages::seasonal_events::SeasonalEventsPage {}
        }
    }
}

// ── Settings & Admin ──

#[component]
fn SettingsPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "settings:read".to_string(),
            pages::settings::SettingsPage {}
        }
    }
}

#[component]
fn IntegrationsPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "settings:read".to_string(),
            pages::integrations::IntegrationsPage {}
        }
    }
}

#[component]
fn UserListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "users:read".to_string(),
            pages::user_list::UserListPage {}
        }
    }
}

#[component]
fn UserCreatePage() -> Element {
    rsx! {
        ProtectedRoute { permission: "users:create".to_string(),
            pages::user_create::UserCreatePage {}
        }
    }
}

#[component]
fn UserEditPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "users:update".to_string(),
            pages::user_edit::UserEditPage { id }
        }
    }
}

#[component]
fn UserDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "users:read".to_string(),
            pages::user_detail::UserDetailPage { id }
        }
    }
}

#[component]
fn RoleListPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "roles:read".to_string(),
            pages::role_list::RoleListPage {}
        }
    }
}

#[component]
fn RoleDetailPage(id: String) -> Element {
    rsx! {
        ProtectedRoute { permission: "roles:read".to_string(),
            pages::role_detail::RoleDetailPage { id }
        }
    }
}

#[component]
fn ActivityLogPage() -> Element {
    rsx! {
        ProtectedRoute { permission: "activity_log:read".to_string(),
            pages::activity_log::ActivityLogPage {}
        }
    }
}

// ── 404 ──

#[component]
fn NotFoundPage(route: Vec<String>) -> Element {
    let route_path = route.join("/");
    rsx! {
        div { class: "page",
            h1 { "404 — Page Not Found" }
            p { "No route matches: /{route_path}" }
            Link { to: Route::DashboardPage {}, "← Back to Dashboard" }
        }
    }
}

// ── Demo ──

#[component]
fn DataGridDemoPage() -> Element {
    let items = demo_items();
    let selected_rows = use_signal(|| std::collections::HashSet::<usize>::new());

    let columns: Vec<ColumnDef<DemoItem>> = vec![
        ColumnDef::text("code", "Code", |item: &DemoItem| item.code.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Text),
        ColumnDef::text("name", "Item Name", |item: &DemoItem| item.name.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
        ColumnDef::text("category", "Category", |item: &DemoItem| item.category.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Select {
                options: vec![
                    "Widgets".to_string(), "Fasteners".to_string(), "Raw Materials".to_string(),
                    "Equipment".to_string(), "Consumables".to_string(), "Electrical".to_string(),
                    "Packaging".to_string(), "Safety".to_string(),
                ],
            }),
        ColumnDef::text("stock", "Stock", |item: &DemoItem| item.stock.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(100))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 })
            .with_cell_class(components::data_grid::CellClassRule::new(
                |item: &DemoItem| {
                    if item.stock == 0 { "text-danger fw-bold".to_string() }
                    else if item.stock <= 10 { "text-warning".to_string() }
                    else { String::new() }
                },
            )),
        ColumnDef::text("price", "Price", |item: &DemoItem| item.price.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Currency { code: "USD", decimals: 2 }),
        ColumnDef::text("status", "Status", |item: &DemoItem| item.status.clone())
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Active", BadgeColor::Green),
                    ("Discontinued", BadgeColor::Red),
                    ("Out of Stock", BadgeColor::Gray),
                    ("Low Stock", BadgeColor::Yellow),
                ],
                default_color: BadgeColor::Blue,
            }),
        ColumnDef::text("updated", "Last Updated", |item: &DemoItem| item.last_updated.clone())
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" }),
    ];

    rsx! {
        div { class: "page",
            h1 { "🔬 DataGrid Demo — Phase 1" }
            p { class: "page-subtitle",
                "A live demonstration of the generic DataGrid component with sorting, ",
                "pagination, selection, cell class rules, and multiple renderers."
            }
            div { class: "demo-features",
                span { class: "demo-badge", "✅ Multi-column sort" }
                span { class: "demo-badge", "✅ Pagination (10/page)" }
                span { class: "demo-badge", "✅ Single/Multi selection" }
                span { class: "demo-badge", "✅ Currency renderer" }
                span { class: "demo-badge", "✅ Badge renderer" }
                span { class: "demo-badge", "✅ Date renderer" }
                span { class: "demo-badge", "✅ Cell class rules" }
                span { class: "demo-badge", "✅ Skeleton loading" }
                span { class: "demo-badge", "✅ Empty state" }
                span { class: "demo-badge", "✅ Row striping" }
                span { class: "demo-badge", "🆕 Text filter" }
                span { class: "demo-badge", "🆕 Select filter" }
                span { class: "demo-badge", "🆕 Filter bar" }
                span { class: "demo-badge", "🆕 Filter dropdowns" }
            }

            div { class: "demo-section",
                h2 { "DataGrid — Full Feature Demo" }
                p { "10 sample items with client-side pagination (10/page), multi-select, and cell class rules for stock levels." }
                DataGrid {
                    columns: columns.clone(),
                    rows: items.clone(),
                    pagination: PaginationMode::Client { page_size: 10 },
                    selection_mode: SelectionMode::Multi,
                    striped: true,
                    hoverable: true,
                    row_height: RowHeight::Standard,
                    selected_rows: selected_rows,
                    on_row_click: move |(idx, item): (usize, DemoItem)| {
                        tracing::info!("Clicked row {}: {}", idx, item.name);
                    },
                }
            }

            div { class: "demo-section",
                h2 { "DataGrid — Skeleton Loading State" }
                p { "Shows 5 shimmer skeleton rows while data is loading." }
                DataGrid {
                    columns: columns.clone(),
                    rows: Vec::<DemoItem>::new(),
                    skeleton: true,
                    skeleton_rows: 5,
                    pagination: PaginationMode::None,
                }
            }

            div { class: "demo-section",
                h2 { "DataGrid — Empty State" }
                p { "Displayed when there are no rows and no loading state is active." }
                DataGrid {
                    columns: columns.clone(),
                    rows: Vec::<DemoItem>::new(),
                    loading: false,
                    empty_message: "No items found".to_string(),
                    empty_hint: Some("Try adjusting your filters or add a new item.".to_string()),
                    pagination: PaginationMode::None,
                }
            }
        }
    }
}

// ============================================================================
// Demo Data
// ============================================================================

#[derive(Clone, PartialEq)]
struct DemoItem {
    code: String,
    name: String,
    category: String,
    stock: i32,
    price: f64,
    status: String,
    last_updated: String,
}

fn demo_items() -> Vec<DemoItem> {
    vec![
        DemoItem { code: "ITM-0001".to_string(), name: "Premium Widget Alpha".to_string(), category: "Widgets".to_string(), stock: 150, price: 29.99, status: "Active".to_string(), last_updated: "2026-06-15".to_string() },
        DemoItem { code: "ITM-0002".to_string(), name: "Industrial Bolt M12".to_string(), category: "Fasteners".to_string(), stock: 3400, price: 0.45, status: "Active".to_string(), last_updated: "2026-06-10".to_string() },
        DemoItem { code: "ITM-0003".to_string(), name: "Steel Rod 12mm x 6m".to_string(), category: "Raw Materials".to_string(), stock: 80, price: 15.75, status: "Active".to_string(), last_updated: "2026-06-01".to_string() },
        DemoItem { code: "ITM-0004".to_string(), name: "Hydraulic Pump HPD-200".to_string(), category: "Equipment".to_string(), stock: 5, price: 1250.00, status: "Discontinued".to_string(), last_updated: "2026-05-20".to_string() },
        DemoItem { code: "ITM-0005".to_string(), name: "Rubber Gasket Set".to_string(), category: "Consumables".to_string(), stock: 0, price: 8.99, status: "Out of Stock".to_string(), last_updated: "2026-06-18".to_string() },
        DemoItem { code: "ITM-0006".to_string(), name: "Copper Wire 2.5mm (100m)".to_string(), category: "Raw Materials".to_string(), stock: 25, price: 45.00, status: "Active".to_string(), last_updated: "2026-06-12".to_string() },
        DemoItem { code: "ITM-0007".to_string(), name: "LED Panel Light 24W".to_string(), category: "Electrical".to_string(), stock: 200, price: 18.50, status: "Active".to_string(), last_updated: "2026-06-14".to_string() },
        DemoItem { code: "ITM-0008".to_string(), name: "Packaging Box 40x30x20cm".to_string(), category: "Packaging".to_string(), stock: 1200, price: 1.20, status: "Active".to_string(), last_updated: "2026-06-16".to_string() },
        DemoItem { code: "ITM-0009".to_string(), name: "Safety Helmet (Yellow)".to_string(), category: "Safety".to_string(), stock: 60, price: 12.00, status: "Low Stock".to_string(), last_updated: "2026-06-08".to_string() },
        DemoItem { code: "ITM-0010".to_string(), name: "Assembly Robot Arm v3".to_string(), category: "Equipment".to_string(), stock: 2, price: 15999.99, status: "Active".to_string(), last_updated: "2026-05-30".to_string() },
    ]
}
