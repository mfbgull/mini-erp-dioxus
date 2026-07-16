use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/accounting/accounts", get(list_accounts))
        .route("/api/accounting/accounts/balances", get(account_balances))
        .route("/api/accounting/accounts/{code}", get(get_account))
        .route("/api/accounting/accounts/{code}/balance", get(account_balance))
        .route("/api/accounting/periods", get(list_periods))
        .route("/api/accounting/periods/{id}", get(get_period).put(update_period))
        // Tax & Payment Terms
        .route("/api/tax-rates", get(list_tax_rates))
        .route("/api/payment-terms", get(list_payment_terms))
        // Expenses
        .route("/api/expenses", get(list_expenses).post(create_expense))
        .route("/api/expenses/{id}", get(get_expense).delete(delete_expense))
        .route("/api/expenses/categories", get(list_expense_categories).post(create_expense_category))
        // Employees
        .route("/api/employees", get(list_employees).post(create_employee))
        .route("/api/employees/{id}", get(get_employee).put(update_employee).delete(delete_employee))
        .route("/api/employees/{id}/salary", post(pay_salary))
        .route("/api/employees/{id}/salary-payments", get(list_salary_payments))
        // Journal Entries
        .route("/api/accounting/journal-entries", get(list_journal_entries).post(create_journal_entry))
        .route("/api/accounting/journal-entries/{id}", get(get_journal_entry))
}

