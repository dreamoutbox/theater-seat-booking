-- 1. Add owner_id to events
ALTER TABLE events ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES users(id) ON DELETE SET NULL;

-- 2. Add gatekeeper value to user_role enum
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'gatekeeper';

-- 3. Create tickets table
CREATE TABLE IF NOT EXISTS tickets (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code             CHAR(6) NOT NULL UNIQUE,
    order_id         UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    seat_booking_id  UUID NOT NULL REFERENCES seat_bookings(id) ON DELETE CASCADE,
    user_id          UUID NOT NULL REFERENCES users(id),
    event_id         UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    showtime_id      UUID NOT NULL REFERENCES showtimes(id) ON DELETE CASCADE,
    status           TEXT NOT NULL DEFAULT 'active', -- active | used | cancelled | expired
    used_at          TIMESTAMPTZ,
    used_by          UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS tickets_seat_booking_id_idx ON tickets(seat_booking_id);
CREATE INDEX IF NOT EXISTS tickets_code_idx ON tickets(code);
CREATE INDEX IF NOT EXISTS tickets_order_id_idx ON tickets(order_id);
CREATE INDEX IF NOT EXISTS tickets_event_id_idx ON tickets(event_id);

-- 4. Create event_gatekeepers join table
CREATE TABLE IF NOT EXISTS event_gatekeepers (
    event_id    UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_by UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (event_id, user_id)
);

CREATE INDEX IF NOT EXISTS event_gatekeepers_user_idx ON event_gatekeepers(user_id);
