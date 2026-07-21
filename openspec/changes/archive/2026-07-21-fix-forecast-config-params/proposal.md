## Why

The forecast model config page (`/forecasts/model-config`) sends rich algorithm-specific parameters (ARIMA p/d/q, Prophet growth mode, neural layers/units/epochs, etc.) to `POST /api/forecasts/config`, but the backend's `ForecastModelConfigForm` only stores `item_id`, `category`, `model_type`, `alpha`, `beta`, `gamma` — all the algorithm-specific parameters are silently dropped. This means the configuration form is effectively decorative: no custom parameters are ever persisted, and the user's choices have no effect.

## What Changes

- Extend the `forecast_model_config` SQLite schema to store algorithm-specific parameters as a JSON blob in a new `params_json` column
- Update `ForecastModelConfig` and `ForecastModelConfigForm` model structs to include the JSON params field
- Update the backend CRUD handlers (`create_model_config`, `update_model_config`, `list_model_configs`) to read/write `params_json`
- Wire the frontend model config page to **load existing configs** from `GET /api/forecasts/config` and populate the form (currently starts with hardcoded defaults every time)
- Add a loader dropdown or selector to pick which config to edit
- Add a "New Config" button to create fresh configs

## Capabilities

### New Capabilities
- `forecast-model-config-persist`: Store and retrieve full algorithm parameters (ARIMA, ETS, Prophet, Neural) as JSON alongside the existing model config fields. The form can save, load, and switch between multiple named configs.

### Modified Capabilities
<!-- No existing forecast specs to modify. This is entirely new capability. -->

## Impact

- **Schema**: `forecast_model_config` table gets new `params_json TEXT` column
- **Backend**: `models.rs` — two structs change; `server/forecast_routes.rs` — CRUD handlers change
- **Frontend**: `pages/forecast_model_config.rs` — add config loading/selection + wire to API
- **API client**: `api.rs` — minor, existing methods may suffice
