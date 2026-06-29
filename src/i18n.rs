use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Ur,
}

impl Lang {
    pub fn code(&self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ur => "ur",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Ur => "اردو",
        }
    }

    pub fn is_rtl(&self) -> bool {
        matches!(self, Lang::Ur)
    }
}

pub struct Translations {
    pub common: HashMap<&'static str, &'static str>,
    pub nav: HashMap<&'static str, &'static str>,
    pub actions: HashMap<&'static str, &'static str>,
    pub status: HashMap<&'static str, &'static str>,
    pub fields: HashMap<&'static str, &'static str>,
    pub messages: HashMap<&'static str, &'static str>,
    pub modules: HashMap<&'static str, &'static str>,
}

fn en_translations() -> Translations {
    let mut common = HashMap::new();
    common.insert("loading", "Loading...");
    common.insert("save", "Save");
    common.insert("cancel", "Cancel");
    common.insert("delete", "Delete");
    common.insert("edit", "Edit");
    common.insert("create", "Create");
    common.insert("search", "Search");
    common.insert("filter", "Filter");
    common.insert("export", "Export");
    common.insert("import", "Import");
    common.insert("print", "Print");
    common.insert("back", "Back");
    common.insert("yes", "Yes");
    common.insert("no", "No");
    common.insert("ok", "OK");
    common.insert("close", "Close");
    common.insert("confirm", "Confirm");
    common.insert("reset", "Reset");
    common.insert("actions", "Actions");
    common.insert("details", "Details");
    common.insert("overview", "Overview");
    common.insert("settings", "Settings");
    common.insert("all", "All");
    common.insert("none", "None");
    common.insert("total", "Total");
    common.insert("subtotal", "Subtotal");
    common.insert("discount", "Discount");
    common.insert("tax", "Tax");
    common.insert("amount", "Amount");
    common.insert("quantity", "Quantity");
    common.insert("price", "Price");
    common.insert("date", "Date");
    common.insert("status", "Status");
    common.insert("description", "Description");
    common.insert("name", "Name");
    common.insert("code", "Code");
    common.insert("email", "Email");
    common.insert("phone", "Phone");
    common.insert("address", "Address");
    common.insert("notes", "Notes");
    common.insert("balance", "Balance");
    common.insert("paid", "Paid");
    common.insert("due", "Due");
    common.insert("from", "From");
    common.insert("to", "To");

    let mut nav = HashMap::new();
    nav.insert("dashboard", "Dashboard");
    nav.insert("inventory", "Inventory");
    nav.insert("items", "Items");
    nav.insert("warehouses", "Warehouses");
    nav.insert("stock_movements", "Stock Movements");
    nav.insert("physical_counts", "Physical Counts");
    nav.insert("sales", "Sales");
    nav.insert("invoices", "Invoices");
    nav.insert("quotations", "Quotations");
    nav.insert("sales_orders", "Sales Orders");
    nav.insert("sales_returns", "Sales Returns");
    nav.insert("pos", "POS Terminal");
    nav.insert("purchases", "Purchases");
    nav.insert("direct_purchases", "Direct Purchases");
    nav.insert("purchase_orders", "Purchase Orders");
    nav.insert("goods_receipts", "Goods Receipts");
    nav.insert("purchase_returns", "Purchase Returns");
    nav.insert("manufacturing", "Manufacturing");
    nav.insert("bom", "Bill of Materials");
    nav.insert("production", "Production");
    nav.insert("customers", "Customers");
    nav.insert("suppliers", "Suppliers");
    nav.insert("employees", "Employees");
    nav.insert("expenses", "Expenses");
    nav.insert("accounting", "Accounting");
    nav.insert("chart_of_accounts", "Chart of Accounts");
    nav.insert("periods", "Accounting Periods");
    nav.insert("reports", "Reports");
    nav.insert("forecasts", "Forecasts");
    nav.insert("users", "Users");
    nav.insert("roles", "Roles");
    nav.insert("activity_log", "Activity Log");
    nav.insert("integrations", "Integrations");

    let mut actions = HashMap::new();
    actions.insert("add_new", "Add New");
    actions.insert("view_all", "View All");
    actions.insert("view_detail", "View Detail");
    actions.insert("create_new", "Create New");
    actions.insert("save_changes", "Save Changes");
    actions.insert("confirm_delete", "Confirm Delete");
    actions.insert("are_you_sure", "Are you sure?");
    actions.insert("cannot_be_undone", "This action cannot be undone.");
    actions.insert("no_items_found", "No items found");
    actions.insert("try_adjusting", "Try adjusting your filters or search.");
    actions.insert("page_loading", "Loading page data...");
    actions.insert("submit", "Submit");
    actions.insert("apply", "Apply");
    actions.insert("clear_filters", "Clear Filters");
    actions.insert("bulk_actions", "Bulk Actions");
    actions.insert("select_all", "Select All");
    actions.insert("deselect_all", "Deselect All");
    actions.insert("refresh", "Refresh");
    actions.insert("download", "Download");
    actions.insert("upload", "Upload");

    let mut status = HashMap::new();
    status.insert("active", "Active");
    status.insert("inactive", "Inactive");
    status.insert("pending", "Pending");
    status.insert("approved", "Approved");
    status.insert("completed", "Completed");
    status.insert("cancelled", "Cancelled");
    status.insert("paid", "Paid");
    status.insert("unpaid", "Unpaid");
    status.insert("partial", "Partially Paid");
    status.insert("overdue", "Overdue");
    status.insert("draft", "Draft");
    status.insert("received", "Received");
    status.insert("returned", "Returned");
    status.insert("delivered", "Delivered");
    status.insert("processing", "Processing");

    let mut fields = HashMap::new();
    fields.insert("item_code", "Item Code");
    fields.insert("item_name", "Item Name");
    fields.insert("category", "Category");
    fields.insert("unit_of_measure", "Unit of Measure");
    fields.insert("current_stock", "Current Stock");
    fields.insert("reorder_level", "Reorder Level");
    fields.insert("standard_cost", "Standard Cost");
    fields.insert("selling_price", "Selling Price");
    fields.insert("customer_name", "Customer Name");
    fields.insert("customer_code", "Customer Code");
    fields.insert("supplier_name", "Supplier Name");
    fields.insert("supplier_code", "Supplier Code");
    fields.insert("invoice_no", "Invoice No");
    fields.insert("quotation_no", "Quotation No");
    fields.insert("po_no", "PO No");
    fields.insert("so_no", "SO No");
    fields.insert("invoice_date", "Invoice Date");
    fields.insert("due_date", "Due Date");
    fields.insert("payment_method", "Payment Method");
    fields.insert("credit_limit", "Credit Limit");
    fields.insert("payment_terms", "Payment Terms");
    fields.insert("warehouse", "Warehouse");
    fields.insert("employee_code", "Employee Code");
    fields.insert("department", "Department");
    fields.insert("designation", "Designation");
    fields.insert("salary", "Salary");
    fields.insert("expense_category", "Expense Category");
    fields.insert("bom_name", "BOM Name");
    fields.insert("finished_item", "Finished Item");
    fields.insert("planned_quantity", "Planned Quantity");
    fields.insert("produced_quantity", "Produced Quantity");

    let mut messages = HashMap::new();
    messages.insert("saved_successfully", "Saved successfully");
    messages.insert("deleted_successfully", "Deleted successfully");
    messages.insert("created_successfully", "Created successfully");
    messages.insert("updated_successfully", "Updated successfully");
    messages.insert("error_occurred", "An error occurred");
    messages.insert("operation_failed", "Operation failed");
    messages.insert("access_denied", "Access denied");
    messages.insert("session_expired", "Session expired");
    messages.insert("login_required", "Please log in to continue");
    messages.insert("welcome_back", "Welcome back");
    messages.insert("goodbye", "Goodbye");

    let mut modules = HashMap::new();
    modules.insert("inventory_management", "Inventory Management");
    modules.insert("sales_crm", "Sales & CRM");
    modules.insert("purchasing", "Purchasing");
    modules.insert("manufacturing", "Manufacturing");
    modules.insert("finance_accounting", "Finance & Accounting");
    modules.insert("hr_management", "HR Management");
    modules.insert("expenses", "Expenses");
    modules.insert("reports_analytics", "Reports & Analytics");
    modules.insert("forecasts", "Forecasts");
    modules.insert("admin", "Administration");

    Translations { common, nav, actions, status, fields, messages, modules }
}

