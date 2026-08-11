-- Migration: Create map_annotations table
CREATE TABLE IF NOT EXISTS map_annotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    map_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    line_type TEXT NOT NULL, -- เช่น 'VIRTUAL_WALL', 'RESTRICTED_ZONE'
    start_x REAL NOT NULL,   -- พิกัดโลก (World Frame: เมตร)
    start_y REAL NOT NULL,
    end_x REAL NOT NULL,
    end_y REAL NOT NULL,
    FOREIGN KEY(map_id) REFERENCES maps(id) ON DELETE CASCADE
);