## ADDED Requirements

### Requirement: Tax summary uses pre-tax amounts
The `GET /api/reports/tax-summary` endpoint SHALL calculate tax amounts using the pre-tax base (quantity × unit_price - discount), NOT the tax-inclusive total_amount. The formula SHALL be: `tax_amount = SUM(quantity × unit_price - discount) × tax_rate / 100`.

#### Scenario: Tax summary calculates correctly
- **WHEN** an invoice has items totaling 1000.00 (pre-tax) with tax_rate=17
- **THEN** the tax_amount = 170.00 (not 1000 × 17/100 = 170, which happens to be the same but the methodology matters for invoices with mixed tax rates)
- **AND** if the total_amount was 1170.00 (tax-inclusive), the tax is still calculated on the 1000.00 base
