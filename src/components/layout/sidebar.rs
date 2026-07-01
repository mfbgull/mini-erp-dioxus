//! Sidebar navigation component.
//!
//! Extracted from `src/main.rs` into a reusable component.
//! Takes the current URL path as a prop for active-route highlighting.

use dioxus::prelude::*;
use std::collections::HashSet;
use crate::i18n::LanguageToggle;
use crate::components::rbac::use_rbac;

// ============================================================================
// Types
// ============================================================================

/// A single item in the sidebar navigation.
pub struct NavItem {
    pub label: &'static str,
    pub icon: &'static str,
    pub route: &'static str,
    pub permission: &'static str,
}

/// A collapsible module group in the sidebar.
pub struct NavModule {
    pub name: &'static str,
    pub icon: &'static str,
    pub items: Vec<NavItem>,
    pub permission: &'static str,
}

// ============================================================================
// Data
// ============================================================================

/// All modules and their items for the sidebar.
pub fn nav_modules() -> Vec<NavModule> {
    vec![
        NavModule {
            name: "Dashboard", icon: "📊", permission: "dashboard:read",
            items: vec![
                NavItem { label: "Overview", icon: "🏠", route: "/", permission: "dashboard:read" },
            ],
        },
        NavModule {
            name: "Inventory", icon: "📦", permission: "inventory:read",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/inventory", permission: "inventory:read" },
                NavItem { label: "Items", icon: "📦", route: "/inventory/items", permission: "inventory:read" },
                NavItem { label: "New Item", icon: "➕", route: "/inventory/items/new", permission: "inventory:create" },
                NavItem { label: "Warehouses", icon: "🏭", route: "/inventory/warehouses", permission: "inventory:read" },
                NavItem { label: "Stock Movements", icon: "📋", route: "/inventory/stock-movements", permission: "inventory:read" },
                NavItem { label: "Stock Ledger", icon: "📋", route: "/inventory/stock-ledger/demo", permission: "inventory:read" },
                NavItem { label: "Physical Counts", icon: "🔢", route: "/inventory/physical-counts", permission: "inventory:read" },
            ],
        },
        NavModule {
            name: "Sales", icon: "💰", permission: "invoices:read",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/sales", permission: "dashboard:read" },
                NavItem { label: "Invoices", icon: "🧾", route: "/sales/invoices", permission: "invoices:read" },
                NavItem { label: "New Invoice", icon: "➕", route: "/sales/invoices/new", permission: "invoices:create" },
                NavItem { label: "Quotations", icon: "📄", route: "/sales/quotations", permission: "quotations:read" },
                NavItem { label: "Sales Orders", icon: "📋", route: "/sales/orders", permission: "sales_orders:read" },
                NavItem { label: "Returns", icon: "↩", route: "/sales/returns", permission: "invoices:read" },
                NavItem { label: "POS", icon: "🏪", route: "/pos", permission: "invoices:create" },
            ],
        },
        NavModule {
            name: "Purchasing", icon: "📥", permission: "purchase_orders:read",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/purchases", permission: "dashboard:read" },
                NavItem { label: "Direct Purchases", icon: "📥", route: "/purchases/direct", permission: "purchase_orders:read" },
                NavItem { label: "Purchase Orders", icon: "📋", route: "/purchases/orders", permission: "purchase_orders:read" },
                NavItem { label: "Goods Receipts", icon: "📦", route: "/purchases/receipts", permission: "purchase_orders:read" },
                NavItem { label: "Returns", icon: "↩", route: "/purchases/returns", permission: "purchase_orders:read" },
            ],
        },
        NavModule {
            name: "Manufacturing", icon: "🏭", permission: "bom:read",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/manufacturing", permission: "dashboard:read" },
                NavItem { label: "BOM", icon: "📋", route: "/manufacturing/boms", permission: "bom:read" },
                NavItem { label: "Production", icon: "⚙", route: "/manufacturing/production", permission: "production:read" },
            ],
        },
        NavModule {
            name: "Customers", icon: "👥", permission: "customers:read",
            items: vec![
                NavItem { label: "All Customers", icon: "👥", route: "/customers", permission: "customers:read" },
            ],
        },
        NavModule {
            name: "Suppliers", icon: "🏢", permission: "suppliers:read",
            items: vec![
                NavItem { label: "All Suppliers", icon: "🏢", route: "/suppliers", permission: "suppliers:read" },
            ],
        },
        NavModule {
            name: "Employees", icon: "👤", permission: "employees:read",
            items: vec![
                NavItem { label: "All Employees", icon: "👤", route: "/employees", permission: "employees:read" },
                NavItem { label: "New Employee", icon: "➕", route: "/employees/new", permission: "employees:create" },
            ],
        },
        NavModule {
            name: "Expenses", icon: "💰", permission: "expenses:read",
            items: vec![
                NavItem { label: "All Expenses", icon: "💰", route: "/expenses", permission: "expenses:read" },
                NavItem { label: "Categories", icon: "📋", route: "/expenses/categories", permission: "expenses:read" },
            ],
        },
        NavModule {
            name: "Accounting", icon: "📊", permission: "accounting:read",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/accounting", permission: "dashboard:read" },
                NavItem { label: "Chart of Accounts", icon: "📋", route: "/accounting/chart-of-accounts", permission: "accounting:read" },
                NavItem { label: "Periods", icon: "📅", route: "/accounting/periods", permission: "accounting:read" },
            ],
        },
        NavModule {
            name: "Reports", icon: "📈", permission: "reports:read",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/reports", permission: "reports:read" },
                NavItem { label: "AR Aging", icon: "📈", route: "/reports/ar-aging", permission: "reports:read" },
                NavItem { label: "Customer Statements", icon: "📈", route: "/reports/customer-statements", permission: "reports:read" },
                NavItem { label: "Sales", icon: "📈", route: "/reports/sales", permission: "reports:read" },
                NavItem { label: "Inventory", icon: "📈", route: "/reports/inventory", permission: "reports:read" },
                NavItem { label: "Financial", icon: "📈", route: "/reports/financial", permission: "reports:read" },
                NavItem { label: "Custom Reports", icon: "📈", route: "/reports/custom", permission: "reports:create" },
                NavItem { label: "Tax Summary", icon: "📈", route: "/reports/tax", permission: "reports:read" },
            ],
        },
        NavModule {
            name: "Forecasts", icon: "🔮", permission: "forecasts:read",
            items: vec![
                NavItem { label: "Dashboard", icon: "🔮", route: "/forecasts", permission: "forecasts:read" },
                NavItem { label: "Demand", icon: "📈", route: "/forecasts/demand", permission: "forecasts:read" },
                NavItem { label: "Trends", icon: "📈", route: "/forecasts/trends", permission: "forecasts:read" },
                NavItem { label: "Accuracy", icon: "📊", route: "/forecasts/accuracy", permission: "forecasts:read" },
                NavItem { label: "Model Config", icon: "⚙", route: "/forecasts/model-config", permission: "forecasts:update" },
                NavItem { label: "Seasonal Events", icon: "📅", route: "/forecasts/seasonal-events", permission: "forecasts:read" },
            ],
        },
        NavModule {
            name: "Admin", icon: "⚙", permission: "settings:read",
            items: vec![
                NavItem { label: "Settings", icon: "⚙", route: "/settings", permission: "settings:read" },
                NavItem { label: "Integrations", icon: "🔗", route: "/settings/integrations", permission: "settings:read" },
                NavItem { label: "Users", icon: "👤", route: "/users", permission: "users:read" },
                NavItem { label: "Roles", icon: "🔐", route: "/roles", permission: "roles:read" },
                NavItem { label: "Activity Log", icon: "📋", route: "/activity-log", permission: "activity_log:read" },
            ],
        },
    ]
}

