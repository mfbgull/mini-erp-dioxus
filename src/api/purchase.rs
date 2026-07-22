use crate::api::ApiClient;
use crate::models::*;
use crate::api::base_url;

impl ApiClient {
    // ── Purchase endpoints ──

    /// GET /api/purchases
    pub async fn list_direct_purchases(&self) -> Result<Vec<DirectPurchase>, String> {
        let url = format!("{}/api/purchases", base_url());
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
        let purchases: Vec<DirectPurchase> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(purchases)
    }

    /// GET /api/purchases/:id
    pub async fn get_direct_purchase(&self, id: i64) -> Result<DirectPurchase, String> {
        let url = format!("{}/api/purchases/{}", base_url(), id);
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
        let purchase: DirectPurchase = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(purchase)
    }

    /// PUT /api/purchases/{id}
    pub async fn update_direct_purchase(
        &self,
        id: i64,
        form: &DirectPurchaseForm,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchases/{}", base_url(), id);
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

    /// GET /api/receipts
    pub async fn list_receipts(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/receipts", base_url());
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

    /// GET /api/purchase-returns
    pub async fn list_purchase_returns(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/purchase-returns", base_url());
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

    // ── Purchasing ──

    /// GET /api/purchase-orders
    pub async fn list_purchase_orders(&self) -> Result<Vec<PurchaseOrder>, String> {
        let url = format!("{}/api/purchase-orders", base_url());
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
        let orders: Vec<PurchaseOrder> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(orders)
    }

    /// GET /api/purchase-orders/:id
    pub async fn get_purchase_order(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders/{}", base_url(), id);
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

    /// POST /api/purchase-orders
    pub async fn create_purchase_order(
        &self,
        form: &PurchaseOrderForm,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders", base_url());
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
        Ok(body)
    }

    /// PUT /api/purchase-orders/:id
    pub async fn update_purchase_order(
        &self,
        id: i64,
        form: &PurchaseOrderForm,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders/{}", base_url(), id);
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

    /// POST /api/purchases
    pub async fn create_direct_purchase(
        &self,
        form: &DirectPurchaseForm,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchases", base_url());
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
        Ok(body)
    }

    /// DELETE /api/purchases/:id
    pub async fn delete_direct_purchase(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchases/{}", base_url(), id);
        let resp = self
            .inner
            .delete(&url)
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

    /// POST /api/purchases/{id}/return
    pub async fn return_direct_purchase(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchases/{}/return", base_url(), id);
        let resp = self
            .inner
            .post(&url)
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

    /// POST /api/purchase-orders/:id/status
    pub async fn update_po_status(
        &self,
        id: i64,
        status: &str,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders/{}/status", base_url(), id);
        let body_json = serde_json::json!({ "status": status });
        let resp = self
            .inner
            .post(&url)
            .headers(self.headers())
            .json(&body_json)
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

    /// GET /api/purchase-orders/:po_id/receipts
    pub async fn list_po_receipts(&self, po_id: i64) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/purchase-orders/{}/receipts", base_url(), po_id);
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

    /// GET /api/purchase-orders/pending
    pub async fn list_pending_pos(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/purchase-orders/pending", base_url());
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

    /// POST /api/purchase-orders/:po_id/receipts
    pub async fn create_goods_receipt(
        &self,
        po_id: i64,
        form: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/purchase-orders/{}/receipts", base_url(), po_id);
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

}
