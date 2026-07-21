//! Database connection, migrations, and seed data for MiniERP.
//!
//! Uses `rusqlite` with SQLite in WAL mode. All queries use prepared statements
//! to prevent SQL injection.

use rusqlite::{Connection, Result};
use std::sync::Mutex;

/// Global database connection (wrapped in Mutex for thread safety).
static DB: once_cell::sync::Lazy<Mutex<Connection>> =
    once_cell::sync::Lazy::new(|| {
        Mutex::new(open_database().expect("Failed to open database"))
    });

/// Get a handle to the global database connection.
pub fn get_db() -> &'static Mutex<Connection> {
    &DB
}

/// Open (or create) the SQLite database at the given path, configure WAL mode,
/// run migrations, and seed initial data.
fn open_database() -> Result<Connection> {
    let db_path = get_db_path();
    tracing::info!("Opening database at: {}", db_path);

    let conn = Connection::open(&db_path)?;

    // WAL mode for better concurrent read performance
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA busy_timeout=15000;
         PRAGMA foreign_keys=ON;"
    )?;

    run_migrations(&conn)?;
    seed_data(&conn)?;

    tracing::info!("Database ready");
    Ok(conn)
}

/// Determine the database file path.
/// Uses the `MINI_ERP_DB_PATH` env var, or defaults to `./mini-erp.db`.
fn get_db_path() -> String {
    std::env::var("MINI_ERP_DB_PATH")
        .unwrap_or_else(|_| "./mini-erp.db".to_string())
}

// ============================================================================
// Migration System
// ============================================================================

/// Run all pending migrations idempotently.
fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migration tracking table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    // Define all migrations in order
    let migrations: Vec<(&str, &str)> = vec![
        ("001_users", MIGRATION_001_USERS),
        ("002_roles_permissions", MIGRATION_002_ROLES_PERMISSIONS),
        ("003_role_permissions", MIGRATION_003_ROLE_PERMISSIONS),
        ("004_settings", MIGRATION_004_SETTINGS),
        ("005_items", MIGRATION_005_ITEMS),
        ("006_warehouses", MIGRATION_006_WAREHOUSES),
        ("007_stock_movements", MIGRATION_007_STOCK_MOVEMENTS),
        ("008_stock_balances", MIGRATION_008_STOCK_BALANCES),
        ("009_stock_batches", MIGRATION_009_STOCK_BATCHES),
        ("010_physical_counts", MIGRATION_010_PHYSICAL_COUNTS),
        ("011_item_locations", MIGRATION_011_ITEM_LOCATIONS),
        ("012_customers", MIGRATION_012_CUSTOMERS),
        ("013_customer_ledger", MIGRATION_013_CUSTOMER_LEDGER),
        ("014_invoices", MIGRATION_014_INVOICES),
        ("015_invoice_items", MIGRATION_015_INVOICE_ITEMS),
        ("016_payments", MIGRATION_016_PAYMENTS),
        ("017_payment_allocations", MIGRATION_017_PAYMENT_ALLOCATIONS),
        ("018_sales_orders", MIGRATION_018_SALES_ORDERS),
        ("019_sales_order_items", MIGRATION_019_SALES_ORDER_ITEMS),
        ("020_quotations", MIGRATION_020_QUOTATIONS),
        ("021_quotation_items", MIGRATION_021_QUOTATION_ITEMS),
        ("022_tax_rates", MIGRATION_022_TAX_RATES),
        ("023_payment_terms", MIGRATION_023_PAYMENT_TERMS),
        ("024_suppliers", MIGRATION_024_SUPPLIERS),
        ("025_supplier_ledger", MIGRATION_025_SUPPLIER_LEDGER),
        ("026_purchase_orders", MIGRATION_026_PURCHASE_ORDERS),
        ("027_purchase_order_items", MIGRATION_027_PURCHASE_ORDER_ITEMS),
        ("028_goods_receipts", MIGRATION_028_GOODS_RECEIPTS),
        ("029_goods_receipt_items", MIGRATION_029_GOODS_RECEIPT_ITEMS),
        ("030_purchases", MIGRATION_030_PURCHASES),
        ("031_boms", MIGRATION_031_BOMS),
        ("032_bom_items", MIGRATION_032_BOM_ITEMS),
        ("033_work_orders", MIGRATION_033_WORK_ORDERS),
        ("034_material_consumption", MIGRATION_034_MATERIAL_CONSUMPTION),
        ("035_productions", MIGRATION_035_PRODUCTIONS),
        ("036_production_inputs", MIGRATION_036_PRODUCTION_INPUTS),
        ("037_chart_of_accounts", MIGRATION_037_CHART_OF_ACCOUNTS),
        ("038_journal_entries", MIGRATION_038_JOURNAL_ENTRIES),
        ("039_journal_lines", MIGRATION_039_JOURNAL_LINES),
        ("040_accounting_periods", MIGRATION_040_ACCOUNTING_PERIODS),
        ("041_employees", MIGRATION_041_EMPLOYEES),
        ("042_employee_documents", MIGRATION_042_EMPLOYEE_DOCUMENTS),
        ("043_salary_payments", MIGRATION_043_SALARY_PAYMENTS),
        ("044_expenses", MIGRATION_044_EXPENSES),
        ("045_expense_categories", MIGRATION_045_EXPENSE_CATEGORIES),
        ("046_demand_forecasts", MIGRATION_046_DEMAND_FORECASTS),
        ("047_forecast_runs", MIGRATION_047_FORECAST_RUNS),
        ("048_forecast_model_config", MIGRATION_048_FORECAST_MODEL_CONFIG),
        ("049_forecast_seasonal_events", MIGRATION_049_FORECAST_SEASONAL_EVENTS),
        ("050_forecast_accuracy", MIGRATION_050_FORECAST_ACCURACY),
        ("051_custom_reports", MIGRATION_051_CUSTOM_REPORTS),
        ("052_dashboard_layouts", MIGRATION_052_DASHBOARD_LAYOUTS),
        ("053_invoice_drafts", MIGRATION_053_INVOICE_DRAFTS),
        ("054_activity_log", MIGRATION_054_ACTIVITY_LOG),
        ("055_bom_add_description_created_by", MIGRATION_055_BOM_DESCRIPTION_CREATED_BY),
        ("056_add_missing_fields", MIGRATION_056_ADD_MISSING_FIELDS),
        ("057_forecast_config_params_json", MIGRATION_057_FORECAST_CONFIG_PARAMS_JSON),
    ];

    for (name, sql) in &migrations {
        let already_applied: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
                [name],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !already_applied {
            tracing::info!("Applying migration: {}", name);
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])?;
            tracing::info!("Migration applied: {}", name);
        }
    }

    Ok(())
}

// ============================================================================
// Migration SQL
// ============================================================================

const MIGRATION_001_USERS: &str = "
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL DEFAULT '',
    password_hash TEXT NOT NULL,
    full_name TEXT NOT NULL DEFAULT '',
    role TEXT NOT NULL DEFAULT 'user',
    role_id INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
";

const MIGRATION_002_ROLES_PERMISSIONS: &str = "
CREATE TABLE IF NOT EXISTS roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    is_system_role INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    permission_name TEXT NOT NULL UNIQUE,
    module TEXT NOT NULL,
    action TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_permissions_module ON permissions(module);
";

const MIGRATION_003_ROLE_PERMISSIONS: &str = "
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);
";

const MIGRATION_004_SETTINGS: &str = "
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT ''
);
";

// ============================================================================
// Inventory Migrations
// ============================================================================

const MIGRATION_005_ITEMS: &str = "
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_code TEXT NOT NULL UNIQUE,
    item_name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT '',
    unit_of_measure TEXT NOT NULL DEFAULT 'pcs',
    current_stock REAL NOT NULL DEFAULT 0,
    reorder_level REAL NOT NULL DEFAULT 0,
    standard_cost REAL NOT NULL DEFAULT 0,
    selling_price REAL NOT NULL DEFAULT 0,
    is_raw_material INTEGER NOT NULL DEFAULT 0,
    is_finished_good INTEGER NOT NULL DEFAULT 0,
    is_purchased INTEGER NOT NULL DEFAULT 1,
    is_manufactured INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_items_code ON items(item_code);
CREATE INDEX IF NOT EXISTS idx_items_category ON items(category);
CREATE INDEX IF NOT EXISTS idx_items_active ON items(is_active);
";

