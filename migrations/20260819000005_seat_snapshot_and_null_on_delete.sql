-- 1. Add snapshot columns to order_seats for historical preservation
ALTER TABLE order_seats
    ADD COLUMN IF NOT EXISTS row_label TEXT,
    ADD COLUMN IF NOT EXISTS seat_number INT,
    ADD COLUMN IF NOT EXISTS zone_name TEXT;

-- 2. Backfill existing order_seats snapshot data
UPDATE order_seats os
SET row_label = s.row_label,
    seat_number = s.seat_number,
    zone_name = z.name
FROM seat_bookings sb
JOIN seats s ON s.id = sb.seat_id
JOIN zones z ON z.id = s.zone_id
WHERE os.seat_booking_id = sb.id
  AND (os.row_label IS NULL OR os.seat_number IS NULL OR os.zone_name IS NULL);

-- 3. Modify seat_bookings foreign key to ON DELETE SET NULL so deleting released seats does not cascade-delete historical order records
ALTER TABLE seat_bookings
    DROP CONSTRAINT IF EXISTS seat_bookings_seat_id_fkey,
    ALTER COLUMN seat_id DROP NOT NULL,
    ADD CONSTRAINT seat_bookings_seat_id_fkey
        FOREIGN KEY (seat_id) REFERENCES seats(id) ON DELETE SET NULL;