async fn list_accounts(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare("SELECT id, code, name, type, normal_balance, parent_id, is_active FROM chart_of_accounts ORDER BY code").unwrap();
    let items: Vec<ChartOfAccount> = stmt.query_map([], |row| {
        Ok(ChartOfAccount {
            id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
            account_type: row.get(3)?, normal_balance: row.get(4)?,
            parent_id: row.get(5)?, is_active: row.get::<_, i64>(6)? != 0,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn account_balances(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT a.id, a.code, a.name, a.type, a.normal_balance,
                COALESCE(SUM(jl.debit), 0) as total_debit,
                COALESCE(SUM(jl.credit), 0) as total_credit,
                COALESCE(SUM(jl.debit), 0) - COALESCE(SUM(jl.credit), 0) as balance
         FROM chart_of_accounts a
         LEFT JOIN journal_lines jl ON a.id = jl.account_id AND jl.voided = 0
         GROUP BY a.id ORDER BY a.code"
    ).unwrap();
    let items: Vec<AccountBalance> = stmt.query_map([], |row| {
        Ok(AccountBalance {
            id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
            account_type: row.get(3)?, normal_balance: row.get(4)?,
            debit: row.get(5)?, credit: row.get(6)?, balance: row.get(7)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_account(State(_state): State<AppState>, Path(code): Path<String>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, code, name, type, normal_balance, parent_id, is_active FROM chart_of_accounts WHERE code = ?1",
        [&code],
        |row| Ok(ChartOfAccount {
            id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
            account_type: row.get(3)?, normal_balance: row.get(4)?,
            parent_id: row.get(5)?, is_active: row.get::<_, i64>(6)? != 0,
        }),
    );
    match result {
        Ok(a) => (StatusCode::OK, Json(json!({ "success": true, "data": a }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Account not found." }))),
    }
}

async fn account_balance(State(_state): State<AppState>, Path(code): Path<String>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT COALESCE(SUM(jl.debit), 0) - COALESCE(SUM(jl.credit), 0)
         FROM journal_lines jl JOIN chart_of_accounts a ON jl.account_id = a.id
         WHERE a.code = ?1 AND jl.voided = 0",
        [&code],
        |row| row.get::<_, f64>(0),
    );
    match result {
        Ok(balance) => (StatusCode::OK, Json(json!({ "success": true, "data": { "balance": balance } }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Account not found." }))),
    }
}

async fn list_periods(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare("SELECT id, period_name, start_date, end_date, status FROM accounting_periods ORDER BY start_date").unwrap();
    let items: Vec<AccountingPeriod> = stmt.query_map([], |row| {
        Ok(AccountingPeriod {
            id: row.get(0)?, period_name: row.get(1)?, start_date: row.get(2)?,
            end_date: row.get(3)?, status: row.get(4)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_period(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, period_name, start_date, end_date, status FROM accounting_periods WHERE id = ?1",
        [id],
        |row| Ok(AccountingPeriod { id: row.get(0)?, period_name: row.get(1)?, start_date: row.get(2)?, end_date: row.get(3)?, status: row.get(4)? }),
    );
    match result {
        Ok(p) => (StatusCode::OK, Json(json!({ "success": true, "data": p }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Period not found." }))),
    }
}

async fn update_period(State(_state): State<AppState>, Path(id): Path<i64>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let status = body.get("status").and_then(|v| v.as_str()).unwrap_or("Open");
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("UPDATE accounting_periods SET status = ?1 WHERE id = ?2", rusqlite::params![status, id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Period updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Period not found." }))),
        Err(e) => { tracing::error!("Failed to update period: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update period." }))) }
    }
}

async fn list_tax_rates(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare("SELECT id, name, rate, is_default, is_active FROM tax_rates WHERE is_active = 1").unwrap();
    let items: Vec<TaxRate> = stmt.query_map([], |row| {
        Ok(TaxRate { id: row.get(0)?, name: row.get(1)?, rate: row.get(2)?, is_default: row.get::<_, i64>(3)? != 0, is_active: row.get::<_, i64>(4)? != 0 })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_payment_terms(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare("SELECT id, name, days, is_default, is_active FROM payment_terms WHERE is_active = 1").unwrap();
    let items: Vec<PaymentTerm> = stmt.query_map([], |row| {
        Ok(PaymentTerm { id: row.get(0)?, name: row.get(1)?, days: row.get(2)?, is_default: row.get::<_, i64>(3)? != 0, is_active: row.get::<_, i64>(4)? != 0 })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

// ============================================================================
// Expenses
// ============================================================================

async fn list_expenses(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, expense_no, category, description, amount, expense_date, status, created_by, created_at
         FROM expenses ORDER BY created_at DESC"
    ).unwrap();
    let items: Vec<Expense> = stmt.query_map([], |row| {
        Ok(Expense {
            id: row.get(0)?, expense_no: row.get(1)?, category: row.get(2)?,
            description: row.get(3)?, amount: row.get(4)?, expense_date: row.get(5)?,
            status: row.get(6)?, created_by: row.get(7)?, created_at: row.get(8)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_expense(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, expense_no, category, description, amount, expense_date, status, created_by, created_at
         FROM expenses WHERE id = ?1",
        [id],
        |row| Ok(Expense { id: row.get(0)?, expense_no: row.get(1)?, category: row.get(2)?, description: row.get(3)?, amount: row.get(4)?, expense_date: row.get(5)?, status: row.get(6)?, created_by: row.get(7)?, created_at: row.get(8)? }),
    );
    match result {
        Ok(e) => (StatusCode::OK, Json(json!({ "success": true, "data": e }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Expense not found." }))),
    }
}

async fn create_expense(State(_state): State<AppState>, Json(form): Json<ExpenseForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM expenses", [], |row| row.get(0)).unwrap_or(1);
    let eno = format!("EXP-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let result = db.execute(
        "INSERT INTO expenses (expense_no, category, description, amount, expense_date) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![eno, form.category, form.description, form.amount, form.expense_date],
    );
    match result {
        Ok(_) => {
            let expense_id = db.last_insert_rowid();
            // Auto-journal: debit Expense (account_id=15 for Rent, or use category-based mapping), credit Cash (account_id=1)
            // For simplicity, use account 15 (Rent Expense) as default; in production, map category to account
            let expense_account_id: i64 = match form.category.to_lowercase().as_str() {
                "salary" | "salaries" => 14,
                "rent" => 15,
                "utilities" => 16,
                "office supplies" => 17,
                _ => 15, // default to Rent Expense
            };
            db.execute(
                "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('expense', ?1, ?2)",
                rusqlite::params![expense_id, form.expense_date],
            ).ok();
            let je_id = db.last_insert_rowid();
            db.execute(
                "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
                 VALUES (?1, ?3, ?2, 0, ?4, ?5),
                        (?1, 1, 0, ?2, ?6, ?5)",
                rusqlite::params![je_id, form.amount, expense_account_id,
                    format!("Expense {}", eno), form.expense_date, format!("Cash - Expense {}", eno)],
            ).ok();
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": expense_id, "expense_no": eno } })))
        }
        Err(e) => { tracing::error!("Failed to create expense: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create expense." }))) }
    }
}

async fn delete_expense(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("DELETE FROM expenses WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Expense deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Expense not found." }))),
        Err(e) => { tracing::error!("Failed to delete expense: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete expense." }))) }
    }
}

async fn list_expense_categories(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare("SELECT id, category_name, is_active FROM expense_categories WHERE is_active = 1 ORDER BY category_name").unwrap();
    let items: Vec<ExpenseCategory> = stmt.query_map([], |row| {
        Ok(ExpenseCategory { id: row.get(0)?, category_name: row.get(1)?, is_active: row.get::<_, i64>(2)? != 0 })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn create_expense_category(State(_state): State<AppState>, Json(form): Json<ExpenseCategoryForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("INSERT INTO expense_categories (category_name) VALUES (?1)", [&form.category_name]);
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid() } }))),
        Err(e) => { tracing::error!("Failed to create category: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create category." }))) }
    }
}

// ============================================================================
// Employees
// ============================================================================

async fn list_employees(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, employee_code, first_name, last_name, email, phone, cnic_no, address, city,
                department, designation, employment_type, salary, bank_name, bank_account_no,
                emergency_contact_name, emergency_contact_phone, is_active, created_at, updated_at
         FROM employees WHERE is_active = 1 ORDER BY employee_code"
    ).unwrap();
    let items: Vec<Employee> = stmt.query_map([], |row| {
        Ok(Employee {
            id: row.get(0)?, employee_code: row.get(1)?, first_name: row.get(2)?,
            last_name: row.get(3)?, email: row.get(4)?, phone: row.get(5)?,
            cnic_no: row.get(6)?, address: row.get(7)?, city: row.get(8)?,
            department: row.get(9)?, designation: row.get(10)?,
            employment_type: row.get(11)?, salary: row.get(12)?,
            bank_name: row.get(13)?, bank_account_no: row.get(14)?,
            emergency_contact_name: row.get(15)?, emergency_contact_phone: row.get(16)?,
            is_active: row.get::<_, i64>(17)? != 0, created_at: row.get(18)?, updated_at: row.get(19)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_employee(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.query_row(
        "SELECT id, employee_code, first_name, last_name, email, phone, cnic_no, address, city,
                department, designation, employment_type, salary, bank_name, bank_account_no,
                emergency_contact_name, emergency_contact_phone, is_active, created_at, updated_at
         FROM employees WHERE id = ?1",
        [id],
        |row| Ok(Employee {
            id: row.get(0)?, employee_code: row.get(1)?, first_name: row.get(2)?,
            last_name: row.get(3)?, email: row.get(4)?, phone: row.get(5)?,
            cnic_no: row.get(6)?, address: row.get(7)?, city: row.get(8)?,
            department: row.get(9)?, designation: row.get(10)?,
            employment_type: row.get(11)?, salary: row.get(12)?,
            bank_name: row.get(13)?, bank_account_no: row.get(14)?,
            emergency_contact_name: row.get(15)?, emergency_contact_phone: row.get(16)?,
            is_active: row.get::<_, i64>(17)? != 0, created_at: row.get(18)?, updated_at: row.get(19)?,
        }),
    );
    match result {
        Ok(e) => (StatusCode::OK, Json(json!({ "success": true, "data": e }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Employee not found." }))),
    }
}

async fn create_employee(State(_state): State<AppState>, Json(form): Json<EmployeeForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM employees", [], |row| row.get(0)).unwrap_or(1);
    let ecode = format!("EMP-{:04}", seq);
    let result = db.execute(
        "INSERT INTO employees (employee_code, first_name, last_name, email, phone, cnic_no, address, city, department, designation, salary, bank_name, bank_account_no, emergency_contact_name, emergency_contact_phone)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        rusqlite::params![ecode, form.first_name, form.last_name, form.email.as_deref().unwrap_or(""),
            form.phone.as_deref().unwrap_or(""), form.cnic_no.as_deref().unwrap_or(""),
            form.address.as_deref().unwrap_or(""), form.city.as_deref().unwrap_or(""),
            form.department.as_deref().unwrap_or(""), form.designation.as_deref().unwrap_or(""),
            form.salary.unwrap_or(0.0), form.bank_name.as_deref().unwrap_or(""),
            form.bank_account_no.as_deref().unwrap_or(""), form.emergency_contact_name.as_deref().unwrap_or(""),
            form.emergency_contact_phone.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid(), "employee_code": ecode } }))),
        Err(e) => { tracing::error!("Failed to create employee: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create employee." }))) }
    }
}

async fn update_employee(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<EmployeeForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "UPDATE employees SET first_name=?1, last_name=?2, email=?3, phone=?4, cnic_no=?5,
         address=?6, city=?7, department=?8, designation=?9, salary=?10, bank_name=?11,
         bank_account_no=?12, emergency_contact_name=?13, emergency_contact_phone=?14, updated_at=datetime('now')
         WHERE id=?15",
        rusqlite::params![form.first_name, form.last_name, form.email.as_deref().unwrap_or(""),
            form.phone.as_deref().unwrap_or(""), form.cnic_no.as_deref().unwrap_or(""),
            form.address.as_deref().unwrap_or(""), form.city.as_deref().unwrap_or(""),
            form.department.as_deref().unwrap_or(""), form.designation.as_deref().unwrap_or(""),
            form.salary.unwrap_or(0.0), form.bank_name.as_deref().unwrap_or(""),
            form.bank_account_no.as_deref().unwrap_or(""), form.emergency_contact_name.as_deref().unwrap_or(""),
            form.emergency_contact_phone.as_deref().unwrap_or(""), id],
    );
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Employee updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Employee not found." }))),
        Err(e) => { tracing::error!("Failed to update employee: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update employee." }))) }
    }
}

async fn delete_employee(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute("UPDATE employees SET is_active = 0, updated_at = datetime('now') WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Employee deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Employee not found." }))),
        Err(e) => { tracing::error!("Failed to delete employee: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete employee." }))) }
    }
}

async fn pay_salary(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<SalaryPaymentForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "INSERT INTO salary_payments (employee_id, amount, payment_date) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, form.amount, form.payment_date],
    );
    match result {
        Ok(_) => {
            let sp_id = db.last_insert_rowid();
            // Auto-journal: debit Salary Expense (account_id=14), credit Cash (account_id=1)
            db.execute(
                "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES ('salary', ?1, ?2)",
                rusqlite::params![sp_id, form.payment_date],
            ).ok();
            let je_id = db.last_insert_rowid();
            db.execute(
                "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date)
                 VALUES (?1, 14, ?2, 0, ?3, ?4),
                        (?1, 1, 0, ?2, ?5, ?4)",
                rusqlite::params![je_id, form.amount,
                    format!("Salary - Employee {}", id), form.payment_date,
                    format!("Cash - Salary Employee {}", id)],
            ).ok();
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": sp_id, "message": "Salary payment recorded." } })))
        }
        Err(e) => { tracing::error!("Failed to record salary: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record salary payment." }))) }
    }
}