// ============================================================================
// CSS
// ============================================================================

pub const SIDEBAR_CSS: &str = r##"
/* ── Sidebar Module Navigation ── */
.sidebar-footer {
    padding: 12px 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    margin-top: auto;
}
.sidebar-module {
    margin-bottom: 2px;
}

.sidebar-module-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-secondary, #6c757d);
    user-select: none;
}

.sidebar-module-header:hover {
    background: rgba(255, 255, 255, 0.06);
}

.sidebar-module-chevron {
    font-size: 10px;
    transition: transform 0.2s ease;
    opacity: 0.6;
}

.sidebar-module-chevron.expanded {
    transform: rotate(90deg);
}

.sidebar-module-items {
    overflow: hidden;
    transition: max-height 0.25s ease, opacity 0.2s ease;
    max-height: 0;
    opacity: 0;
}

.sidebar-module-items.open {
    max-height: 600px;
    opacity: 1;
}

.sidebar-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 16px 6px 28px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.7);
    text-decoration: none;
    border-radius: 4px;
    transition: all 0.12s ease;
    cursor: pointer;
    position: relative;
}

.sidebar-item:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
}

.sidebar-item.active {
    background: rgba(74, 144, 217, 0.2);
    color: #ffffff;
    font-weight: 500;
}

.sidebar-item.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 6px;
    bottom: 6px;
    width: 3px;
    background: var(--accent, #4a90d9);
    border-radius: 0 2px 2px 0;
}

