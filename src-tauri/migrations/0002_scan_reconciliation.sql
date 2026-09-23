ALTER TABLE sources
    ADD COLUMN scan_generation INTEGER NOT NULL DEFAULT 0;

ALTER TABLE media
    ADD COLUMN is_present INTEGER NOT NULL DEFAULT 1;

ALTER TABLE media
    ADD COLUMN last_seen_generation INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_media_present_id
    ON media(is_present, id);

CREATE INDEX IF NOT EXISTS idx_media_source_present_id
    ON media(source_id, is_present, id);

PRAGMA user_version = 2;
