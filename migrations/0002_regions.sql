-- ── REGIONS ──────────────────────────────────────────────────────────────────
-- Region is a first-class dimension. Each supported market is a row here.
CREATE TABLE regions (
    code         TEXT PRIMARY KEY,          -- 'GB', 'US', 'CA', 'AU'
    name         TEXT NOT NULL,             -- 'United Kingdom'
    currency     TEXT NOT NULL,             -- 'GBP', 'USD', 'CAD', 'AUD'
    currency_sym TEXT NOT NULL,             -- '£', '$', 'C$', 'A$'
    active       BOOLEAN NOT NULL DEFAULT FALSE  -- flip on when launched
);

INSERT INTO regions (code, name, currency, currency_sym, active) VALUES
    ('GB', 'United Kingdom', 'GBP', '£',  TRUE),   -- launch region
    ('US', 'United States',  'USD', '$',  FALSE),
    ('CA', 'Canada',         'CAD', 'C$', FALSE),
    ('AU', 'Australia',      'AUD', 'A$', FALSE);

-- ── PRODUCT GROUPS ───────────────────────────────────────────────────────────
-- Links the "same" product across regions (UK blazer ↔ US blazer).
-- Optional — an item can exist without a group.
CREATE TABLE product_groups (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    canonical_name TEXT NOT NULL,           -- 'ASOS Oversized Linen Blazer'
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── ADD REGION TO EXISTING TABLES ────────────────────────────────────────────

-- Brands: which network covers this brand IN THIS REGION
-- A brand can have multiple rows — one per region it operates in.
ALTER TABLE brands ADD COLUMN region TEXT NOT NULL DEFAULT 'GB' REFERENCES regions(code);
-- Drop old unique constraint on slug, make it unique per region instead
ALTER TABLE brands DROP CONSTRAINT brands_slug_key;
ALTER TABLE brands ADD CONSTRAINT brands_slug_region_key UNIQUE (slug, region);

-- Items: each regional variant is its own row
ALTER TABLE items ADD COLUMN region           TEXT NOT NULL DEFAULT 'GB' REFERENCES regions(code);
ALTER TABLE items ADD COLUMN currency         TEXT NOT NULL DEFAULT 'GBP';
ALTER TABLE items ADD COLUMN product_group_id UUID REFERENCES product_groups(id) ON DELETE SET NULL;
-- SKU is now unique per brand AND region (same SKU can appear in UK and US)
ALTER TABLE items DROP CONSTRAINT items_sku_brand_id_key;
ALTER TABLE items ADD CONSTRAINT items_sku_brand_region_key UNIQUE (sku, brand_id, region);

CREATE INDEX idx_items_region       ON items(region);
CREATE INDEX idx_items_group        ON items(product_group_id);
CREATE INDEX idx_items_region_drop  ON items(region, drop_percent DESC);  -- most common query

-- Users: their home region drives what they see + default currency
ALTER TABLE users ADD COLUMN region TEXT NOT NULL DEFAULT 'GB' REFERENCES regions(code);