const MIGRATION_006_WAREHOUSES: &str = "
CREATE TABLE IF NOT EXISTS warehouses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    warehouse_code TEXT NOT NULL UNIQUE,
    warehouse_name TEXT NOT NULL,
    location TEXT NOT NULL DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_warehouses_code ON warehouses(warehouse_code);
";

const MIGRATION_007_STOCK_MOVEMENTS: &str = "
CREATE TABLE IF NOT EXISTS stock_movements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    movement_no TEXT NOT NULL UNIQUE,
    item_id INTEGER NOT NULL,
    warehouse_id INTEGER NOT NULL,
    movement_type TEXT NOT NULL,
    quantity REAL NOT NULL,
    unit_cost REAL NOT NULL DEFAULT 0,
    reference_doctype TEXT,
    reference_docno TEXT,
    batch_id INTEGER,
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (item_id) REFERENCES items(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_stock_movements_item ON stock_movements(item_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_warehouse ON stock_movements(warehouse_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_type ON stock_movements(movement_type);
CREATE INDEX IF NOT EXISTS idx_stock_movements_date ON stock_movements(created_at);
";

const MIGRATION_008_STOCK_BALANCES: &str = "
CREATE TABLE IF NOT EXISTS stock_balances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL,
    warehouse_id INTEGER NOT NULL,
    quantity REAL NOT NULL DEFAULT 0,
    UNIQUE(item_id, warehouse_id),
    FOREIGN KEY (item_id) REFERENCES items(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_stock_balances_item ON stock_balances(item_id);
CREATE INDEX IF NOT EXISTS idx_stock_balances_warehouse ON stock_balances(warehouse_id);
";

const MIGRATION_009_STOCK_BATCHES: &str = "
CREATE TABLE IF NOT EXISTS stock_batches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_no TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    warehouse_id INTEGER NOT NULL,
    source_type TEXT,
    source_id INTEGER,
    quantity_original REAL NOT NULL,
    quantity_remaining REAL NOT NULL,
    unit_cost REAL NOT NULL DEFAULT 0,
    received_date TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (item_id) REFERENCES items(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_stock_batches_item ON stock_batches(item_id);
CREATE INDEX IF NOT EXISTS idx_stock_batches_warehouse ON stock_batches(warehouse_id);
CREATE INDEX IF NOT EXISTS idx_stock_batches_fifo ON stock_batches(item_id, warehouse_id, received_date, id);
";

const MIGRATION_010_PHYSICAL_COUNTS: &str = "
CREATE TABLE IF NOT EXISTS physical_counts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    count_no TEXT NOT NULL UNIQUE,
    count_date TEXT NOT NULL DEFAULT (datetime('now')),
    warehouse_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'Draft',
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_physical_counts_warehouse ON physical_counts(warehouse_id);
CREATE INDEX IF NOT EXISTS idx_physical_counts_status ON physical_counts(status);

CREATE TABLE IF NOT EXISTS physical_count_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    count_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    system_quantity REAL NOT NULL DEFAULT 0,
    counted_quantity REAL,
    variance REAL,
    FOREIGN KEY (count_id) REFERENCES physical_counts(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_physical_count_items_count ON physical_count_items(count_id);
";

const MIGRATION_011_ITEM_LOCATIONS: &str = "
CREATE TABLE IF NOT EXISTS item_locations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL,
    warehouse_id INTEGER NOT NULL,
    rack_no TEXT NOT NULL DEFAULT '',
    is_primary INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (item_id) REFERENCES items(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_item_locations_item ON item_locations(item_id);
";

// ============================================================================
// CRM / Sales Migrations
// ============================================================================

const MIGRATION_012_CUSTOMERS: &str = "
CREATE TABLE IF NOT EXISTS customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_code TEXT NOT NULL UNIQUE,
    customer_name TEXT NOT NULL,
    email TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    billing_address TEXT NOT NULL DEFAULT '',
    shipping_address TEXT NOT NULL DEFAULT '',
    payment_terms TEXT NOT NULL DEFAULT 'Net 30',
    credit_limit REAL NOT NULL DEFAULT 0,
    credit_balance REAL NOT NULL DEFAULT 0,
    current_balance REAL NOT NULL DEFAULT 0,
    opening_balance REAL NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_customers_code ON customers(customer_code);
CREATE INDEX IF NOT EXISTS idx_customers_active ON customers(is_active);
";

const MIGRATION_013_CUSTOMER_LEDGER: &str = "
CREATE TABLE IF NOT EXISTS customer_ledger (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    transaction_date TEXT NOT NULL DEFAULT (datetime('now')),
    type TEXT NOT NULL,
    reference_no TEXT NOT NULL DEFAULT '',
    debit REAL NOT NULL DEFAULT 0,
    credit REAL NOT NULL DEFAULT 0,
    balance REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);
CREATE INDEX IF NOT EXISTS idx_customer_ledger_customer ON customer_ledger(customer_id);
";

const MIGRATION_014_INVOICES: &str = "
CREATE TABLE IF NOT EXISTS invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_no TEXT NOT NULL UNIQUE,
    customer_id INTEGER NOT NULL,
    so_id INTEGER,
    quotation_id INTEGER,
    source_type TEXT NOT NULL DEFAULT 'DIRECT',
    invoice_date TEXT NOT NULL DEFAULT (datetime('now')),
    due_date TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'Unpaid',
    total_amount REAL NOT NULL DEFAULT 0,
    paid_amount REAL NOT NULL DEFAULT 0,
    balance_amount REAL NOT NULL DEFAULT 0,
    returned_amount REAL NOT NULL DEFAULT 0,
    discount_scope TEXT,
    discount_type TEXT,
    discount_value REAL DEFAULT 0,
    tax_rate REAL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT '',
    warehouse_id INTEGER,
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (customer_id) REFERENCES customers(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_invoices_customer ON invoices(customer_id);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
CREATE INDEX IF NOT EXISTS idx_invoices_date ON invoices(invoice_date);
";

const MIGRATION_015_INVOICE_ITEMS: &str = "
CREATE TABLE IF NOT EXISTS invoice_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    quantity REAL NOT NULL,
    returned_qty REAL NOT NULL DEFAULT 0,
    unit_price REAL NOT NULL,
    amount REAL NOT NULL,
    tax_rate REAL NOT NULL DEFAULT 0,
    discount_type TEXT,
    discount_value REAL DEFAULT 0,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_invoice_items_invoice ON invoice_items(invoice_id);
";

const MIGRATION_016_PAYMENTS: &str = "
CREATE TABLE IF NOT EXISTS payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payment_no TEXT NOT NULL UNIQUE,
    customer_id INTEGER NOT NULL,
    invoice_id INTEGER,
    payment_date TEXT NOT NULL DEFAULT (datetime('now')),
    amount REAL NOT NULL,
    payment_method TEXT NOT NULL DEFAULT 'Cash',
    reference TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (customer_id) REFERENCES customers(id),
    FOREIGN KEY (invoice_id) REFERENCES invoices(id)
);
CREATE INDEX IF NOT EXISTS idx_payments_customer ON payments(customer_id);
CREATE INDEX IF NOT EXISTS idx_payments_invoice ON payments(invoice_id);
";

const MIGRATION_017_PAYMENT_ALLOCATIONS: &str = "
CREATE TABLE IF NOT EXISTS payment_allocations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payment_id INTEGER NOT NULL,
    invoice_id INTEGER NOT NULL,
    amount REAL NOT NULL,
    FOREIGN KEY (payment_id) REFERENCES payments(id) ON DELETE CASCADE,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id)
);
CREATE INDEX IF NOT EXISTS idx_payment_allocations_payment ON payment_allocations(payment_id);
CREATE INDEX IF NOT EXISTS idx_payment_allocations_invoice ON payment_allocations(invoice_id);
";

const MIGRATION_018_SALES_ORDERS: &str = "
CREATE TABLE IF NOT EXISTS sales_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    so_no TEXT NOT NULL UNIQUE,
    customer_id INTEGER NOT NULL,
    so_date TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'Pending',
    source_type TEXT,
    source_id INTEGER,
    total_amount REAL NOT NULL DEFAULT 0,
    warehouse_id INTEGER,
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (customer_id) REFERENCES customers(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_sales_orders_customer ON sales_orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_sales_orders_status ON sales_orders(status);
";

const MIGRATION_019_SALES_ORDER_ITEMS: &str = "
CREATE TABLE IF NOT EXISTS sales_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    so_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    quantity REAL NOT NULL,
    delivered_quantity REAL NOT NULL DEFAULT 0,
    unit_price REAL NOT NULL,
    amount REAL NOT NULL,
    FOREIGN KEY (so_id) REFERENCES sales_orders(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_sales_order_items_so ON sales_order_items(so_id);
";

const MIGRATION_020_QUOTATIONS: &str = "
CREATE TABLE IF NOT EXISTS quotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    quotation_no TEXT NOT NULL UNIQUE,
    customer_id INTEGER NOT NULL,
    quotation_date TEXT NOT NULL DEFAULT (datetime('now')),
    expiry_date TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'Draft',
    total_amount REAL NOT NULL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);
CREATE INDEX IF NOT EXISTS idx_quotations_customer ON quotations(customer_id);
CREATE INDEX IF NOT EXISTS idx_quotations_status ON quotations(status);
";

const MIGRATION_021_QUOTATION_ITEMS: &str = "
CREATE TABLE IF NOT EXISTS quotation_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    quotation_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    quantity REAL NOT NULL,
    unit_price REAL NOT NULL,
    discount REAL NOT NULL DEFAULT 0,
    tax REAL NOT NULL DEFAULT 0,
    amount REAL NOT NULL,
    FOREIGN KEY (quotation_id) REFERENCES quotations(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_quotation_items_quotation ON quotation_items(quotation_id);
";

const MIGRATION_022_TAX_RATES: &str = "
CREATE TABLE IF NOT EXISTS tax_rates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    rate REAL NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1
);
";

const MIGRATION_023_PAYMENT_TERMS: &str = "
CREATE TABLE IF NOT EXISTS payment_terms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    days INTEGER NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1
);
";

// ============================================================================
// Purchasing Migrations
// ============================================================================

const MIGRATION_024_SUPPLIERS: &str = "
CREATE TABLE IF NOT EXISTS suppliers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    supplier_code TEXT NOT NULL UNIQUE,
    supplier_name TEXT NOT NULL,
    email TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    address TEXT NOT NULL DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_suppliers_code ON suppliers(supplier_code);
";

const MIGRATION_025_SUPPLIER_LEDGER: &str = "
CREATE TABLE IF NOT EXISTS supplier_ledger (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    supplier_id INTEGER NOT NULL,
    transaction_date TEXT NOT NULL DEFAULT (datetime('now')),
    type TEXT NOT NULL,
    reference_no TEXT NOT NULL DEFAULT '',
    debit REAL NOT NULL DEFAULT 0,
    credit REAL NOT NULL DEFAULT 0,
    balance REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (supplier_id) REFERENCES suppliers(id)
);
CREATE INDEX IF NOT EXISTS idx_supplier_ledger_supplier ON supplier_ledger(supplier_id);
";

const MIGRATION_026_PURCHASE_ORDERS: &str = "
CREATE TABLE IF NOT EXISTS purchase_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    po_no TEXT NOT NULL UNIQUE,
    supplier_id INTEGER NOT NULL,
    po_date TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'Draft',
    total_amount REAL NOT NULL DEFAULT 0,
    warehouse_id INTEGER,
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (supplier_id) REFERENCES suppliers(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_supplier ON purchase_orders(supplier_id);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_status ON purchase_orders(status);
";

const MIGRATION_027_PURCHASE_ORDER_ITEMS: &str = "
CREATE TABLE IF NOT EXISTS purchase_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    po_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    quantity REAL NOT NULL,
    received_quantity REAL NOT NULL DEFAULT 0,
    returned_quantity REAL NOT NULL DEFAULT 0,
    unit_price REAL NOT NULL,
    amount REAL NOT NULL,
    FOREIGN KEY (po_id) REFERENCES purchase_orders(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_purchase_order_items_po ON purchase_order_items(po_id);
";

const MIGRATION_028_GOODS_RECEIPTS: &str = "
CREATE TABLE IF NOT EXISTS goods_receipts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    receipt_no TEXT NOT NULL UNIQUE,
    po_id INTEGER NOT NULL,
    receipt_date TEXT NOT NULL DEFAULT (datetime('now')),
    warehouse_id INTEGER,
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (po_id) REFERENCES purchase_orders(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_goods_receipts_po ON goods_receipts(po_id);
";

const MIGRATION_029_GOODS_RECEIPT_ITEMS: &str = "
CREATE TABLE IF NOT EXISTS goods_receipt_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    receipt_id INTEGER NOT NULL,
    po_item_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    received_quantity REAL NOT NULL,
    FOREIGN KEY (receipt_id) REFERENCES goods_receipts(id) ON DELETE CASCADE,
    FOREIGN KEY (po_item_id) REFERENCES purchase_order_items(id),
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_goods_receipt_items_receipt ON goods_receipt_items(receipt_id);
";

const MIGRATION_030_PURCHASES: &str = "
CREATE TABLE IF NOT EXISTS purchases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    purchase_no TEXT NOT NULL UNIQUE,
    item_id INTEGER NOT NULL,
    warehouse_id INTEGER NOT NULL,
    batch_id INTEGER,
    quantity REAL NOT NULL,
    unit_cost REAL NOT NULL,
    total_cost REAL NOT NULL,
    supplier_name TEXT NOT NULL DEFAULT '',
    purchase_date TEXT NOT NULL DEFAULT (datetime('now')),
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (item_id) REFERENCES items(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_purchases_item ON purchases(item_id);
";

// ============================================================================
// Manufacturing Migrations
// ============================================================================

const MIGRATION_031_BOMS: &str = "
CREATE TABLE IF NOT EXISTS boms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bom_no TEXT NOT NULL UNIQUE,
    bom_name TEXT NOT NULL,
    finished_item_id INTEGER NOT NULL,
    quantity REAL NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (finished_item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_boms_finished_item ON boms(finished_item_id);
";

const MIGRATION_032_BOM_ITEMS: &str = "
CREATE TABLE IF NOT EXISTS bom_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bom_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    quantity REAL NOT NULL,
    unit_cost REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (bom_id) REFERENCES boms(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_bom_items_bom ON bom_items(bom_id);
";

const MIGRATION_033_WORK_ORDERS: &str = "
CREATE TABLE IF NOT EXISTS work_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wo_no TEXT NOT NULL UNIQUE,
    bom_id INTEGER NOT NULL,
    finished_item_id INTEGER NOT NULL,
    planned_quantity REAL NOT NULL,
    produced_quantity REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'Pending',
    warehouse_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (bom_id) REFERENCES boms(id),
    FOREIGN KEY (finished_item_id) REFERENCES items(id)
);
";

const MIGRATION_034_MATERIAL_CONSUMPTION: &str = "
CREATE TABLE IF NOT EXISTS material_consumption (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wo_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    consumed_quantity REAL NOT NULL,
    consumption_date TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (wo_id) REFERENCES work_orders(id),
    FOREIGN KEY (item_id) REFERENCES items(id)
);
";

const MIGRATION_035_PRODUCTIONS: &str = "
CREATE TABLE IF NOT EXISTS productions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    production_no TEXT NOT NULL UNIQUE,
    output_item_id INTEGER NOT NULL,
    output_quantity REAL NOT NULL,
    warehouse_id INTEGER NOT NULL,
    bom_id INTEGER,
    overhead_cost REAL NOT NULL DEFAULT 0,
    batch_id INTEGER,
    unit_cost REAL NOT NULL DEFAULT 0,
    total_material_cost REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'Completed',
    notes TEXT NOT NULL DEFAULT '',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (output_item_id) REFERENCES items(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id),
    FOREIGN KEY (bom_id) REFERENCES boms(id)
);
CREATE INDEX IF NOT EXISTS idx_productions_output_item ON productions(output_item_id);
";

const MIGRATION_036_PRODUCTION_INPUTS: &str = "
CREATE TABLE IF NOT EXISTS production_inputs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    production_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    quantity REAL NOT NULL,
    warehouse_id INTEGER NOT NULL,
    FOREIGN KEY (production_id) REFERENCES productions(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id),
    FOREIGN KEY (warehouse_id) REFERENCES warehouses(id)
);
CREATE INDEX IF NOT EXISTS idx_production_inputs_production ON production_inputs(production_id);
";

// ============================================================================
// Accounting Migrations
// ============================================================================

const MIGRATION_037_CHART_OF_ACCOUNTS: &str = "
CREATE TABLE IF NOT EXISTS chart_of_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    normal_balance TEXT NOT NULL DEFAULT 'Debit',
    parent_id INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (parent_id) REFERENCES chart_of_accounts(id)
);
CREATE INDEX IF NOT EXISTS idx_coa_code ON chart_of_accounts(code);
CREATE INDEX IF NOT EXISTS idx_coa_type ON chart_of_accounts(type);
";

const MIGRATION_038_JOURNAL_ENTRIES: &str = "
CREATE TABLE IF NOT EXISTS journal_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    reference_type TEXT,
    reference_id INTEGER,
    entry_date TEXT NOT NULL DEFAULT (datetime('now')),
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_journal_entries_date ON journal_entries(entry_date);
CREATE INDEX IF NOT EXISTS idx_journal_entries_ref ON journal_entries(reference_type, reference_id);
";

const MIGRATION_039_JOURNAL_LINES: &str = "
CREATE TABLE IF NOT EXISTS journal_lines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    journal_entry_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    debit REAL NOT NULL DEFAULT 0,
    credit REAL NOT NULL DEFAULT 0,
    description TEXT NOT NULL DEFAULT '',
    line_date TEXT NOT NULL DEFAULT (datetime('now')),
    reference_type TEXT,
    reference_id INTEGER,
    voided INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id) ON DELETE CASCADE,
    FOREIGN KEY (account_id) REFERENCES chart_of_accounts(id)
);
CREATE INDEX IF NOT EXISTS idx_journal_lines_entry ON journal_lines(journal_entry_id);
CREATE INDEX IF NOT EXISTS idx_journal_lines_account ON journal_lines(account_id);
CREATE INDEX IF NOT EXISTS idx_journal_lines_date ON journal_lines(line_date);
";

const MIGRATION_040_ACCOUNTING_PERIODS: &str = "
CREATE TABLE IF NOT EXISTS accounting_periods (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    period_name TEXT NOT NULL UNIQUE,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Open'
);
CREATE INDEX IF NOT EXISTS idx_accounting_periods_status ON accounting_periods(status);
";

// ============================================================================
// HR Migrations
// ============================================================================

const MIGRATION_041_EMPLOYEES: &str = "
CREATE TABLE IF NOT EXISTS employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_code TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    cnic_no TEXT NOT NULL DEFAULT '',
    address TEXT NOT NULL DEFAULT '',
    city TEXT NOT NULL DEFAULT '',
    department TEXT NOT NULL DEFAULT '',
    designation TEXT NOT NULL DEFAULT '',
    salary REAL NOT NULL DEFAULT 0,
    bank_name TEXT NOT NULL DEFAULT '',
    bank_account_no TEXT NOT NULL DEFAULT '',
    emergency_contact_name TEXT NOT NULL DEFAULT '',
    emergency_contact_phone TEXT NOT NULL DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_employees_code ON employees(employee_code);
";

const MIGRATION_042_EMPLOYEE_DOCUMENTS: &str = "
CREATE TABLE IF NOT EXISTS employee_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    document_name TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT '',
    file_path TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (employee_id) REFERENCES employees(id) ON DELETE CASCADE
);
";

const MIGRATION_043_SALARY_PAYMENTS: &str = "
CREATE TABLE IF NOT EXISTS salary_payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL,
    amount REAL NOT NULL,
    payment_date TEXT NOT NULL DEFAULT (datetime('now')),
    journal_entry_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (employee_id) REFERENCES employees(id)
);
CREATE INDEX IF NOT EXISTS idx_salary_payments_employee ON salary_payments(employee_id);
";

// ============================================================================
// Expense Migrations
// ============================================================================

const MIGRATION_044_EXPENSES: &str = "
CREATE TABLE IF NOT EXISTS expenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    expense_no TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    amount REAL NOT NULL,
    expense_date TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'Approved',
    created_by INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_expenses_category ON expenses(category);
CREATE INDEX IF NOT EXISTS idx_expenses_date ON expenses(expense_date);
";

const MIGRATION_045_EXPENSE_CATEGORIES: &str = "
CREATE TABLE IF NOT EXISTS expense_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_name TEXT NOT NULL UNIQUE,
    is_active INTEGER NOT NULL DEFAULT 1
);
";

// ============================================================================
// Forecast Migrations
// ============================================================================

const MIGRATION_046_DEMAND_FORECASTS: &str = "
CREATE TABLE IF NOT EXISTS demand_forecasts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL,
    forecast_date TEXT NOT NULL DEFAULT (datetime('now')),
    period TEXT NOT NULL,
    predicted_quantity REAL NOT NULL,
    confidence_level REAL NOT NULL DEFAULT 0,
    trend_direction TEXT NOT NULL DEFAULT 'stable',
    model_type TEXT NOT NULL DEFAULT 'weighted_moving_average',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_demand_forecasts_item ON demand_forecasts(item_id);
CREATE INDEX IF NOT EXISTS idx_demand_forecasts_date ON demand_forecasts(forecast_date);
";

const MIGRATION_047_FORECAST_RUNS: &str = "
CREATE TABLE IF NOT EXISTS forecast_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL UNIQUE,
    run_type TEXT NOT NULL DEFAULT 'auto',
    status TEXT NOT NULL DEFAULT 'running',
    items_processed INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);
";

const MIGRATION_048_FORECAST_MODEL_CONFIG: &str = "
CREATE TABLE IF NOT EXISTS forecast_model_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER,
    category TEXT,
    model_type TEXT NOT NULL DEFAULT 'weighted_moving_average',
    alpha REAL,
    beta REAL,
    gamma REAL,
    params_json TEXT,
    model_name TEXT,
    FOREIGN KEY (item_id) REFERENCES items(id)
);
";

