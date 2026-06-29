//! Sidebar navigation component.
//!
//! Extracted from `src/main.rs` into a reusable component.
//! Takes the current URL path as a prop for active-route highlighting.

use dioxus::prelude::*;
use std::collections::HashSet;
use crate::i18n::LanguageToggle;

// ============================================================================
// Types
// ============================================================================

/// A single item in the sidebar navigation.
pub struct NavItem {
    pub label: &'static str,
    pub icon: &'static str,
    pub route: &'static str,
}

/// A collapsible module group in the sidebar.
pub struct NavModule {
    pub name: &'static str,
    pub icon: &'static str,
    pub items: Vec<NavItem>,
}

// ============================================================================
// Data
// ============================================================================

/// All modules and their items for the sidebar.
pub fn nav_modules() -> Vec<NavModule> {
    vec![
        NavModule {
            name: "Dashboard", icon: "📊",
            items: vec![
                NavItem { label: "Overview", icon: "🏠", route: "/" },
            ],
        },
        NavModule {
            name: "Inventory", icon: "📦",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/inventory" },
                NavItem { label: "Items", icon: "📦", route: "/inventory/items" },
                NavItem { label: "New Item", icon: "➕", route: "/inventory/items/new" },
                NavItem { label: "Warehouses", icon: "🏭", route: "/inventory/warehouses" },
                NavItem { label: "Stock Movements", icon: "📋", route: "/inventory/stock-movements" },
                NavItem { label: "Stock Ledger", icon: "📋", route: "/inventory/stock-ledger/demo" },
                NavItem { label: "Physical Counts", icon: "🔢", route: "/inventory/physical-counts" },
            ],
        },
        NavModule {
            name: "Sales", icon: "💰",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/sales" },
                NavItem { label: "Invoices", icon: "🧾", route: "/sales/invoices" },
                NavItem { label: "New Invoice", icon: "➕", route: "/sales/invoices/new" },
                NavItem { label: "Quotations", icon: "📄", route: "/sales/quotations" },
                NavItem { label: "Sales Orders", icon: "📋", route: "/sales/orders" },
                NavItem { label: "Returns", icon: "↩", route: "/sales/returns" },
                NavItem { label: "POS", icon: "🏪", route: "/pos" },
            ],
        },
        NavModule {
            name: "Purchasing", icon: "📥",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/purchases" },
                NavItem { label: "Direct Purchases", icon: "📥", route: "/purchases/direct" },
                NavItem { label: "Purchase Orders", icon: "📋", route: "/purchases/orders" },
                NavItem { label: "Goods Receipts", icon: "📦", route: "/purchases/receipts" },
                NavItem { label: "Returns", icon: "↩", route: "/purchases/returns" },
            ],
        },
        NavModule {
            name: "Manufacturing", icon: "🏭",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/manufacturing" },
                NavItem { label: "BOM", icon: "📋", route: "/manufacturing/bom" },
                NavItem { label: "Production", icon: "⚙", route: "/manufacturing/production" },
            ],
        },
        NavModule {
            name: "Customers", icon: "👥",
            items: vec![
                NavItem { label: "All Customers", icon: "👥", route: "/customers" },
            ],
        },
        NavModule {
            name: "Suppliers", icon: "🏢",
            items: vec![
                NavItem { label: "All Suppliers", icon: "🏢", route: "/suppliers" },
            ],
        },
        NavModule {
            name: "Employees", icon: "👤",
            items: vec![
                NavItem { label: "All Employees", icon: "👤", route: "/employees" },
                NavItem { label: "New Employee", icon: "➕", route: "/employees/new" },
            ],
        },
        NavModule {
            name: "Expenses", icon: "💰",
            items: vec![
                NavItem { label: "All Expenses", icon: "💰", route: "/expenses" },
                NavItem { label: "Categories", icon: "📋", route: "/expenses/categories" },
            ],
        },
        NavModule {
            name: "Accounting", icon: "📊",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/accounting" },
                NavItem { label: "Chart of Accounts", icon: "📋", route: "/accounting/chart-of-accounts" },
                NavItem { label: "Periods", icon: "📅", route: "/accounting/periods" },
            ],
        },
        NavModule {
            name: "Reports", icon: "📈",
            items: vec![
                NavItem { label: "Dashboard", icon: "📊", route: "/reports" },
                NavItem { label: "AR Aging", icon: "📈", route: "/reports/ar-aging" },
                NavItem { label: "Customer Statements", icon: "📈", route: "/reports/customer-statements" },
                NavItem { label: "Sales", icon: "📈", route: "/reports/sales" },
                NavItem { label: "Inventory", icon: "📈", route: "/reports/inventory" },
                NavItem { label: "Financial", icon: "📈", route: "/reports/financial" },
                NavItem { label: "Custom Reports", icon: "📈", route: "/reports/custom" },
                NavItem { label: "Tax Summary", icon: "📈", route: "/reports/tax" },
            ],
        },
        NavModule {
            name: "Forecasts", icon: "🔮",
            items: vec![
                NavItem { label: "Dashboard", icon: "🔮", route: "/forecasts" },
                NavItem { label: "Demand", icon: "📈", route: "/forecasts/demand" },
                NavItem { label: "Trends", icon: "📈", route: "/forecasts/trends" },
                NavItem { label: "Accuracy", icon: "📊", route: "/forecasts/accuracy" },
                NavItem { label: "Model Config", icon: "⚙", route: "/forecasts/model-config" },
                NavItem { label: "Seasonal Events", icon: "📅", route: "/forecasts/seasonal-events" },
            ],
        },
        NavModule {
            name: "Admin", icon: "⚙",
            items: vec![
                NavItem { label: "Settings", icon: "⚙", route: "/settings" },
                NavItem { label: "Integrations", icon: "🔗", route: "/settings/integrations" },
                NavItem { label: "Users", icon: "👤", route: "/users" },
                NavItem { label: "Roles", icon: "🔐", route: "/roles" },
                NavItem { label: "Activity Log", icon: "📋", route: "/activity-log" },
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

    rsx! {
        aside { class: "app-sidebar",
            div { class: "sidebar-logo", "MiniERP" }
            nav { class: "sidebar-nav",
                {modules.into_iter().map(|module| {
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
                                {module.items.into_iter().map(|item| {
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
                LanguageToggle {}
            }
        }
    }
}
