-- Add original_price_cents to order_seats to preserve initial purchase price across seat changes
ALTER TABLE order_seats
    ADD COLUMN IF NOT EXISTS original_price_cents BIGINT;

-- Backfill original_price_cents for existing records:
-- 1. Try to take the earliest old_price_cents from ticket_seat_changes if this seat was previously changed
-- 2. Fall back to current price_cents
UPDATE order_seats os
SET original_price_cents = COALESCE(
    (
        SELECT tsc.old_price_cents
        FROM ticket_seat_changes tsc
        WHERE tsc.order_id = os.order_id
          AND tsc.seat_booking_id = os.seat_booking_id
        ORDER BY tsc.created_at ASC
        LIMIT 1
    ),
    os.price_cents
)
WHERE os.original_price_cents IS NULL;