const MIGRATION_049_FORECAST_SEASONAL_EVENTS: &str = "
CREATE TABLE IF NOT EXISTS forecast_seasonal_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    multiplier REAL NOT NULL DEFAULT 1.0,
    applies_to_category TEXT,
    applies_to_item_id INTEGER,
    FOREIGN KEY (applies_to_item_id) REFERENCES items(id)
);
";

const MIGRATION_050_FORECAST_ACCURACY: &str = "
CREATE TABLE IF NOT EXISTS forecast_accuracy (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    forecast_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    period TEXT NOT NULL,
    mape REAL NOT NULL DEFAULT 0,
    mae REAL NOT NULL DEFAULT 0,
    smape REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (forecast_id) REFERENCES demand_forecasts(id),
    FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_forecast_accuracy_item ON forecast_accuracy(item_id);
";

// ============================================================================
// Reports / Dashboard Migrations
// ============================================================================

const MIGRATION_051_CUSTOM_REPORTS: &str = "
CREATE TABLE IF NOT EXISTS custom_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    config TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
";

const MIGRATION_052_DASHBOARD_LAYOUTS: &str = "
CREATE TABLE IF NOT EXISTS dashboard_layouts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    layout_name TEXT NOT NULL DEFAULT 'Default',
    blocks TEXT NOT NULL DEFAULT '[]',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, layout_name),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
