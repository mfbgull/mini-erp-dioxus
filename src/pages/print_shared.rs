//! Shared print template CSS and utilities for Invoice, Quotation, and PO print pages.

/// Shared CSS for all print templates (A4 layout, typography, tables, totals, footer).
pub const PRINT_CSS: &str = r#"
@page { margin: 15mm; size: A4; }
@media print {
    @page { margin: 15mm; size: A4; }
    body { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
    .print-no-print { display: none !important; }
}
.print-page { max-width: 800px; margin: 0 auto; background: #fff; padding: 32px; font-family: 'Inter', 'Segoe UI', Arial, sans-serif; color: #1a1a1a; }
.print-page * { box-sizing: border-box; }
.print-header { display: flex; justify-content: space-between; align-items: flex-start; border-bottom: 2px solid #e0e0e0; padding-bottom: 20px; margin-bottom: 24px; }
.print-company { display: flex; flex-direction: column; gap: 2px; }
.print-company h1 { font-size: 24px; font-weight: 700; margin: 0; color: #1a1a1a; }
.print-company-subtitle { font-size: 12px; color: #6c757d; }
.print-title { font-size: 28px; font-weight: 700; margin: 0; }
.print-info-row { display: flex; justify-content: space-between; gap: 32px; margin-bottom: 24px; }
.print-info-block { flex: 1; }
.print-info-block h3 { font-size: 12px; font-weight: 600; color: #6c757d; text-transform: uppercase; letter-spacing: 0.5px; margin: 0 0 8px 0; }
.print-info-block p { margin: 2px 0; font-size: 13px; color: #1a1a1a; line-height: 1.5; }
.print-table { width: 100%; border-collapse: collapse; margin-bottom: 24px; }
.print-table thead th { background: #f5f5f5; padding: 8px 10px; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.3px; color: #6c757d; border-bottom: 2px solid #e0e0e0; text-align: left; }
.print-table thead th.text-right { text-align: right; }
.print-table tbody td { padding: 8px 10px; font-size: 13px; border-bottom: 1px solid #eee; color: #1a1a1a; }
.print-table tbody td.text-right { text-align: right; font-family: 'Courier New', monospace; }
.print-table tbody tr:nth-child(even) { background: #fafafa; }
.print-table tfoot td { padding: 8px 10px; font-size: 13px; font-weight: 600; border-top: 2px solid #e0e0e0; }
.print-totals { width: 300px; margin-left: auto; }
.print-totals table { width: 100%; }
.print-totals td { padding: 4px 10px; font-size: 13px; }
.print-totals td.text-right { text-align: right; font-family: 'Courier New', monospace; }
.print-totals .total-row td { padding-top: 8px; font-weight: 700; font-size: 16px; border-top: 2px solid #1a1a1a; }
.print-notes { background: #f9f9f9; padding: 12px 16px; border-radius: 4px; font-size: 12px; color: #6c757d; margin-bottom: 24px; }
.print-payment-info { background: #f0f7ff; border: 1px solid #b3d4fc; padding: 12px 16px; border-radius: 4px; font-size: 12px; margin-bottom: 24px; }
.print-payment-info h4 { margin: 0 0 8px 0; font-size: 13px; color: #1a1a1a; }
.print-payment-info p { margin: 2px 0; color: #333; }
.print-terms { background: #fffde7; border: 1px solid #ffe082; padding: 12px 16px; border-radius: 4px; font-size: 12px; color: #5d4037; margin-bottom: 24px; }
.print-terms h4 { margin: 0 0 8px 0; font-size: 13px; color: #5d4037; }
.print-terms p { margin: 2px 0; }
.print-signature { display: flex; justify-content: flex-end; margin-top: 48px; padding-top: 16px; }
.print-signature-box { width: 200px; text-align: center; }
.print-signature-line { border-top: 1px solid #1a1a1a; margin-top: 60px; padding-top: 6px; font-size: 12px; color: #6c757d; }
.print-footer { margin-top: 32px; padding-top: 16px; border-top: 1px solid #e0e0e0; font-size: 11px; color: #6c757d; text-align: center; }
@media (max-width: 768px) {
    .print-page { padding: 16px; }
    .print-header { flex-direction: column; gap: 12px; }
    .print-info-row { flex-direction: column; gap: 16px; }
    .print-totals { width: 100%; }
}
"#;

/// Company information — can be swapped for settings/config later.
pub struct CompanyInfo {
    pub name: &'static str,
    pub address: &'static str,
    pub phone_email: &'static str,
    pub tax_id: &'static str,
}

pub const DEFAULT_COMPANY: CompanyInfo = CompanyInfo {
    name: "MiniERP",
    address: "123 Business Avenue, Gulberg, Lahore 54660",
    phone_email: "Phone: +92 42 111 222 333  |  Email: accounts@minierp.pk",
    tax_id: "NTN: 1234567-8  |  STRN: 9876543210",
};

/// Trigger browser print dialog — works on both WASM and native desktop.
pub fn trigger_print() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Some(d) = w.document() {
                if let Some(v) = d.default_view() {
                    let _ = v.print();
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // On native desktop, no browser print API available.
        // User can use Ctrl+P or the OS print dialog.
    }
}
