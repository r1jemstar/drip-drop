CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE category AS ENUM ('womenswear','menswear','footwear','accessories','workwear');
CREATE TYPE tag_type  AS ENUM ('system','community','personal');

CREATE TABLE brands (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name       TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,
    awin_id    TEXT,
    cj_id      TEXT,
    commission NUMERIC(5,4) NOT NULL DEFAULT 0.06,
    active     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE items (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sku           TEXT NOT NULL,
    brand_id      UUID NOT NULL REFERENCES brands(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    category      category NOT NULL,
    current_price NUMERIC(10,2) NOT NULL,
    was_price     NUMERIC(10,2) NOT NULL,
    drop_percent  NUMERIC(5,2) NOT NULL DEFAULT 0,
    affiliate_url TEXT NOT NULL,
    image_url     TEXT NOT NULL DEFAULT '',
    sizes         TEXT[] NOT NULL DEFAULT '{}',
    in_stock      BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at    TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sku, brand_id)
);

CREATE INDEX idx_items_brand    ON items(brand_id);
CREATE INDEX idx_items_category ON items(category);
CREATE INDEX idx_items_drop     ON items(drop_percent DESC);
CREATE INDEX idx_items_price    ON items(current_price);
CREATE INDEX idx_items_updated  ON items(updated_at DESC);

-- Only insert when price ACTUALLY changes (checked in Rust before insert)
CREATE TABLE price_history (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    item_id     UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    price       NUMERIC(10,2) NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_ph_item_time ON price_history(item_id, recorded_at DESC);

CREATE TABLE affiliate_links (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    item_id    UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    network    TEXT NOT NULL,
    url        TEXT NOT NULL,
    commission NUMERIC(5,4)
);

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email           TEXT NOT NULL UNIQUE,
    display_name    TEXT NOT NULL DEFAULT '',
    password_hash   TEXT,
    is_premium      BOOLEAN NOT NULL DEFAULT FALSE,
    excluded_brands UUID[] NOT NULL DEFAULT '{}',
    theme           JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE alerts (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    item_id      UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    target_price NUMERIC(10,2),
    active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, item_id)
);

CREATE TABLE tags (
    id       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    label    TEXT NOT NULL,
    tag_type tag_type NOT NULL DEFAULT 'community',
    upvotes  INT NOT NULL DEFAULT 0
);
CREATE TABLE item_tags (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    tag_id  UUID NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    PRIMARY KEY (item_id, tag_id)
);
CREATE INDEX idx_tags_upvotes ON tags(upvotes DESC) WHERE tag_type = 'community';

CREATE TABLE style_boards (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    is_shared  BOOLEAN NOT NULL DEFAULT FALSE,
    share_slug TEXT UNIQUE,
    item_ids   UUID[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW(); RETURN NEW; END; $$ LANGUAGE plpgsql;
CREATE TRIGGER items_updated_at BEFORE UPDATE ON items
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