fn ur_translations() -> Translations {
    let mut common = HashMap::new();
    common.insert("loading", "...لوڈ ہو رہا ہے");
    common.insert("save", "محفوظ کریں");
    common.insert("cancel", "منسوخ کریں");
    common.insert("delete", "حذف کریں");
    common.insert("edit", "ترمیم کریں");
    common.insert("create", "بنائیں");
    common.insert("search", "تلاش کریں");
    common.insert("filter", "فلٹر");
    common.insert("export", "برآمد");
    common.insert("import", "درآمد");
    common.insert("print", "پرنٹ");
    common.insert("back", "واپس");
    common.insert("yes", "ہاں");
    common.insert("no", "نہیں");
    common.insert("ok", "ٹھیک ہے");
    common.insert("close", "بند کریں");
    common.insert("confirm", "تصدیق کریں");
    common.insert("reset", "ری سیٹ");
    common.insert("actions", "اقدامات");
    common.insert("details", "تفصیلات");
    common.insert("overview", "جائزہ");
    common.insert("settings", "ترتیبات");
    common.insert("all", "سب");
    common.insert("none", "کوئی نہیں");
    common.insert("total", "کل");
    common.insert("subtotal", "ذیلی کل");
    common.insert("discount", "رعایت");
    common.insert("tax", "ٹیکس");
    common.insert("amount", "رقم");
    common.insert("quantity", "مقدار");
    common.insert("price", "قیمت");
    common.insert("date", "تاریخ");
    common.insert("status", "حالت");
    common.insert("description", "تفصیل");
    common.insert("name", "نام");
    common.insert("code", "کوڈ");
    common.insert("email", "ای میل");
    common.insert("phone", "فون");
    common.insert("address", "پتہ");
    common.insert("notes", "نوٹس");
    common.insert("balance", "بیلنس");
    common.insert("paid", "ادا شدہ");
    common.insert("due", "واجب الادا");
    common.insert("from", "سے");
    common.insert("to", "تک");

    let mut nav = HashMap::new();
    nav.insert("dashboard", "ڈیش بورڈ");
    nav.insert("inventory", "انوینٹری");
    nav.insert("items", "اشیاء");
    nav.insert("warehouses", "گودام");
    nav.insert("stock_movements", "اسٹاک movements");
    nav.insert("physical_counts", "فیزیکل کاؤنٹ");
    nav.insert("sales", "فروخت");
    nav.insert("invoices", "انوائسز");
    nav.insert("quotations", "کوٹیشنز");
    nav.insert("sales_orders", "سیلز آرڈرز");
    nav.insert("sales_returns", "سیلز ریٹرنز");
    nav.insert("pos", "POS ٹرمینل");
    nav.insert("purchases", "خریداری");
    nav.insert("direct_purchases", "براہ راست خریداری");
    nav.insert("purchase_orders", "پرچیز آرڈرز");
    nav.insert("goods_receipts", "مال وصولی");
    nav.insert("purchase_returns", "خریداری ریٹرنز");
    nav.insert("manufacturing", "manufacturing");
    nav.insert("bom", "بِل آف مٹیریلز");
    nav.insert("production", "پیداوار");
    nav.insert("customers", "صارفین");
    nav.insert("suppliers", "سپلائرز");
    nav.insert("employees", "ملازمین");
    nav.insert("expenses", "اخراجات");
    nav.insert("accounting", "اکاؤنٹنگ");
    nav.insert("chart_of_accounts", "اکاؤنٹس چارٹ");
    nav.insert("periods", "اکاؤنٹنگ پیریڈز");
    nav.insert("reports", "رپورٹس");
    nav.insert("forecasts", "پیش گوئیاں");
    nav.insert("users", "صارفین");
    nav.insert("roles", "کردار");
    nav.insert("activity_log", "سرگرمی لاگ");
    nav.insert("integrations", "انٹیگریشنز");

    let mut actions = HashMap::new();
    actions.insert("add_new", "نیا شامل کریں");
    actions.insert("view_all", "سب دیکھیں");
    actions.insert("view_detail", "تفصیل دیکھیں");
    actions.insert("create_new", "نیا بنائیں");
    actions.insert("save_changes", "تبدیلیاں محفوظ کریں");
    actions.insert("confirm_delete", "حذف کی تصدیق کریں");
    actions.insert("are_you_sure", "کیا آپ کو یقین ہے؟");
    actions.insert("cannot_be_undone", "یہ عمل واپس نہیں ہو سکتا۔");
    actions.insert("no_items_found", "کوئی آئٹم نہیں ملا");
    actions.insert("try_adjusting", "اپنے فلٹرز یا تلاش کو ترمیم کریں۔");
    actions.insert("page_loading", "...صفحہ لوڈ ہو رہا ہے");
    actions.insert("submit", "جمع کریں");
    actions.insert("apply", "لاگو کریں");
    actions.insert("clear_filters", "فلٹرز صاف کریں");
    actions.insert("bulk_actions", "جملہ اقدامات");
    actions.insert("select_all", "سب منتخب کریں");
    actions.insert("deselect_all", "انتخاب صاف کریں");
    actions.insert("refresh", "ریفریش");
    actions.insert("download", "ڈاؤن لوڈ");
    actions.insert("upload", "اپ لوڈ");

    let mut status = HashMap::new();
    status.insert("active", "فعال");
    status.insert("inactive", "غیر فعال");
    status.insert("pending", "زیر التوا");
    status.insert("approved", "منظور شدہ");
    status.insert("completed", "مکمل");
    status.insert("cancelled", "منسوخ");
    status.insert("paid", "ادا شدہ");
    status.insert("unpaid", "غیر ادا شدہ");
    status.insert("partial", "جزوی ادا شدہ");
    status.insert("overdue", "واجب الادا");
    status.insert("draft", "مسودہ");
    status.insert("received", "وصول شدہ");
    status.insert("returned", "واپس شدہ");
    status.insert("delivered", "ڈلیور شدہ");
    status.insert("processing", "عمل جاری ہے");

    let mut fields = HashMap::new();
    fields.insert("item_code", "آئٹم کوڈ");
    fields.insert("item_name", "آئٹم کا نام");
    fields.insert("category", "زمرہ");
    fields.insert("unit_of_measure", "پیمانہ");
    fields.insert("current_stock", "موجودہ اسٹاک");
    fields.insert("reorder_level", "ری آرڈر لیول");
    fields.insert("standard_cost", "معیاری لاگت");
    fields.insert("selling_price", "فروخت قیمت");
    fields.insert("customer_name", "صارف کا نام");
    fields.insert("customer_code", "صارف کوڈ");
    fields.insert("supplier_name", "سپلائر کا نام");
    fields.insert("supplier_code", "سپلائر کوڈ");
    fields.insert("invoice_no", "انوائس نمبر");
    fields.insert("quotation_no", "کوٹیشن نمبر");
    fields.insert("po_no", "پرچیز آرڈر نمبر");
    fields.insert("so_no", "سیلز آرڈر نمبر");
    fields.insert("invoice_date", "انوائس کی تاریخ");
    fields.insert("due_date", "واجب الادا تاریخ");
    fields.insert("payment_method", "ادائیگی کا طریقہ");
    fields.insert("credit_limit", "کریڈٹ حد");
    fields.insert("payment_terms", "ادائیگی کی شرائط");
    fields.insert("warehouse", "گودام");
    fields.insert("employee_code", "ملازم کوڈ");
    fields.insert("department", "محکمہ");
    fields.insert("designation", "عہدہ");
    fields.insert("salary", "تنخواہ");
    fields.insert("expense_category", "اخراجات کا زمرہ");
    fields.insert("bom_name", "BOM کا نام");
    fields.insert("finished_item", "تیار شدہ آئٹم");
    fields.insert("planned_quantity", "منصوبہ مقدار");
    fields.insert("produced_quantity", "پیدا شدہ مقدار");

    let mut messages = HashMap::new();
    messages.insert("saved_successfully", "کامیابی سے محفوظ ہو گیا");
    messages.insert("deleted_successfully", "کامیابی سے حذف ہو گیا");
    messages.insert("created_successfully", "کامیابی سے بن گیا");
    messages.insert("updated_successfully", "کامیابی سے اپ ڈیٹ ہو گیا");
    messages.insert("error_occurred", "خرابی ہوئی");
    messages.insert("operation_failed", "آپریشن ناکام");
    messages.insert("access_denied", "رسائی منع ہے");
    messages.insert("session_expired", "سیشن ختم ہو گیا");
    messages.insert("login_required", "براہ کرم لاگ ان کریں");
    messages.insert("welcome_back", "خوش آمدید");
    messages.insert("goodbye", "خدا حافظ");

    let mut modules = HashMap::new();
    modules.insert("inventory_management", "انوینٹری مینجمنٹ");
    modules.insert("sales_crm", "فروخت اور CRM");
    modules.insert("purchasing", "خریداری");
    modules.insert("manufacturing", "manufacturing");
    modules.insert("finance_accounting", "مالیات و اکاؤنٹنگ");
    modules.insert("hr_management", "HR مینجمنٹ");
    modules.insert("expenses", "اخراجات");
    modules.insert("reports_analytics", "رپورٹس و تجزیات");
    modules.insert("forecasts", "پیش گوئیاں");
    modules.insert("admin", "انتظامیہ");

    Translations { common, nav, actions, status, fields, messages, modules }
}

