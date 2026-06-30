# Invoice Edit Flow — Full Walkthrough

## 1. Entry Points (where you click Edit)

**Three paths into the edit form:**

| Source | File | Navigation |
|--------|------|-----------|
| Sales list (AG Grid) | `SalesPage.tsx` | Dropdown menu "Edit" → `navigate(\`/sales/invoice/${id}?mode=edit\`)` |
| Invoice view page | `InvoiceViewPage.tsx` | Toolbar "Edit" button → `navigate(\`/sales/invoice/${id}?mode=edit\`)` |
| Smart router | `InvoiceRouter.tsx` | Route `/sales/invoice/:id` (no `?mode=`) auto-decides: **Draft/Unpaid → edit**, Paid/Partially Paid/Cancelled → view |

## 2. Routing — URL to Component

Defined in `App.tsx`:

| Route | Component | Behaviour |
|-------|-----------|-----------|
| `/sales/invoice` | `SalesInvoicePage` | New invoice (blank form) |
| `/sales/invoice/:id` | `InvoiceRouter` | Fetches invoice, decides edit vs view by status |
| `/sales/invoice/:id/view` | `InvoiceViewPage` | Read-only view (forced) |
| `/sales/invoice/:id/edit` | `InvoiceRouter` | Edit mode (forced via `defaultMode="edit"`) |

## 3. Page Component — `SalesInvoicePage.tsx`

When the URL param `id` is present, the page runs four parallel fetches:

```
GET  /invoices/:id               → invoice header + line items
GET  /customers/:id              → customer info (credit limit, etc.)
GET  /customers/:id/balance      → customer current balance
GET  /invoices/:id/payments      → existing payments
```

These run inside a `useEffect` triggered by `[invoiceId]`. The response maps into `InvoiceFormState`:

```ts
setInvoice({
  ...invoiceData,
  items: padItemsToMinimum(formattedItems),  // pads to at least N rows so user has blank lines
  customer_id, customer_name, customer_email, customer_phone, customer_address,
  customer_current_balance, customer_credit_limit, customer_credit_utilization,
  discountScope: invoiceData.discount_scope || 'item',
  discount: { type: invoiceData.discount_type || 'flat', value: invoiceData.discount_value || 0 },
  notes, terms,
  company: { name, email, phone, address, taxId },   // from settings
  payment: {
    record_payment: false,
    payment_date: today,
    payment_amount: invoiceData.balance_amount,
    payment_method: 'Cash',
    reference_no: '',
    payment_notes: '',
  },
  paymentMethods: [{ id: Date.now(), method: 'Cash', amount: 0, reference_no: '' }],
})
```

`padItemsToMinimum` ensures an existing invoice with 2 saved lines still shows a few empty rows so the user can add items inline without clicking "+ Add Item" first.

## 4. Form Layout — Three Sub-Components

```
┌──────────────────────────────────────────┐
│  InvoiceFormHeader                        │
│  ┌──────┬──────────────┬────────────────┐│
│  │Title │ Customer      │ Invoice Date   ││
│  │INV-23│(searchable    │ Due Date       ││
│  │      │ select)       │                ││
│  └──────┴──────────────┴────────────────┘│
│  [Cancel]        [Update Invoice]         │
├──────────────────────────────────────────┤
│  InvoiceItemsTable                        │
│  ┌──────┬──────┬────┬────┬───┬──────────┐│
│  │Item  │Qty   │Rate│Tax │Disc│ Total    ││
│  ├──────┼──────┼────┼────┼───┼──────────┤│
│  │[▼sel]│  2   │10  │16% │ 5% │  23.20   ││
│  │[editable cells — click to edit]       ││
│  ├──────┴──────┴────┴────┴───┴──────────┤│
│  │ Subtotal: XX   Tax: YY   Total: ZZ    ││
│  │ [+ Add Item]                          ││
│  └──────────────────────────────────────┘│
├──────────────────────────────────────────┤
│  InvoicePaymentPanel                      │
│  ┌──────────────────────────────────────┐│
│  │ Existing Payments table              ││
│  │ [Edit] [Delete] per row              ││
│  │ ──────────────────────────────────   ││
│  │ New Payment toggle                   ││
│  │ Method  │ Amount  │ Ref              ││
│  │ [Record Payment]                     ││
│  └──────────────────────────────────────┘│
└──────────────────────────────────────────┘
```

