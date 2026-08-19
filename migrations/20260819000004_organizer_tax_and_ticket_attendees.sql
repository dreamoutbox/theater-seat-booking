-- Add organizer tax information to events table
ALTER TABLE events ADD COLUMN IF NOT EXISTS tax_business_name VARCHAR(255);
ALTER TABLE events ADD COLUMN IF NOT EXISTS tax_id VARCHAR(100);
ALTER TABLE events ADD COLUMN IF NOT EXISTS tax_address TEXT;
ALTER TABLE events ADD COLUMN IF NOT EXISTS tax_email VARCHAR(255);
ALTER TABLE events ADD COLUMN IF NOT EXISTS tax_phone VARCHAR(50);

-- Add customer billing & tax information to orders table
ALTER TABLE orders ADD COLUMN IF NOT EXISTS billing_name VARCHAR(255);
ALTER TABLE orders ADD COLUMN IF NOT EXISTS tax_id VARCHAR(100);
ALTER TABLE orders ADD COLUMN IF NOT EXISTS billing_address TEXT;

-- Add attendee name and phone to order_seats join table
ALTER TABLE order_seats ADD COLUMN IF NOT EXISTS attendee_name VARCHAR(255);
ALTER TABLE order_seats ADD COLUMN IF NOT EXISTS attendee_phone VARCHAR(100);

-- Add attendee name and phone to tickets table
ALTER TABLE tickets ADD COLUMN IF NOT EXISTS attendee_name VARCHAR(255);
ALTER TABLE tickets ADD COLUMN IF NOT EXISTS attendee_phone VARCHAR(100);
