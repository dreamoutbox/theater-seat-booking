-- Create ticket_seat_changes table to track seat changes with program-readable metadata
CREATE TABLE IF NOT EXISTS ticket_seat_changes (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id         UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    ticket_id        UUID REFERENCES tickets(id) ON DELETE SET NULL,
    seat_booking_id  UUID REFERENCES seat_bookings(id) ON DELETE SET NULL,
    showtime_id      UUID NOT NULL REFERENCES showtimes(id) ON DELETE CASCADE,
    old_seat_id      UUID REFERENCES seats(id) ON DELETE SET NULL,
    new_seat_id      UUID REFERENCES seats(id) ON DELETE SET NULL,
    old_row_label    VARCHAR(10) NOT NULL,
    old_seat_number  INT NOT NULL,
    old_zone_name    VARCHAR(100) NOT NULL,
    old_price_cents  BIGINT NOT NULL,
    new_row_label    VARCHAR(10) NOT NULL,
    new_seat_number  INT NOT NULL,
    new_zone_name    VARCHAR(100) NOT NULL,
    new_price_cents  BIGINT NOT NULL,
    changed_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    reason           TEXT,
    metadata         JSONB,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ticket_seat_changes_order_id_idx ON ticket_seat_changes(order_id);
CREATE INDEX IF NOT EXISTS ticket_seat_changes_ticket_id_idx ON ticket_seat_changes(ticket_id);
CREATE INDEX IF NOT EXISTS ticket_seat_changes_showtime_id_idx ON ticket_seat_changes(showtime_id);