async fn list_salary_payments(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = db.prepare(
        "SELECT id, employee_id, amount, payment_date FROM salary_payments WHERE employee_id = ?1 ORDER BY payment_date DESC"
    ).unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([id], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "employee_id": row.get::<_, i64>(1)?,
            "amount": row.get::<_, f64>(2)?,
            "payment_date": row.get::<_, String>(3)?,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_journal_entries(
    State(_state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let from_date = params.get("from_date").map(|s| s.as_str()).unwrap_or("2000-01-01");
    let to_date = params.get("to_date").map(|s| s.as_str()).unwrap_or("2099-12-31");
    let mut stmt = db.prepare(
        "SELECT id, reference_type, reference_id, entry_date, created_by, created_at
         FROM journal_entries WHERE entry_date BETWEEN ?1 AND ?2 ORDER BY entry_date DESC, id DESC"
    ).unwrap();
    let entries: Vec<JournalEntry> = stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        Ok(JournalEntry {
            id: row.get(0)?, reference_type: row.get(1)?, reference_id: row.get(2)?,
            entry_date: row.get(3)?, created_by: row.get(4)?, created_at: row.get(5)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": entries })))
}

async fn create_journal_entry(
    State(_state): State<AppState>,
    Json(form): Json<JournalEntryForm>,
) -> impl IntoResponse {
    if form.lines.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "At least one journal line is required." })));
    }
    let total_debit: f64 = form.lines.iter().map(|l| l.debit).sum();
    let total_credit: f64 = form.lines.iter().map(|l| l.credit).sum();
    if (total_debit - total_credit).abs() > 0.01 {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "success": false,
            "error": format!("Debits ({}) must equal credits ({}).", total_debit, total_credit)
        })));
    }
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let result = db.execute(
        "INSERT INTO journal_entries (reference_type, reference_id, entry_date) VALUES (?1, ?2, ?3)",
        rusqlite::params![form.reference_type, form.reference_id, form.entry_date],
    );
    match result {
        Ok(_) => {
            let entry_id = db.last_insert_rowid();
            for line in &form.lines {
                db.execute(
                    "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date, reference_type, reference_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![entry_id, line.account_id, line.debit, line.credit,
                        line.description.as_deref().unwrap_or(""), form.entry_date,
                        form.reference_type, form.reference_id],
                ).ok();
            }
            (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": entry_id } })))
        }
        Err(e) => {
            tracing::error!("Failed to create journal entry: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create journal entry." })))
        }
    }
}

