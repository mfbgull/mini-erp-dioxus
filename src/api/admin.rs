use crate::api::ApiClient;
use crate::models::*;
use crate::api::base_url;

impl ApiClient {
    // ── Employee endpoints ──

    /// GET /api/employees
    pub async fn list_employees(&self) -> Result<Vec<Employee>, String> {
        let url = format!("{}/api/employees", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let employees: Vec<Employee> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(employees)
    }

    /// GET /api/employees/{id}
    pub async fn get_employee(&self, id: i64) -> Result<Employee, String> {
        let url = format!("{}/api/employees/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let employee: Employee = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(employee)
    }

    /// POST /api/employees
    pub async fn create_employee(&self, form: &EmployeeForm) -> Result<Employee, String> {
        let url = format!("{}/api/employees", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let employee: Employee = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(employee)
    }

    /// PUT /api/employees/{id}
    pub async fn update_employee(
        &self,
        id: i64,
        form: &EmployeeForm,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/employees/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/employees/{employee_id}/salary
    pub async fn pay_salary(
        &self,
        employee_id: i64,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/employees/{}/salary", base_url(), employee_id);
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Role endpoints ──

    /// GET /api/roles
    pub async fn list_roles(&self) -> Result<Vec<Role>, String> {
        let url = format!("{}/api/roles", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Role> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/roles/{id}
    pub async fn get_role(&self, id: i64) -> Result<Role, String> {
        let url = format!("{}/api/roles/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let item: Role = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// GET /api/roles/permissions
    pub async fn list_all_permissions(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/roles/permissions", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// GET /api/roles/{role_id}/permissions
    pub async fn get_role_permissions(
        &self,
        role_id: i64,
    ) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/roles/{}/permissions", base_url(), role_id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    /// PUT /api/roles/{role_id}/permissions
    pub async fn update_role_permissions(
        &self,
        role_id: i64,
        permissions: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/roles/{}/permissions", base_url(), role_id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(permissions)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/roles
    pub async fn create_role(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/roles", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// DELETE /api/roles/{id}
    pub async fn delete_role(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/roles/{}", base_url(), id);
        let resp = self
            .inner
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/roles/{id}/users
    pub async fn list_role_users(&self, role_id: i64) -> Result<Vec<User>, String> {
        let url = format!("{}/api/roles/{}/users", base_url(), role_id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<User> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── User endpoints ──

    /// GET /api/users
    pub async fn list_users(&self) -> Result<Vec<User>, String> {
        let url = format!("{}/api/users", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<User> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/users/{id}
    pub async fn get_user(&self, id: i64) -> Result<User, String> {
        let url = format!("{}/api/users/{}", base_url(), id);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let item: User = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// PUT /api/users/{id}
    pub async fn update_user(
        &self,
        id: i64,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users/{}", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// POST /api/users
    pub async fn create_user(&self, form: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// DELETE /api/users/{id}
    pub async fn delete_user(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users/{}", base_url(), id);
        let resp = self
            .inner
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/users/{id}/reset-password
    pub async fn reset_user_password(
        &self,
        id: i64,
        new_password: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users/{}/reset-password", base_url(), id);
        let body_json = serde_json::json!({ "password": new_password });
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(&body_json)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/users/{id}/toggle-status
    pub async fn toggle_user_status(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/users/{}/toggle-status", base_url(), id);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Expense endpoints ──

    /// GET /api/expenses
    pub async fn list_expenses(&self) -> Result<Vec<Expense>, String> {
        let url = format!("{}/api/expenses", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Expense> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// POST /api/expenses
    pub async fn create_expense(&self, form: &ExpenseForm) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/expenses", base_url());
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// GET /api/expenses/categories
    pub async fn list_expense_categories(&self) -> Result<Vec<ExpenseCategory>, String> {
        let url = format!("{}/api/expenses/categories", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<ExpenseCategory> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── Activity Log endpoints ──

    /// GET /api/activity-logs
    pub async fn list_activity_logs(&self) -> Result<Vec<ActivityLog>, String> {
        let url = format!("{}/api/activity-logs", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<ActivityLog> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── Integration endpoints ──

    /// GET /api/integrations
    pub async fn list_integrations(&self) -> Result<Vec<Integration>, String> {
        let url = format!("{}/api/integrations", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let items: Vec<Integration> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/integrations/{service}
    pub async fn get_integration(&self, service: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/integrations/{}", base_url(), service);
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/integrations/{service}
    pub async fn update_integration(
        &self,
        service: &str,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/integrations/{}", base_url(), service);
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Settings endpoints ──

    /// GET /api/settings
    pub async fn get_settings(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/settings", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    /// PUT /api/settings
    pub async fn update_settings(
        &self,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/settings", base_url());
        let resp = self
            .inner
            .put(&url)
            .headers(self.headers())
            .json(form)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body["data"].clone())
    }

    // ── Dashboard Layouts ──

    /// GET /api/dashboard/layout
    pub async fn list_dashboard_layouts(&self) -> Result<Vec<DashboardLayout>, String> {
        let url = format!("{}/api/dashboard/layout", base_url());
        let resp = self
            .inner
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let layouts: Vec<DashboardLayout> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(layouts)
    }

    /// POST /api/dashboard/layout
    pub async fn create_dashboard_layout(
        &self,
        name: &str,
        blocks: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/layout", base_url());
        let body = serde_json::json!({ "layout_name": name, "blocks": blocks });
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let err_body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(err_body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let resp_body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(resp_body)
    }

    /// DELETE /api/dashboard/layout/:id
    pub async fn delete_dashboard_layout(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/dashboard/layout/{}", base_url(), id);
        let resp = self
            .inner
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            return Err(body["error"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string());
        }
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(body)
    }
}
