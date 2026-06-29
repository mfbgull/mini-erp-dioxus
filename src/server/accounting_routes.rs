use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
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
}

async fn list_accounts(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE accounting_periods SET status = ?1 WHERE id = ?2", rusqlite::params![status, id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Period updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Period not found." }))),
        Err(e) => { tracing::error!("Failed to update period: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update period." }))) }
    }
}

async fn list_tax_rates(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT id, name, rate, is_default, is_active FROM tax_rates WHERE is_active = 1").unwrap();
    let items: Vec<TaxRate> = stmt.query_map([], |row| {
        Ok(TaxRate { id: row.get(0)?, name: row.get(1)?, rate: row.get(2)?, is_default: row.get::<_, i64>(3)? != 0, is_active: row.get::<_, i64>(4)? != 0 })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_payment_terms(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
    let seq: i64 = db.query_row("SELECT COUNT(*) + 1 FROM expenses", [], |row| row.get(0)).unwrap_or(1);
    let eno = format!("EXP-{}-{:04}", chrono::Utc::now().format("%Y"), seq);
    let result = db.execute(
        "INSERT INTO expenses (expense_no, category, description, amount, expense_date) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![eno, form.category, form.description, form.amount, form.expense_date],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid(), "expense_no": eno } }))),
        Err(e) => { tracing::error!("Failed to create expense: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create expense." }))) }
    }
}

async fn delete_expense(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("DELETE FROM expenses WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Expense deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Expense not found." }))),
        Err(e) => { tracing::error!("Failed to delete expense: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete expense." }))) }
    }
}

async fn list_expense_categories(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT id, category_name, is_active FROM expense_categories WHERE is_active = 1 ORDER BY category_name").unwrap();
    let items: Vec<ExpenseCategory> = stmt.query_map([], |row| {
        Ok(ExpenseCategory { id: row.get(0)?, category_name: row.get(1)?, is_active: row.get::<_, i64>(2)? != 0 })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn create_expense_category(State(_state): State<AppState>, Json(form): Json<ExpenseCategoryForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT id, employee_code, first_name, last_name, email, phone, cnic_no, address, city,
                department, designation, salary, bank_name, bank_account_no,
                emergency_contact_name, emergency_contact_phone, is_active, created_at, updated_at
         FROM employees WHERE is_active = 1 ORDER BY employee_code"
    ).unwrap();
    let items: Vec<Employee> = stmt.query_map([], |row| {
        Ok(Employee {
            id: row.get(0)?, employee_code: row.get(1)?, first_name: row.get(2)?,
            last_name: row.get(3)?, email: row.get(4)?, phone: row.get(5)?,
            cnic_no: row.get(6)?, address: row.get(7)?, city: row.get(8)?,
            department: row.get(9)?, designation: row.get(10)?, salary: row.get(11)?,
            bank_name: row.get(12)?, bank_account_no: row.get(13)?,
            emergency_contact_name: row.get(14)?, emergency_contact_phone: row.get(15)?,
            is_active: row.get::<_, i64>(16)? != 0, created_at: row.get(17)?, updated_at: row.get(18)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_employee(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT id, employee_code, first_name, last_name, email, phone, cnic_no, address, city,
                department, designation, salary, bank_name, bank_account_no,
                emergency_contact_name, emergency_contact_phone, is_active, created_at, updated_at
         FROM employees WHERE id = ?1",
        [id],
        |row| Ok(Employee {
            id: row.get(0)?, employee_code: row.get(1)?, first_name: row.get(2)?,
            last_name: row.get(3)?, email: row.get(4)?, phone: row.get(5)?,
            cnic_no: row.get(6)?, address: row.get(7)?, city: row.get(8)?,
            department: row.get(9)?, designation: row.get(10)?, salary: row.get(11)?,
            bank_name: row.get(12)?, bank_account_no: row.get(13)?,
            emergency_contact_name: row.get(14)?, emergency_contact_phone: row.get(15)?,
            is_active: row.get::<_, i64>(16)? != 0, created_at: row.get(17)?, updated_at: row.get(18)?,
        }),
    );
    match result {
        Ok(e) => (StatusCode::OK, Json(json!({ "success": true, "data": e }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Employee not found." }))),
    }
}

async fn create_employee(State(_state): State<AppState>, Json(form): Json<EmployeeForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
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
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE employees SET is_active = 0, updated_at = datetime('now') WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Employee deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Employee not found." }))),
        Err(e) => { tracing::error!("Failed to delete employee: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete employee." }))) }
    }
}

async fn pay_salary(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<SalaryPaymentForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute(
        "INSERT INTO salary_payments (employee_id, amount, payment_date) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, form.amount, form.payment_date],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid(), "message": "Salary payment recorded." } }))),
        Err(e) => { tracing::error!("Failed to record salary: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to record salary payment." }))) }
    }
}