async fn get_journal_entry(
    State(_state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap_or_else(|e| e.into_inner());
    let entry = db.query_row(
        "SELECT id, reference_type, reference_id, entry_date, created_by, created_at
         FROM journal_entries WHERE id = ?1",
        [id],
        |row| Ok(JournalEntry {
            id: row.get(0)?, reference_type: row.get(1)?, reference_id: row.get(2)?,
            entry_date: row.get(3)?, created_by: row.get(4)?, created_at: row.get(5)?,
        }),
    );
    match entry {
        Ok(e) => {
            let mut stmt = db.prepare(
                "SELECT jl.id, jl.journal_entry_id, jl.account_id, a.code, a.name,
                        jl.debit, jl.credit, jl.description, jl.line_date,
                        jl.reference_type, jl.reference_id, jl.voided
                 FROM journal_lines jl
                 LEFT JOIN chart_of_accounts a ON jl.account_id = a.id
                 WHERE jl.journal_entry_id = ?1 ORDER BY jl.id"
            ).unwrap();
            let lines: Vec<JournalLine> = stmt.query_map([id], |row| {
                Ok(JournalLine {
                    id: row.get(0)?, journal_entry_id: row.get(1)?, account_id: row.get(2)?,
                    account_code: row.get(3)?, account_name: row.get(4)?,
                    debit: row.get(5)?, credit: row.get(6)?, description: row.get(7)?,
                    line_date: row.get(8)?, reference_type: row.get(9)?,
                    reference_id: row.get(10)?, voided: row.get::<_, i64>(11)? != 0,
                })
            }).unwrap().filter_map(|r| r.ok()).collect();
            (StatusCode::OK, Json(json!({ "success": true, "data": { "entry": e, "lines": lines } })))
        }
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Journal entry not found." }))),
    }
}
