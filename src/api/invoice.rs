use crate::api::ApiClient;
use crate::models::*;
use crate::api::base_url;

impl ApiClient {
    // ── Invoice endpoints ──

    /// GET /api/invoices
    pub async fn list_invoices(&self) -> Result<Vec<Invoice>, String> {
        let url = format!("{}/api/invoices", base_url());
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
        let invoices: Vec<Invoice> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(invoices)
    }

    /// GET /api/invoices/:id
    pub async fn get_invoice(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/invoices/{}", base_url(), id);
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
        Ok(body)
    }

    /// POST /api/invoices
    pub async fn create_invoice(&self, form: &InvoiceForm) -> Result<Invoice, String> {
        let url = format!("{}/api/invoices", base_url());
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
        let invoice: Invoice = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(invoice)
    }

    /// PUT /api/invoices/:id
    pub async fn update_invoice(
        &self,
        id: i64,
        form: &InvoiceForm,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/invoices/{}", base_url(), id);
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
        Ok(body)
    }

    /// PUT /api/invoices/:id/cancel
    pub async fn cancel_invoice(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/invoices/{}/cancel", base_url(), id);
        let resp = self
            .inner
            .put(&url)
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
        Ok(body)
    }

    /// GET /api/invoices/:id/payments
    pub async fn get_invoice_payments(&self, id: i64) -> Result<Vec<Payment>, String> {
        let url = format!("{}/api/invoices/{}/payments", base_url(), id);
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
        let payments: Vec<Payment> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(payments)
    }

    /// POST /api/invoices/:id/return
    pub async fn return_invoice(
        &self,
        id: i64,
        items: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/invoices/{}/return", base_url(), id);
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(items)
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

    /// GET /api/invoices/returns
    pub async fn list_invoice_returns(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/invoices/returns", base_url());
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
        let items: Vec<serde_json::Value> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    // ── POS endpoints ──

    /// POST /api/pos/sale
    pub async fn create_pos_sale(
        &self,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/pos/sale", base_url());
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

    /// GET /api/pos/transactions
    pub async fn list_pos_transactions(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/pos/transactions", base_url());
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

    // ── Mobile invoice endpoints ──

    /// POST /api/mobile-invoices/draft
    pub async fn create_mobile_draft(
        &self,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/mobile-invoices/draft", base_url());
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

    /// GET /api/mobile-invoices/items/search?q={query}
    pub async fn search_items(&self, query: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = format!(
            "{}/api/mobile-invoices/items/search?q={}",
            base_url(),
            query
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

    /// GET /api/mobile-invoices/customers/search?q={query}
    pub async fn search_customers(&self, query: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = format!(
            "{}/api/mobile-invoices/customers/search?q={}",
            base_url(),
            query
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

    /// GET /api/mobile-invoices/tax-rates
    pub async fn get_mobile_tax_rates(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/mobile-invoices/tax-rates", base_url());
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

    /// GET /api/mobile-invoices/payment-terms
    pub async fn get_mobile_payment_terms(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/mobile-invoices/payment-terms", base_url());
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

    /// POST /api/mobile-invoices/submit
    pub async fn submit_mobile_invoice(
        &self,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/mobile-invoices/submit", base_url());
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

    // ── Payment endpoints ──

    /// GET /api/payments
    pub async fn list_payments(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/payments", base_url());
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
        let items: Vec<serde_json::Value> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/payments/:id
    pub async fn get_payment(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/payments/{}", base_url(), id);
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
        Ok(body["data"].clone())
    }

    /// POST /api/payments
    pub async fn create_payment(
        &self,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/payments", base_url());
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

    // ── Quotation endpoints ──

    /// GET /api/sales/quotations
    pub async fn list_quotations(&self) -> Result<Vec<Quotation>, String> {
        let url = format!("{}/api/sales/quotations", base_url());
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
        let items: Vec<Quotation> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/sales/quotations/{id}
    pub async fn get_quotation(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/quotations/{}", base_url(), id);
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
        Ok(body["data"].clone())
    }

    /// POST /api/sales/quotations
    pub async fn create_quotation(&self, form: &QuotationForm) -> Result<Quotation, String> {
        let url = format!("{}/api/sales/quotations", base_url());
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
        let q: Quotation = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(q)
    }

    /// POST /api/sales/quotations/:id/convert
    pub async fn convert_quotation(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/quotations/{}/convert", base_url(), id);
        let resp = self
            .inner
            .post(&url)
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
        Ok(body["data"].clone())
    }

    // ── Sales Order endpoints ──

    /// GET /api/sales/sales-orders
    pub async fn list_sales_orders(&self) -> Result<Vec<SalesOrder>, String> {
        let url = format!("{}/api/sales/sales-orders", base_url());
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
        let items: Vec<SalesOrder> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/sales/sales-orders/{id}
    pub async fn get_sales_order(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/sales-orders/{}", base_url(), id);
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
        Ok(body["data"].clone())
    }

    /// POST /api/sales/sales-orders
    pub async fn create_sales_order(&self, form: &SalesOrderForm) -> Result<SalesOrder, String> {
        let url = format!("{}/api/sales/sales-orders", base_url());
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
        let order: SalesOrder = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(order)
    }

    /// POST /api/sales/sales-orders/:id/cancel
    pub async fn cancel_sales_order(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/sales-orders/{}/cancel", base_url(), id);
        let resp = self
            .inner
            .post(&url)
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
        Ok(body["data"].clone())
    }

    /// POST /api/sales/sales-orders/:id/convert
    pub async fn convert_sales_order(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/sales/sales-orders/{}/convert", base_url(), id);
        let resp = self
            .inner
            .post(&url)
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
        Ok(body["data"].clone())
    }

    /// GET /api/sales-returns
    pub async fn list_sales_returns(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/sales-returns", base_url());
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
