CREATE INDEX IF NOT EXISTS idx_media_present_type_id
    ON media(is_present, media_type, id);

CREATE INDEX IF NOT EXISTS idx_media_present_extension_id
    ON media(is_present, extension, id);

CREATE INDEX IF NOT EXISTS idx_media_present_date_id
    ON media(is_present, COALESCE(modified_at_fs, created_at_fs, 0), id);

CREATE INDEX IF NOT EXISTS idx_media_present_size_id
    ON media(is_present, size_bytes, id);

CREATE INDEX IF NOT EXISTS idx_media_present_name_id
    ON media(is_present, file_name COLLATE NOCASE, id);

PRAGMA user_version = 3;
