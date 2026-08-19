-- Add sequential event running number for accounting, invoicing, and tax report statement references
ALTER TABLE events ADD COLUMN IF NOT EXISTS event_number BIGSERIAL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_events_event_number ON events (event_number);
