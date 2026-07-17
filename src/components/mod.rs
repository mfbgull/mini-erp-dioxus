//! Reusable UI components for MiniERP.
//!
//! Each sub-module encapsulates a self-contained component or component family.
//! Components are organized by function then by module.
//!
//! # Current Modules
//!
//! | Module | Description | Status |
//! |--------|-------------|--------|
//! | `data_grid` | Generic DataGrid<T> table component (AG-Grid replacement) | Phase 1 ✅ |
//! | `layout` | Sidebar, TopMenu, ContentArea shell | Phase 0 ⬜ |
//! | `common` | Button, FormInput, Modal, Toast, PageLoader | Phase 0 ⬜ |
//! | `invoice` | InvoiceForm, InvoiceItemsTable, Print templates | Phase 2 ⬜ |
//! | `dashboard` | DashboardLayout, DashboardBlock, chart wrappers | Phase 3 ⬜ |
//! | `inventory` | ItemList, WarehouseCard, StockMovement | Phase 1 ⬜ |
//! | `customer` | CustomerDetail, LedgerView, PaymentList | Phase 1 ⬜ |
//! | `purchasing` | POForm, ReceiptList, SupplierCard | Phase 2 ⬜ |
//! | `manufacturing` | BOMList, ProductionForm, BOMDetail | Phase 2 ⬜ |

// Phase 0 — Core UI component library
pub mod common;

// Phase 1 — DataGrid core component
pub mod data_grid;

// Charts module
pub mod charts;

// RBAC permission gating
pub mod rbac;

// Keyboard shortcuts
pub mod shortcuts;

// Future modules (unlocked as migration progresses)
pub mod layout;
pub mod invoice;
pub mod dashboard;
pub mod customer;
pub mod inventory;
