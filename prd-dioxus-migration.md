# PRD: MiniERP — Dioxus (Rust) Rebuild

**Version:** 1.1  
**Date:** 2026-06-27  
**Status:** Draft  
**Target Framework:** [Dioxus](https://dioxuslabs.com/) 0.6+ (Rust/WASM)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current Architecture](#2-current-architecture)
3. [Target Architecture](#3-target-architecture)
4. [Database Schema](#4-database-schema)
5. [API Surface — All Endpoints](#5-api-surface--all-endpoints)
6. [Frontend Pages & Routes](#6-frontend-pages--routes)
7. [UI Component Inventory](#7-ui-component-inventory)
8. [Business Logic & Services](#8-business-logic--services)
9. [Authentication & Security](#9-authentication--security)
10. [State Management & Data Flow](#10-state-management--data-flow)
11. [Form Validation](#11-form-validation)
12. [Calculations (Pure Functions)](#12-calculations-pure-functions)
13. [Reporting Engine](#13-reporting-engine)
14. [Forecasting Engine](#14-forecasting-engine)
15. [Desktop/Mobile UI Patterns](#15-desktopmobile-ui-patterns)
16. [i18n](#16-i18n)
17. [Dioxus-Specific Considerations](#17-dioxus-specific-considerations)
18. [Migration Phases](#18-migration-phases)
19. [Risks & Mitigations](#19-risks--mitigations)

---

## 1. Executive Summary

MiniERP is a full-featured ERP system for small businesses covering: inventory management, sales (invoices, quotations, sales orders, POS), purchasing (POs, goods receipt, returns), manufacturing (BOM, work orders, production), accounting (double-entry, AR aging, financial reports), HR (employees, salary), expenses, forecasting, dashboards, and role-based access control.

### Current Stack
- **Frontend:** React 19 + TypeScript + Vite + AG-Grid + TanStack Query + Chart.js + CSS Custom Properties
- **Backend:** Express 5 + TypeScript + better-sqlite3 (raw SQL, no ORM)
- **Desktop:** Electron wrapper (spawns Express as child process)
- **Auth:** JWT (24h, httpOnly cookie)
- **DB:** SQLite (WAL mode, synchronous)
- **Lines of code:** ~456 TypeScript/TSX files, ~10,000+ lines CSS, ~1,200 lines i18n

### Target Stack
- **Frontend:** Dioxus 0.6+ (WASM/RSX)
- **Backend:** Dioxus Server Functions (Rust) or standalone Axum
- **Desktop:** Dioxus Desktop (native WebView)
- **Mobile:** Dioxus Mobile (iOS/Android)
- **DB:** rusqlite (SQLite, same schema)
- **Auth:** JWT (same approach)
- **Charts:** plotters crate or custom canvas
- **Table:** Custom table component (no AG-Grid equivalent in Rust ecosystem)

---

## 2. Current Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Electron Shell                     │
│  ┌──────────────────────────────────────────────┐   │
│  │         Dioxus Desktop (Native Window)        │   │
│  │  ┌────────────────────────────────────────┐  │   │
│  │  │          Dioxus Router (RSX)           │  │   │
│  │  │  ┌─────┐ ┌──────┐ ┌──────┐ ┌──────┐  │  │   │
│  │  │  │Dashboard│Invoice│Inventory│Reports│ ...│  │   │
│  │  │  └─────┘ └──────┘ └──────┘ └──────┘  │  │   │
│  │  │  ┌────────────────────────────────┐  │  │   │
│  │  │  │  State: Signals + Context      │  │  │   │
│  │  │  └────────────────────────────────┘  │  │   │
│  │  └────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Rust Backend (Server Functions / Actix)      │   │
│  │  ┌──────┐ ┌────────┐ ┌────────┐ ┌────────┐  │   │
│  │  │Routes │Controllers│ Services│  Models  │  │   │
│  │  └──────┘ └────────┘ └────────┘ └────────┘  │   │
│  │  ┌────────────────────────────────────────┐  │   │
│  │  │  rusqlite (SQLite, WAL mode)            │  │   │
│  │  └────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

### Key Architecture Decisions for Dioxus

Straightforward 1:1 swaps:

| Current | Dioxus Equivalent | Notes |
|---------|-------------------|-------|
| React + React Router | Dioxus Router | Native support, similar patterns |
| React Context | Dioxus Context API (`use_context_provider`) | Same pattern |
| CSS Custom Properties | CSS in Dioxus | Global stylesheets still work |
| localStorage | `dioxus-storage` / browser Storage API | Persists auth token |
| Electron (desktop) | Dioxus Desktop | Native target, no Electron wrapper needed |
| Capacitor (mobile) | Dioxus Mobile | Native target |
| Multer (file uploads) | `reqwest` multipart | Server-side file handling |
| date-fns | `chrono` crate | All date logic moves to Rust |

The harder swaps — AG-Grid, Chart.js, TanStack Query, jsPDF, and other libraries without a drop-in Rust equivalent — are covered with risk ratings in **§17.2**.

---

## 3. Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Dioxus App (cross-platform binary)                         │
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Dioxus Router                                     │    │
│  │  / → Dashboard                                     │    │
│  │  /inventory → Items | Warehouses | Stock | Counts  │    │
│  │  /sales → Invoices | Quotations | SO | POS         │    │
│  │  /purchases → Direct | PO | Receipts               │    │
│  │  /manufacturing → BOM | Production                  │    │
│  │  /accounting → GL | Reports | Periods              │    │
│  │  /customers | /suppliers | /employees | /expenses  │    │
│  │  /reports | /forecasts | /settings | /users | /roles│   │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Desktop Layout       │  Mobile Layout              │    │
│  │  ┌────────┬─────────┐ │  ┌──────────────────┐     │    │
│  │  │Sidebar │ Content │ │  │Top Bar + Nav     │     │    │
│  │  │(260px) │ (Grid)  │ │  │Compact Card List │     │    │
│  │  └────────┴─────────┘ │  │Detail Modal      │     │    │
│  │  Breakpoint: >768px   │  │Action Bar        │     │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Signals & State                                    │    │
│  │  - AuthContext (user, token, permissions)           │    │
│  │  - ThemeContext (sidebar, mobile)                   │    │
│  │  - Page-level signals for lists, forms, modals      │    │
│  │  - Resource (async data fetching)                    │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Server Functions / API Calls                       │    │
│  │  - Dioxus Server Functions (if fullstack)           │    │
│  │  - reqwest calls to standalone Rust API server      │    │
│  │  - Call pattern: call_server(function_name, args)   │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Pure Functions (same logic, Rust impl)            │    │
│  │  - invoiceCalculations (discounts, taxes, totals)  │    │
│  │  - customerCalculations (metrics, aging)           │    │
│  │  - stockCalculations (FIFO, valuations)            │    │
│  │  - reportCalculations (financial math)             │    │
│  └────────────────────────────────────────────────────┘    │
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Rust Backend (same machine or server)             │    │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌────────┐ │    │
│  │  │ Handlers │ │ Services │ │ Models │ │ Middle │ │    │
│  │  │(axum/actix)│ │ (Biz Logic)│ │ (SQL)  │ │ (Auth) │ │    │
│  │  └──────────┘ └──────────┘ └────────┘ └────────┘ │    │
│  │  ┌────────────────────────────────────────────┐   │    │
│  │  │  rusqlite - SQLite database (.db file)       │   │    │
│  │  │  ~56 tables, WAL mode, same schema           │   │    │
│  │  └────────────────────────────────────────────┘   │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Database Schema

SQLite via rusqlite. All tables, columns, types, indexes, FKs preserved from current schema.

### Complete Table List (56 tables)

#### Core
| # | Table | Key Columns | Notes |
|---|-------|-------------|-------|
| 1 | `users` | id, username, email, password_hash, full_name, role, role_id, is_active | JWT auth |
| 2 | `settings` | key (PK), value, description | Key-value config |
| 3 | `roles` | id, role_name, description, is_system_role, is_active | RBAC |
| 4 | `permissions` | id, permission_name, module, action, description | ~130 permissions |
| 5 | `role_permissions` | role_id, permission_id | M2M |
| 6 | `activity_log` | id, user_id, action, entity_type, entity_id, metadata, ip_address | Audit trail |

#### Inventory
| # | Table | Key Columns |
|---|-------|-------------|
| 7 | `items` | id, item_code, item_name, category, unit_of_measure, current_stock, reorder_level, standard_cost/selling_price, is_raw_material/finished_good/purchased/manufactured |
| 8 | `warehouses` | id, warehouse_code, warehouse_name, location, is_active |
| 9 | `stock_movements` | id, movement_no, item_id, warehouse_id, movement_type, quantity, unit_cost, reference_doctype/docno, batch_id |
| 10 | `stock_balances` | item_id, warehouse_id, quantity | UNIQUE(item_id, warehouse_id) |
| 11 | `item_locations` | id, item_id, warehouse_id, rack_no, is_primary |
| 12 | `stock_batches` | id, batch_no, item_id, warehouse_id, source_type, source_id, quantity_original/remaining, unit_cost, received_date |
| 13 | `physical_counts` | id, count_no, count_date, warehouse_id, status |
| 14 | `physical_count_items` | id, count_id, item_id, system_quantity, counted_quantity, variance |

#### Sales & CRM
| # | Table | Key Columns |
|---|-------|-------------|
| 15 | `customers` | id, customer_code, customer_name, email, phone, billing/shipping_address, payment_terms, credit_limit, credit_balance, current_balance, opening_balance |
| 16 | `invoices` | id, invoice_no, customer_id, so_id, quotation_id, source_type, invoice_date, due_date, status, total_amount, paid_amount, balance_amount, returned_amount, discount_scope/type/value |
| 17 | `invoice_items` | id, invoice_id, item_id, quantity, returned_qty, unit_price, amount, tax_rate, discount_type/value |
| 18 | `payments` | id, payment_no, customer_id, invoice_id, payment_date, amount, payment_method |
| 19 | `payment_allocations` | id, payment_id, invoice_id, amount |
| 20 | `customer_ledger` | id, customer_id, transaction_date, type, reference_no, debit, credit, balance |
| 21 | `sales_orders` | id, so_no, customer_id, so_date, status, source_type, source_id, total_amount, warehouse_id |
| 22 | `sales_order_items` | id, so_id, item_id, quantity, delivered_quantity, unit_price, amount |
| 23 | `quotations` | id, quotation_no, customer_id, quotation_date, expiry_date, status, total_amount |
| 24 | `quotation_items` | id, quotation_id, item_id, quantity, unit_price, discount, tax, amount |
| 25 | `tax_rates` | id, name, rate, is_default, is_active |
| 26 | `payment_terms` | id, name, days, is_default, is_active |
| 27 | `invoice_drafts` | id, session_id, customer_id, items_data (JSON), status, expires_at |

#### Purchasing
| # | Table | Key Columns |
|---|-------|-------------|
| 28 | `suppliers` | id, supplier_code, supplier_name, email, phone, address, is_active |
| 29 | `purchase_orders` | id, po_no, supplier_id, po_date, status, total_amount, warehouse_id |
| 30 | `purchase_order_items` | id, po_id, item_id, quantity, received_quantity, returned_quantity, unit_price, amount |
| 31 | `goods_receipts` | id, receipt_no, po_id, receipt_date, warehouse_id |
| 32 | `goods_receipt_items` | id, receipt_id, po_item_id, item_id, received_quantity |
| 33 | `purchases` | id, purchase_no, item_id, warehouse_id, batch_id, quantity, unit_cost, total_cost, supplier_name |
| 34 | `supplier_ledger` | id, supplier_id, transaction_date, type, reference_no, debit, credit, balance |

#### Manufacturing
| # | Table | Key Columns |
|---|-------|-------------|
| 35 | `boms` | id, bom_no, bom_name, finished_item_id, quantity, is_active |
| 36 | `bom_items` | id, bom_id, item_id (raw_material_id), quantity, unit_cost |
| 37 | `work_orders` | id, wo_no, bom_id, finished_item_id, planned_quantity, produced_quantity, status, warehouse_id |
| 38 | `material_consumption` | id, wo_id, item_id, consumed_quantity, consumption_date |
| 39 | `productions` | id, production_no, output_item_id, output_quantity, warehouse_id, bom_id, overhead_cost, batch_id, unit_cost, total_material_cost |
| 40 | `production_inputs` | id, production_id, item_id, quantity, warehouse_id |

#### Finance & Accounting
| # | Table | Key Columns |
|---|-------|-------------|
| 41 | `chart_of_accounts` | id, code, name, type, normal_balance, parent_id | 17 seeded accounts |
| 42 | `journal_entries` | id, reference_type, reference_id, entry_date, debit_account, credit_account, amount | Legacy, single-entry |
| 43 | `journal_lines` | id, journal_entry_id, account_id, debit, credit, description, line_date, voided | Canonical double-entry |
| 44 | `accounting_periods` | id, period_name, start_date, end_date, status (open/closed) |

#### HR
| # | Table | Key Columns |
|---|-------|-------------|
| 45 | `employees` | id, employee_code, first/last_name, email, phone, cnic_no, address, city, department, designation, salary, bank info, emergency contacts |
| 46 | `employee_documents` | id, employee_id, document_name, type, file_path |
| 47 | `salary_payments` | id, employee_id, amount, payment_date, journal_entry_id |

#### Expenses
| # | Table | Key Columns |
|---|-------|-------------|
| 48 | `expenses` | id, expense_no, category, description, amount, expense_date, status |
| 49 | `expense_categories` | id, category_name, is_active | 15 seeded |

#### Forecasting
| # | Table | Key Columns |
|---|-------|-------------|
| 50 | `demand_forecasts` | id, item_id, forecast_date, period, predicted_quantity, confidence_level, trend_direction, model_type |
| 51 | `forecast_runs` | id, run_id, run_type, status, items_processed |
| 52 | `forecast_model_config` | id, item_id, category, model_type, alpha/beta/gamma params |
| 53 | `forecast_seasonal_events` | id, event_name, start_date, end_date, multiplier, applies_to_category/item_id | 5 seeded |
| 54 | `forecast_accuracy` | id, forecast_id, item_id, period, mape, mae, smape |

#### Reports & Dashboards
| # | Table | Key Columns |
|---|-------|-------------|
| 55 | `custom_reports` | id, user_id, name, config (JSON), is_active, last_run_at |
| 56 | `dashboard_layouts` | id, user_id, layout_name, blocks (JSON), is_active | UNIQUE(user_id, layout_name) |

> Note: `invoice_drafts` (table #27, session-based with auto-expiry) is also the mechanism behind the mobile draft-invoice endpoints in §5.9.

### Key DB Patterns to Replicate

1. **Prepared statements only** — All queries via `rusqlite::Connection.prepare()` + parameter binding
2. **Transactions** — `conn.transaction()` for multi-step writes (invoice creation = 6+ tables)
3. **WAL mode** — `PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;`
4. **Document numbering** — Sequence stored in `settings` table, generated as: `INV-{year}-{seq:04}`, `PO-{year}-{seq:04}`, etc.
5. **Soft deletes** — `is_active` boolean on most tables
6. **Ledger pattern** — Running balance computed on INSERT via `SELECT balance FROM table WHERE customer_id=? ORDER BY id DESC LIMIT 1`
7. **FIFO costing** — `SELECT * FROM stock_batches WHERE item_id=? AND warehouse_id=? AND quantity_remaining > 0 ORDER BY received_date ASC, id ASC`
8. **Migration system** — Check `pragma_table_info` or `sqlite_master` for table/column existence, run SQL files idempotently

---

## 5. API Surface — All Endpoints (~155 endpoints, 25 route files)

### 5.1 Auth (4)
| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/auth/login` | Rate-limited | Username + password → JWT cookie |
| POST | `/api/auth/logout` | Token | Clear cookie |
| GET | `/api/auth/me` | Token | Current user profile |
| POST | `/api/auth/change-password` | Token + rate-limited | Current + new password |

### 5.2 Dashboard (16)
| Method | Path | Permission | Description |
|--------|------|-----------|-------------|
| GET | `/api/dashboard/summary` | dashboard:read | Aggregated KPI cards |
| GET | `/api/dashboard/top-customers` | dashboard:read | Top 5 by revenue |
| GET | `/api/dashboard/sales-summary` | dashboard:read | Today/week/month sales |
| GET | `/api/dashboard/expense-summary` | dashboard:read | Weekly/monthly expenses |
| GET | `/api/dashboard/production-status` | dashboard:read | Active/completed production |
| GET | `/api/dashboard/stock-movement-summary` | dashboard:read | Recent 7 days movements |
| GET | `/api/dashboard/kpi` | dashboard:read | Stock health metric |
| GET | `/api/dashboard/ar-summary` | dashboard:read | AR aging buckets |
| GET/POST/PUT/PATCH/DELETE | `/api/dashboard/layout*` | dashboard:* | CRUD custom layouts |

### 5.3 Inventory (24)
| Method | Path | Permission | Description |
|--------|------|-----------|-------------|
| GET/POST | `/api/inventory/items` | inventory:* | List/Create items |
| GET/PUT/DELETE | `/api/inventory/items/:id` | inventory:* | CRUD single item |
| GET | `/api/inventory/items-categories` | inventory:read | Distinct categories |
| GET | `/api/inventory/items-low-stock` | inventory:read | Below reorder level |
| GET | `/api/inventory/items-uom` | inventory:read | Units of measure |
| GET/POST | `/api/inventory/warehouses` | inventory:* | List/Create warehouses |
| GET/PUT/DELETE | `/api/inventory/warehouses/:id` | inventory:* | CRUD warehouse |
| GET/POST | `/api/inventory/stock-movements` | inventory:* | List/Create movements |
| GET | `/api/inventory/stock-summary` | inventory:read | Per-warehouse summary |
| GET | `/api/inventory/stock-ledger/:itemId` | inventory:read | Item movement history |
| GET | `/api/inventory/stock-balances` | inventory:read | Current balances |
| GET/POST | `/api/inventory/physical-counts` | inventory:* | CRUD physical counts |
| GET | `/api/inventory/physical-counts/:id` | inventory:read | Count detail |
| POST | `/api/inventory/physical-counts/:id/items` | inventory:update | Record count item |
| POST | `/api/inventory/physical-counts/:id/complete` | inventory:update | Finalize count |
| POST | `/api/inventory/physical-counts/:id/cancel` | inventory:update | Abort count |
| DELETE | `/api/inventory/physical-counts/:id` | inventory:delete | Remove count |

### 5.4 Sales & Customers (28)
| Method | Path | Permission | Description |
|--------|------|-----------|-------------|
| GET/POST | `/api/customers` | customers:* | List/Create |
| GET/PUT/DELETE | `/api/customers/:id` | customers:* | CRUD |
| GET | `/api/customers/:id/ledger` | customers:read | Customer ledger |
| GET | `/api/customers/:id/statement` | customers:read | Date-ranged statement |
| GET | `/api/customers/:id/balance` | customers:read | Current balance |
| POST | `/api/customers/recalculate-balances` | customers:update | Bulk rebuild |
| GET/POST | `/api/invoices` | invoices:* | List/Create |
| GET/PUT/DELETE | `/api/invoices/:id` | invoices:* | CRUD |
| PUT | `/api/invoices/:id/cancel` | invoices:update | Cancel invoice |
| POST | `/api/invoices/:id/return` | invoices:update | Return items |
| GET | `/api/invoices/:id/payments` | invoices:read | Invoice payments |
| GET | `/api/invoices/returns` | invoices:read | Return history |
| POST | `/api/payments` | payments:create | Create payment |
| GET/PUT/DELETE | `/api/payments/:id` | payments:* | CRUD payment |
| GET/POST | `/api/sales/quotations` | quotations:* | CRUD |
| PUT/DELETE | `/api/sales/quotations/:id` | quotations:* | Update/Delete |
| POST | `/api/sales/quotations/:id/convert` | quotations:update | Quotation → SO |
| GET | `/api/sales/quotations/:id/cycle-chain` | quotations:read | Chain tracking |
| GET/POST | `/api/sales/sales-orders` | sales_orders:* | CRUD |
| PUT/DELETE | `/api/sales/sales-orders/:id` | sales_orders:* | Update/Delete |
| POST | `/api/sales/sales-orders/:id/cancel` | sales_orders:update | Cancel |
| POST | `/api/sales/sales-orders/:id/convert` | sales_orders:create | SO → Invoice |
| GET | `/api/sales/sales-orders/:id/cycle-chain` | sales_orders:read | Chain tracking |
| GET | `/api/sales/dashboard` | sales:read | Sales dashboard |
| POST | `/api/pos/sale` | pos:create | Walk-in sale |
| GET | `/api/pos/transactions` | pos:read | POS history |

### 5.5 Purchasing & Suppliers (21)
| Method | Path | Permission | Description |
|--------|------|-----------|-------------|
| GET/POST | `/api/suppliers` | suppliers:* | List/Create |
| GET/PUT/DELETE | `/api/suppliers/:id` | suppliers:* | CRUD |
| GET | `/api/suppliers/next-code` | suppliers:read | Auto-code |
| GET/POST | `/api/purchase-orders` | purchase_orders:* | List/Create |
| GET/PUT/DELETE | `/api/purchase-orders/:id` | purchase_orders:* | CRUD |
| POST | `/api/purchase-orders/:id/items` | purchase_orders:create | Add line |
| PUT/DELETE | `/api/purchase-orders/:id/items/:itemId` | purchase_orders:* | Update/remove line |
| POST | `/api/purchase-orders/:id/status` | purchase_orders:update | Status transition |
| POST | `/api/purchase-orders/:id/receipts` | purchase_orders:create | Goods receipt |
| POST | `/api/purchase-orders/:id/return-receipt` | purchase_orders:update | Return receipt |
| GET | `/api/purchase-orders/pending` | purchase_orders:read | Pending orders |
| GET | `/api/purchase-orders/:id/receipts` | purchase_orders:read | Receipt history |
| GET | `/api/purchase-orders/summary/supplier/:id` | purchase_orders:read | By supplier |
| GET | `/api/purchase-orders/suppliers/:id/balance` | purchase_orders:read | Balance |
| GET | `/api/purchase-orders/suppliers/:id/transactions` | purchase_orders:read | Transactions |
| GET/POST | `/api/purchases` | purchases:* | Direct purchase |
| GET/DELETE | `/api/purchases/:id` | purchases:* | CRUD |
| POST | `/api/purchases/:id/return` | purchases:update | Return items |

### 5.6 Manufacturing (10)
| Method | Path | Permission | Description |
|--------|------|-----------|-------------|
| GET/POST | `/api/bom` | bom:* | List/Create BOMs |
| GET/PUT/DELETE | `/api/bom/:id` | bom:* | CRUD |
| GET | `/api/bom/by-item/:itemId` | bom:read | By finished item |
| PATCH | `/api/bom/:id/toggle-active` | bom:update | Toggle active |
| GET/POST | `/api/production/productions` | production:* | List/Create |
| GET/DELETE | `/api/production/productions/:id` | production:* | CRUD |
| GET | `/api/production/productions/summary/item/:id` | production:read | Summary |

### 5.7 Accounting & Reports (29)
| Method | Path | Permission | Description |
|--------|------|-----------|-------------|
| GET | `/api/accounting/accounts` | accounting:read | Chart of accounts |
| GET | `/api/accounting/accounts/balances` | accounting:read | Trial balance |
| GET | `/api/accounting/accounts/:code` | accounting:read | Single account |
| GET | `/api/accounting/accounts/:code/balance` | accounting:read | Balance |
| GET | `/api/accounting/periods` | accounting:read | Period list |
| GET/POST | `/api/accounting/periods/:id` | accounting:* | Open/close |
| GET | `/api/activity-logs/*` | activity_log:read | 10 endpoints |
| GET | `/api/reports/ar-aging` | reports:read | AR aging report |
| GET | `/api/reports/customer-statements` | reports:read | Customer statements |
| GET | `/api/reports/top-debtors` | reports:read | Top debtors |
| GET | `/api/reports/dso` | reports:read | Days Sales Outstanding |
| GET | `/api/reports/ar-summary` | reports:read | Receivables summary |
| GET | `/api/reports/sales-summary` | reports:read | Sales summary |
| GET | `/api/reports/sales-by-customer` | reports:read | Sales per customer |
| GET | `/api/reports/sales-by-item` | reports:read | Sales per item |
| GET | `/api/reports/stock-level` | reports:read | Stock levels |
| GET | `/api/reports/low-stock` | reports:read | Low stock |
| GET | `/api/reports/stock-valuation` | reports:read | Stock valuation |
| GET | `/api/reports/inventory-movement` | reports:read | Movement report |
| GET | `/api/reports/profit-loss` | reports:read | P&L statement |
| GET | `/api/reports/cash-flow` | reports:read | Cash flow |
| GET | `/api/reports/purchase-summary` | reports:read | Purchase report |
| GET | `/api/reports/supplier-analysis` | reports:read | Supplier analysis |
| GET | `/api/reports/production-summary` | reports:read | Production efficiency |
| GET | `/api/reports/bom-usage` | reports:read | BOM usage |
| GET | `/api/reports/expenses` | reports:read | Expense report |
| GET | `/api/reports/trial-balance` | reports:read | Trial balance |
| GET | `/api/reports/general-ledger` | reports:read | GL report |
| GET | `/api/reports/balance-sheet` | reports:read | Balance sheet |
| GET | `/api/reports/income-statement` | reports:read | Income statement |
| GET | `/api/reports/tax-summary` | reports:read | Tax summary |
| GET | `/api/reports/batch-traceability/:itemId` | reports:read | Batch trace |

### 5.8 Admin (14)
| Method | Path | Permission | Description |
|--------|------|-----------|-------------|
| GET/POST | `/api/users` | users:* | List/Create |
| GET/PUT/DELETE | `/api/users/:id` | users:* | CRUD |
| PUT | `/api/users/:id/reset-password` | users:update | Admin reset |
| PUT | `/api/users/:id/toggle-status` | users:update | Activate/deactivate |
| GET/POST | `/api/roles` | roles:* | List/Create |
| GET | `/api/roles/permissions` | roles:read | All permissions |
| GET | `/api/roles/:id/permissions` | roles:read | Role permissions |
| PUT/DELETE | `/api/roles/:id` | roles:* | Update/Delete |
| PUT | `/api/roles/:id/permissions` | roles:update | Set permissions |
| GET/PUT | `/api/settings` | settings:* | List/Update settings |
| GET/POST | `/api/expenses/*` | expenses:* | 14 endpoints |
| GET/POST | `/api/employees/*` | employees:* | 12 endpoints |
| GET | `/api/forecasts/*` | forecasts:* | 16 endpoints |
| GET/POST | `/api/reports/custom/*` | reports:* | 11 endpoints |
| GET | `/api/integrations` | integrations:read | Integration settings |
| PUT | `/api/integrations/:service` | integrations:update | Update integration |

### 5.9 Mobile-specific (9)
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/mobile-invoices/draft` | Save draft |
| PUT | `/api/mobile-invoices/draft/:id` | Update draft |
| GET | `/api/mobile-invoices/draft/:id` | Get draft |
| DELETE | `/api/mobile-invoices/draft/:id` | Delete draft |
| GET | `/api/mobile-invoices/items/search` | Item search |
| GET | `/api/mobile-invoices/customers/search` | Customer search |
| GET | `/api/mobile-invoices/tax-rates` | Tax rate list |
| GET | `/api/mobile-invoices/payment-terms` | Payment term list |
| POST | `/api/mobile-invoices/submit` | Submit from mobile |

---

## 6. Frontend Pages & Routes

### Complete Route Map

```
/                                    → Dashboard (16 block types, customizable)
/login                               → Login page
/pos                                 → POS terminal

/inventory                           → Inventory Dashboard
/inventory/items                     → Item List (AG-Grid / Compact Cards)
/inventory/items/new                 → Item Create Form
/inventory/items/:id                 → Item Detail / Edit
/inventory/warehouses                → Warehouse List
/inventory/warehouses/new            → Warehouse Create Form
/inventory/warehouses/:id            → Warehouse Detail
/inventory/stock-movements           → Stock Movement List
/inventory/stock-movements/new       → New Stock Movement
/inventory/stock-ledger/:itemId      → Item Movement History
/inventory/physical-counts           → Physical Count List
/inventory/physical-counts/new       → New Physical Count
/inventory/physical-counts/:id       → Count Detail / Record

/sales                               → Sales Dashboard
/sales/invoices                      → Invoice List
/sales/invoices/new                  → Invoice Create (full form)
/sales/invoices/:id                  → Invoice Detail / Edit
/sales/invoices/:id/print            → Invoice Print (A4 / Thermal)
/sales/quotations                    → Quotation List
/sales/quotations/new                → Quotation Create
/sales/quotations/:id                → Quotation Detail
/sales/quotations/:id/print          → Quotation Print
/sales/orders                        → Sales Order List
/sales/orders/new                    → Sales Order Create
/sales/orders/:id                    → Sales Order Detail
/sales/returns                       → Sales Return History

/purchases                           → Purchases Dashboard
/purchases/direct                    → Direct Purchase List
/purchases/direct/new                → New Direct Purchase
/purchases/direct/:id                → Purchase Detail
/purchases/orders                    → Purchase Order List
/purchases/orders/new                → New Purchase Order
/purchases/orders/:id                → PO Detail (with receipts)
/purchases/receipts                  → Goods Receipt History
/purchases/returns                   → Purchase Return History

/manufacturing                       → Manufacturing Dashboard
/manufacturing/bom                   → BOM List
/manufacturing/bom/new               → Create BOM
/manufacturing/bom/:id               → BOM Detail
/manufacturing/production            → Production List
/manufacturing/production/new        → Record Production
/manufacturing/production/:id        → Production Detail

/customers                           → Customer List
/customers/:id                       → Customer Detail (Overview / Invoices / Payments / Ledger)

/suppliers                           → Supplier List
/suppliers/:id                       → Supplier Detail

/employees                           → Employee List
/employees/new                       → New Employee
/employees/:id                       → Employee Detail / Edit

/expenses                            → Expense List
/expenses/new                        → New Expense
/expenses/categories                 → Category Management

/accounting                          → Accounting Dashboard
/accounting/chart-of-accounts        → COA List
/accounting/periods                  → Periods Management
/accounting/journal                  → General Ledger view (read-only; entries are posted by services, not edited directly)

/reports                             → Reports Dashboard
/reports/ar-aging                    → AR Aging Report
/reports/customer-statements         → Customer Statements
/reports/sales                       → Sales Reports
/reports/inventory                   → Inventory Reports
/reports/financial                   → Financial Reports (P&L, Balance Sheet, Cash Flow)
/reports/custom                      → Custom Report Builder
/reports/tax                         → Tax Summary

/forecasts                           → Forecast Dashboard
/forecasts/demand                    → Demand Forecast
/forecasts/trends                    → Trend Analysis
/forecasts/accuracy                  → Accuracy Metrics
/forecasts/model-config              → Model Configuration
/forecasts/seasonal-events           → Event Calendar

/settings                            → Settings Page
/settings/integrations               → Integration Config

/users                               → User List
/users/:id                           → User Detail / Edit

/roles                               → Role List
/roles/:id                           → Role Detail / Permissions

/activity-log                        → Activity Log Viewer

/dashboard                           → Dashboard (aliased)
```

### Layout Structure

```rust
// Desktop (breakpoint > 768px)
┌──────────┬──────────────────────────────────────┐
│          │  Top Menu (Search, Notifications,     │
│ Sidebar  │        User Menu)                     │
│ (260px)  ├──────────────────────────────────────┤
│          │                                      │
│ - Logo   │  Content Area (Page Router Outlet)   │
│ - Nav    │                                      │
│   items  │  ┌────────────────────────────────┐  │
│          │  │  Data Table (Desktop)           │  │
│          │  │  or Compact Cards (Mobile)      │  │
│          │  └────────────────────────────────┘  │
│          │                                      │
│          │  Floating Action Button (Mobile)      │
└──────────┴──────────────────────────────────────┘

// Mobile (breakpoint ≤ 768px)
┌──────────────────────────────────┐
│ Top Bar (Hamburger + Title + Search)│
├──────────────────────────────────┤
│                                  │
│ Compact Card List                │
│ ┌──────────────────────────────┐ │
│ │ Card 1: Summary + Chevron    │ │
│ └──────────────────────────────┘ │
│ ┌──────────────────────────────┐ │
│ │ Card 2: Summary + Chevron    │ │
│ └──────────────────────────────┘ │
│                                  │
├──────────────────────────────────┤
│ Mobile Action Bar (Add, Search,   │
│   Filter, Menu)                   │
└──────────────────────────────────┘
```

---

## 7. UI Component Inventory

### 7.1 Common/Shared Components (to port as Dioxus components)

| Component | Props/State | Features | Desktop | Mobile |
|-----------|-------------|----------|---------|--------|
| **MiniERPGrid** | columns, rows, onRowClick, loading | Sort, filter, paginate, row selection, column resize | ✅ AG-Grid equivalent | ❌ Hidden |
| **CompactCardShell** | cards[], onCardClick, search, loading | Search bar, card list, pull-to-refresh | ❌ Hidden | ✅ Main list |
| **Compact*Card** (20 variants) | item data, actions | Summary line, expandable detail, action menu | ❌ Shown on mobile | ✅ |
| **Modal** | title, children, size, onClose | Overlay, close button, sizes (sm/md/lg/xl) | ✅ | ✅ (bottom-sheet on ≤768px) |
| **SearchModal** | onSelect, items, renderItem | Search input, filtered list, keyboard nav | ✅ | ✅ |
| **SearchableSelect** | options, value, onChange | Typeahead, dropdown, keyboard selection | ✅ | ✅ |
| **FormInput** | label, value, onChange, error, type | Label, input, error message, required indicator | ✅ | ✅ |
| **Button** | variant, size, loading, icon, onClick | Primary/secondary/danger/ghost, icon support | ✅ | ✅ (full-width on mobile) |
| **DropdownMenu** | items[], trigger | Positioned menu, click-away close | ✅ | ✅ |
| **DateRangePicker** | start, end, onChange | Two date inputs | ✅ | ✅ |
| **StatCard** | title, value, icon, trend | KPI display | ✅ | ✅ |
| **SummaryCard** | title, values[], variant | Multi-value summary | ✅ | ✅ |
| **Toast** | message, type, duration | Auto-dismiss, error/success/info | ✅ | ✅ |
| **PageLoader** | none | Full-page loading spinner | ✅ | ✅ |
| **KeyboardShortcutsHelp** | none | ? key overlay | ✅ | ❌ |
| **ErrorBoundary** | children | Catch fallback | ✅ | ✅ |
| **LanguageToggle** | none | EN/UR switch | ✅ | ✅ |

### 7.2 Layout Components

| Component | Features |
|-----------|----------|
| **Sidebar** | Logo, navigation items (collapsible), active state, user info, collapse toggle |
| **TopMenu** | Search bar, notifications bell, user dropdown, settings, language toggle |
| **FloatingActionButton** | Speed dial menu for mobile (Add Invoice, Customer, Item, etc.) |
| **QuickActionsPanel** | Slide-out panel with shortcuts |
| **ShortcutBar** | Bottom nav bar for mobile (Dashboard, Sales, Inventory, More) |

### 7.3 Business Components (per module)

**Invoice:**
- `InvoiceFormHeader` — Customer select, dates, discount, terms
- `InvoiceItemsTable` — Editable grid (description/qty/rate/discount/tax/amount)
- `InvoiceEditableCell` — Inline text/number editing
- `InvoiceSearchableCell` — Item search popup
- `InvoicePaymentPanel` — Payment method, amount, reference
- `InvoiceRouter` — Desktop vs Mobile form switch
- `InvoiceTemplateA4` — Print template (A4)
- `ThermalInvoiceTemplate` — Print template (thermal 80mm)
- `QuotationTemplateA4` — Print template
- `PurchaseOrderTemplateA4` — Print template
- `PriceHistoryHint` — Price suggestion tooltip


**Customer Detail (Tab-based):**
- `CustomerHeader` — Name, code, balance, credit
- `CustomerModals` — Edit/Deactivate modals
- `OverviewTab` — Metrics, charts, recent activity
- `InvoicesTab` — Invoice list (AG-Grid / cards)
- `PaymentsTab` — Payment list
- `LedgerTab` — Running balance ledger
- `EditPaymentForm` — Edit payment modal

**Dashboard:**
- `DashboardLayout` — Grid of blocks, drag-reorderable
- `DashboardBlock` — Individual block wrapper (loading, resize)
- `DashboardBlockPalette` — Block picker sidebar
- `DashboardCustomizationBar` — Edit mode toolbar
- Block types (16): StatCards, SalesSummary, ExpenseSummary, ARSummary, TopCustomers, StockByCategory, StockMovementSummary, LowStockAlerts, RecentActivity, KPIGauge, ForecastSnapshot, ProductionStatus, QuickActions, CustomText, SalesPurchasesChart, DeprecatedBlock

### 7.4 Form Validation (Zod → Rust)

Every module currently validates with a per-form Zod schema (`z.object({ field: z.string().min(1).max(100), ... })`). Port these to `garde` derive macros or manual checks — full rule-by-rule mapping is in **§11**.

### 7.5 AG-Grid Features to Replace

The custom table component must support:
- Column definitions (field, headerName, width, sortable, filter, editable, cellRenderer)
- Server-side pagination (or client-side for small datasets)
- Sorting (multi-column)
- Filtering (text, number, date, select)
- Row selection (single/multi)
- Cell editing (inline)
- Cell renderers (status badges, currency, dates, action buttons)
- Column resize/reorder
- Row grouping (for reports)
- Pinned columns (actions column)
- Row height variants (compact/standard/comfortable)
- Cell class rules (status-based coloring)

**Dioxus approach:** Build a generic `DataGrid<T>` component using virtual scrolling (for performance) and CSS-based rendering. No ready-made equivalent — this is the single biggest effort item.

---

## 8. Business Logic & Services

### 8.1 Core Services (all to port to Rust)

| Service | Key Functions | DB Tables | Transaction? | Complexity |
|---------|--------------|-----------|-------------|------------|
| **InvoiceService** | create, update, cancel, return | invoices, invoice_items, stock_movements, stock_batches, payments, payment_allocations, customer_ledger, journal_lines (6+) | ✅ Yes | **High** |
| **SalesOrderService** | create, convert to invoice, cancel | sales_orders, sales_order_items, stock_movements, stock_batches | ✅ Yes | High |
| **QuotationService** | create, convert to SO | quotations, quotation_items | ✅ Yes | Medium |
| **POSService** | create walk-in sale | invoices, invoice_items, stock_movements, payments, customer_ledger | ✅ Yes | Medium |
| **PurchaseOrderService** | create, receive goods, return | purchase_orders, purchase_order_items, goods_receipts, stock_movements, supplier_ledger | ✅ Yes | High |
| **PurchaseService** | record, return | purchases, stock_movements, stock_batches | ✅ Yes | Medium |
| **ProductionService** | record production | productions, production_inputs, stock_batches, stock_movements, journal_lines | ✅ Yes | High |
| **BOMService** | create, update, toggle | boms, bom_items | ✅ Yes | Low |
| **CustomerService** | create (with opening balance), ledger, statement, recalculate | customers, customer_ledger | ✅ Partial | Medium |
| **SupplierService** | CRUD, balance | suppliers, supplier_ledger | ❌ | Low |
| **EmployeeService** | CRUD, pay salary | employees, salary_payments, journal_lines | ✅ Yes | Medium |
| **ExpenseService** | CRUD, categories | expenses, expense_categories | ❌ | Low |
| **StockMovementService** | record (FIFO), ledger, summary | stock_movements, stock_balances, stock_batches, items | ✅ Yes | Medium |
| **PhysicalCountService** | create, record, complete | physical_counts, physical_count_items, stock_movements, journal_lines | ✅ Yes | Medium |
| **AccountingService** | post entries, balances, periods | journal_lines, chart_of_accounts, accounting_periods | ✅ Yes | High |
| **DashboardService** | 8 aggregated queries | Multiple tables | ❌ | Low |
| **ReportService** | 25 financial/operational reports | Multiple tables | ❌ | Medium |
| **ForecastService** | 5 models, generate, accuracy, events | demand_forecasts, forecast_accuracy, forecast_model_config, forecast_seasonal_events | ✅ Partial | **High** |
| **AuthService** | login, change-password | users | ❌ | Low |
| **ActivityLogService** | log, query, cleanup | activity_log | ✅ (batch) | Low |
| **ReportQueryEngine** | Dynamic SQL from JSON config | All 17 registered entities | ❌ | **High** |

### 8.2 Invoice Creation Flow (most complex transaction)

```
createInvoice(data, userId):
  BEGIN TRANSACTION
    1. Validate: customer exists, items have stock, dates valid
    2. INSERT INTO invoices (status='Unpaid', amounts computed)
    3. FOR EACH item in data.items:
       a. INSERT INTO invoice_items
       b. IF source_type = 'SALES_ORDER' OR source_type = 'DIRECT':
          - consumeFromOldestBatches(item, warehouse, quantity)
          - FOR EACH batch consumed:
            INSERT INTO stock_movements (type='SALE', quantity=-qty, unit_cost=batch.cost)
            UPDATE stock_batches SET quantity_remaining -= qty
            UPDATE stock_balances SET quantity -= qty
       c. ELSE IF source_type = 'POS':
          - Same FIFO consumption
    4. POST GL JOURNAL ENTRIES:
       a. AccountingService.postInvoiceEntry: Dr AR, Cr Sales Revenue, Cr Tax Payable
       b. AccountingService.postCOGSEntry: Dr COGS, Cr Inventory
    5. IF data.record_payment:
       a. INSERT INTO payments (amount=paid_amount, payment_method)
       b. INSERT INTO payment_allocations (payment_id, invoice_id, amount)
       c. INSERT INTO customer_ledger (type='PAYMENT', credit=amount)
       d. AccountingService.postPaymentEntry: Dr Cash, Cr AR
       e. UPDATE invoices SET paid_amount += amount, balance_amount = total - paid
       f. IF balance_amount = 0 → SET status = 'Paid'
    6. INSERT INTO customer_ledger (type='INVOICE', debit=total_amount, balance=running)
    7. UPDATE customers SET current_balance += total_amount
    8. Log activity: "Invoice INV-2026-0001 created"
  COMMIT
  Return { invoice, items, stock_movements, payments }
```

### 8.3 Stock Consumption Flow (FIFO)

```
consumeFromOldestBatches(itemId, warehouseId, totalQty):
  batches = SELECT * FROM stock_batches 
    WHERE item_id = itemId AND warehouse_id = warehouseId 
      AND quantity_remaining > 0 
    ORDER BY received_date ASC, id ASC
  
  remaining = totalQty
  consumed = []
  
  FOR batch in batches:
    qty_to_take = MIN(batch.quantity_remaining, remaining)
    consumed.push({ batch, quantity: qty_to_take, unit_cost: batch.unit_cost })
    batch.quantity_remaining -= qty_to_take
    UPDATE stock_batches SET quantity_remaining = batch.quantity_remaining WHERE id = batch.id
    remaining -= qty_to_take
    IF remaining == 0: BREAK
  
  IF remaining > 0: THROW "Insufficient stock"
  RETURN consumed  // Array of { batch, quantity, unit_cost }
```

### 8.4 Accounting Double-Entry Posting

```
postEntry(db, input):
  // Input: { lines: [{ account_id, debit, credit, description }], entry_date, reference_type, reference_id }
  VALIDATE: 
    - entry_date falls within an OPEN accounting_period
    - Each line has either debit > 0 or credit > 0 (not both)
    - At least 2 lines
    - SUM(debits) ≈ SUM(credits) (within 0.01)
  
  journal_entry_id = SELECT MAX(journal_entry_id) FROM journal_lines + 1
  
  FOR line in input.lines:
    INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, description, line_date, reference_type, reference_id)
```

---

## 9. Authentication & Security

### Auth Flow (same pattern, Rust implementation)

```
Login Flow:
  1. POST /api/auth/login { username, password }
  2. SELECT * FROM users WHERE username = ? AND is_active = 1
  3. bcrypt::verify(password, user.password_hash)
  4. Generate JWT (HS256):
     - Payload: { id, username, email, role, exp: now + 24h, iss: "mini-erp", aud: "mini-erp-client" }
     - Sign with secret key (from env)
  5. Set httpOnly cookie: token=<jwt>; Path=/; HttpOnly; SameSite=Strict; Max-Age=86400
  6. Return { user: { id, username, full_name, email, role, is_active } }
```

### JWT Implementation (Rust crates)
- `jsonwebtoken` crate (same algorithm, HS256)
- Secret from env var `JWT_SECRET` (required, crash if unset)
- 24h expiry, fixed issuer/audience verification
- Algorithm whitelist: only HS256 (no `none` attack)

### Password Hashing
- `bcrypt` crate, cost factor 12
- Default admin seeded on first run (env var `DEFAULT_ADMIN_PASSWORD`)

### RBAC Middleware Pattern
```rust
// Middleware: require_permission(module: &str, action: &str)
// 1. Extract user from request extension (set by auth middleware)
// 2. If user.role == "admin" → pass (bypass)
// 3. Query: SELECT 1 FROM role_permissions rp
//    JOIN permissions p ON rp.permission_id = p.id
//    WHERE rp.role_id = ? AND p.module = ? AND p.action = ?
// 4. If no row → return 403 Forbidden
```

### Security Checklist (port all)
- [ ] JWT httpOnly cookie (SameSite=Strict)
- [ ] Password hashing (bcrypt, 12 rounds)
- [ ] RBAC (dual-layer: role string + permission tables)
- [ ] Rate limiting (login: 5/15min, password-change: 3/hr, API: 100/min)
- [ ] Helmet security headers (CSP, X-Frame-Options, etc.)
- [ ] CORS whitelist
- [ ] Input validation (all endpoints)
- [ ] Parameterized SQL (no injection)
- [ ] AES-256-GCM encryption for sensitive settings
- [ ] Activity audit logging (60+ action types)
- [ ] File upload MIME validation (10mb limit)
- [ ] Error handling (no stack leakage in production)

### Rate Limiting (Rust crates)
- `governor` crate for token-bucket rate limiting
- Auth limiter: 5 requests / 15 min per IP
- Password change: 3 / hour per IP
- API: 100 / min per IP
- Sensitive operations: 10 / min per IP

---

## 10. State Management & Data Flow

### 10.1 Current React Patterns (to replicate in Dioxus)

**TanStack Query (server state):**
```typescript
// Each module has a custom hook pattern:
function useItems(filters: ItemFilters) {
  return useQuery({
    queryKey: ['items', filters],
    queryFn: () => api.get('/api/inventory/items', { params: filters }).then(r => r.data)
  });
}

// Mutations:
function useCreateItem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data) => api.post('/api/inventory/items', data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['items'] })
  });
}
```

**Dioxus equivalent:** Use `use_resource` + `use_server_future` for data fetching, manual cache invalidation via signals:

```rust
#[component]
fn ItemList(filters: Filters) -> Element {
    let items = use_resource(move || get_items(filters.clone()));
    // ...
}

// Mutations: use callbacks that invalidate resources
fn use_create_item(cx: &ScopeState) -> UseMutation<ItemFormData, Item> {
    // POST → refresh items resource
}
```

### 10.2 State Categories

| State Type | Example | Storage | Dioxus Equivalent |
|-----------|---------|---------|-------------------|
| Server data | Items list, invoice detail | API-fetched | `use_resource`, signals |
| Auth state | User, token, permissions | localStorage + memory | `use_context_provider` + storage API |
| UI state | Sidebar open, active tab, modal | Component-local | `use_signal` |
| Form state | Invoice form fields | Component-local | `use_signal` |
| Filter state | Search, sort, pagination | URL params + state | `use_query` (URL) + signals |
| Theme state | Sidebar collapsed, mobile | Context | `use_context_provider` |
| Toast notifications | Success/error messages | Global signal | Signal + overlay component |
| Dashboard layout | Block positions (JSON) | API + reorder | `use_resource` + drag signals |
| i18n | Active language | localStorage + context | `use_context_provider` |

### 10.3 Auth State Flow

```rust
// AuthProvider (context)
struct AuthState {
    user: Signal<Option<User>>,
    token: Signal<Option<String>>,
    permissions: Signal<HashMap<String, Vec<String>>>,
}

// On app init:
fn init_auth() {
    // Check localStorage for stored user
    // If found, assume authenticated (no /me call at startup)
    // Set user signal
    
    // On 401 response from any API call → clear auth → redirect to /login
    // On 403 → show toast "Access denied"
}

// Login: POST /auth/login → store response user in signal + localStorage
// Logout: POST /auth/logout → clear signal + localStorage → redirect
```

---

## 11. Form Validation

### Current Zod Schemas (port to Rust `garde` or manual)

Key validation patterns:
- `z.string().min(1, "Required")` → `#[garde(length(min=1))]`
- `z.number().positive()` → `#[garde(range(min=0.01))]`
- `z.string().email()` → `#[garde(email)]`
- `z.array().min(1)` → `#[garde(length(min=1))]`

### All Form Validation Rules

| Form | Fields | Rules |
|------|--------|-------|
| **Item** | item_code, item_name, category, standard_cost, standard_selling_price, reorder_level | code: 1-50 unique, name: 1-200, cost/price ≥ 0, reorder ≥ 0 |
| **Customer** | customer_name, email, phone, credit_limit, payment_terms_days | name: 1-200, email: valid format, credit_limit ≥ 0, days: integer ≥ 0 |
| **Supplier** | supplier_name, email, phone | name: 1-200, email: valid |
| **Invoice** | customer_id, invoice_date, items[] | ≥1 item, amounts match line totals, discount ≤ total |
| **Invoice Item** | item_id, quantity, unit_price, tax_rate | quantity > 0, unit_price ≥ 0, tax_rate ≥ 0 |
| **Payment** | customer_id, amount, invoice_allocations | amount > 0, allocations sum = amount, each invoice balance sufficient |
| **Purchase Order** | supplier_id, po_date, items[] | ≥1 item, quantities > 0, prices ≥ 0 |
| **Goods Receipt** | po_id, receipt_date, items[] | quantities ≤ PO remaining quantities |
| **Sales Order** | customer_id, so_date, items[] | ≥1 item, quantities > 0 |
| **Quotation** | customer_id, quotation_date, items[] | ≥1 item, dates valid |
| **BOM** | bom_name, finished_item_id, items[] | name: 1-200, ≥1 raw material, quantities > 0 |
| **Production** | output_item_id, output_quantity, warehouse_id, bom_id | quantity > 0, BOM exists, stock sufficient for inputs |
| **User** | username, email, password, full_name | username: 3-50 unique, email: valid, password: ≥6, name: 1-100 |
| **Employee** | first_name, last_name, email, phone, salary | names: 1-100, email: valid, salary ≥ 0 |
| **Expense** | expense_category, amount, expense_date | category: exists, amount > 0, date: valid |
| **Stock Movement** | item_id, warehouse_id, movement_type, quantity | quantity ≠ 0, outgoing ≤ available, type valid |
| **Role** | role_name | name: 1-50 unique |

---

## 12. Calculations (Pure Functions — port to Rust)

These are synchronous, side-effect-free functions currently in TypeScript. Port as pure Rust functions in a `calculations` module.

### 12.1 Invoice Calculations (`invoiceCalculations.ts` → `calculations::invoice`)

```rust
pub fn calculate_item_discount(quantity: f64, unit_price: f64, discount_type: &str, discount_value: f64) -> f64
pub fn calculate_item_total(quantity: f64, unit_price: f64, discount: f64) -> f64
pub fn calculate_subtotal(items: &[InvoiceItem]) -> f64
pub fn calculate_tax(subtotal: f64, tax_rate: f64) -> f64
pub fn calculate_discount(subtotal: f64, discount_type: &str, discount_value: f64) -> f64
pub fn calculate_total(subtotal: f64, discount: f64, tax: f64) -> f64
pub fn create_empty_item_row() -> InvoiceFormItem
pub fn pad_items_to_minimum(items: &mut Vec<InvoiceFormItem>, min: usize)
pub fn get_expected_status(total: f64, paid: f64, balance: f64) -> &'static str
// Logic: if paid == 0 → "Unpaid", if paid >= total → "Paid", else "Partially Paid"
pub fn compute_invoice_metrics(items: &[InvoiceItem], discount: Discount) -> InvoiceMetrics
```

### 12.2 Quotation Calculations (`quotationCalculations.ts` → `calculations::quotation`)

```rust
pub fn calculate_item_discount(item: &QuotationFormItem) -> f64  // Handles flat vs percentage
pub fn calculate_item_total(item: &QuotationFormItem) -> f64
pub fn calculate_subtotal(items: &[QuotationFormItem]) -> f64
pub fn calculate_discount(subtotal: f64, discount_type: &str, discount_value: f64) -> f64
pub fn calculate_tax(subtotal: f64, tax_rate: f64) -> f64
pub fn calculate_total(items: &[QuotationFormItem], discount: Discount) -> f64
pub fn create_empty_item_row() -> QuotationFormItem
pub fn pad_items_to_minimum(items: &mut Vec<QuotationFormItem>, min: usize)
pub fn get_sellable_items(items: &[Item]) -> Vec<Item>  // is_finished_good || is_purchased
pub fn filter_filled_items(items: &[QuotationFormItem]) -> Vec<QuotationFormItem>
```

### 12.3 Sales Order Calculations (`salesOrderCalculations.ts` → `calculations::sales_order`)

Same shape as Quotation Calculations (§12.2) — item discount, subtotal, tax, and total functions — plus:

```rust
pub fn calculate_delivered_percentage(item: &SalesOrderItem) -> f64  // delivered_quantity / quantity
pub fn get_undelivered_items(items: &[SalesOrderItem]) -> Vec<SalesOrderItem>
```

### 12.4 Customer Calculations (`customerCalculations.ts` → `calculations::customer`)

```rust
pub fn calculate_ledger_totals(entries: &[LedgerEntry]) -> (f64, f64, f64)  // (total_debit, total_credit, balance)
pub fn calculate_current_balance(entries: &[LedgerEntry]) -> f64
pub fn calculate_total_invoiced(entries: &[LedgerEntry]) -> f64
pub fn calculate_total_paid(entries: &[LedgerEntry]) -> f64
pub fn calculate_total_outstanding(invoices: &[Invoice]) -> f64
pub fn calculate_credit_utilization(balance: f64, credit_limit: f64) -> f64  // percentage
pub fn calculate_overdue_invoices(invoices: &[Invoice]) -> Vec<Invoice>
pub fn calculate_average_days_to_pay(entries: &[LedgerEntry]) -> f64
pub fn compute_customer_metrics(customer: &Customer, entries: &[LedgerEntry], invoices: &[Invoice], payments: &[Payment]) -> CustomerMetrics
pub fn format_currency(amount: f64) -> String  // "Rs. 1,234.56"
pub fn format_date(date: &NaiveDate) -> String  // "Jan 15, 2024"
```

### 12.5 AR Aging Calculations

```rust
pub fn calculate_aging_buckets(invoices: &[Invoice], as_of_date: NaiveDate) -> AgingBuckets
// Buckets:
//   Current: due_date > as_of_date
//   1-30 days: as_of_date - due_date in [1, 30]
//   31-60 days: in [31, 60]
//   61-90 days: in [61, 90]
//   90+ days: > 90
pub fn calculate_dso(receivables: f64, credit_sales: f64, days: i64) -> f64  // (receivables / credit_sales) * days
```

### 12.6 Stock Calculations

```rust
pub fn compute_fifo_cogs(consumptions: &[BatchConsumption]) -> f64
// SUM(quantity * unit_cost) for each consumed batch

pub fn compute_stock_value(items: &[StockBalance], costs: &HashMap<i64, f64>) -> f64
// SUM(balance.quantity * cost[item_id])

pub fn calculate_reorder_quantity(daily_usage: f64, lead_time_days: f64, safety_stock: f64) -> f64
// (daily_usage * lead_time_days) + safety_stock - current_stock
```

### 12.7 Formatting Utilities (`formatters.ts`)

```rust
pub fn format_currency(amount: f64) -> String  // "Rs. 1,234.00"
pub fn format_quantity(amount: f64, uom: &str) -> String  // "100.00 Nos"
pub fn format_percent(value: f64) -> String  // "45.5%"
pub fn format_date(date: NaiveDate) -> String  // "2024-01-15"
pub fn format_datetime(dt: NaiveDateTime) -> String  // "2024-01-15 14:30:00"
pub fn format_phone(phone: &str) -> String  // "+92 300 1234567"
```

---

## 13. Reporting Engine

### 13.1 Standard Reports (25, read-only SQL queries)

Each is a parameterized SQL query returning serde-serializable structs:

| Report | Key Query Pattern | Parameters |
|--------|-------------------|------------|
| AR Aging | Customers LEFT JOIN invoices, GROUP BY aging bucket | asOfDate |
| Customer Statement | customer_ledger WHERE customer_id AND date range | customerId, startDate, endDate |
| Top Debtors | customers ORDER BY current_balance DESC LIMIT N | limit, asOfDate |
| DSO | (AR / credit sales) × days | fromDate, toDate |
| Sales Summary | invoices GROUP BY date/customer | startDate, endDate |
| Sales by Customer | invoices JOIN items GROUP BY customer | startDate, endDate |
| Sales by Item | invoice_items JOIN items GROUP BY item | startDate, endDate |
| Stock Level | items + stock_balances | none |
| Low Stock | items WHERE current_stock ≤ reorder_level | none |
| Stock Valuation | items × standard_cost WITH warehouse subtotals | none |
| Inventory Movement | stock_movements WITH date range + item filter | startDate, endDate, itemId |
| P&L | Revenue accounts - Expense accounts | fromDate, toDate |
| Cash Flow | Operating/Investing/Financing from journal_lines | fromDate, toDate |
| Purchase Summary | purchases GROUP BY date/supplier | startDate, endDate |
| Supplier Analysis | purchases GROUP BY supplier | startDate, endDate |
| Production Summary | productions WITH BOM + inputs | startDate, endDate |
| BOM Usage | boms JOIN bom_items JOIN work_orders | startDate, endDate, itemId |
| Expenses | expenses GROUP BY category | fromDate, toDate, category |
| Trial Balance | chart_of_accounts WITH balance from journal_lines | asOfDate |
| General Ledger | journal_lines WITH account filter | startDate, endDate |
| Balance Sheet | Assets - Liabilities - Equity | asOfDate |
| Income Statement | Revenue - Expense accounts | fromDate, toDate |
| Tax Summary | invoice_items WITH tax_rate | fromDate, toDate |
| Batch Traceability | stock_batches WITH movements BY item | itemId |

### 13.2 Custom Report Builder (Dynamic SQL)

The ad-hoc report builder allows users to create reports from 17 registered entities. Port the `reportQueryEngine.ts` logic:

```rust
struct ReportConfig {
    entity: String,           // "invoices", "items", etc.
    fields: Vec<FieldDef>,    // { field, alias, aggregate? }
    filters: Vec<FilterGroup>, // AND/OR nested conditions
    sort: Vec<SortDef>,        // { field, direction }
    limit: Option<i64>,
}

impl ReportConfig {
    fn build_sql(&self, entity_registry: &EntityRegistry) -> (String, Vec<rusqlite::types::Value>) {
        // 1. Validate entity exists in registry
        // 2. Build SELECT with aliases and aggregate functions
        // 3. Build FROM with LEFT JOINs as needed
        // 4. Build WHERE from filter groups (operators: eq, neq, gt, lt, contains, etc.)
        // 5. Handle relative dates (this_month, last_30_days, etc.)
        // 6. Build GROUP BY / HAVING for aggregates
        // 7. Build ORDER BY
        // 8. Return parameterized SQL
    }
}
```

Entity registry: 17 entities with typed fields, aggregate capabilities (SUM, COUNT, AVG, MIN, MAX), and relationship joins.

---

## 14. Forecasting Engine

Port `forecastService.ts` (~750 lines) to Rust.

### 14.1 Models (5 algorithms)

```rust
// Weighted Moving Average (weights: 0.5, 0.3, 0.2, last 3 periods)
fn calculate_wma(sales: &[f64]) -> f64;

// Simple Exponential Smoothing
fn calculate_ses(sales: &[f64], alpha: f64) -> f64;

// Holt's Linear Trend
fn calculate_holts(sales: &[f64], alpha: f64, beta: f64) -> f64;

// Holt-Winters Triple Exponential Smoothing (with seasonality)
fn calculate_holt_winters(sales: &[f64], periods: usize, alpha: f64, beta: f64, gamma: f64) -> f64;

// Simple AR(1) with differencing
fn calculate_simple_arima(sales: &[f64]) -> f64;
```

### 14.2 Auto-Model Selection

```rust
fn auto_select_model(item_id: i64) -> &'static str {
    // Read forecast_accuracy table for this item
    // Return model type with lowest MAPE (min 3 samples)
    // Fallback: "weighted_moving_average"
}
```

### 14.3 Forecast Generation Pipeline

```rust
fn generate_all_forecasts() -> Result<ForecastRun> {
    // 1. Get all active items with sales history
    // 2. For each item:
    //    a. Load model config (item-specific → category default → global default)
    //    b. Fetch 12 months of historical sales from invoice_items
    //    c. Apply seasonal multipliers
    //    d. Run model → predicted quantities (week, month, quarter)
    //    e. Calculate confidence (coefficient of variation: >0.5→50%, >0.3→70%, else→90%)
    //    f. Detect trend direction (>5%→growing, <-5%→declining, else→stable)
    //    g. Apply bias correction from historical accuracy
    //    h. Upsert into demand_forecasts table
    // 3. Compute safety stock: Z × √(lead_time_days) × σ_daily_demand
    // 4. Create forecast_runs record
    // 5. Log completion
}
```

### 14.4 Accuracy Computation

```rust
fn compute_forecast_accuracy() -> Result<()> {
    // For each past forecast with a matching actual sales period:
    //   MAPE = (|actual - predicted| / actual) × 100
    //   MAE  = |actual - predicted|
    //   sMAPE = (|actual - predicted| / ((|actual| + |predicted|) / 2)) × 100
    // Upsert into forecast_accuracy
}
```

### 14.5 Seasonal Events

5 seeded events (New Year, Eid al-Fitr, Eid al-Adha, Black Friday, Back to School) with date ranges and multiplier values. Recurring events auto-shifted to current year.

---

## 15. Desktop/Mobile UI Patterns

### 15.1 Responsive Breakpoints

```css
/* Already defined in CSS variables — port to Dioxus conditional rendering */
--breakpoint-mobile: 768px;
--breakpoint-small-phone: 413px;
```

| Breakpoint | Layout | Table Display | Form Display | Modal |
|-----------|--------|---------------|-------------|-------|
| >768px | Sidebar (260px) + Content | DataGrid (full table) | Inline form page | Centered overlay |
| ≤768px | Top bar + full-width content | Compact cards | Single-column wizard | Bottom sheet |
| ≤413px | Same as mobile | Smaller cards, stacked buttons | Full-width inputs | Same |

### 15.2 Desktop Data Display (DataGrid)

Replace AG-Grid with a custom virtual-scrolling table:

```rust
#[component]
fn DataGrid<T: Serializable + 'static>(
    columns: Vec<ColumnDef>,
    rows: Vec<T>,
    on_row_click: Option<Callback<T>>,
    loading: bool,
    page_size: Option<usize>,  // default 50
    sortable: bool,
    filterable: bool,
) -> Element
```

Features to implement:
- Fixed header row
- Virtual scroll for performance (only render visible rows + buffer)
- Column resizing via mouse drag
- Column sorting (click header)
- Text filter per column
- Row striping
- Cell class rules (status → color)
- Action column (edit, delete, view buttons)
- Loading skeleton state
- Empty state message

### 15.3 Mobile Data Display (Compact Card System)

```rust
#[component]
fn CompactCardList<T: Serializable + 'static>(
    cards: Vec<CompactCardDef<T>>,
    on_card_click: Callback<T>,
    search_placeholder: &'static str,
    loading: bool,
    action_menu: Option<Vec<ActionDef>>,
) -> Element

// Each card renders as:
// ┌──────────────────────────────────┐
// │ MAIN LABEL           STATUS BADGE│
// │ Subtitle line                    │
// │ Amount/Value        → chevron    │
// └──────────────────────────────────┘
```

20 card variants (CompactInvoiceCard, CompactItemCard, CompactCustomerCard, etc.), each with:
- Main label + status badge (colored)
- Subtitle/secondary info
- Primary value/amount
- Right chevron for navigation
- Long-press action menu (mobile)

### 15.4 Mobile Form Wizard

For complex forms (Invoice, Sales Order, PO), use a step wizard:

```rust
// Step 1: Select Customer (search modal)
// Step 2: Add Items (search + inline add)
// Step 3: Set Discounts/Tax
// Step 4: Payment (optional)
// Step 5: Review & Submit
```

### 15.5 Mobile Action Bar

Fixed bottom bar on mobile:
```rust
// ┌──────────────────────────────────┐
// │ [+ Add]  [🔍 Search]  [⚙ Filter] │
// │          [☰ Menu]                │
// └──────────────────────────────────┘
```

### 15.6 Print Templates (Invoice, Quotation, PO)

CSS-based print layout rather than jsPDF. Dioxus can render to a hidden printable element that uses `@media print` styles.

Three template types to port:
- **A4 Invoice** — Full company header, customer info, itemized table, totals, payment info
- **Thermal 80mm** — Compact receipt-style format
- **A4 Quotation/PO** — Similar to invoice with appropriate labels

---

## 16. i18n

### 16.1 Supported Languages

| Code | Language | Direction |
|------|----------|-----------|
| `en` | English | LTR |
| `ur` | Urdu | RTL |

### 16.2 Translation Structure (~600 keys per language)

```rust
struct Translations {
    dashboard: DashboardTranslations,
    customers: ModuleTranslations,
    suppliers: ModuleTranslations,
    reports: HashMap<String, String>,  // 25+ report names
    quotations: ModuleTranslations,
    sales_orders: ModuleTranslations,
    sales: ModuleTranslations,
    inventory: ModuleTranslations,
    payments: ModuleTranslations,
    purchases: ModuleTranslations,
    forecasts: ModuleTranslations,
    employees: ModuleTranslations,
    expenses: ModuleTranslations,
    bom: ModuleTranslations,
    production: ModuleTranslations,
    warehouses: ModuleTranslations,
    stock_movements: ModuleTranslations,
    settings: ModuleTranslations,
    integrations: ModuleTranslations,
    user_management: ModuleTranslations,
    custom_reports: CustomReportTranslations,
    nav: HashMap<String, String>,  // 40+ nav items
    actions: HashMap<String, String>,  // 30 common actions
    messages: HashMap<String, String>,  // saved, deleted, error, loading
    fields: HashMap<String, String>,  // 32 common field labels
    status: HashMap<String, String>,  // active, inactive, pending, completed
    common: HashMap<String, String>,  // 40 common terms
    shortcuts: HashMap<String, String>,
    errors: HashMap<String, String>,
}
```

### 16.3 Context & Switching

```rust
#[derive(Clone, Copy)]
struct I18nContext {
    locale: Signal<Locale>,  // "en" | "ur"
    translations: Signal<Translations>,
}

// Usage: t().dashboard.welcome  or  t_field("quantity")
// RTL: if locale == "ur" { rtl.css } else { ltr.css }
```

---

## 17. Dioxus-Specific Considerations

### 17.1 Crate Dependencies

The full dependency list, with feature flags and release profile, lives in **Appendix B** as a ready-to-use `Cargo.toml`. Headline crates: `dioxus` (web/desktop/mobile/router), `rusqlite` (bundled SQLite), `jsonwebtoken` + `bcrypt` (auth), `chrono` (dates), `garde` (validation), `reqwest` (HTTP client), and `axum` + `tokio` if running a standalone API server rather than Dioxus server functions.

### 17.2 Critical Gaps (no direct Rust/Dioxus equivalent)

| Feature | Current | Dioxus Solution | Risk |
|---------|---------|----------------|------|
| **AG-Grid** | 35MB community grid | Custom `<DataGrid>` component | **HIGH** — most complex UI component |
| **Chart.js** | Canvas charts | `plotters` crate or embed JS | MEDIUM — plotters works in WASM |
| **jsPDF** | PDF export | Server-side PDF (printpdf) or CSS `@media print` | MEDIUM |
| **html2canvas** | Screenshot export | Use browser `html2canvas` JS via eval | LOW |
| **lucide-react** | Tree-shakeable SVG | Pick icon set, bundle as data | LOW |
| **react-hot-toast** | Toast notifications | Simple signal-based SVG toast | LOW |
| **@dnd-kit** | Drag & drop | HTML5 drag events or pointer events | LOW |
| **date-fns** | Date formatting | `chrono` crate formatting | LOW |
| **TanStack Query** | Cache + refetch | Manual `use_resource` + signal-based cache | MEDIUM |
| **zod** | Schema validation | `garde` crate or manual | LOW |
| **react-router-dom** | SPA routing | Dioxus Router (built-in) | LOW |
| **axios** | HTTP client | `reqwest` or server functions | LOW |

### 17.3 AG-Grid Replacement Strategy

**Phase 1:** Basic table with columns, sort, pagination
**Phase 2:** Column resize, filter, row selection
**Phase 3:** Inline editing, cell renderers, cell class rules
**Phase 4:** Virtual scrolling, column reorder, row grouping

Build a generic `DataGrid<T>` from scratch rather than finding a third-party lib — the feature set is specific enough that any prebuilt solution will need significant customization.

### 17.4 Chart Replacement (plotters)

Charts used:
- Dashboard sales chart (line)
- Stock by category (bar)
- Expense summary (pie)
- AR aging (bar)
- Forecast trends (line with confidence band)
- KPI gauge (semi-circle)

`plotters` works in WASM via canvas. Each chart type becomes a component.

### 17.5 PDF Generation

Two options:
1. **CSS `@media print`:** Simplest for A4 invoice/PO/quote templates. Dioxus renders a hidden element, user presses Ctrl+P → native print dialog → "Save as PDF".
2. **Server-side PDF:** Use `printpdf` or `wkhtmltopdf` sidecar for programmatic generation.

### 17.6 WASM Size Considerations

Current React SPA: ~2MB gzipped
Dioxus WASM target: Keep under 3MB by:
- Using `wee_alloc` for smaller allocator
- Only importing needed crates
- Feature-flagging chart libraries for desktop-only
- Code splitting via Dioxus lazy components

---

## 18. Migration Phases

### Phase 0: Foundation (2 weeks)
- [ ] Set up Dioxus project (web target first)
- [ ] Port CSS design tokens and global styles
- [ ] Create basic layout (Sidebar + TopBar + ContentArea)
- [ ] Port Dioxus Router with all route definitions
- [ ] Set up i18n framework (en + ur)
- [ ] Create common components (Button, FormInput, Modal, Toast, PageLoader)
- [ ] Set up rusqlite database connection and migration system
- [ ] Port auth system (JWT, bcrypt, login page)
- [ ] Port RBAC middleware

### Phase 1: Core Data (4 weeks)
- [ ] Port ALL database models (~56 tables → rusqlite structs + CRUD)
- [ ] Port ALL pure calculation functions (~40 functions)
- [ ] Build DataGrid component (basic version)
- [ ] Build CompactCard system (generic + first 5 variants)
- [ ] Port Inventory module (items, warehouses, stock movements, stock balances)
- [ ] Port Customer/Supplier modules
- [ ] Port User/Role management

### Phase 2: Transactions (6 weeks)
- [ ] Port Invoice module (full create/update/cancel/return with FIFO + GL)
- [ ] Port Sales Order + Quotation modules (with convert chain)
- [ ] Port POS module
- [ ] Port Purchase Order module (with goods receipt + returns)
- [ ] Port Direct Purchase module
- [ ] Port BOM + Production modules
- [ ] Port Employee module (with salary payment)

### Phase 3: Finance & Reports (4 weeks)
- [ ] Port Accounting module (chart of accounts, journal lines, periods)
- [ ] Port ALL 25 standard reports (SQL queries)
- [ ] Port Custom Report Builder (dynamic SQL engine)
- [ ] Port Dashboard (16 block types + custom layout)
- [ ] Port Activity Log viewer
- [ ] Port Settings + Integrations pages

### Phase 4: Intelligence (3 weeks)
- [ ] Port Forecasting Engine (5 models + accuracy + seasonal events)
- [ ] Port CSV/PDF export
- [ ] WebMCP integration (AI assistant tools)
- [ ] PWA / service worker setup

### Phase 5: Cross-Platform (3 weeks)
- [ ] Test and polish Desktop target (Dioxus Desktop)
- [ ] Test and polish Mobile target (Dioxus Mobile)
- [ ] Responsive testing (all breakpoints)
- [ ] Print template testing (A4, thermal)
- [ ] Performance profiling and WASM size optimization
- [ ] Security audit (pen test)

### Phase 6: Polish & Ship (2 weeks)
- [ ] TypeScript-level type safety (Rust compiler guarantees this already)
- [ ] Keyboard shortcuts
- [ ] RTL Urdu testing
- [ ] Package for Windows/Mac (desktop native)
- [ ] Package for iOS/Android (mobile native)
- [ ] Documentation

**Total estimated effort: 24 weeks (6 months) for a team of 2-3 Rust developers.**

---

## 19. Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| **AG-Grid replacement scope** | HIGH | Build DataGrid iteratively; launch Phase 1 with basic table, enhance later |
| **No production-grade WASM chart lib** | MEDIUM | Use plotters for basic charts; embed Chart.js via wasm-bindgen as fallback |
| **Rust learning curve for team** | HIGH | Pair JS devs with Rust mentor; start with backend/model layer (simple SQL) |
| **Dioxus ecosystem immaturity** | MEDIUM | Dioxus 0.6 is stable; pin version, avoid nightly features |
| **Mobile testing complexity** | MEDIUM | Start with web target (works everywhere); add mobile after web is solid |
| **WASM size bloat** | LOW | Monitor bundle size per module; use `twiggy` for analysis; lazy-load heavy components |
| **Electron → Dioxus Desktop differences** | LOW | Both use WebView under the hood; Dioxus Desktop uses wry (same as Tauri) |
| **Data migration from existing app** | MEDIUM | Same SQLite schema; just copy the `.db` file — zero migration needed |
| **Print PDF quality** | LOW | CSS `@media print` is well-supported; server-side printpdf for programmatic use |
| **Custom report engine complexity** | MEDIUM | Already isolated in reportQueryEngine.ts; straightforward SQL builder port |

---

## Appendix A: File Mapping (Current → Dioxus)

| Current File | Dioxus Equivalent | Effort |
|-------------|-------------------|--------|
| `client/src/App.tsx` | `src/main.rs` (router + providers) | 1 day |
| `client/src/components/common/*` | `src/components/common/*.rs` | 2 weeks |
| `client/src/components/layout/*` | `src/components/layout/*.rs` | 1 week |
| `client/src/components/invoice/*` | `src/components/invoice/*.rs` | 2 weeks |
| `client/src/components/dashboard/*` | `src/components/dashboard/*.rs` | 2 weeks |
| `client/src/components/customer/*` | `src/components/customer/*.rs` | 1 week |
| `client/src/components/bom/*` | `src/components/manufacturing/*.rs` | 3 days |
| `client/src/components/production/*` | `src/components/manufacturing/*.rs` | 3 days |
| `client/src/components/inventory/*` | `src/components/inventory/*.rs` | 1 week |
| `client/src/components/purchases/*` | `src/components/purchasing/*.rs` | 1 week |
| `client/src/components/expenses/*` | `src/components/expenses/*.rs` | 2 days |
| `client/src/components/integration/*` | `src/components/settings/*.rs` | 1 day |
| `client/src/components/ErrorBoundary.tsx` | Inline in `main.rs` | 0.5 day |
| `client/src/pages/*` | `src/pages/*.rs` | 3 weeks |
| `client/src/types/index.ts` | `src/types.rs` | 1 week |
| `client/src/hooks/*` | Inline or `src/hooks.rs` | 1 week |
| `client/src/context/*` | `src/context.rs` | 2 days |
| `client/src/utils/*` | `src/calculations/*.rs`, `src/utils.rs` | 2 weeks |
| `client/src/styles/*.css` | `src/styles/*.css` (direct include) | 1 week |
| `client/src/locales/*.json` | `src/i18n/*.rs` (embedded) | 3 days |
| `client/src/schemas/*` | `src/validation.rs` (garde) | 1 week |
| `client/src/main.tsx` | `src/main.rs` (entry) | 1 day |
| `server/src/routes/*` | `src/server/routes/*.rs` | 3 weeks |
| `server/src/controllers/*` | `src/server/handlers/*.rs` | 3 weeks |
| `server/src/services/*` | `src/server/services/*.rs` | 4 weeks |
| `server/src/models/*` | `src/server/models/*.rs` | 3 weeks |
| `server/src/middleware/*` | `src/server/middleware/*.rs` | 1 week |
| `server/src/config/*` | `src/server/config.rs` | 2 days |
| `server/src/migrations/*` | `src/server/migrations.rs` | 1 week |
| `server/src/types/*` | `src/server/types.rs` | 3 days |
| `server/src/utils/*` | `src/server/utils.rs` | 3 days |
| `server/src/app.ts` | `src/main.rs` (server bootstrap) | 2 days |
| `electron/main.js` | Dioxus Desktop (replaces entirely) | 1 week |
| `electron/preload.js` | Not needed (Dioxus has native IPC) | N/A |
| Root `package.json` | `Cargo.toml` | 1 day |

## Appendix B: Dioxus Cargo.toml Template

```toml
[package]
name = "mini-erp"
version = "1.0.0"
edition = "2021"

[dependencies]
# Core
dioxus = { version = "0.6", features = ["web", "desktop", "mobile", "router", "fermi"] }
dioxus-storage = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }

# Backend (when running as server)
rusqlite = { version = "0.32", features = ["bundled"] }
jsonwebtoken = "9"
bcrypt = "0.16"
uuid = { version = "1", features = ["v4"] }
tracing = "0.1"
tracing-subscriber = "0.3"

# HTTP
reqwest = { version = "0.12", features = ["json"] }

# Validation
garde = "0.21"

# Utilities
csv = "1.3"
rand = "0.8"

# Optional: Charts
plotters = { version = "0.3", optional = true }

# Desktop/mobile targets (enabled by feature flags)
[target.'cfg(target_os = "windows")'.dependencies]
dioxus-desktop = "0.6"

[target.'cfg(target_os = "macos")'.dependencies]
dioxus-desktop = "0.6"

[target.'cfg(target_os = "ios")'.dependencies]
dioxus-mobile = "0.6"

[target.'cfg(target_os = "android")'.dependencies]
dioxus-mobile = "0.6"

[features]
default = ["web"]
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
mobile = ["dioxus/mobile"]
charts = ["plotters"]

[profile.release]
opt-level = "z"     # Size optimization
lto = true
codegen-units = 1
strip = true
```

---

*End of PRD*
