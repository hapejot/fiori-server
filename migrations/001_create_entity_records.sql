-- Entity records: stores all OData entity data as JSONB.
-- One row per entity record per draft state (is_active).
CREATE TABLE IF NOT EXISTS entity_records (
    entity_set  TEXT    NOT NULL,
    key_value   TEXT    NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    data        JSONB   NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (entity_set, key_value, is_active)
);

-- Index for fast collection queries per entity set.
CREATE INDEX IF NOT EXISTS idx_entity_records_set
    ON entity_records (entity_set, is_active);

-- GIN index on data for $filter queries inside JSONB.
CREATE INDEX IF NOT EXISTS idx_entity_records_data
    ON entity_records USING GIN (data);
