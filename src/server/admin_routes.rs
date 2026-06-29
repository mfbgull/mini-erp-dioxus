use crate::models::*;
use crate::server::auth_routes::AppState;
use crate::server::db;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        // Users
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/api/users/{id}/reset-password", put(reset_password))
        .route("/api/users/{id}/toggle-status", put(toggle_user_status))
        // Roles
        .route("/api/roles", get(list_roles).post(create_role))
        .route("/api/roles/permissions", get(list_all_permissions))
        .route("/api/roles/{id}", get(get_role).put(update_role).delete(delete_role))
        .route("/api/roles/{id}/permissions", get(get_role_permissions).put(update_role_permissions))
        // Settings
        .route("/api/settings", get(get_settings).put(update_settings))
        // Activity Logs
        .route("/api/activity-logs", get(list_activity_logs))
        // Integrations
        .route("/api/integrations", get(list_integrations))
        .route("/api/integrations/{service}", put(update_integration))
}

// ============================================================================
// Users
// ============================================================================

async fn list_users(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT u.id, u.username, u.email, u.full_name, u.role, u.role_id, u.is_active
         FROM users u ORDER BY u.username"
    ).unwrap();
    let items: Vec<UserProfile> = stmt.query_map([], |row| {
        Ok(UserProfile {
            id: row.get(0)?, username: row.get(1)?, full_name: row.get(3)?,
            email: row.get(2)?, role: row.get(4)?, role_id: row.get(5)?,
            is_active: row.get::<_, i64>(6)? != 0,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_user(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT id, username, email, full_name, role, role_id, is_active FROM users WHERE id = ?1",
        [id],
        |row| Ok(UserProfile { id: row.get(0)?, username: row.get(1)?, full_name: row.get(3)?, email: row.get(2)?, role: row.get(4)?, role_id: row.get(5)?, is_active: row.get::<_, i64>(6)? != 0 }),
    );
    match result {
        Ok(u) => (StatusCode::OK, Json(json!({ "success": true, "data": u }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "User not found." }))),
    }
}

async fn create_user(State(_state): State<AppState>, Json(form): Json<UserForm>) -> impl IntoResponse {
    if form.username.trim().is_empty() || form.full_name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Username and full name are required." })));
    }
    let password = form.password.as_deref().unwrap_or("password123");
    let hash = bcrypt::hash(password, 12).expect("Failed to hash password");
    let db = db::get_db().lock().unwrap();
    let exists: bool = db.query_row("SELECT COUNT(*) > 0 FROM users WHERE username = ?1", [&form.username], |row| row.get(0)).unwrap_or(false);
    if exists { return (StatusCode::CONFLICT, Json(json!({ "success": false, "error": "Username already exists." }))); }
    let role_name = db.query_row("SELECT role_name FROM roles WHERE id = ?1", [form.role_id], |row| row.get::<_, String>(0)).unwrap_or_else(|_| "user".to_string());
    let result = db.execute(
        "INSERT INTO users (username, email, password_hash, full_name, role, role_id, is_active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![form.username, form.email, hash, form.full_name, role_name, form.role_id, form.is_active as i64],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid(), "username": form.username } }))),
        Err(e) => { tracing::error!("Failed to create user: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create user." }))) }
    }
}

async fn update_user(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<UserForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let role_name = db.query_row("SELECT role_name FROM roles WHERE id = ?1", [form.role_id], |row| row.get::<_, String>(0)).unwrap_or_else(|_| "user".to_string());
    let result = if let Some(pw) = &form.password {
        let hash = bcrypt::hash(pw, 12).expect("Failed to hash password");
        db.execute(
            "UPDATE users SET username=?1, email=?2, password_hash=?3, full_name=?4, role=?5, role_id=?6, is_active=?7, updated_at=datetime('now') WHERE id=?8",
            rusqlite::params![form.username, form.email, hash, form.full_name, role_name, form.role_id, form.is_active as i64, id],
        )
    } else {
        db.execute(
            "UPDATE users SET username=?1, email=?2, full_name=?3, role=?4, role_id=?5, is_active=?6, updated_at=datetime('now') WHERE id=?7",
            rusqlite::params![form.username, form.email, form.full_name, role_name, form.role_id, form.is_active as i64, id],
        )
    };
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "User updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "User not found." }))),
        Err(e) => { tracing::error!("Failed to update user: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update user." }))) }
    }
}