### How editable cells work

- Each cell is identified by `data-cell-id="<rowId>-<field>"` (e.g. `"3-description"`, `"3-quantity"`)
- Clicking a cell → `onSetEditingCell(cellId)` → that cell's `editingCell` prop matches → it renders an `<input>` instead of a label
- **SearchableCell** (product name column): opens a dropdown that filters items from the product master as you type. Selecting one auto-fills `item_id`, `description`, and `rate`.
- **Rate cell**: when focused, a `useEffect` fires `fetchPriceHistory(itemId, customerId)` → `GET /sales/item-customer-history?item_id=X&customer_id=Y` → shows a floating hint with last transaction prices
- Tab / Enter advances to the next field (order defined by `getNextField()`). Enter on the last field auto-adds a new row.

## 5. Submit — What happens when you click "Update Invoice"

### Client side (`handleSubmit`)

```ts
1. filterFilledItems(items)           // discard empty rows
2. validateInvoiceSubmission(...)      // check customer_id + at least one item
3. validate(validationData)            // Zod schema (invoiceSchema) pass
4. Build submitData: {
     customer_id, invoice_date, due_date,
     total_amount, discount_scope, discount_type, discount_value,
     notes, terms,
     items: [{ item_id, quantity, unit_price, tax_rate, discount_type, discount_value }],
     ...(deletedPayments.length && { deleted_payments: [id1, id2] }),
     ...(record_payment && {
       record_payment: true,
       payment: { payment_date, amount, payment_method, reference_no, notes }
     })
   }
5. mutation.mutate(submitData)
```

### Mutation hook (`useInvoiceMutations.ts`)

```ts
function useSaveInvoice(invoiceId, customerId) {
  return useMutation({
    mutationFn: (data) => {
      if (invoiceId) return api.put(`/invoices/${invoiceId}`, data)   // ← PUT happens here
      return api.post('/invoices', data)
    },
    onSuccess: () => {
      toast.success('Invoice updated successfully!')
      navigate(`/customers/${customerId}`)
    },
    onError: (error) => toast.error(error.response?.data?.error)
  })
}
```

## 6. Backend — `PUT /api/invoices/:id`

Handler: **`invoiceController.updateInvoice`** (`server/src/controllers/invoiceController.ts`)

Everything runs inside one `db.transaction()`:

| Step | What | Tables/Models Involved |
|------|------|----------------------|
| 1 | Parse & validate request body | — |
| 2 | Check invoice exists (fast-fail) | `invoices` |
| 3 | **BEGIN TRANSACTION** | — |
| 4 | Re-read original invoice inside txn | `invoices` |
| 5 | **Handle deleted payments**: get allocations → delete allocations → delete payment rows → delete GL entries → recalc balances on all affected invoices | `payments`, `payment_allocations`, `customer_ledger`, `invoices` |
| 6 | Rebuild customer ledger running balances | `customer_ledger` |
| 7 | **Handle new payment** (if `record_payment=true`): atomic payment_no generation → insert payment → insert allocation → create ledger entry → post GL journal | `payments`, `payment_allocations`, `customer_ledger`, `journal_lines` |
| 8 | Recalculate `paid_amount` / `balance_amount` (factors in `returned_amount`) | — |
| 9 | Determine status: `Paid` / `Partially Paid` / `Unpaid` / `Returned` | — |
| 10 | **Update invoice header** (no, date, customer, totals, discount, notes, terms) | `InvoiceModel.updateInvoice()` → `UPDATE invoices SET ...` |
| 11 | **Reverse old stock**: fetch old invoice items, create ADJUSTMENT stock movements to add stock back | `stock_movements` |
| 12 | Delete old invoice line items | `DELETE FROM invoice_items WHERE invoice_id = ?` |
| 13 | **Insert new items** + create SALE stock movements (FIFO batch consumption) | `invoice_items`, `stock_movements` |
| 14 | Delete old ledger entry, create new one with updated total | `DELETE/INSERT INTO customer_ledger` |
| 15 | Update customer `current_balance` (both old and new customer if changed) | `customers` |
| 16 | **COMMIT TRANSACTION** | — |
| 17 | Fetch & return updated invoice | `InvoiceModel.getWithCustomer()` → `res.json(updatedInvoice)` |

