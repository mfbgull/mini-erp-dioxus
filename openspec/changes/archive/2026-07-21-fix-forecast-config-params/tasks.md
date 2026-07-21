## 1. Database migration

- [x] 1.1 Add `params_json TEXT` and `model_name TEXT` columns to `forecast_model_config` in the schema creation SQL (src/server/db.rs) — nullable, no default
- [x] 1.2 Add idempotent ALTER TABLE migration in `seed_data` or migration block so existing databases get the new columns without errors

## 2. Backend model structs

- [x] 2.1 Add `params_json: Option<String>` and `model_name: Option<String>` fields to `ForecastModelConfig` struct in src/models.rs
- [x] 2.2 Add `params: Option<serde_json::Value>` and `model_name: Option<String>` fields to `ForecastModelConfigForm` struct in src/models.rs
- [x] 2.3 Add serde rename/skip_serializing_if annotations so `params` serializes as a JSON object in API responses

## 3. Backend CRUD handlers

- [x] 3.1 Update `list_model_configs` in src/server/forecast_routes.rs to SELECT the new columns and return `params` as a parsed JSON object
- [x] 3.2 Update `create_model_config` to deserialize `params` from the form, serialize to JSON string for INSERT, and include `model_name`
- [x] 3.3 Update `update_model_config` to include `params_json` and `model_name` in the UPDATE SET clause
- [x] 3.4 Add `GET /api/forecasts/config/{id}` handler (route already declared but handler may be missing — check and implement)

## 4. API client

- [x] 4.1 Update `list_forecast_configs` in src/api.rs if the response shape needs adjustment (should already work if backend returns `data` array with new fields)
- [x] 4.2 Verify `create_forecast_config` and `update_forecast_config` send the full body including `params` and `model_name` (likely already works since the client passes the JSON through)

## 5. Frontend page — config loading & selector

- [x] 5.1 Add a `use_resource` or `use_effect` on `ForecastModelConfigPage` to fetch existing configs from the API on mount
- [x] 5.2 Add a config selector dropdown above the form with "New Config…" as the first option, followed by saved config names
- [x] 5.3 Track `selected_config_id: Signal<Option<i64>>` — `None` for new, `Some(id)` for editing
- [x] 5.4 When user selects an existing config, populate all form signals (model_name, algorithm, training dates, params per algorithm type) from the fetched config
- [x] 5.5 Add a "New Config" button that resets the form to defaults and sets `selected_config_id` to `None`

## 6. Frontend page — save logic update

- [x] 6.1 Update `on_save` handler to include `"params"` in the JSON payload (constructed from the algorithm-specific form signals)
- [x] 6.2 Include `"model_name"` in the JSON payload (already present, but ensure it's read from the signal, not hardcoded)
- [x] 6.3 Branch on `selected_config_id`: if `Some(id)`, call `update_forecast_config(id, &payload)` (PUT); if `None`, call `create_forecast_config(&payload)` (POST)
- [x] 6.4 On successful save (create or update), refresh the config list and select the saved config

## 7. Verify

- [x] 7.1 Build: `cargo check` passes with no new warnings
- [ ] 7.2 Start the app, navigate to /forecasts/model-config, create a new config with ARIMA params, save it, verify it appears in the dropdown
- [ ] 7.3 Select the saved config, verify params populate the form fields
- [ ] 7.4 Modify params, save again, verify PUT was called and changes persist