async fn delete_user(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE users SET is_active = 0, updated_at = datetime('now') WHERE id = ?1 AND username != 'admin'", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "User deactivated." } }))),
        Ok(_) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Cannot delete admin user." }))),
        Err(e) => { tracing::error!("Failed to delete user: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete user." }))) }
    }
}

async fn reset_password(State(_state): State<AppState>, Path(id): Path<i64>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let new_pw = body.get("new_password").and_then(|v| v.as_str()).unwrap_or("password123");
    let hash = bcrypt::hash(new_pw, 12).expect("Failed to hash password");
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE users SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2", rusqlite::params![hash, id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Password reset." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "User not found." }))),
        Err(e) => { tracing::error!("Failed to reset password: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to reset password." }))) }
    }
}

async fn toggle_user_status(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE users SET is_active = CASE WHEN is_active = 1 THEN 0 ELSE 1 END, updated_at = datetime('now') WHERE id = ?1 AND username != 'admin'", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "User status toggled." } }))),
        Ok(_) => (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Cannot toggle admin status." }))),
        Err(e) => { tracing::error!("Failed to toggle status: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to toggle status." }))) }
    }
}

// ============================================================================
// Roles
// ============================================================================

async fn list_roles(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT id, role_name, description, is_system_role, is_active FROM roles ORDER BY role_name").unwrap();
    let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "role_name": row.get::<_, String>(1)?,
            "description": row.get::<_, String>(2)?, "is_system_role": row.get::<_, i64>(3)? != 0,
            "is_active": row.get::<_, i64>(4)? != 0,
        }))
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_role(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.query_row(
        "SELECT id, role_name, description, is_system_role, is_active FROM roles WHERE id = ?1",
        [id],
        |row| Ok(json!({
            "id": row.get::<_, i64>(0)?, "role_name": row.get::<_, String>(1)?,
            "description": row.get::<_, String>(2)?, "is_system_role": row.get::<_, i64>(3)? != 0,
            "is_active": row.get::<_, i64>(4)? != 0,
        })),
    );
    match result {
        Ok(r) => (StatusCode::OK, Json(json!({ "success": true, "data": r }))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Role not found." }))),
    }
}

async fn create_role(State(_state): State<AppState>, Json(form): Json<RoleForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute(
        "INSERT INTO roles (role_name, description) VALUES (?1, ?2)",
        rusqlite::params![form.role_name, form.description.as_deref().unwrap_or("")],
    );
    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": db.last_insert_rowid() } }))),
        Err(e) => { tracing::error!("Failed to create role: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to create role." }))) }
    }
}

async fn update_role(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<RoleForm>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let result = db.execute("UPDATE roles SET role_name=?1, description=?2 WHERE id=?3",
        rusqlite::params![form.role_name, form.description.as_deref().unwrap_or(""), id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Role updated." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Role not found." }))),
        Err(e) => { tracing::error!("Failed to update role: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to update role." }))) }
    }
}

async fn delete_role(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let is_system: bool = db.query_row("SELECT is_system_role FROM roles WHERE id = ?1", [id], |row| row.get::<_, i64>(0)).map(|v| v != 0).unwrap_or(true);
    if is_system { return (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Cannot delete system role." }))); }
    db.execute("DELETE FROM role_permissions WHERE role_id = ?1", [id]).ok();
    let result = db.execute("DELETE FROM roles WHERE id = ?1", [id]);
    match result {
        Ok(rows) if rows > 0 => (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Role deleted." } }))),
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({ "success": false, "error": "Role not found." }))),
        Err(e) => { tracing::error!("Failed to delete role: {}", e); (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "success": false, "error": "Failed to delete role." }))) }
    }
}

