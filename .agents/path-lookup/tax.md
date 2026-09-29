### Thai tax running numbers for orders, refunds, and financial statements. If you want to configure, generate, or display sequential Thai tax running numbers, then change:

- `migrations/20260825000001_thai_tax_running_numbers.sql` = database table `document_sequences` and columns `orders.tax_invoice_number`, `refunds.refund_number`
- `crates/repository/src/sequence_repository.rs` = atomic sequence generator using PostgreSQL `ON CONFLICT DO UPDATE` to guarantee gapless numbers
- `crates/repository/src/models.rs` = models with `tax_invoice_number` on `OrderDetailRow`/`OrderDetail`, `refund_number` on `Refund`/`RefundDetail`, and `DocumentSequence`
- `crates/repository/src/order_repository.rs` = database queries returning `tax_invoice_number` and methods `update_tax_invoice_number` & `find_event_id_by_order_id`
- `crates/repository/src/refund_repository.rs` = database query saving `refund_number` on refund creation and fetching it in `find_by_order_id`
- `crates/repository/src/report_repository.rs` = `ReportRepository::save` accepting transactional executor for atomic `statement_ref` save
- `src/routes/order.rs` = assigns `INV-YYYYMM-XXXXX` upon payment confirmation and `CN-YYYYMM-XXXXX` upon ticket refunds
- `src/routes/webhook.rs` = assigns `INV-YYYYMM-XXXXX` when Stripe webhook payment succeeds
- `src/routes/report.rs` = assigns `FIN-YYYYMM-XXXXX` when event financial report statement is generated
- `src/pdf.rs` = renders `Tax Inv/Receipt: <tax_invoice_number>` on official order receipt PDF
- `src/email.rs` = includes tax invoice running number in order confirmation email subject line
- `frontend/src/components/OrdersTable.vue` = displays `tax_invoice_number` with short ID fallback and enables search filtering by tax invoice number
- `frontend/src/views/OrderDetailView.vue` = displays tax invoice chip, credit note number in refunds table, and official tax invoice number in printable tax receipt
