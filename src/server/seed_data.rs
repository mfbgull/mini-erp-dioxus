use rusqlite::{Connection, Result};

/// Seed initial data if tables are empty.
pub fn seed_data(conn: &Connection) -> Result<()> {
    // Seed admin role
    let role_count: i64 = conn.query_row("SELECT COUNT(*) FROM roles", [], |row| row.get(0))?;

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
        let admin_password =
            std::env::var("DEFAULT_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
        let hash = bcrypt::hash(&admin_password, 12).expect("Failed to hash admin password");

        conn.execute(
            "INSERT INTO users (username, email, password_hash, full_name, role, is_active)
             VALUES ('admin', 'admin@minierp.local', ?1, 'System Administrator', 'admin', 1)",
            [&hash],
        )?;
    }

    // Seed permissions
    let perm_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM permissions", [], |row| row.get(0))?;

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
            (
                "customers.create",
                "customers",
                "create",
                "Create customers",
            ),
            (
                "customers.update",
                "customers",
                "update",
                "Update customers",
            ),
            (
                "customers.delete",
                "customers",
                "delete",
                "Delete customers",
            ),
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
    let wh_count: i64 = conn.query_row("SELECT COUNT(*) FROM warehouses", [], |row| row.get(0))?;

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
    let item_count: i64 = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))?;

    if item_count == 0 {
        tracing::info!("Seeding items…");
        let items = vec![
            (
                "ITM-0001",
                "Premium Widget Alpha",
                "High-quality widget",
                "Widgets",
                "pcs",
                150.0,
                50.0,
                25.0,
                29.99,
                0,
                0,
                1,
                0,
            ),
            (
                "ITM-0002",
                "Industrial Bolt M12",
                "Stainless steel bolt",
                "Fasteners",
                "pcs",
                3400.0,
                500.0,
                0.35,
                0.45,
                0,
                0,
                1,
                0,
            ),
            (
                "ITM-0003",
                "Steel Rod 12mm x 6m",
                "Raw steel material",
                "Raw Materials",
                "pcs",
                80.0,
                100.0,
                12.0,
                15.75,
                1,
                0,
                1,
                0,
            ),
            (
                "ITM-0004",
                "Hydraulic Pump HPD-200",
                "Industrial hydraulic pump",
                "Equipment",
                "pcs",
                5.0,
                10.0,
                980.0,
                1250.0,
                0,
                0,
                1,
                0,
            ),
            (
                "ITM-0005",
                "Rubber Gasket Set",
                "Replacement gasket kit",
                "Consumables",
                "pcs",
                0.0,
                50.0,
                6.5,
                8.99,
                0,
                0,
                1,
                0,
            ),
            (
                "ITM-0006",
                "Copper Wire 2.5mm (100m)",
                "Electrical copper wire",
                "Raw Materials",
                "rolls",
                25.0,
                50.0,
                38.0,
                45.00,
                1,
                0,
                1,
                0,
            ),
            (
                "ITM-0007",
                "LED Panel Light 24W",
                "Office ceiling light",
                "Electrical",
                "pcs",
                200.0,
                250.0,
                14.0,
                18.50,
                0,
                0,
                1,
                0,
            ),
            (
                "ITM-0008",
                "Packaging Box 40x30x20cm",
                "Corrugated shipping box",
                "Packaging",
                "pcs",
                1200.0,
                200.0,
                0.85,
                1.20,
                0,
                0,
                1,
                0,
            ),
            (
                "ITM-0009",
                "Safety Helmet (Yellow)",
                "Construction safety helmet",
                "Safety",
                "pcs",
                60.0,
                100.0,
                8.0,
                12.00,
                0,
                0,
                1,
                0,
            ),
            (
                "ITM-0010",
                "Assembly Robot Arm v3",
                "Automated assembly arm",
                "Equipment",
                "pcs",
                2.0,
                5.0,
                12000.0,
                15999.99,
                0,
                0,
                1,
                0,
            ),
        ];

        for (code, name, desc, cat, uom, stock, reorder, cost, price, raw, fg, purch, mfg) in &items
        {
            conn.execute(
                "INSERT INTO items (item_code, item_name, description, category, unit_of_measure,
                    current_stock, reorder_level, standard_cost, selling_price,
                    is_raw_material, is_finished_good, is_purchased, is_manufactured)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    code, name, desc, cat, uom, stock, reorder, cost, price, raw, fg, purch, mfg
                ],
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
        conn.execute(
            "INSERT INTO tax_rates (name, rate, is_default, is_active) VALUES ('No Tax', 0, 1, 1)",
            [],
        )?;
        conn.execute("INSERT INTO tax_rates (name, rate, is_default, is_active) VALUES ('Standard 17%', 17.0, 0, 1)", [])?;
        conn.execute("INSERT INTO tax_rates (name, rate, is_default, is_active) VALUES ('Reduced 5%', 5.0, 0, 1)", [])?;
    }

    // ── Seed Payment Terms ──
    let pt_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM payment_terms", [], |row| row.get(0))?;
    if pt_count == 0 {
        tracing::info!("Seeding payment terms…");
        conn.execute("INSERT INTO payment_terms (name, days, is_default, is_active) VALUES ('Due on Receipt', 0, 1, 1)", [])?;
        conn.execute("INSERT INTO payment_terms (name, days, is_default, is_active) VALUES ('Net 15', 15, 0, 1)", [])?;
        conn.execute("INSERT INTO payment_terms (name, days, is_default, is_active) VALUES ('Net 30', 30, 0, 1)", [])?;
        conn.execute("INSERT INTO payment_terms (name, days, is_default, is_active) VALUES ('Net 60', 60, 0, 1)", [])?;
    }

    // ── Seed Expense Categories ──
    let ec_count: i64 = conn.query_row("SELECT COUNT(*) FROM expense_categories", [], |row| {
        row.get(0)
    })?;
    if ec_count == 0 {
        tracing::info!("Seeding expense categories…");
        let cats = vec![
            "Rent",
            "Utilities",
            "Salaries",
            "Office Supplies",
            "Travel",
            "Marketing",
            "Insurance",
            "Maintenance",
            "Telecommunications",
            "Professional Services",
            "Taxes",
            "Shipping",
            "Raw Materials",
            "Miscellaneous",
            "Depreciation",
        ];
        for cat in &cats {
            conn.execute(
                "INSERT INTO expense_categories (category_name, is_active) VALUES (?1, 1)",
                [cat],
            )?;
        }
    }

    // ── Seed Chart of Accounts ──
    let coa_count: i64 = conn.query_row("SELECT COUNT(*) FROM chart_of_accounts", [], |row| {
        row.get(0)
    })?;
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
    let ap_count: i64 = conn.query_row("SELECT COUNT(*) FROM accounting_periods", [], |row| {
        row.get(0)
    })?;
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
    let se_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM forecast_seasonal_events", [], |row| {
            row.get(0)
        })?;
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
    let emp_count: i64 = conn.query_row("SELECT COUNT(*) FROM employees", [], |row| row.get(0))?;

    if emp_count == 0 {
        tracing::info!("Seeding employees…");
        let employees = vec![
            (
                "EMP-001",
                "Ahmed",
                "Hassan",
                "ahmed.hassan@minierp.local",
                "+92-300-111-0001",
                "61101-1234567-1",
                "12 Main Street, Gulberg",
                "Lahore",
                "Production",
                "Production Manager",
                120000.0,
                "HBL",
                "PK12HBLB1234567890",
                "Fatima Hassan",
                "+92-300-999-0001",
                1,
                "Permanent",
            ),
            (
                "EMP-002",
                "Sara",
                "Khan",
                "sara.khan@minierp.local",
                "+92-300-111-0002",
                "42201-2345678-3",
                "45 Clifton Road",
                "Karachi",
                "Finance",
                "Senior Accountant",
                95000.0,
                "UBL",
                "PK12UBLB0987654321",
                "Ali Khan",
                "+92-300-999-0002",
                1,
                "Permanent",
            ),
            (
                "EMP-003",
                "Usman",
                "Malik",
                "usman.malik@minierp.local",
                "+92-300-111-0003",
                "35202-3456789-5",
                "78 Faisal Town",
                "Islamabad",
                "Sales",
                "Sales Representative",
                70000.0,
                "MCB",
                "PK12MCBC1122334455",
                "Ayesha Malik",
                "+92-300-999-0003",
                1,
                "Contract",
            ),
            (
                "EMP-004",
                "Zara",
                "Qureshi",
                "zara.qureshi@minierp.local",
                "+92-300-111-0004",
                "63301-4567890-7",
                "23 Satellite Town",
                "Rawalpindi",
                "HR",
                "HR Coordinator",
                55000.0,
                "ABL",
                "PK12ABLB2233445566",
                "Imran Qureshi",
                "+92-300-999-0004",
                1,
                "Probation",
            ),
            (
                "EMP-005",
                "Bilal",
                "Ahmed",
                "bilal.ahmed@minierp.local",
                "+92-300-111-0005",
                "44101-5678901-9",
                "56 Garden Town",
                "Lahore",
                "IT",
                "IT Support Engineer",
                65000.0,
                "HBL",
                "PK12HBLB3344556677",
                "Nadia Ahmed",
                "+92-300-999-0005",
                1,
                "Permanent",
            ),
        ];

        for (
            code,
            first,
            last,
            email,
            phone,
            cnic,
            addr,
            city,
            dept,
            desig,
            salary,
            bank,
            acct,
            e_contact,
            e_phone,
            active,
            emp_type,
        ) in &employees
        {
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
    let bom_count: i64 = conn.query_row("SELECT COUNT(*) FROM boms", [], |row| row.get(0))?;

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
    let sup_count: i64 = conn.query_row("SELECT COUNT(*) FROM suppliers", [], |row| row.get(0))?;

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
    let cust_count: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |row| row.get(0))?;

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
    let inv_count: i64 = conn.query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))?;

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
    let po_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM purchase_orders", [], |row| row.get(0))?;

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
    let so_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM sales_orders", [], |row| row.get(0))?;

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
    let prod_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM productions", [], |row| row.get(0))?;

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
    let je_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM journal_entries", [], |row| row.get(0))?;
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