async fn list_all_permissions(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT id, permission_name, module, action, description FROM permissions ORDER BY module, action").unwrap();
    let items: Vec<Permission> = stmt.query_map([], |row| {
        Ok(Permission { id: row.get(0)?, permission_name: row.get(1)?, module: row.get(2)?, action: row.get(3)?, description: row.get(4)? })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn get_role_permissions(State(_state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT p.id, p.permission_name, p.module, p.action, p.description
         FROM permissions p JOIN role_permissions rp ON p.id = rp.permission_id
         WHERE rp.role_id = ?1 ORDER BY p.module, p.action"
    ).unwrap();
    let items: Vec<Permission> = stmt.query_map([id], |row| {
        Ok(Permission { id: row.get(0)?, permission_name: row.get(1)?, module: row.get(2)?, action: row.get(3)?, description: row.get(4)? })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn update_role_permissions(State(_state): State<AppState>, Path(id): Path<i64>, Json(form): Json<RolePermissionUpdate>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    db.execute("DELETE FROM role_permissions WHERE role_id = ?1", [id]).ok();
    for pid in &form.permission_ids {
        db.execute("INSERT INTO role_permissions (role_id, permission_id) VALUES (?1, ?2)", rusqlite::params![id, pid]).ok();
    }
    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Permissions updated." } })))
}

// ============================================================================
// Settings
// ============================================================================

async fn get_settings(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare("SELECT key, value, description FROM settings ORDER BY key").unwrap();
    let items: Vec<Setting> = stmt.query_map([], |row| {
        Ok(Setting { key: row.get(0)?, value: row.get(1)?, description: row.get(2)? })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn update_settings(State(_state): State<AppState>, Json(form): Json<SettingsUpdate>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    for s in &form.settings {
        db.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
            rusqlite::params![s.key, s.value],
        ).ok();
    }
    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": "Settings updated." } })))
}

// ============================================================================
// Activity Logs
// ============================================================================

async fn list_activity_logs(State(_state): State<AppState>) -> impl IntoResponse {
    let db = db::get_db().lock().unwrap();
    let mut stmt = db.prepare(
        "SELECT al.id, al.user_id, u.username, al.action, al.entity_type, al.entity_id,
                al.metadata, al.ip_address, al.created_at
         FROM activity_log al LEFT JOIN users u ON al.user_id = u.id
         ORDER BY al.created_at DESC LIMIT 100"
    ).unwrap();
    let items: Vec<ActivityLog> = stmt.query_map([], |row| {
        Ok(ActivityLog {
            id: row.get(0)?, user_id: row.get(1)?, username: row.get(2)?,
            action: row.get(3)?, entity_type: row.get(4)?, entity_id: row.get(5)?,
            metadata: row.get(6)?, ip_address: row.get(7)?, created_at: row.get(8)?,
        })
    }).unwrap().filter_map(|r| r.ok()).collect();
    (StatusCode::OK, Json(json!({ "success": true, "data": items })))
}

async fn list_integrations(State(_state): State<AppState>) -> impl IntoResponse {
    let integrations = vec![
        json!({ "service": "email", "is_configured": false, "settings": {} }),
        json!({ "service": "sms", "is_configured": false, "settings": {} }),
        json!({ "service": "accounting", "is_configured": false, "settings": {} }),
        json!({ "service": "ecommerce", "is_configured": false, "settings": {} }),
    ];
    (StatusCode::OK, Json(json!({ "success": true, "data": integrations })))
}

async fn update_integration(State(_state): State<AppState>, Path(service): Path<String>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    tracing::info!("Integration update for {}: {:?}", service, body);
    (StatusCode::OK, Json(json!({ "success": true, "data": { "message": format!("Integration '{}' updated.", service) } })))
}
