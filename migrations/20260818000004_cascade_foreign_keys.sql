-- Fix foreign key constraints on booking_queue, seat_bookings, orders, and related tables to CASCADE/SET NULL on seat or parent deletion

-- 1. booking_queue
ALTER TABLE booking_queue
    DROP CONSTRAINT IF EXISTS booking_queue_seat_id_fkey,
    ADD CONSTRAINT booking_queue_seat_id_fkey
        FOREIGN KEY (seat_id) REFERENCES seats(id) ON DELETE CASCADE;

ALTER TABLE booking_queue
    DROP CONSTRAINT IF EXISTS booking_queue_showtime_id_fkey,
    ADD CONSTRAINT booking_queue_showtime_id_fkey
        FOREIGN KEY (showtime_id) REFERENCES showtimes(id) ON DELETE CASCADE;

ALTER TABLE booking_queue
    DROP CONSTRAINT IF EXISTS booking_queue_zone_id_fkey,
    ADD CONSTRAINT booking_queue_zone_id_fkey
        FOREIGN KEY (zone_id) REFERENCES zones(id) ON DELETE CASCADE;

ALTER TABLE booking_queue
    DROP CONSTRAINT IF EXISTS booking_queue_user_id_fkey,
    ADD CONSTRAINT booking_queue_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 2. seat_bookings
ALTER TABLE seat_bookings
    DROP CONSTRAINT IF EXISTS seat_bookings_seat_id_fkey,
    ADD CONSTRAINT seat_bookings_seat_id_fkey
        FOREIGN KEY (seat_id) REFERENCES seats(id) ON DELETE CASCADE;

ALTER TABLE seat_bookings
    DROP CONSTRAINT IF EXISTS seat_bookings_showtime_id_fkey,
    ADD CONSTRAINT seat_bookings_showtime_id_fkey
        FOREIGN KEY (showtime_id) REFERENCES showtimes(id) ON DELETE CASCADE;

ALTER TABLE seat_bookings
    DROP CONSTRAINT IF EXISTS seat_bookings_user_id_fkey,
    ADD CONSTRAINT seat_bookings_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE seat_bookings
    DROP CONSTRAINT IF EXISTS seat_bookings_booking_queue_id_fkey,
    ADD CONSTRAINT seat_bookings_booking_queue_id_fkey
        FOREIGN KEY (booking_queue_id) REFERENCES booking_queue(id) ON DELETE SET NULL;

-- 3. orders
ALTER TABLE orders
    DROP CONSTRAINT IF EXISTS orders_user_id_fkey,
    ADD CONSTRAINT orders_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE orders
    DROP CONSTRAINT IF EXISTS orders_showtime_id_fkey,
    ADD CONSTRAINT orders_showtime_id_fkey
        FOREIGN KEY (showtime_id) REFERENCES showtimes(id) ON DELETE CASCADE;

-- 4. order_seats
ALTER TABLE order_seats
    DROP CONSTRAINT IF EXISTS order_seats_seat_booking_id_fkey,
    ADD CONSTRAINT order_seats_seat_booking_id_fkey
        FOREIGN KEY (seat_booking_id) REFERENCES seat_bookings(id) ON DELETE CASCADE;

-- 5. payment_events
ALTER TABLE payment_events
    DROP CONSTRAINT IF EXISTS payment_events_order_id_fkey,
    ADD CONSTRAINT payment_events_order_id_fkey
        FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE SET NULL;
