-- Create document_sequences table for gapless, sequential Thai tax running numbers
CREATE TABLE IF NOT EXISTS document_sequences (
    scope_id UUID NOT NULL,
    doc_type VARCHAR(32) NOT NULL,
    period VARCHAR(10) NOT NULL,
    last_number BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (scope_id, doc_type, period)
);

-- Add tax_invoice_number to orders table
ALTER TABLE orders ADD COLUMN IF NOT EXISTS tax_invoice_number VARCHAR(64);
CREATE UNIQUE INDEX IF NOT EXISTS idx_orders_tax_invoice_number 
    ON orders (tax_invoice_number) 
    WHERE tax_invoice_number IS NOT NULL;

-- Add refund_number to refunds table
ALTER TABLE refunds ADD COLUMN IF NOT EXISTS refund_number VARCHAR(64);
CREATE UNIQUE INDEX IF NOT EXISTS idx_refunds_refund_number 
    ON refunds (refund_number) 
    WHERE refund_number IS NOT NULL;
