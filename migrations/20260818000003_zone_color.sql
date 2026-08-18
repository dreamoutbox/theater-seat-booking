ALTER TABLE zones
ADD COLUMN IF NOT EXISTS color TEXT NOT NULL DEFAULT '#14b8a6';

-- Assign distinct default colors to known zone names
UPDATE zones SET color = '#f59e0b' WHERE LOWER(name) LIKE '%vip%' AND (color IS NULL OR color = '#14b8a6');
UPDATE zones SET color = '#6366f1' WHERE LOWER(name) LIKE '%premium%' AND (color IS NULL OR color = '#14b8a6');
UPDATE zones SET color = '#14b8a6' WHERE LOWER(name) LIKE '%standard%' AND (color IS NULL OR color = '#14b8a6');