";

const MIGRATION_053_INVOICE_DRAFTS: &str = "
CREATE TABLE IF NOT EXISTS invoice_drafts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    customer_id INTEGER,
    items_data TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL DEFAULT 'active',
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_invoice_drafts_session ON invoice_drafts(session_id);
";

const MIGRATION_055_BOM_DESCRIPTION_CREATED_BY: &str = "
ALTER TABLE boms ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE boms ADD COLUMN created_by INTEGER REFERENCES users(id);
";

const MIGRATION_056_ADD_MISSING_FIELDS: &str = "
ALTER TABLE employees ADD COLUMN employment_type TEXT NOT NULL DEFAULT 'Permanent';
ALTER TABLE purchases ADD COLUMN status TEXT NOT NULL DEFAULT 'Completed';
ALTER TABLE productions ADD COLUMN end_date TEXT;
ALTER TABLE productions ADD COLUMN completed_qty REAL NOT NULL DEFAULT 0;
ALTER TABLE purchase_orders ADD COLUMN expected_date TEXT;
ALTER TABLE sales_orders ADD COLUMN delivery_date TEXT;
ALTER TABLE customers ADD COLUMN customer_type TEXT NOT NULL DEFAULT 'Regular';
ALTER TABLE customers ADD COLUMN notes TEXT NOT NULL DEFAULT '';
ALTER TABLE customers ADD COLUMN total_invoiced REAL NOT NULL DEFAULT 0;
ALTER TABLE customers ADD COLUMN total_paid REAL NOT NULL DEFAULT 0;
ALTER TABLE customers ADD COLUMN last_invoice_date TEXT;
ALTER TABLE users ADD COLUMN last_login TEXT;
ALTER TABLE warehouses ADD COLUMN capacity REAL NOT NULL DEFAULT 0;
ALTER TABLE boms ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
";

