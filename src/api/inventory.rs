use crate::api::ApiClient;
use crate::models::*;
use crate::api::base_url;

impl ApiClient {
    // ── Inventory endpoints ──

    /// GET /api/inventory/items
    pub async fn list_items(&self) -> Result<Vec<Item>, String> {
        let url = format!("{}/api/inventory/items", base_url());
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
        let items: Vec<Item> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(items)
    }

    /// GET /api/inventory/items/:id
    pub async fn get_item(&self, id: i64) -> Result<Item, String> {
        let url = format!("{}/api/inventory/items/{}", base_url(), id);
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
        let item: Item = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// POST /api/inventory/items
    pub async fn create_item(&self, form: &ItemForm) -> Result<Item, String> {
        let url = format!("{}/api/inventory/items", base_url());
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
        let item: Item = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// PUT /api/inventory/items/:id
    pub async fn update_item(&self, id: i64, form: &ItemForm) -> Result<Item, String> {
        let url = format!("{}/api/inventory/items/{}", base_url(), id);
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
        let item: Item = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(item)
    }

    /// DELETE /api/inventory/items/:id
    pub async fn delete_item(&self, id: i64) -> Result<(), String> {
        let url = format!("{}/api/inventory/items/{}", base_url(), id);
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

        Ok(())
    }

    /// GET /api/inventory/warehouses
    pub async fn list_warehouses(&self) -> Result<Vec<Warehouse>, String> {
        let url = format!("{}/api/inventory/warehouses", base_url());
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
        let warehouses: Vec<Warehouse> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(warehouses)
    }

    /// GET /api/inventory/stock-movements
    pub async fn list_stock_movements(&self) -> Result<Vec<StockMovement>, String> {
        let url = format!("{}/api/inventory/stock-movements", base_url());
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
        let movements: Vec<StockMovement> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(movements)
    }

    /// GET /api/inventory/stock-movements/item/{item_id}
    pub async fn list_stock_movements_by_item(
        &self,
        item_id: i64,
    ) -> Result<Vec<StockMovement>, String> {
        let url = format!(
            "{}/api/inventory/stock-movements/item/{}",
            base_url(),
            item_id
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
            let msg = body["error"].as_str().unwrap_or("Request failed");
            return Err(msg.to_string());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        let movements: Vec<StockMovement> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(movements)
    }

    /// GET /api/inventory/stock-balances
    pub async fn list_stock_balances(&self) -> Result<Vec<StockBalance>, String> {
        let url = format!("{}/api/inventory/stock-balances", base_url());
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
        let balances: Vec<StockBalance> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(balances)
    }

    /// GET /api/inventory/physical-counts
    pub async fn list_physical_counts(&self) -> Result<Vec<PhysicalCount>, String> {
        let url = format!("{}/api/inventory/physical-counts", base_url());
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
        let counts: Vec<PhysicalCount> = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(counts)
    }

    /// GET /api/inventory/warehouses/:id
    pub async fn get_warehouse(&self, id: i64) -> Result<Warehouse, String> {
        let url = format!("{}/api/inventory/warehouses/{}", base_url(), id);
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
        let warehouse: Warehouse = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(warehouse)
    }

    /// POST /api/inventory/warehouses
    pub async fn create_warehouse(&self, form: &WarehouseForm) -> Result<Warehouse, String> {
        let url = format!("{}/api/inventory/warehouses", base_url());
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
        let warehouse: Warehouse = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(warehouse)
    }

    /// PUT /api/inventory/warehouses/:id
    pub async fn update_warehouse(
        &self,
        id: i64,
        form: &WarehouseForm,
    ) -> Result<Warehouse, String> {
        let url = format!("{}/api/inventory/warehouses/{}", base_url(), id);
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
        let warehouse: Warehouse = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(warehouse)
    }

    /// DELETE /api/inventory/warehouses/:id
    pub async fn delete_warehouse(&self, id: i64) -> Result<(), String> {
        let url = format!("{}/api/inventory/warehouses/{}", base_url(), id);
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

        Ok(())
    }

    /// POST /api/inventory/stock-movements
    pub async fn create_stock_movement(
        &self,
        form: &StockMovementForm,
    ) -> Result<StockMovement, String> {
        let url = format!("{}/api/inventory/stock-movements", base_url());
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
        let movement: StockMovement = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(movement)
    }

    /// GET /api/inventory/physical-counts/:id
    pub async fn get_physical_count(&self, id: i64) -> Result<PhysicalCount, String> {
        let url = format!("{}/api/inventory/physical-counts/{}", base_url(), id);
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
        // Backend returns { "data": { "count": {...}, "items": [...] } }
        let count_data = body["data"]["count"].clone();
        let count: PhysicalCount =
            serde_json::from_value(count_data).map_err(|e| format!("Parse error: {}", e))?;
        Ok(count)
    }

    /// GET /api/inventory/physical-counts/{id} — returns count with items
    pub async fn get_physical_count_with_items(
        &self,
        id: i64,
    ) -> Result<(PhysicalCount, Vec<serde_json::Value>), String> {
        let url = format!("{}/api/inventory/physical-counts/{}", base_url(), id);
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
        // Backend returns { "data": { "count": {...}, "items": [...] } }
        let count_data = body["data"]["count"].clone();
        let items = body["data"]["items"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let count: PhysicalCount =
            serde_json::from_value(count_data).map_err(|e| format!("Parse error: {}", e))?;
        Ok((count, items))
    }

    /// PUT /api/inventory/physical-counts/{count_id}/items/{item_id}
    pub async fn update_count_item(
        &self,
        count_id: i64,
        item_id: i64,
        counted_qty: f64,
    ) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/api/inventory/physical-counts/{}/items/{}",
            base_url(),
            count_id,
            item_id
        );
        let body = serde_json::json!({ "counted_quantity": counted_qty });
        let resp = self
            .inner
            .put(&url)
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

    /// DELETE /api/inventory/physical-counts/{id}
    pub async fn delete_physical_count(&self, id: i64) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/inventory/physical-counts/{}", base_url(), id);
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

    /// POST /api/inventory/physical-counts
    pub async fn create_physical_count(
        &self,
        form: &PhysicalCountForm,
    ) -> Result<PhysicalCount, String> {
        let url = format!("{}/api/inventory/physical-counts", base_url());
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
        let count: PhysicalCount = serde_json::from_value(body["data"].clone())
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(count)
    }

    /// GET /api/inventory/stock-summary
    pub async fn get_stock_summary(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/inventory/stock-summary", base_url());
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

    /// GET /api/inventory/items-categories
    pub async fn list_item_categories(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/inventory/items-categories", base_url());
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
        serde_json::from_value(body["data"].clone()).map_err(|e| format!("Parse error: {}", e))
    }

    /// GET /api/inventory/items-low-stock
    pub async fn list_low_stock_items(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/inventory/items-low-stock", base_url());
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

    /// GET /api/inventory/items-uom
    pub async fn list_uom(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/inventory/items-uom", base_url());
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
        serde_json::from_value(body["data"].clone()).map_err(|e| format!("Parse error: {}", e))
    }

    /// GET /api/inventory/physical-counts/{count_id}/items
    pub async fn get_physical_count_items(
        &self,
        count_id: i64,
    ) -> Result<Vec<serde_json::Value>, String> {
        let url = format!(
            "{}/api/inventory/physical-counts/{}/items",
            base_url(),
            count_id
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

    /// POST /api/inventory/physical-counts/{count_id}/complete
    pub async fn complete_physical_count(
        &self,
        count_id: i64,
    ) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/api/inventory/physical-counts/{}/complete",
            base_url(),
            count_id
        );
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
        Ok(body["data"].clone())
    }

    /// POST /api/inventory/physical-counts/{count_id}/cancel
    pub async fn cancel_physical_count(&self, count_id: i64) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/api/inventory/physical-counts/{}/cancel",
            base_url(),
            count_id
        );
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
        Ok(body["data"].clone())
    }

    /// PUT /api/inventory/physical-counts/{id}
    pub async fn update_physical_count(
        &self,
        id: i64,
        form: &PhysicalCountForm,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/inventory/physical-counts/{}", base_url(), id);
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

    // ── Catalog alias ──

    /// GET /api/inventory/items (alias for list_items, same endpoint)
    pub async fn list_items_catalog(&self) -> Result<Vec<Item>, String> {
        self.list_items().await
    }

}
