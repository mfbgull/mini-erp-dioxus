use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct Permissions {
    pub role: String,
    pub permissions: HashSet<String>,
}

impl Permissions {
    pub fn new(role: &str) -> Self {
        let permissions = match role {
            "admin" => vec![
                "inventory:read", "inventory:create", "inventory:update", "inventory:delete",
                "customers:read", "customers:create", "customers:update", "customers:delete",
                "suppliers:read", "suppliers:create", "suppliers:update", "suppliers:delete",
                "invoices:read", "invoices:create", "invoices:update", "invoices:delete",
                "quotations:read", "quotations:create", "quotations:update", "quotations:delete",
                "sales_orders:read", "sales_orders:create", "sales_orders:update", "sales_orders:delete",
                "purchase_orders:read", "purchase_orders:create", "purchase_orders:update", "purchase_orders:delete",
                "bom:read", "bom:create", "bom:update", "bom:delete",
                "production:read", "production:create", "production:update", "production:delete",
                "employees:read", "employees:create", "employees:update", "employees:delete",
                "expenses:read", "expenses:create", "expenses:update", "expenses:delete",
                "accounting:read", "accounting:create", "accounting:update", "accounting:delete",
                "reports:read", "reports:create",
                "forecasts:read", "forecasts:create", "forecasts:update",
                "settings:read", "settings:update",
                "users:read", "users:create", "users:update", "users:delete",
                "roles:read", "roles:create", "roles:update", "roles:delete",
                "dashboard:read",
                "activity_log:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "manager" => vec![
                "inventory:read", "inventory:create", "inventory:update",
                "customers:read", "customers:create", "customers:update",
                "suppliers:read", "suppliers:create", "suppliers:update",
                "invoices:read", "invoices:create", "invoices:update",
                "quotations:read", "quotations:create", "quotations:update",
                "sales_orders:read", "sales_orders:create", "sales_orders:update",
                "purchase_orders:read", "purchase_orders:create", "purchase_orders:update",
                "bom:read", "bom:create", "bom:update",
                "production:read", "production:create", "production:update",
                "employees:read", "employees:create", "employees:update",
                "expenses:read", "expenses:create", "expenses:update",
                "accounting:read",
                "reports:read",
                "forecasts:read", "forecasts:create",
                "settings:read",
                "dashboard:read",
                "activity_log:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "viewer" => vec![
                "inventory:read",
                "customers:read",
                "suppliers:read",
                "invoices:read",
                "quotations:read",
                "sales_orders:read",
                "purchase_orders:read",
                "bom:read",
                "production:read",
                "employees:read",
                "expenses:read",
                "accounting:read",
                "reports:read",
                "forecasts:read",
                "dashboard:read",
                "activity_log:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            _ => HashSet::new(),
        };
        Self {
            role: role.to_string(),
            permissions,
        }
    }

    pub fn has(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    pub fn has_any(&self, perms: &[&str]) -> bool {
        perms.iter().any(|p| self.has(p))
    }

    pub fn has_all(&self, perms: &[&str]) -> bool {
        perms.iter().all(|p| self.has(p))
    }
}

#[derive(Clone, Copy)]
pub struct RbacContext {
    pub permissions: Signal<Permissions>,
}

impl RbacContext {
    pub fn new(role: &str) -> Self {
        let permissions = use_signal(|| Permissions::new(role));
        Self { permissions }
    }

    pub fn has(&self, permission: &str) -> bool {
        self.permissions.read().has(permission)
    }

    pub fn set_role(&self, role: &str) {
        let mut perms = self.permissions.clone();
        perms.set(Permissions::new(role));
    }
}

pub fn use_rbac() -> RbacContext {
    use_context::<RbacContext>()
}

#[component]
pub fn Can(permission: String, children: Element) -> Element {
    let rbac = use_rbac();
    if rbac.has(&permission) {
        children
    } else {
        rsx! {}
    }
}

#[component]
pub fn Cannot(permission: String, children: Element) -> Element {
    let rbac = use_rbac();
    if !rbac.has(&permission) {
        children
    } else {
        rsx! {}
    }
}

#[component]
pub fn ProtectedRoute(permission: String, children: Element) -> Element {
    let rbac = use_rbac();
    let navigator = use_navigator();
    if rbac.has(&permission) {
        children
    } else {
        rsx! {
            div { class: "page", style: "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 60vh; gap: 16px;",
                div { style: "font-size: 48px;", "🔒" }
                h2 { style: "margin: 0; color: var(--text-primary);", "Access Denied" }
                p { style: "color: var(--text-secondary);", "You don't have permission to access this page." }
                p { style: "color: var(--text-secondary); font-size: 13px;", "Required: {permission}" }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| { let _ = navigator.push("/"); },
                    "← Back to Dashboard"
                }
            }
        }
    }
}
