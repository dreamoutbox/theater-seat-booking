-- Migration: Event-Owned Seat Maps
-- Re-architect zones and seats to belong to events rather than theaters

-- 1. Add event_id column to zones and seats
ALTER TABLE zones
    ADD COLUMN IF NOT EXISTS event_id UUID REFERENCES events(id) ON DELETE CASCADE;

ALTER TABLE seats
    ADD COLUMN IF NOT EXISTS event_id UUID REFERENCES events(id) ON DELETE CASCADE;

-- 2. Backfill event_id from existing showtimes where possible
UPDATE zones z
SET event_id = sub.event_id
FROM (
    SELECT DISTINCT ON (theater_id) theater_id, event_id
    FROM showtimes
    ORDER BY theater_id, starts_at ASC
) sub
WHERE z.theater_id = sub.theater_id
  AND z.event_id IS NULL;

UPDATE seats s
SET event_id = sub.event_id
FROM (
    SELECT DISTINCT ON (theater_id) theater_id, event_id
    FROM showtimes
    ORDER BY theater_id, starts_at ASC
) sub
WHERE s.theater_id = sub.theater_id
  AND s.event_id IS NULL;

-- Remove any orphaned seats or zones that cannot be associated with an event
DELETE FROM seats WHERE event_id IS NULL;
DELETE FROM zones WHERE event_id IS NULL;

-- 3. Make event_id NOT NULL and theater_id nullable
ALTER TABLE zones
    ALTER COLUMN event_id SET NOT NULL,
    ALTER COLUMN theater_id DROP NOT NULL;

ALTER TABLE seats
    ALTER COLUMN event_id SET NOT NULL,
    ALTER COLUMN theater_id DROP NOT NULL;

-- 4. Replace unique constraint on seats to be scoped by event_id
ALTER TABLE seats
    DROP CONSTRAINT IF EXISTS seats_theater_id_row_label_seat_number_key;

ALTER TABLE seats
    DROP CONSTRAINT IF EXISTS seats_event_id_row_label_seat_number_key,
    ADD CONSTRAINT seats_event_id_row_label_seat_number_key UNIQUE (event_id, row_label, seat_number);

-- 5. Add performance indexes
CREATE INDEX IF NOT EXISTS idx_zones_event_id ON zones(event_id);
CREATE INDEX IF NOT EXISTS idx_seats_event_id ON seats(event_id);
