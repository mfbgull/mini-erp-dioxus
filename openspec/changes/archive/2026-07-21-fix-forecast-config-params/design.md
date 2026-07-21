## Context

The `/forecasts/model-config` page lets users configure forecasting models (ARIMA, ETS, Prophet, Neural/LSTM) with algorithm-specific parameters. Currently, the backend `ForecastModelConfigForm` struct only accepts `item_id`, `category`, `model_type`, `alpha`, `beta`, `gamma` — fields designed for an older Holt-Winters-only model. The frontend sends a `params` JSON object with algorithm-specific keys (e.g., `{"p":"2","d":"1","q":"2"}` for ARIMA), but the backend maps to a different schema and the `params` field is never deserialized or stored.

Additionally, the page always starts with hardcoded defaults — it never loads existing configs from the server, so you can't edit a saved config.

## Goals / Non-Goals

**Goals:**
- Persist algorithm-specific parameters (ARIMA p/d/q, ETS error/trend/seasonal, Prophet growth/seasonality, neural layers/units/epochs) alongside model configs
- Load existing configs into the form for viewing/editing
- Support multiple named configs with a selector
- Minimal schema change — one new nullable TEXT column

**Non-Goals:**
- Actual forecast computation engine (the backend stub `run_forecast` remains a stub)
- Auto-tune backend implementation
- Validation of algorithm parameter values (backend stores what frontend sends)
- Deleting configs from the UI (exists at API level but no UI wire-up needed in this change)

## Decisions

**1. Store algorithm params as a JSON string in one column rather than separate columns per algorithm type.**
- *Rationale:* Each algorithm has a different set of params. Adding columns for each (arima_p, arima_d, arima_q, prophet_growth, neural_layers, etc.) would bloat the schema and require migrations for every new algorithm. A single `params_json TEXT` column is flexible — the frontend controls the shape, the backend just stores/retrieves it.
- *Alternative considered:* Separate tables per algorithm type. Rejected — over-engineered for the current use case; the JSON approach matches how the frontend already sends the data.

**2. Keep the existing scalar columns (alpha, beta, gamma) for backward compatibility.**
- *Rationale:* The table may already contain rows with alpha/beta/gamma values. Dropping them would break existing data. The new `params_json` column supplements, not replaces, these fields. New configs will set alpha/beta/gamma to NULL and put everything in `params_json`.

**3. Use the existing `model_name` field from the frontend form as the display label for config selection.**
- *Rationale:* The frontend already sends `"model_name"` in the JSON payload (e.g., "Demand Forecast ARIMA"). However, the current `ForecastModelConfigForm` doesn't have a name field. Rather than altering the form struct to add a name, we'll add a `model_name TEXT` column to the config table so each config has a human-readable label. This is cleaner than using the ID.

**4. Config selector as a simple dropdown before the form, not part of the form itself.**
- *Rationale:* Easier UX — user picks "New Config" or an existing named config, then the form populates. Keeps the form as the editing surface.

## Risks / Trade-offs

- **JSON schema drift** → Frontend and backend have no shared schema for the params JSON. Mitigation: the backend stores whatever JSON the frontend sends; validation is frontend-only for now. If the frontend changes the param shape, old saved configs may render empty fields when loaded.
- **SQLite TEXT column for JSON** → No native JSON validation or indexing. Acceptable since config count is low (dozens, not millions) and we never query by param values.
- **Existing alpha/beta/gamma data** → Old rows will have `params_json = NULL`. The frontend falls back to empty params. No data loss.