pub fn get_translations(lang: Lang) -> Translations {
    match lang {
        Lang::En => en_translations(),
        Lang::Ur => ur_translations(),
    }
}

pub fn use_i18n() -> Signal<Lang> {
    use_context::<Signal<Lang>>()
}

pub fn t(category: &str, key: &str, lang: Lang) -> String {
    let translations = get_translations(lang);
    let map = match category {
        "common" => &translations.common,
        "nav" => &translations.nav,
        "actions" => &translations.actions,
        "status" => &translations.status,
        "fields" => &translations.fields,
        "messages" => &translations.messages,
        "modules" => &translations.modules,
        _ => return key.to_string(),
    };
    map.get(key).unwrap_or(&key).to_string()
}

#[component]
pub fn LanguageToggle() -> Element {
    let mut lang = use_i18n();
    let current = *lang.read();

    rsx! {
        button {
            style: "background: none; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 4px 10px; cursor: pointer; font-size: 12px; color: var(--text-secondary, #6c757d); display: flex; align-items: center; gap: 4px;",
            onclick: move |_| {
                let cur = *lang.read();
                let new_lang = match cur {
                    Lang::En => Lang::Ur,
                    Lang::Ur => Lang::En,
                };
                lang.set(new_lang);
            },
            span { style: "font-size: 14px;", "🌐" }
            span { "{current.name()}" }
        }
    }
}