.sidebar-item-icon {
    width: 18px;
    text-align: center;
    flex-shrink: 0;
    font-size: 14px;
    opacity: 0.8;
}

/* ── Sidebar Scrollable ── */
.app-sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}

.sidebar-nav {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 4px 8px 16px 8px;
}

.sidebar-nav::-webkit-scrollbar {
    width: 4px;
}

.sidebar-nav::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 2px;
}

.sidebar-nav::-webkit-scrollbar-track {
    background: transparent;
}
"##;

// ============================================================================
// Sidebar Component
// ============================================================================

/// Props for the Sidebar component.
#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    /// The current URL path for active-route highlighting.
    /// Provided by the parent layout via `use_route::<Route>().to_string()`.
    pub current_path: String,
}

/// Full-featured sidebar with collapsible module groups and active-route highlighting.
#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    // ── Collapsible module state ──
    let expanded: Signal<HashSet<String>> = use_signal(|| {
        let mut s = HashSet::new();
        s.insert("Dashboard".to_string());
        s.insert("Sales".to_string());
        s
    });

    let toggle_module = {
        let mut exp = expanded.clone();
        move |name: String| {
            let mut set = exp.write();
            if !set.insert(name.clone()) {
                set.remove(&name);
            }
        }
    };

    let modules = nav_modules();
    let current_path = props.current_path;
    let lang = crate::i18n::use_i18n();
    let rbac = use_rbac();

    rsx! {
        aside { class: "app-sidebar",
            div { class: "sidebar-logo", "MiniERP" }
            nav { class: "sidebar-nav",
                {modules.into_iter().filter(|m| rbac.has(m.permission)).map(|module| {
                    let mod_name = module.name.to_string();
                    let is_expanded = expanded.read().contains(&mod_name);
                    let chevron_class = if is_expanded { "sidebar-module-chevron expanded" } else { "sidebar-module-chevron" };
                    let items_class = if is_expanded { "sidebar-module-items open" } else { "sidebar-module-items" };
                    let on_toggle = {
                        let mut t = toggle_module.clone();
                        let n = mod_name.clone();
                        move |_| t(n.clone())
                    };
                    let nav_key = mod_name.to_lowercase().replace(' ', "_");
                    let translations = crate::i18n::get_translations(lang.read().clone());
                    let mod_label = translations.nav.get(nav_key.as_str()).unwrap_or(&module.name).to_string();

                    rsx! {
                        div { key: "{mod_name}", class: "sidebar-module",
                            div {
                                class: "sidebar-module-header",
                                onclick: on_toggle,
                                span { "{module.icon} {mod_label}" }
                                span { class: "{chevron_class}", "▶" }
                            }
                            div { class: "{items_class}",
                                {module.items.into_iter().filter(|i| rbac.has(i.permission)).map(|item| {
                                    let label = item.label;
                                    let icon = item.icon;
                                    let route_path = item.route;
                                    let item_class = {
                                        // Exact match (e.g., "/sales/invoices" matches "/sales/invoices")
                                        if current_path == route_path {
                                            "sidebar-item active"
                                        }
                                        // Prefix match for detail pages (e.g., "/sales/invoices/42" matches "/sales/invoices")
                                        else if route_path.len() > 1
                                            && current_path.starts_with(route_path)
                                            && current_path[route_path.len()..].starts_with('/')
                                        {
                                            "sidebar-item active"
                                        } else {
                                            "sidebar-item"
                                        }
                                    };

                                    let item_label_key = label.to_lowercase().replace(' ', "_");
                                    let item_translations = crate::i18n::get_translations(lang.read().clone());
                                    let item_label = item_translations.nav.get(item_label_key.as_str()).unwrap_or(&label).to_string();
                                    rsx! {
                                        Link {
                                            key: "{label}",
                                            class: "{item_class}",
                                            to: "{route_path}",
                                            span { class: "sidebar-item-icon", "{icon}" }
                                            span { "{item_label}" }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                })}
            }

            div { class: "sidebar-footer",
                Link { class: "sidebar-item", to: "/profile",
                    span { class: "sidebar-item-icon", "👤" }
                    span { "My Profile" }
                }
                LanguageToggle {}
            }
        }
    }
}
