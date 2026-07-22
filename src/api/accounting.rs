use crate::api::ApiClient;
use crate::models::*;
use crate::api::base_url;

impl ApiClient {
    // ── Accounting endpoints ──

    /// GET /api/accounting/accounts
    /// Response: { success: bool, data: Vec<ChartOfAccount> }
    pub async fn list_accounts(&self) -> Result<Vec<ChartOfAccount>, String> {
        let url = format!("{}/api/accounting/accounts", base_url());
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
        if body.get("success").and_then(|v| v.as_bool()) != Some(true) {
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }
        let items: Vec<ChartOfAccount> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/accounting/periods
    pub async fn list_accounting_periods(&self) -> Result<Vec<AccountingPeriod>, String> {
        let url = format!("{}/api/accounting/periods", base_url());
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
        let items: Vec<AccountingPeriod> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── Accounting endpoints (balances) ──

    /// GET /api/accounting/accounts/balances
    pub async fn list_account_balances(&self) -> Result<Vec<AccountBalance>, String> {
        let url = format!("{}/api/accounting/accounts/balances", base_url());
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
        let items: Vec<AccountBalance> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── Tax & Payment endpoints ──

    /// GET /api/tax-rates
    pub async fn list_tax_rates(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/tax-rates", base_url());
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

    /// GET /api/payment-terms
    pub async fn list_payment_terms(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/payment-terms", base_url());
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

    // ── Journal Entries ──

    /// GET /api/accounting/journal-entries?from_date=...&to_date=...
    pub async fn list_journal_entries(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<JournalEntry>, String> {
        let url = format!(
            "{}/api/accounting/journal-entries?from_date={}&to_date={}",
            base_url(),
            from_date,
            to_date
        );
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
        let entries: Vec<JournalEntry> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(entries)
    }

    /// POST /api/accounting/journal-entries
    pub async fn create_journal_entry(
        &self,
        entry_date: &str,
        reference_type: &str,
        reference_id: Option<i64>,
        lines: &[(i64, f64, f64, String)],
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/accounting/journal-entries", base_url());
        let lines_json: Vec<serde_json::Value> = lines
            .iter()
            .map(|(account_id, debit, credit, desc)| {
                serde_json::json!({
                    "account_id": account_id,
                    "debit": debit,
                    "credit": credit,
                    "description": desc,
                })
            })
            .collect();
        let body = serde_json::json!({
            "entry_date": entry_date,
            "reference_type": if reference_type.is_empty() { serde_json::Value::Null } else { serde_json::json!(reference_type) },
            "reference_id": reference_id,
            "lines": lines_json,
        });
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

    /// GET /api/accounting/journal-entries/:id
    pub async fn get_journal_entry(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/accounting/journal-entries/{}", base_url(), id);
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

    /// GET /api/employees/:id/salary-payments
    pub async fn list_salary_payments(
        &self,
        employee_id: i64,
    ) -> Result<Vec<serde_json::Value>, String> {
        let url = format!(
            "{}/api/employees/{}/salary-payments",
            base_url(),
            employee_id
        );
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

}
