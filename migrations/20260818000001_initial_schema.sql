-- Initial Schema for Theater Seat Booking POC

CREATE TYPE user_role AS ENUM ('admin','staff','event_organizer','customer');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'customer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE theaters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    address TEXT
);

CREATE TABLE zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    theater_id UUID NOT NULL REFERENCES theaters(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    price_cents BIGINT NOT NULL CHECK (price_cents >= 0)
);

CREATE TABLE seats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    theater_id UUID NOT NULL REFERENCES theaters(id) ON DELETE CASCADE,
    zone_id UUID NOT NULL REFERENCES zones(id) ON DELETE CASCADE,
    row_label TEXT NOT NULL,
    seat_number INT NOT NULL,
    UNIQUE (theater_id, row_label, seat_number)
);

CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL
);

CREATE TABLE showtimes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    theater_id UUID NOT NULL REFERENCES theaters(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL
);

-- The queue: one row per seat-click request
CREATE TABLE booking_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    showtime_id UUID NOT NULL REFERENCES showtimes(id) ON DELETE CASCADE,
    zone_id UUID NOT NULL REFERENCES zones(id) ON DELETE CASCADE,
    seat_id UUID NOT NULL REFERENCES seats(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','processing','success','failed')),
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_booking_queue_partition
    ON booking_queue (showtime_id, zone_id, status, created_at);

-- Source of truth for seat occupancy
CREATE TABLE seat_bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    showtime_id UUID NOT NULL REFERENCES showtimes(id) ON DELETE CASCADE,
    seat_id UUID NOT NULL REFERENCES seats(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    booking_queue_id UUID REFERENCES booking_queue(id) ON DELETE SET NULL,
    status TEXT NOT NULL CHECK (status IN ('held','confirmed','released')),
    held_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

-- Defense-in-depth: DB physically cannot hold two active claims on one seat
CREATE UNIQUE INDEX uniq_active_seat_per_showtime
    ON seat_bookings (showtime_id, seat_id)
    WHERE status IN ('held','confirmed');

CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    showtime_id UUID NOT NULL REFERENCES showtimes(id) ON DELETE CASCADE,
    total_amount_cents BIGINT NOT NULL CHECK (total_amount_cents >= 0),
    status TEXT NOT NULL DEFAULT 'pending_payment'
        CHECK (status IN ('pending_payment','paid','cancelled','expired')),
    stripe_payment_intent_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE order_seats (
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    seat_booking_id UUID NOT NULL REFERENCES seat_bookings(id) ON DELETE CASCADE,
    price_cents BIGINT NOT NULL,
    PRIMARY KEY (order_id, seat_booking_id)
);

-- Stripe webhook idempotency
CREATE TABLE payment_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stripe_event_id TEXT UNIQUE NOT NULL,
    order_id UUID REFERENCES orders(id) ON DELETE SET NULL,
    type TEXT NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
