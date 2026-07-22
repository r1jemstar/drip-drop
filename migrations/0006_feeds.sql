CREATE TABLE IF NOT EXISTS feeds (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    label         TEXT NOT NULL,
    url           TEXT NOT NULL,
    network       TEXT NOT NULL DEFAULT 'awin',
    region        TEXT NOT NULL DEFAULT 'CA',
    fallback_brand TEXT NOT NULL DEFAULT 'Unknown',
    interval_hours INT  NOT NULL DEFAULT 24,
    active        BOOLEAN NOT NULL DEFAULT true,
    last_run_at   TIMESTAMPTZ,
    last_status   TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS feeds_url_idx ON feeds (url);