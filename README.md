# MiniERP

Full-featured ERP system rebuilt in Dioxus (Rust) for cross-platform desktop, web, and mobile.

## Features

- **Multi-platform**: Runs on web (WASM), desktop (WebView), and mobile (iOS/Android)
- **Modules**: Sales, Purchases, Inventory, Manufacturing, Accounting, HR, CRM
- **Backend**: Axum-based REST API with SQLite storage
- **Frontend**: Dioxus 0.7 with router, reactive UI, and charting support

## Development

### Prerequisites

- Rust (2021 edition)
- Dioxus CLI (`cargo install dioxus-cli`)

### Building

```bash
# Web (WASM) - default
cargo build --features web

# Desktop
cargo build --features desktop

# Server (native)
cargo build --bin mini-erp-server --features server
```

### Running

```bash
# Web server (includes both frontend and backend)
dx serve --features web

# Desktop
dx serve --features desktop

# Server only
cargo run --bin mini-erp-server --features server
```

## Project Structure

```
src/
├── api.rs           # HTTP client and API endpoints
├── lib.rs           # Shared library code
├── main.rs          # Frontend entry point
├── server_main.rs   # Backend entry point
├── components/      # Reusable UI components
├── pages/           # Application pages/views
└── server/          # Axum backend routes
```

## License

MIT