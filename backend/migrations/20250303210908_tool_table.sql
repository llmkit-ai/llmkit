CREATE TABLE prompt_version_tool (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    prompt_version_id INTEGER NOT NULL,
    name TEXT NOT NULL,  
    description TEXT NOT NULL,
    parameters JSON NOT NULL, -- denormalized JSON object here
    strict BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (prompt_version_id) REFERENCES prompt_version(id)
);
