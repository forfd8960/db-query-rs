-- Database connections table
CREATE TABLE IF NOT EXISTS database_connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    connection_url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Table metadata cache
CREATE TABLE IF NOT EXISTS table_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    database_id INTEGER NOT NULL,
    schema_name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    table_type TEXT NOT NULL, -- 'TABLE' or 'VIEW'
    cached_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (database_id) REFERENCES database_connections(id) ON DELETE CASCADE,
    UNIQUE (database_id, schema_name, table_name)
);

-- Column metadata cache
CREATE TABLE IF NOT EXISTS column_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_id INTEGER NOT NULL,
    column_name TEXT NOT NULL,
    data_type TEXT NOT NULL,
    is_nullable BOOLEAN NOT NULL,
    column_default TEXT,
    ordinal_position INTEGER NOT NULL,
    is_primary_key BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (table_id) REFERENCES table_metadata(id) ON DELETE CASCADE,
    UNIQUE (table_id, column_name)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_table_metadata_db ON table_metadata(database_id);
CREATE INDEX IF NOT EXISTS idx_column_metadata_table ON column_metadata(table_id);
