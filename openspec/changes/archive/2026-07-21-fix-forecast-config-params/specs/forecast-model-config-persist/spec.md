## ADDED Requirements

### Requirement: Backend stores algorithm-specific parameters as JSON
The system SHALL persist algorithm-specific parameters sent by the frontend in a `params_json TEXT` column on the `forecast_model_config` table.

The backend SHALL:
- Accept a `params` JSON object in the `POST /api/forecasts/config` and `PUT /api/forecasts/config/{id}` request bodies
- Store it as a serialized JSON string in the `params_json` column
- Return the `params_json` field (as a parsed JSON object) in `GET /api/forecasts/config` and `GET /api/forecasts/config/{id}` responses
- Set `alpha`, `beta`, `gamma` to NULL when `params_json` is provided, to avoid confusion with the old fields

#### Scenario: Create config with ARIMA params
- **WHEN** client sends `POST /api/forecasts/config` with body `{"model_name": "Test ARIMA", "model_type": "arima", "params": {"p": "2", "d": "1", "q": "2"}}`
- **THEN** the server stores the config with `params_json` = `'{"p":"2","d":"1","q":"2"}'` and returns `{"success": true, "data": {"id": <new_id>}}`

#### Scenario: Create config with Neural params
- **WHEN** client sends `POST /api/forecasts/config` with body `{"model_name": "LSTM v1", "model_type": "neural", "params": {"layers": "3", "units": "128", "epochs": "200"}}`
- **THEN** the server stores the config with `params_json` = the sent JSON and returns success

#### Scenario: List configs returns params_json
- **WHEN** client calls `GET /api/forecasts/config`
- **THEN** each config in the response includes `params` as a parsed JSON object (not an escaped string)

#### Scenario: Old configs without params_json return null
- **WHEN** a config was created before this migration (has `params_json = NULL`)
- **THEN** the `GET` response includes `"params": null` for that config

### Requirement: Configs have a human-readable name
The system SHALL store a `model_name TEXT` column on `forecast_model_config` so configs can be displayed in a selector.

#### Scenario: Save config with name
- **WHEN** client sends `POST /api/forecasts/config` with `"model_name": "My Config"`
- **THEN** the stored row has `model_name = "My Config"` and the response includes it

#### Scenario: List configs returns name
- **WHEN** client calls `GET /api/forecasts/config`
- **THEN** each config in the response includes `"model_name": "<saved name>"`

### Requirement: Frontend loads existing configs on mount
The `ForecastModelConfigPage` SHALL fetch existing configs from `GET /api/forecasts/config` on mount and display them in a dropdown selector above the form.

#### Scenario: Page loads with config list
- **WHEN** the page mounts and the API returns 3 configs
- **THEN** a dropdown shows "New Config…", "Config 1", "Config 2", "Config 3" as options
- **THEN** "New Config…" is selected by default, and the form shows empty/default values

#### Scenario: Select existing config populates form
- **WHEN** user selects "Config 1" from the dropdown
- **THEN** the form fields populate with that config's name, algorithm, dates, and params
- **THEN** the "Save Model" button updates the existing config (PUT) instead of creating a new one (POST)

#### Scenario: Save new config creates new row
- **WHEN** user fills the form with "New Config…" selected and clicks "Save Model"
- **THEN** the page sends `POST /api/forecasts/config` with all form fields including `params`
- **THEN** on success, the new config appears in the dropdown and is selected

#### Scenario: Save edits existing config
- **WHEN** user selects an existing config, modifies params, and clicks "Save Model"
- **THEN** the page sends `PUT /api/forecasts/config/{id}` with the updated fields
- **THEN** on success, the toast confirms the update

### Requirement: Frontend includes params in the save payload
The frontend SHALL include the algorithm-specific `params` JSON object in the save request body, matching the shape shown in the form for the selected algorithm.

#### Scenario: ARIMA form saves params
- **WHEN** user has ARIMA selected with p=3, d=1, q=2 and clicks "Save Model"
- **THEN** the request body includes `"params": {"p": "3", "d": "1", "q": "2"}`

#### Scenario: Neural form saves params
- **WHEN** user has Neural selected with layers=2, units=64, epochs=100 and clicks "Save Model"
- **THEN** the request body includes `"params": {"layers": "2", "units": "64", "epochs": "100"}`
