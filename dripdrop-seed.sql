-- ══════════════ DRIP DROP SEED DATA ══════════════
-- Run with: psql -U postgres -d dripdrop -f seed.sql
-- Realistic multi-region test data (GB / US / CA)

-- ── BRANDS (one row per brand per region) ──
INSERT INTO brands (id, name, slug, region, awin_id, commission, active) VALUES
  ('11111111-1111-1111-1111-111111111101', 'ASOS',     'asos',     'GB', 'awin-asos-gb', 0.06, TRUE),
  ('11111111-1111-1111-1111-111111111102', 'ASOS',     'asos',     'US', 'cj-asos-us',    0.05, TRUE),
  ('11111111-1111-1111-1111-111111111103', 'ASOS',     'asos',     'CA', 'rak-asos-ca',   0.05, TRUE),
  ('11111111-1111-1111-1111-111111111104', 'Nike',     'nike',     'GB', 'awin-nike-gb', 0.04, TRUE),
  ('11111111-1111-1111-1111-111111111105', 'Nike',     'nike',     'US', 'cj-nike-us',    0.04, TRUE),
  ('11111111-1111-1111-1111-111111111106', 'Nike',     'nike',     'CA', 'rak-nike-ca',   0.04, TRUE),
  ('11111111-1111-1111-1111-111111111107', 'Aritzia',  'aritzia',  'CA', 'rak-aritzia-ca',0.07, TRUE),
  ('11111111-1111-1111-1111-111111111108', 'Zara',     'zara',     'GB', 'awin-zara-gb', 0.05, TRUE);

-- ── PRODUCT GROUPS (link same item across regions) ──
INSERT INTO product_groups (id, canonical_name) VALUES
  ('22222222-2222-2222-2222-222222222201', 'ASOS Oversized Linen Blazer'),
  ('22222222-2222-2222-2222-222222222202', 'Nike Air Max 95');

-- ── ITEMS ──
-- The linen blazer in all 3 regions (same group, different price/currency/link)
INSERT INTO items (sku, brand_id, name, category, current_price, was_price, drop_percent, affiliate_url, image_url, sizes, region, currency, product_group_id, expires_at) VALUES
  ('BLZ-001', '11111111-1111-1111-1111-111111111101', 'Oversized Linen Blazer — Camel', 'womenswear', 34.00, 55.00, 38, 'https://asos.com/gb/blazer?aff=dripdrop', '', ARRAY['XS','S','M','L'], 'GB', 'GBP', '22222222-2222-2222-2222-222222222201', NOW() + INTERVAL '6 hours'),
  ('BLZ-001', '11111111-1111-1111-1111-111111111102', 'Oversized Linen Blazer — Camel', 'womenswear', 47.00, 72.00, 35, 'https://asos.com/us/blazer?aff=dripdrop', '', ARRAY['XS','S','M','L'], 'US', 'USD', '22222222-2222-2222-2222-222222222201', NOW() + INTERVAL '6 hours'),
  ('BLZ-001', '11111111-1111-1111-1111-111111111103', 'Oversized Linen Blazer — Camel', 'womenswear', 58.00, 89.00, 35, 'https://asos.com/ca/blazer?aff=dripdrop', '', ARRAY['XS','S','M','L'], 'CA', 'CAD', '22222222-2222-2222-2222-222222222201', NOW() + INTERVAL '6 hours'),

  -- Nike Air Max in all 3 regions
  ('AM95-01', '11111111-1111-1111-1111-111111111104', 'Air Max 95 — White/Grey', 'footwear', 89.00, 112.00, 20, 'https://nike.com/gb/am95?aff=dripdrop', '', ARRAY['5','6','7','8'], 'GB', 'GBP', '22222222-2222-2222-2222-222222222202', NOW() + INTERVAL '48 hours'),
  ('AM95-01', '11111111-1111-1111-1111-111111111105', 'Air Max 95 — White/Grey', 'footwear', 115.00, 150.00, 23, 'https://nike.com/us/am95?aff=dripdrop', '', ARRAY['5','6','7','8'], 'US', 'USD', '22222222-2222-2222-2222-222222222202', NOW() + INTERVAL '48 hours'),
  ('AM95-01', '11111111-1111-1111-1111-111111111106', 'Air Max 95 — White/Grey', 'footwear', 155.00, 200.00, 22, 'https://nike.com/ca/am95?aff=dripdrop', '', ARRAY['5','6','7','8'], 'CA', 'CAD', '22222222-2222-2222-2222-222222222202', NOW() + INTERVAL '48 hours'),

  -- Canada-only: Aritzia (tests region-exclusive brands)
  ('ARZ-99', '11111111-1111-1111-1111-111111111107', 'Wilfred Free Sculpt Knit Top', 'womenswear', 45.00, 68.00, 34, 'https://aritzia.com/ca/top?aff=dripdrop', '', ARRAY['XS','S','M','L'], 'CA', 'CAD', NULL, NOW() + INTERVAL '12 hours'),

  -- GB-only: Zara
  ('ZR-CG2', '11111111-1111-1111-1111-111111111108', 'Cargo Wide Leg Trousers — Khaki', 'womenswear', 25.00, 35.00, 29, 'https://zara.com/gb/cargo?aff=dripdrop', '', ARRAY['XS','S','M','L','XL'], 'GB', 'GBP', NULL, NOW() + INTERVAL '14 hours');

-- ── PRICE HISTORY (for the blazer GB — powers the chart) ──
DO $$
DECLARE blazer_gb UUID;
BEGIN
  SELECT id INTO blazer_gb FROM items WHERE sku='BLZ-001' AND region='GB';
  INSERT INTO price_history (item_id, price, recorded_at) VALUES
    (blazer_gb, 55.00, NOW() - INTERVAL '8 weeks'),
    (blazer_gb, 55.00, NOW() - INTERVAL '7 weeks'),
    (blazer_gb, 52.00, NOW() - INTERVAL '6 weeks'),
    (blazer_gb, 55.00, NOW() - INTERVAL '5 weeks'),
    (blazer_gb, 48.00, NOW() - INTERVAL '4 weeks'),
    (blazer_gb, 44.00, NOW() - INTERVAL '3 weeks'),
    (blazer_gb, 38.00, NOW() - INTERVAL '2 weeks'),
    (blazer_gb, 34.00, NOW() - INTERVAL '1 week');
END $$;

SELECT 'Seeded ' || COUNT(*) || ' items' AS result FROM items;