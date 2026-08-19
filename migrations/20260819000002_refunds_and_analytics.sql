-- 1. Add refunded_amount_cents to orders
ALTER TABLE orders ADD COLUMN IF NOT EXISTS refunded_amount_cents BIGINT NOT NULL DEFAULT 0;

-- 2. Update orders status check constraint to support refund statuses
ALTER TABLE orders DROP CONSTRAINT IF EXISTS orders_status_check;
ALTER TABLE orders ADD CONSTRAINT orders_status_check 
    CHECK (status IN ('pending_payment', 'paid', 'partially_refunded', 'refunded', 'cancelled', 'expired'));

-- 3. Add refund tracking columns to tickets
ALTER TABLE tickets ADD COLUMN IF NOT EXISTS refunded_at TIMESTAMPTZ;
ALTER TABLE tickets ADD COLUMN IF NOT EXISTS refunded_by UUID REFERENCES users(id) ON DELETE SET NULL;

-- 4. Create refunds audit table
CREATE TABLE IF NOT EXISTS refunds (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id         UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    amount_cents     BIGINT NOT NULL CHECK (amount_cents > 0),
    stripe_refund_id TEXT,
    refunded_by      UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    reason           TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 5. Create refund_tickets join table
CREATE TABLE IF NOT EXISTS refund_tickets (
    refund_id   UUID NOT NULL REFERENCES refunds(id) ON DELETE CASCADE,
    ticket_id   UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    price_cents BIGINT NOT NULL,
    PRIMARY KEY (refund_id, ticket_id)
);

CREATE INDEX IF NOT EXISTS refunds_order_id_idx ON refunds(order_id);
CREATE INDEX IF NOT EXISTS refund_tickets_ticket_id_idx ON refund_tickets(ticket_id);
