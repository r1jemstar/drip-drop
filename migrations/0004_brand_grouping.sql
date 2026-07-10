-- Brand tier: how we categorise + let users filter
CREATE TYPE brand_tier AS ENUM ('mainstream', 'boutique', 'luxury');

-- How often the ingest scheduler should check this brand's prices
CREATE TYPE check_freq AS ENUM ('daily', 'weekly', 'monthly');

ALTER TABLE brands ADD COLUMN tier         brand_tier NOT NULL DEFAULT 'mainstream';
ALTER TABLE brands ADD COLUMN check_freq   check_freq NOT NULL DEFAULT 'weekly';
ALTER TABLE brands ADD COLUMN is_canadian  BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE brands ADD COLUMN has_affiliate BOOLEAN NOT NULL DEFAULT FALSE;

-- Index so scheduler can grab "all brands due for a check" fast
CREATE INDEX idx_brands_freq ON brands(check_freq) WHERE active = TRUE;