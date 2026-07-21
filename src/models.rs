//! Shared data types for MiniERP.
//!
//! These types are compiled for both the frontend (WASM) and server (native)
//! binaries. They use only `serde` for serialization — no platform-specific deps.

use serde::{Deserialize, Serialize};

// ============================================================================
// Auth Types
// ============================================================================

/// Login request payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserProfile,
    pub token: String,
}

/// Logout request (no body needed, token is in cookie/header).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub message: String,
}

/// Authenticated user profile returned by `/api/auth/me`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    pub id: i64,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub role_id: Option<i64>,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub last_login: Option<String>,
}

/// JWT claims payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User ID
    pub sub: i64,
    /// Username
    pub username: String,
    /// User role name
    pub role: String,
    /// Expiration timestamp (epoch seconds)
    pub exp: usize,
    /// Issued at timestamp
    pub iat: usize,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
}

// ============================================================================
// Permission Types
// ============================================================================

/// A permission entry from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: i64,
    pub permission_name: String,
    pub module: String,
    pub action: String,
    pub description: String,
}

/// A role with its permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i64,
    pub role_name: String,
    pub description: String,
    pub is_system_role: bool,
    pub is_active: bool,
    #[serde(default)]
    pub user_count: i64,
    #[serde(default)]
    pub permissions: Vec<Permission>,
}

// ============================================================================
// API Response Wrappers
// ============================================================================

/// Standard API response with a data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Paginated list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

// ============================================================================
// User Management Types
// ============================================================================

/// User record from the database (server-side, not sent to client).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub password_hash: String,
    pub full_name: String,
    pub role: String,
    pub role_id: Option<i64>,
    pub is_active: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_login: Option<String>,
}

/// User create/update payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserForm {
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub password: Option<String>,
    pub role_id: i64,
    pub is_active: bool,
}

/// Password change request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// ============================================================================
// Settings Type
// ============================================================================

/// Key-value setting from the `settings` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub description: String,
}

// ============================================================================
// Inventory Types
// ============================================================================

/// Item record from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub item_code: String,
    pub item_name: String,
    pub description: String,
    pub category: String,
    pub unit_of_measure: String,
    pub current_stock: f64,
    pub reorder_level: f64,
    pub standard_cost: f64,
    pub selling_price: f64,
    pub is_raw_material: bool,
    pub is_finished_good: bool,
    pub is_purchased: bool,
    pub is_manufactured: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Item create/update payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemForm {
    pub item_code: String,
    pub item_name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit_of_measure: Option<String>,
    pub reorder_level: Option<f64>,
    pub standard_cost: Option<f64>,
    pub selling_price: Option<f64>,
    pub is_raw_material: Option<bool>,
    pub is_finished_good: Option<bool>,
    pub is_purchased: Option<bool>,
    pub is_manufactured: Option<bool>,
}

/// Warehouse record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: i64,
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub location: String,
    pub capacity: f64,
    pub is_active: bool,
    pub created_at: String,
}

/// Warehouse create/update payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseForm {
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub location: Option<String>,
}

/// Stock movement record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovement {
    pub id: i64,
    pub movement_no: String,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub warehouse_id: i64,
    pub warehouse_name: Option<String>,
    pub movement_type: String,
    pub quantity: f64,
    pub unit_cost: f64,
    pub reference_doctype: Option<String>,
    pub reference_docno: Option<String>,
    pub batch_id: Option<i64>,
    pub notes: String,
    pub created_by: Option<i64>,
    pub created_at: String,
}

/// Stock movement create payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovementForm {
    pub item_id: i64,
    pub warehouse_id: i64,
    pub movement_type: String,
    pub quantity: f64,
    pub unit_cost: Option<f64>,
    pub reference_doctype: Option<String>,
    pub reference_docno: Option<String>,
    pub notes: Option<String>,
}