### Key details

- **Stock reversal** (`InvoiceModel.reverseStockForItems`): old items get positive quantity movement (type `ADJUSTMENT`), effectively putting stock back on the shelf before new items deduct it again.
- **FIFO batch consumption** (`InvoiceModel.consumeFromOldestBatches`): for each new line item, stock is drawn from the oldest batches first.
- **Payment deletion**: only payments that the user explicitly removed (marked as deleted in the UI) are cleaned up. Previously recorded payments that *weren't* touched stay intact.
- **Customer balance**: rebuilt from scratch (`rebuildLedgerBalances`) after any ledger mutations to guarantee consistency.

## 7. File Manifest

| Layer | File | Role |
|-------|------|------|
| Routes | `App.tsx:247` | Registers `/sales/invoice/:id`, `:id/view`, `:id/edit` |
| Router | `InvoiceRouter.tsx` | Smart switch between edit/view based on `?mode=` or status |
| List | `SalesPage.tsx` | AG Grid invoice table with "Edit" dropdown action |
| View | `InvoiceViewPage.tsx` | Read-only invoice display with "Edit" toolbar button |
| Form | `SalesInvoicePage.tsx` | Main edit page — orchestrates data fetching, state, and sub-components |
| Header | `InvoiceFormHeader.tsx` | Customer selector, date pickers, Save/Cancel buttons |
| Items | `InvoiceItemsTable.tsx` + `SearchableCell.tsx` + `EditableCell.tsx` | Editable line-item table |
| Payments | `InvoicePaymentPanel.tsx` | Existing payments table + new payment form |
| Hooks | `useInvoiceMutations.ts` | `useSaveInvoice` — decides POST vs PUT |
| Schema | `schemas/index.ts` | Zod `invoiceSchema` for client-side validation |
| Types | `types/index.ts:1901` | `InvoiceFormState`, `InvoiceFormItem`, `ExistingPayment` etc. |
| Utils | `utils/invoiceCalculations.ts` | `calculateTotal`, `calculateTax`, `padItemsToMinimum`, etc. |
| Utils | `utils/invoiceRules.ts` | `filterFilledItems`, `validateInvoiceSubmission` |
| Backend | `invoiceController.ts` | `updateInvoice()` — ~120 line transaction |
| Backend | `InvoiceModel.ts:676` | `updateInvoice()` SQL, `reverseStockForItems()` |
| Backend | `routes/invoices.ts:14` | `router.put('/:id', ..., updateInvoice)` |

## 8. Data Flow Diagram

```
You click Edit
    ↓
navigate('/sales/invoice/{id}?mode=edit')
    ↓
InvoiceRouter → checks ?mode= param → renders SalesInvoicePage
    ↓
SalesInvoicePage → useEffect([invoiceId])
    ↓
4 parallel GETs:
  /invoices/{id}          → invoice header + items
  /customers/{id}         → customer details + credit limit
  /customers/{id}/balance → current balance
  /invoices/{id}/payments → existing payments
    ↓
Map into InvoiceFormState → render form (3 components)
    ↓
  You edit cells    → click-to-type on each field
  You search items  → SearchableCell dropdown filter
  You edit rate     → price history hint popup
  You add/remove rows → handleAddNewItem / handleRemoveItem
  You add/delete payments → handleRecordPayment / handleDeletePayment
    ↓
You click "Update Invoice"
    ↓
handleSubmit → validate → mutation.mutate(submitData)
    ↓
PUT /api/invoices/{id}  ← full invoice payload
    ↓
invoiceController.updateInvoice
  → single SQLite transaction:
    1. handle deleted payments (reverse allocations + ledger)
    2. handle new payments (create payment + allocation + GL)
    3. recalc paid_amount / balance_amount
    4. update invoice header
    5. reverse old stock movements
    6. delete old items, insert new items
    7. FIFO batch consumption for new items
    8. rebuild customer ledger + balance
  → commit
    ↓
Return updated invoice → toast "Updated!" → redirect to customer page
```