const MIGRATION_057_FORECAST_CONFIG_PARAMS_JSON: &str = "
ALTER TABLE forecast_model_config ADD COLUMN params_json TEXT;
ALTER TABLE forecast_model_config ADD COLUMN model_name TEXT;
";

const MIGRATION_054_ACTIVITY_LOG: &str = "
CREATE TABLE IF NOT EXISTS activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER,
    metadata TEXT,
    ip_address TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_activity_log_user ON activity_log(user_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_created ON activity_log(created_at);
";

// ============================================================================
// Seed Data
// ============================================================================

/// Seed initial data if tables are empty.
fn seed_data(conn: &Connection) -> Result<()> {
    // Seed admin role
    let role_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM roles",
        [],
        |row| row.get(0),
    )?;

    if role_count == 0 {
        tracing::info!("Seeding roles…");
        conn.execute(
            "INSERT INTO roles (role_name, description, is_system_role, is_active)
             VALUES ('admin', 'System Administrator — full access', 1, 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO roles (role_name, description, is_system_role, is_active)
             VALUES ('user', 'Standard user — role-based access', 1, 1)",
            [],
        )?;
    }

    // Seed default admin user
    let user_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE username = 'admin'",
        [],
        |row| row.get(0),
    )?;

    if user_count == 0 {
        tracing::info!("Seeding admin user…");
        let admin_password = std::env::var("DEFAULT_ADMIN_PASSWORD")
            .unwrap_or_else(|_| "admin123".to_string());
        let hash = bcrypt::hash(&admin_password, 12)
            .expect("Failed to hash admin password");

        conn.execute(
            "INSERT INTO users (username, email, password_hash, full_name, role, is_active)
             VALUES ('admin', 'admin@minierp.local', ?1, 'System Administrator', 'admin', 1)",
            [&hash],
        )?;
    }

    // Seed permissions
    let perm_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM permissions",
        [],
        |row| row.get(0),
    )?;

    if perm_count == 0 {
        tracing::info!("Seeding permissions…");
        let permissions = vec![
            ("dashboard.read", "dashboard", "read", "View dashboard"),
            ("users.read", "users", "read", "View users"),
            ("users.create", "users", "create", "Create users"),
            ("users.update", "users", "update", "Update users"),
            ("users.delete", "users", "delete", "Delete users"),
            ("roles.read", "roles", "read", "View roles"),
            ("roles.create", "roles", "create", "Create roles"),
            ("roles.update", "roles", "update", "Update roles"),
            ("roles.delete", "roles", "delete", "Delete roles"),
            ("inventory.read", "inventory", "read", "View inventory"),
            ("inventory.create", "inventory", "create", "Create items"),
            ("inventory.update", "inventory", "update", "Update items"),
            ("inventory.delete", "inventory", "delete", "Delete items"),
            ("invoices.read", "invoices", "read", "View invoices"),
            ("invoices.create", "invoices", "create", "Create invoices"),
            ("invoices.update", "invoices", "update", "Update invoices"),
            ("invoices.delete", "invoices", "delete", "Delete invoices"),
            ("customers.read", "customers", "read", "View customers"),
            ("customers.create", "customers", "create", "Create customers"),
            ("customers.update", "customers", "update", "Update customers"),
            ("customers.delete", "customers", "delete", "Delete customers"),
            ("settings.read", "settings", "read", "View settings"),
            ("settings.update", "settings", "update", "Update settings"),
        ];

        for (name, module, action, desc) in &permissions {
            conn.execute(
                "INSERT INTO permissions (permission_name, module, action, description)
                 VALUES (?1, ?2, ?3, ?4)",
                [name, module, action, desc],
            )?;
        }
    }

    // Assign all permissions to admin role
    let admin_role_id: i64 = conn.query_row(
        "SELECT id FROM roles WHERE role_name = 'admin'",
        [],
        |row| row.get(0),
    )?;

    let rp_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM role_permissions WHERE role_id = ?1",
        [admin_role_id],
        |row| row.get(0),
    )?;

    if rp_count == 0 {
        tracing::info!("Assigning all permissions to admin role…");
        conn.execute(
            "INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
             SELECT ?1, id FROM permissions",
            [admin_role_id],
        )?;
    }

    // ── Seed Warehouses ──
    let wh_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM warehouses",
        [],
        |row| row.get(0),
    )?;

    if wh_count == 0 {
        tracing::info!("Seeding warehouses…");
        conn.execute(
            "INSERT INTO warehouses (warehouse_code, warehouse_name, location, is_active)
             VALUES ('WH-001', 'Main Warehouse', 'Building A, Floor 1', 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO warehouses (warehouse_code, warehouse_name, location, is_active)
             VALUES ('WH-002', 'Secondary Warehouse', 'Building B, Floor 1', 1)",
            [],
        )?;
    }

    // ── Seed Items ──
    let item_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM items",
        [],
        |row| row.get(0),
    )?;

    if item_count == 0 {
        tracing::info!("Seeding items…");
        let items = vec![
            ("ITM-0001", "Premium Widget Alpha", "High-quality widget", "Widgets", "pcs", 150.0, 50.0, 25.0, 29.99, 0, 0, 1, 0),
            ("ITM-0002", "Industrial Bolt M12", "Stainless steel bolt", "Fasteners", "pcs", 3400.0, 500.0, 0.35, 0.45, 0, 0, 1, 0),
            ("ITM-0003", "Steel Rod 12mm x 6m", "Raw steel material", "Raw Materials", "pcs", 80.0, 100.0, 12.0, 15.75, 1, 0, 1, 0),
            ("ITM-0004", "Hydraulic Pump HPD-200", "Industrial hydraulic pump", "Equipment", "pcs", 5.0, 10.0, 980.0, 1250.0, 0, 0, 1, 0),
            ("ITM-0005", "Rubber Gasket Set", "Replacement gasket kit", "Consumables", "pcs", 0.0, 50.0, 6.5, 8.99, 0, 0, 1, 0),
            ("ITM-0006", "Copper Wire 2.5mm (100m)", "Electrical copper wire", "Raw Materials", "rolls", 25.0, 50.0, 38.0, 45.00, 1, 0, 1, 0),
            ("ITM-0007", "LED Panel Light 24W", "Office ceiling light", "Electrical", "pcs", 200.0, 250.0, 14.0, 18.50, 0, 0, 1, 0),
            ("ITM-0008", "Packaging Box 40x30x20cm", "Corrugated shipping box", "Packaging", "pcs", 1200.0, 200.0, 0.85, 1.20, 0, 0, 1, 0),
            ("ITM-0009", "Safety Helmet (Yellow)", "Construction safety helmet", "Safety", "pcs", 60.0, 100.0, 8.0, 12.00, 0, 0, 1, 0),
            ("ITM-0010", "Assembly Robot Arm v3", "Automated assembly arm", "Equipment", "pcs", 2.0, 5.0, 12000.0, 15999.99, 0, 0, 1, 0),
        ];

        for (code, name, desc, cat, uom, stock, reorder, cost, price, raw, fg, purch, mfg) in &items {
            conn.execute(
                "INSERT INTO items (item_code, item_name, description, category, unit_of_measure,
                    current_stock, reorder_level, standard_cost, selling_price,
                    is_raw_material, is_finished_good, is_purchased, is_manufactured)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![code, name, desc, cat, uom, stock, reorder, cost, price, raw, fg, purch, mfg],
            )?;
        }

        // Seed stock balances for warehouse 1
        tracing::info!("Seeding stock balances…");
        for item_id in 1..=10 {
            let stock: f64 = conn.query_row(
                "SELECT current_stock FROM items WHERE id = ?1",
                [item_id],
                |row| row.get(0),
            )?;
            if stock > 0.0 {
                conn.execute(
                    "INSERT INTO stock_balances (item_id, warehouse_id, quantity)
                     VALUES (?1, 1, ?2)",
                    rusqlite::params![item_id, stock],
                )?;
            }
        }
    }

    // ── Seed Tax Rates ──
    let tax_count: i64 = conn.query_row("SELECT COUNT(*) FROM tax_rates", [], |row| row.get(0))?;
    if tax_count == 0 {
        tracing::info!("Seeding tax rates…");
        conn.execute("INSERT INTO tax_rates (name, rate, is_default, is_active) VALUES ('No Tax', 0, 1, 1)", [])?;
        conn.execute("INSERT INTO tax_rates (name, rate, is_default, is_active) VALUES ('Standard 17%', 17.0, 0, 1)", [])?;
        conn.execute("INSERT INTO tax_rates (name, rate, is_default, is_active) VALUES ('Reduced 5%', 5.0, 0, 1)", [])?;
    }

    // ── Seed Payment Terms ──
    let pt_count: i64 = conn.query_row("SELECT COUNT(*) FROM payment_terms", [], |row| row.get(0))?;
    if pt_count == 0 {
        tracing::info!("Seeding payment terms…");
        conn.execute("INSERT INTO payment_terms (name, days, is_default, is_active) VALUES ('Due on Receipt', 0, 1, 1)", [])?;
        conn.execute("INSERT INTO payment_terms (name, days, is_default, is_active) VALUES ('Net 15', 15, 0, 1)", [])?;
        conn.execute("INSERT INTO payment_terms (name, days, is_default, is_active) VALUES ('Net 30', 30, 0, 1)", [])?;
        conn.execute("INSERT INTO payment_terms (name, days, is_default, is_active) VALUES ('Net 60', 60, 0, 1)", [])?;
    }

    // ── Seed Expense Categories ──
    let ec_count: i64 = conn.query_row("SELECT COUNT(*) FROM expense_categories", [], |row| row.get(0))?;
    if ec_count == 0 {
        tracing::info!("Seeding expense categories…");
        let cats = vec![
            "Rent", "Utilities", "Salaries", "Office Supplies", "Travel",
            "Marketing", "Insurance", "Maintenance", "Telecommunications",
            "Professional Services", "Taxes", "Shipping", "Raw Materials",
            "Miscellaneous", "Depreciation",
        ];
        for cat in &cats {
            conn.execute("INSERT INTO expense_categories (category_name, is_active) VALUES (?1, 1)", [cat])?;
        }
    }

    // ── Seed Chart of Accounts ──
    let coa_count: i64 = conn.query_row("SELECT COUNT(*) FROM chart_of_accounts", [], |row| row.get(0))?;
    if coa_count == 0 {
        tracing::info!("Seeding chart of accounts…");
        let accounts = vec![
            ("1000", "Cash", "Asset", "Debit"),
            ("1100", "Accounts Receivable", "Asset", "Debit"),
            ("1200", "Inventory", "Asset", "Debit"),
            ("1300", "Prepaid Expenses", "Asset", "Debit"),
            ("1500", "Fixed Assets", "Asset", "Debit"),
            ("2000", "Accounts Payable", "Liability", "Credit"),
            ("2100", "Tax Payable", "Liability", "Credit"),
            ("2200", "Accrued Expenses", "Liability", "Credit"),
            ("3000", "Owner's Equity", "Equity", "Credit"),
            ("3100", "Retained Earnings", "Equity", "Credit"),
            ("4000", "Sales Revenue", "Revenue", "Credit"),
            ("4100", "Service Revenue", "Revenue", "Credit"),
            ("5000", "Cost of Goods Sold", "Expense", "Debit"),
            ("6000", "Salary Expense", "Expense", "Debit"),
            ("6100", "Rent Expense", "Expense", "Debit"),
            ("6200", "Utilities Expense", "Expense", "Debit"),
            ("6300", "Office Supplies Expense", "Expense", "Debit"),
            // Stock-adjustment accounts (must stay LAST → ids 18, 19; hardcoded by
            // create_stock_movement's journal posting. Append only, never reorder.)
            ("5100", "Inventory Shrinkage", "Expense", "Debit"),
            ("4200", "Inventory Adjustment Gain", "Revenue", "Credit"),
        ];
        for (code, name, atype, nb) in &accounts {
            conn.execute(
                "INSERT INTO chart_of_accounts (code, name, type, normal_balance, is_active) VALUES (?1, ?2, ?3, ?4, 1)",
                [code, name, atype, nb],
            )?;
        }
    }

    // ── Ensure stock-adjustment accounts exist on pre-existing DBs (append-only) ──
    // Older databases seeded only 17 accounts; add the two new ones idempotently so
    // their ids remain 18 and 19 (code is UNIQUE → INSERT OR IGNORE is safe).
    for (code, name, atype, nb) in &[
        ("5100", "Inventory Shrinkage", "Expense", "Debit"),
        ("4200", "Inventory Adjustment Gain", "Revenue", "Credit"),
    ] {
        conn.execute(
            "INSERT OR IGNORE INTO chart_of_accounts (code, name, type, normal_balance, is_active) VALUES (?1, ?2, ?3, ?4, 1)",
            [code, name, atype, nb],
        )?;
    }

    // ── Seed Accounting Periods ──
    let ap_count: i64 = conn.query_row("SELECT COUNT(*) FROM accounting_periods", [], |row| row.get(0))?;
    if ap_count == 0 {
        tracing::info!("Seeding accounting periods…");
        let periods = vec![
            ("FY2025-Q1", "2025-01-01", "2025-03-31", "Closed"),
            ("FY2025-Q2", "2025-04-01", "2025-06-30", "Closed"),
            ("FY2025-Q3", "2025-07-01", "2025-09-30", "Closed"),
            ("FY2025-Q4", "2025-10-01", "2025-12-31", "Closed"),
            ("FY2026-Q1", "2026-01-01", "2026-03-31", "Closed"),
            ("FY2026-Q2", "2026-04-01", "2026-06-30", "Open"),
        ];
        for (name, start, end, status) in &periods {
            conn.execute(
                "INSERT INTO accounting_periods (period_name, start_date, end_date, status) VALUES (?1, ?2, ?3, ?4)",
                [name, start, end, status],
            )?;
        }
    }

    // ── Seed Seasonal Events ──
    let se_count: i64 = conn.query_row("SELECT COUNT(*) FROM forecast_seasonal_events", [], |row| row.get(0))?;
    if se_count == 0 {
        tracing::info!("Seeding seasonal events…");
        let events = vec![
            ("New Year", "2026-01-01", "2026-01-15", 1.5),
            ("Eid al-Fitr", "2026-03-20", "2026-03-30", 1.8),
            ("Eid al-Adha", "2026-05-27", "2026-06-05", 1.6),
            ("Black Friday", "2026-11-27", "2026-11-30", 2.0),
            ("Back to School", "2026-08-01", "2026-08-31", 1.3),
        ];
        for (name, start, end, mult) in &events {
            conn.execute(
                "INSERT INTO forecast_seasonal_events (event_name, start_date, end_date, multiplier) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![name, start, end, mult],
            )?;
        }
    }

    // ── Seed Employees ──
    let emp_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM employees",
        [],
        |row| row.get(0),
    )?;

    if emp_count == 0 {
        tracing::info!("Seeding employees…");
        let employees = vec![
            ("EMP-001", "Ahmed", "Hassan", "ahmed.hassan@minierp.local", "+92-300-111-0001",
             "61101-1234567-1", "12 Main Street, Gulberg", "Lahore", "Production", "Production Manager",
             120000.0, "HBL", "PK12HBLB1234567890", "Fatima Hassan", "+92-300-999-0001", 1, "Permanent"),
            ("EMP-002", "Sara", "Khan", "sara.khan@minierp.local", "+92-300-111-0002",
             "42201-2345678-3", "45 Clifton Road", "Karachi", "Finance", "Senior Accountant",
             95000.0, "UBL", "PK12UBLB0987654321", "Ali Khan", "+92-300-999-0002", 1, "Permanent"),
            ("EMP-003", "Usman", "Malik", "usman.malik@minierp.local", "+92-300-111-0003",
             "35202-3456789-5", "78 Faisal Town", "Islamabad", "Sales", "Sales Representative",
             70000.0, "MCB", "PK12MCBC1122334455", "Ayesha Malik", "+92-300-999-0003", 1, "Contract"),
            ("EMP-004", "Zara", "Qureshi", "zara.qureshi@minierp.local", "+92-300-111-0004",
             "63301-4567890-7", "23 Satellite Town", "Rawalpindi", "HR", "HR Coordinator",
             55000.0, "ABL", "PK12ABLB2233445566", "Imran Qureshi", "+92-300-999-0004", 1, "Probation"),
            ("EMP-005", "Bilal", "Ahmed", "bilal.ahmed@minierp.local", "+92-300-111-0005",
             "44101-5678901-9", "56 Garden Town", "Lahore", "IT", "IT Support Engineer",
             65000.0, "HBL", "PK12HBLB3344556677", "Nadia Ahmed", "+92-300-999-0005", 1, "Permanent"),
        ];

        for (code, first, last, email, phone, cnic, addr, city, dept, desig, salary, bank, acct, e_contact, e_phone, active, emp_type) in &employees {
            conn.execute(
                "INSERT INTO employees (employee_code, first_name, last_name, email, phone,
                    cnic_no, address, city, department, designation, salary,
                    bank_name, bank_account_no, emergency_contact_name, emergency_contact_phone,
                    is_active, employment_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                rusqlite::params![code, first, last, email, phone, cnic, addr, city, dept, desig, salary, bank, acct, e_contact, e_phone, active, emp_type],
            )?;
        }
    }

    // ── Seed BOMs ──
    let bom_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM boms",
        [],
        |row| row.get(0),
    )?;

    if bom_count == 0 {
        tracing::info!("Seeding BOMs…");

        // BOM 1: Premium Widget Alpha (item_id=1) requires Steel Rod + Industrial Bolt + Copper Wire
        conn.execute(
            "INSERT INTO boms (bom_no, bom_name, finished_item_id, quantity, is_active, version, description)
             VALUES ('BOM-001', 'Premium Widget Assembly', 1, 1.0, 1, 2,
                     'Standard assembly for Premium Widget Alpha — v2') ",
            [],
        )?;
        // BOM 1 items: Steel Rod 12mm x 6m (item 3), Industrial Bolt M12 (item 2), Copper Wire (item 6)
        conn.execute(
            "INSERT INTO bom_items (bom_id, item_id, quantity, unit_cost)
             VALUES (1, 3, 2.0, 12.00),
                    (1, 2, 8.0, 0.35),
                    (1, 6, 0.5, 38.00)",
            [],
        )?;

        // BOM 2: LED Panel Light 24W (item_id=7) requires Rubber Gasket + Packaging Box
        conn.execute(
            "INSERT INTO boms (bom_no, bom_name, finished_item_id, quantity, is_active, version, description)
             VALUES ('BOM-002', 'LED Light Panel Assembly', 7, 1.0, 1, 1,
                     'Assembly for 24W LED panel light') ",
            [],
        )?;
        // BOM 2 items: Rubber Gasket Set (item 5), Packaging Box (item 8)
        conn.execute(
            "INSERT INTO bom_items (bom_id, item_id, quantity, unit_cost)
             VALUES (2, 5, 1.0, 6.50),
                    (2, 8, 1.0, 0.85)",
            [],
        )?;
    }

    // ── Seed Suppliers ──
    let sup_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM suppliers",
        [],
        |row| row.get(0),
    )?;

    if sup_count == 0 {
        tracing::info!("Seeding suppliers…");
        conn.execute(
            "INSERT INTO suppliers (supplier_code, supplier_name, email, phone, address)
             VALUES ('SUP-001', 'TechSupply Corp', 'orders@techsupply.com', '+92-42-111-0001', '12 Industrial Zone, Lahore')",
            [],
        )?;
        conn.execute(
            "INSERT INTO suppliers (supplier_code, supplier_name, email, phone, address)
             VALUES ('SUP-002', 'RawMaterials Ltd', 'sales@rawmat.com', '+92-21-111-0002', '45 Port Road, Karachi')",
            [],
        )?;
    }

    // ── Seed Customers ──
    let cust_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM customers",
        [],
        |row| row.get(0),
    )?;

    if cust_count == 0 {
        tracing::info!("Seeding customers…");
        conn.execute(
            "INSERT INTO customers (customer_code, customer_name, email, phone, billing_address,
                shipping_address, payment_terms, credit_limit, opening_balance, is_active,
                customer_type, notes, total_invoiced, total_paid)
             VALUES ('CUST-001', 'Al-Rashid Traders', 'info@alrashid.pk', '+92-51-111-0001',
                     '10 Mall Road, Islamabad', '10 Mall Road, Islamabad',
                     'Net 30', 500000.0, 0.0, 1, 'Regular', 'Preferred customer since 2024', 4498.50, 0.0)",
            [],
        )?;
        conn.execute(
            "INSERT INTO customers (customer_code, customer_name, email, phone, billing_address,
                shipping_address, payment_terms, credit_limit, opening_balance, is_active,
                customer_type, notes, total_invoiced, total_paid)
             VALUES ('CUST-002', 'Gulfam Enterprises', 'orders@gulfam.com', '+92-42-111-0002',
                     '55 Gulberg, Lahore', '55 Gulberg, Lahore',
                     'Net 15', 250000.0, 0.0, 1, 'Wholesale', 'Bulk purchaser', 569.50, 569.50)",
            [],
        )?;
    }

    // ── Seed Invoices ──
    let inv_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM invoices",
        [],
        |row| row.get(0),
    )?;

    if inv_count == 0 {
        tracing::info!("Seeding invoices…");
        conn.execute(
            "INSERT INTO invoices (invoice_no, customer_id, invoice_date, due_date, status,
                total_amount, paid_amount, balance_amount, notes)
             VALUES ('INV-2026-0001', 1, '2026-06-01', '2026-07-01', 'Unpaid',
                     4498.50, 0.0, 4498.50, 'Widget order for Q3')",
            [],
        )?;
        conn.execute(
            "INSERT INTO invoices (invoice_no, customer_id, invoice_date, due_date, status,
                total_amount, paid_amount, balance_amount, notes)
             VALUES ('INV-2026-0002', 2, '2026-06-15', '2026-06-30', 'Paid',
                     569.50, 569.50, 0.0, 'Gasket and helmet supply')",
            [],
        )?;

        // Invoice items
        conn.execute(
            "INSERT INTO invoice_items (invoice_id, item_id, description, quantity, unit_price, amount)
             VALUES (1, 1, 'Premium Widget Alpha', 150, 29.99, 4498.50)",
            [],
        )?;
        conn.execute(
            "INSERT INTO invoice_items (invoice_id, item_id, description, quantity, unit_price, amount)
             VALUES (2, 5, 'Rubber Gasket Set', 50, 8.99, 449.50)",
            [],
        )?;
        conn.execute(
            "INSERT INTO invoice_items (invoice_id, item_id, description, quantity, unit_price, amount)
             VALUES (2, 9, 'Safety Helmet (Yellow)', 10, 12.00, 120.00)",
            [],
        )?;

        // Payment for invoice 2
        conn.execute(
            "INSERT INTO payments (payment_no, customer_id, invoice_id, payment_date, amount, payment_method, reference)
             VALUES ('PAY-2026-0001', 2, 2, '2026-06-16', 569.50, 'Bank Transfer', 'TRX-001')",
            [],
        )?;

        // Update customer totals to match
        conn.execute(
            "UPDATE customers SET total_invoiced = 4498.50, current_balance = 4498.50 WHERE id = 1",
            [],
        )?;
        conn.execute(
            "UPDATE customers SET total_invoiced = 569.50, total_paid = 569.50, last_invoice_date = '2026-06-15' WHERE id = 2",
            [],
        )?;
    }

    // ── Seed Purchase Orders ──
    let po_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM purchase_orders",
        [],
        |row| row.get(0),
    )?;

    if po_count == 0 {
        tracing::info!("Seeding purchase orders…");
        conn.execute(
            "INSERT INTO purchase_orders (po_no, supplier_id, po_date, status, total_amount, warehouse_id, expected_date, notes)
             VALUES ('PO-2026-0001', 1, '2026-06-15', 'Approved', 7200.00, 1, '2026-07-15',
                     'Monthly steel and bolt supply')",
            [],
        )?;
        conn.execute(
            "INSERT INTO purchase_orders (po_no, supplier_id, po_date, status, total_amount, warehouse_id, expected_date, notes)
             VALUES ('PO-2026-0002', 2, '2026-06-20', 'Draft', 3420.00, 1, '2026-07-01',
                     'Copper wire and rod order')",
            [],
        )?;

        // PO items
        conn.execute(
            "INSERT INTO purchase_order_items (po_id, item_id, description, quantity, unit_price, amount)
             VALUES (1, 3, 'Steel Rod 12mm x 6m', 300, 12.00, 3600.00),
                    (1, 2, 'Industrial Bolt M12', 6000, 0.35, 2100.00),
                    (1, 6, 'Copper Wire 2.5mm (100m)', 20, 75.00, 1500.00)",
            [],
        )?;
        conn.execute(
            "INSERT INTO purchase_order_items (po_id, item_id, description, quantity, unit_price, amount)
             VALUES (2, 3, 'Steel Rod 12mm x 6m', 100, 12.00, 1200.00),
                    (2, 6, 'Copper Wire 2.5mm (100m)', 30, 74.00, 2220.00)",
            [],
        )?;
    }

    // ── Seed Sales Orders ──
    let so_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sales_orders",
        [],
        |row| row.get(0),
    )?;

    if so_count == 0 {
        tracing::info!("Seeding sales orders…");
        conn.execute(
            "INSERT INTO sales_orders (so_no, customer_id, so_date, status, total_amount, warehouse_id, delivery_date, notes)
             VALUES ('SO-2026-0001', 1, '2026-06-10', 'Pending', 1499.50, 1, '2026-06-25',
                     'Widget order from Al-Rashid')",
            [],
        )?;
        conn.execute(
            "INSERT INTO sales_orders (so_no, customer_id, so_date, status, total_amount, warehouse_id, delivery_date, notes)
             VALUES ('SO-2026-0002', 2, '2026-06-18', 'Confirmed', 2500.00, 1, '2026-07-05',
                     'Hydraulic pump for Gulfam')",
            [],
        )?;

        // SO items
        conn.execute(
            "INSERT INTO sales_order_items (so_id, item_id, description, quantity, unit_price, amount)
             VALUES (1, 1, 'Premium Widget Alpha', 50, 29.99, 1499.50)",
            [],
        )?;
        conn.execute(
            "INSERT INTO sales_order_items (so_id, item_id, description, quantity, unit_price, amount)
             VALUES (2, 4, 'Hydraulic Pump HPD-200', 2, 1250.00, 2500.00)",
            [],
        )?;
    }

    // ── Seed Productions ──
    let prod_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM productions",
        [],
        |row| row.get(0),
    )?;

    if prod_count == 0 {
        tracing::info!("Seeding productions…");
        conn.execute(
            "INSERT INTO productions (production_no, output_item_id, output_quantity, warehouse_id, bom_id,
                overhead_cost, unit_cost, total_material_cost, status, completed_qty, end_date, notes)
             VALUES ('PROD-2026-0001', 1, 50, 1, 1,
                     500.0, 56.0, 2300.0, 'Completed', 50, '2026-06-20',
                     'Batch production of Premium Widgets')",
            [],
        )?;
        conn.execute(
            "INSERT INTO productions (production_no, output_item_id, output_quantity, warehouse_id, bom_id,
                overhead_cost, unit_cost, total_material_cost, status, completed_qty, end_date, notes)
             VALUES ('PROD-2026-0002', 7, 100, 1, 2,
                     200.0, 15.0, 1300.0, 'In Progress', 40, NULL,
                     'First batch of LED light panels')",
            [],
        )?;

        // Production inputs
        conn.execute(
            "INSERT INTO production_inputs (production_id, item_id, quantity, warehouse_id)
             VALUES (1, 3, 100, 1),
                    (1, 2, 400, 1),
                    (1, 6, 25, 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO production_inputs (production_id, item_id, quantity, warehouse_id)
             VALUES (2, 5, 100, 1),
                    (2, 8, 100, 1)",
            [],
        )?;
    }

    // ── Seed Journal Entries ──
    // Chart of Accounts IDs (auto-incremented): 1=Cash, 2=AR, 3=Inventory, 4=Prepaid, 5=Fixed,
    // 6=AP, 7=Tax Payable, 8=Accrued, 9=Equity, 10=Retained, 11=Revenue, 12=Service Rev,
    // 13=COGS, 14=Salary Exp, 15=Rent Exp, 16=Utilities Exp, 17=Office Supplies Exp
    let je_count: i64 = conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get(0))?;
    if je_count == 0 {
        tracing::info!("Seeding journal entries…");

        // Invoice 1: INV-2026-0001 — debit AR, credit Revenue for 4498.50
        conn.execute(
            "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('invoice', 1, '2026-06-01')",
            [],
        )?;
        let je1_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
             VALUES (?1, 2, 4498.50, 0, 'Invoice INV-2026-0001 - AR', '2026-06-01'),
                    (?1, 11, 0, 4498.50, 'Invoice INV-2026-0001 - Revenue', '2026-06-01')",
            [je1_id],
        )?;

        // Invoice 2: INV-2026-0002 — debit AR, credit Revenue for 569.50
        conn.execute(
            "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('invoice', 2, '2026-06-15')",
            [],
        )?;
        let je2_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
             VALUES (?1, 2, 569.50, 0, 'Invoice INV-2026-0002 - AR', '2026-06-15'),
                    (?1, 11, 0, 569.50, 'Invoice INV-2026-0002 - Revenue', '2026-06-15')",
            [je2_id],
        )?;

        // Payment 1: PAY-2026-0001 — debit Cash, credit AR for 569.50
        conn.execute(
            "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('payment', 1, '2026-06-16')",
            [],
        )?;
        let je3_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
             VALUES (?1, 1, 569.50, 0, 'Payment PAY-2026-0001 - Cash', '2026-06-16'),
                    (?1, 2, 0, 569.50, 'Payment PAY-2026-0001 - AR', '2026-06-16')",
            [je3_id],
        )?;

        // Purchase Order 1: PO-2026-0001 — debit Inventory, credit AP for 7200.00
        conn.execute(
            "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('purchase_order', 1, '2026-06-15')",
            [],
        )?;
        let je4_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
             VALUES (?1, 3, 7200.00, 0, 'PO-2026-0001 - Inventory', '2026-06-15'),
                    (?1, 6, 0, 7200.00, 'PO-2026-0001 - Accounts Payable', '2026-06-15')",
            [je4_id],
        )?;

        // Purchase Order 2: PO-2026-0002 — debit Inventory, credit AP for 3420.00
        conn.execute(
            "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('purchase_order', 2, '2026-06-20')",
            [],
        )?;
        let je5_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
             VALUES (?1, 3, 3420.00, 0, 'PO-2026-0002 - Inventory', '2026-06-20'),
                    (?1, 6, 0, 3420.00, 'PO-2026-0002 - Accounts Payable', '2026-06-20')",
            [je5_id],
        )?;

        // Seed customer ledger entries
        conn.execute(
            "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (1, '2026-06-01', 'INVOICE', 'INV-2026-0001', 4498.50, 0, 4498.50)",
            [],
        )?;
        conn.execute(
            "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (2, '2026-06-15', 'INVOICE', 'INV-2026-0002', 569.50, 0, 569.50)",
            [],
        )?;
        conn.execute(
            "INSERT INTO customer_ledger (customer_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (2, '2026-06-16', 'PAYMENT', 'PAY-2026-0001', 0, 569.50, 0)",
            [],
        )?;

        // Seed supplier ledger entries
        conn.execute(
            "INSERT INTO supplier_ledger (supplier_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (1, '2026-06-15', 'PURCHASE', 'PO-2026-0001', 7200.00, 0, 7200.00)",
            [],
        )?;
        conn.execute(
            "INSERT INTO supplier_ledger (supplier_id, transaction_date, type, reference_no, debit, credit, balance)
             VALUES (2, '2026-06-20', 'PURCHASE', 'PO-2026-0002', 3420.00, 0, 3420.00)",
            [],
        )?;
    }

    Ok(())
}