/// Stock balance record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBalance {
    pub id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub category: Option<String>,
    pub unit_of_measure: Option<String>,
    pub warehouse_id: i64,
    pub warehouse_name: Option<String>,
    pub quantity: f64,
}

/// Physical count record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalCount {
    pub id: i64,
    pub count_no: String,
    pub count_date: String,
    pub warehouse_id: i64,
    pub warehouse_name: Option<String>,
    pub status: String,
    pub notes: String,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Physical count item record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalCountItem {
    pub id: i64,
    pub count_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub system_quantity: f64,
    pub counted_quantity: Option<f64>,
    pub variance: Option<f64>,
    #[serde(default)]
    pub unit_cost: Option<f64>,
}

/// Physical count create payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalCountForm {
    pub warehouse_id: i64,
    pub notes: Option<String>,
}

/// Physical count item record payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalCountItemForm {
    pub item_id: i64,
    pub counted_quantity: f64,
}

// ============================================================================
// Customer Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: i64,
    pub customer_code: String,
    pub customer_name: String,
    pub email: String,
    pub phone: String,
    pub billing_address: String,
    pub shipping_address: String,
    pub payment_terms: String,
    pub credit_limit: f64,
    pub credit_balance: f64,
    pub current_balance: f64,
    pub opening_balance: f64,
    pub is_active: bool,
    pub customer_type: String,
    pub notes: String,
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub last_invoice_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerForm {
    pub customer_code: String,
    pub customer_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub billing_address: Option<String>,
    pub shipping_address: Option<String>,
    pub payment_terms: Option<String>,
    pub credit_limit: Option<f64>,
    pub opening_balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerLedgerEntry {
    pub id: i64,
    pub customer_id: i64,
    pub transaction_date: String,
    pub transaction_type: String,
    pub reference_no: String,
    pub debit: f64,
    pub credit: f64,
    pub balance: f64,
}

// ============================================================================
// Invoice Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: i64,
    pub invoice_no: String,
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub customer_code: Option<String>,
    pub so_id: Option<i64>,
    pub quotation_id: Option<i64>,
    pub source_type: String,
    pub invoice_date: String,
    pub due_date: String,
    pub status: String,
    pub total_amount: f64,
    pub paid_amount: f64,
    pub balance_amount: f64,
    pub returned_amount: f64,
    pub discount_scope: Option<String>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub tax_rate: Option<f64>,
    pub notes: Option<String>,
    pub warehouse_id: Option<i64>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub item_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub id: i64,
    pub invoice_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub returned_qty: f64,
    pub unit_price: f64,
    pub amount: f64,
    pub tax_rate: f64,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItemForm {
    pub item_id: i64,
    pub description: Option<String>,
    pub quantity: f64,
    pub unit_price: f64,
    pub tax_rate: Option<f64>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceForm {
    pub customer_id: i64,
    pub invoice_date: String,
    pub due_date: Option<String>,
    pub source_type: Option<String>,
    pub warehouse_id: Option<i64>,
    pub discount_scope: Option<String>,
    pub discount_type: Option<String>,
    pub discount_value: Option<f64>,
    pub tax_rate: Option<f64>,
    pub notes: Option<String>,
    pub items: Vec<InvoiceItemForm>,
    pub record_payment: Option<bool>,
    pub payment_amount: Option<f64>,
    pub payment_method: Option<String>,
    #[serde(default)]
    pub deleted_payment_ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceReturnRequest {
    pub items: Vec<InvoiceReturnItem>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceReturnItem {
    pub item_id: i64,
    pub quantity: f64,
}

// ============================================================================
// Payment Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: i64,
    pub payment_no: String,
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub invoice_id: Option<i64>,
    pub payment_date: String,
    pub amount: f64,
    pub payment_method: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentForm {
    pub customer_id: i64,
    pub invoice_id: Option<i64>,
    pub payment_date: String,
    pub amount: f64,
    pub payment_method: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub allocations: Option<Vec<PaymentAllocationForm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAllocation {
    pub id: i64,
    pub payment_id: i64,
    pub invoice_id: i64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAllocationForm {
    pub invoice_id: i64,
    pub amount: f64,
}

// ============================================================================
// Sales Order Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrder {
    pub id: i64,
    pub so_no: String,
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub customer_code: Option<String>,
    pub so_date: String,
    pub status: String,
    pub delivery_date: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<i64>,
    pub total_amount: f64,
    pub warehouse_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub item_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrderItem {
    pub id: i64,
    pub so_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub delivered_quantity: f64,
    pub unit_price: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrderItemForm {
    pub item_id: i64,
    pub description: Option<String>,
    pub quantity: f64,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrderForm {
    pub customer_id: i64,
    pub so_date: String,
    pub warehouse_id: Option<i64>,
    pub notes: Option<String>,
    pub items: Vec<SalesOrderItemForm>,
}

// ============================================================================
// Quotation Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotation {
    pub id: i64,
    pub quotation_no: String,
    pub customer_id: i64,
    pub customer_name: Option<String>,
    pub quotation_date: String,
    pub expiry_date: String,
    pub status: String,
    pub total_amount: f64,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotationItem {
    pub id: i64,
    pub quotation_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount: f64,
    pub tax: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotationItemForm {
    pub item_id: i64,
    pub description: Option<String>,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount: Option<f64>,
    pub tax_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotationForm {
    pub customer_id: i64,
    pub quotation_date: String,
    pub expiry_date: String,
    pub notes: Option<String>,
    pub items: Vec<QuotationItemForm>,
}

// ============================================================================
// Supplier Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: i64,
    pub supplier_code: String,
    pub supplier_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierForm {
    pub supplier_code: String,
    pub supplier_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierLedgerEntry {
    pub id: i64,
    pub supplier_id: i64,
    pub transaction_date: String,
    pub transaction_type: String,
    pub reference_no: String,
    pub debit: f64,
    pub credit: f64,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierPaymentForm {
    pub payment_date: String,
    pub amount: f64,
    pub payment_method: Option<String>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

// ============================================================================
// Purchase Order Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: i64,
    pub po_no: String,
    pub supplier_id: i64,
    pub supplier_name: Option<String>,
    pub supplier_code: Option<String>,
    pub po_date: String,
    pub status: String,
    pub total_amount: f64,
    pub expected_date: Option<String>,
    pub warehouse_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub item_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItem {
    pub id: i64,
    pub po_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub received_quantity: f64,
    pub returned_quantity: f64,
    pub unit_price: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItemForm {
    pub item_id: i64,
    pub description: Option<String>,
    pub quantity: f64,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderForm {
    pub supplier_id: i64,
    pub po_date: String,
    pub warehouse_id: Option<i64>,
    pub notes: Option<String>,
    pub items: Vec<PurchaseOrderItemForm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderStatusUpdate {
    pub status: String,
}

// ============================================================================
// Goods Receipt Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceipt {
    pub id: i64,
    pub receipt_no: String,
    pub po_id: i64,
    pub receipt_date: String,
    pub warehouse_id: Option<i64>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptItem {
    pub id: i64,
    pub receipt_id: i64,
    pub po_item_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub received_quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptForm {
    pub po_id: i64,
    pub receipt_date: String,
    pub warehouse_id: Option<i64>,
    pub notes: Option<String>,
    pub items: Vec<GoodsReceiptItemForm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptItemForm {
    pub po_item_id: i64,
    pub item_id: i64,
    pub received_quantity: f64,
}

// ============================================================================
// Direct Purchase Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectPurchase {
    pub id: i64,
    pub purchase_no: String,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub warehouse_id: i64,
    pub warehouse_name: Option<String>,
    pub batch_id: Option<i64>,
    pub quantity: f64,
    pub unit_cost: f64,
    pub total_cost: f64,
    pub supplier_name: String,
    pub purchase_date: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectPurchaseForm {
    pub item_id: i64,
    pub warehouse_id: i64,
    pub quantity: f64,
    pub unit_cost: f64,
    pub supplier_name: String,
    pub purchase_date: String,
    pub notes: Option<String>,
}

// ============================================================================
// BOM Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bom {
    pub id: i64,
    pub bom_no: String,
    pub bom_name: String,
    pub finished_item_id: i64,
    pub finished_item_name: Option<String>,
    pub finished_item_code: Option<String>,
    pub quantity: f64,
    pub version: i64,
    pub total_cost: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomItem {
    pub id: i64,
    pub bom_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub quantity: f64,
    pub unit_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomItemForm {
    pub item_id: i64,
    pub quantity: f64,
    pub unit_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomForm {
    pub bom_name: String,
    pub finished_item_id: i64,
    pub quantity: f64,
    pub description: Option<String>,
    pub items: Vec<BomItemForm>,
}

// ============================================================================
// Production Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Production {
    pub id: i64,
    pub production_no: String,
    pub output_item_id: i64,
    pub output_item_name: Option<String>,
    pub output_item_code: Option<String>,
    pub output_quantity: f64,
    pub completed_qty: f64,
    pub end_date: Option<String>,
    pub warehouse_id: i64,
    pub warehouse_name: Option<String>,
    pub bom_id: Option<i64>,
    pub bom_name: Option<String>,
    pub overhead_cost: f64,
    pub batch_id: Option<i64>,
    pub unit_cost: f64,
    pub total_material_cost: f64,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionInput {
    pub id: i64,
    pub production_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub quantity: f64,
    pub warehouse_id: i64,
    pub unit_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionInputForm {
    pub item_id: i64,
    pub quantity: f64,
    pub warehouse_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionForm {
    pub output_item_id: i64,
    pub output_quantity: f64,
    pub warehouse_id: i64,
    pub bom_id: Option<i64>,
    pub overhead_cost: Option<f64>,
    pub notes: Option<String>,
    pub inputs: Vec<ProductionInputForm>,
}

// ============================================================================
// Employee Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: i64,
    pub employee_code: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub cnic_no: String,
    pub address: String,
    pub city: String,
    pub department: String,
    pub designation: String,
    pub employment_type: String,
    pub salary: f64,
    pub bank_name: String,
    pub bank_account_no: String,
    pub emergency_contact_name: String,
    pub emergency_contact_phone: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeForm {
    pub employee_code: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub cnic_no: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub salary: Option<f64>,
    pub bank_name: Option<String>,
    pub bank_account_no: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryPayment {
    pub id: i64,
    pub employee_id: i64,
    pub amount: f64,
    pub payment_date: String,
    pub journal_entry_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryPaymentForm {
    pub amount: f64,
    pub payment_date: String,
}

// ============================================================================
// Expense Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: i64,
    pub expense_no: String,
    pub category: String,
    pub description: String,
    pub amount: f64,
    pub expense_date: String,
    pub status: String,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseForm {
    pub category: String,
    pub description: String,
    pub amount: f64,
    pub expense_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseCategory {
    pub id: i64,
    pub category_name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseCategoryForm {
    pub category_name: String,
}

// ============================================================================
// Accounting Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOfAccount {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub normal_balance: String,
    pub parent_id: Option<i64>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub normal_balance: String,
    pub debit: f64,
    pub credit: f64,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i64,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub entry_date: String,
    pub created_by: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLine {
    pub id: i64,
    pub journal_entry_id: i64,
    pub account_id: i64,
    pub account_code: Option<String>,
    pub account_name: Option<String>,
    pub debit: f64,
    pub credit: f64,
    pub description: String,
    pub line_date: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub voided: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryForm {
    pub entry_date: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub lines: Vec<JournalLineForm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLineForm {
    pub account_id: i64,
    pub debit: f64,
    pub credit: f64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingPeriod {
    pub id: i64,
    pub period_name: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
}

// ============================================================================
// Activity Log Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub metadata: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

// ============================================================================
// Dashboard Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_items: i64,
    pub total_customers: i64,
    pub total_suppliers: i64,
    pub total_invoices: i64,
    pub total_revenue: f64,
    pub total_expenses: f64,
    pub outstanding_ar: f64,
    pub outstanding_ap: f64,
    pub low_stock_count: i64,
    pub stock_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopCustomer {
    pub customer_id: i64,
    pub customer_name: String,
    pub total_revenue: f64,
    pub invoice_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesSummary {
    pub today: f64,
    pub this_week: f64,
    pub this_month: f64,
    pub invoice_count_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseSummary {
    pub this_week: f64,
    pub this_month: f64,
    pub by_category: Vec<CategoryExpense>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryExpense {
    pub category: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArSummary {
    pub current: f64,
    pub days_1_30: f64,
    pub days_31_60: f64,
    pub days_61_90: f64,
    pub days_90_plus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub id: i64,
    pub user_id: i64,
    pub layout_name: String,
    pub blocks: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayoutForm {
    pub layout_name: String,
    pub blocks: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovementSummary {
    pub date: String,
    pub inbound: f64,
    pub outbound: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStatusSummary {
    pub in_progress: i64,
    pub completed: i64,
    pub total_output_quantity: f64,
}

// ============================================================================
// Tax & Payment Terms Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRate {
    pub id: i64,
    pub name: String,
    pub rate: f64,
    pub is_default: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTerm {
    pub id: i64,
    pub name: String,
    pub days: i64,
    pub is_default: bool,
    pub is_active: bool,
}

// ============================================================================
// Custom Report Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReport {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub config: String,
    pub is_active: bool,
    pub last_run_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomReportForm {
    pub name: String,
    pub config: String,
}

// ============================================================================
// Forecast Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandForecast {
    pub id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub item_code: Option<String>,
    pub forecast_date: String,
    pub period: String,
    pub predicted_quantity: f64,
    pub confidence_level: f64,
    pub trend_direction: String,
    pub model_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastRun {
    pub id: i64,
    pub run_id: String,
    pub run_type: String,
    pub status: String,
    pub items_processed: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastModelConfig {
    pub id: i64,
    pub item_id: Option<i64>,
    pub category: Option<String>,
    pub model_type: String,
    pub alpha: Option<f64>,
    pub beta: Option<f64>,
    pub gamma: Option<f64>,
    #[serde(skip)]
    pub params_json: Option<String>,
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastModelConfigForm {
    pub item_id: Option<i64>,
    pub category: Option<String>,
    pub model_type: String,
    pub alpha: Option<f64>,
    pub beta: Option<f64>,
    pub gamma: Option<f64>,
    pub params: Option<serde_json::Value>,
    pub model_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalEvent {
    pub id: i64,
    pub event_name: String,
    pub start_date: String,
    pub end_date: String,
    pub multiplier: f64,
    pub applies_to_category: Option<String>,
    pub applies_to_item_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalEventForm {
    pub event_name: String,
    pub start_date: String,
    pub end_date: String,
    pub multiplier: f64,
    pub applies_to_category: Option<String>,
    pub applies_to_item_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastAccuracy {
    pub id: i64,
    pub forecast_id: i64,
    pub item_id: i64,
    pub item_name: Option<String>,
    pub period: String,
    pub mape: f64,
    pub mae: f64,
    pub smape: f64,
}

// ============================================================================
// Integration Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub service: String,
    pub is_configured: bool,
    pub settings: std::collections::HashMap<String, String>,
}

// ============================================================================
// Role Management Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleForm {
    pub role_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissionUpdate {
    pub permission_ids: Vec<i64>,
}

// ============================================================================
// Settings Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsUpdate {
    pub settings: Vec<SettingUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingUpdate {
    pub key: String,
    pub value: String,
}
