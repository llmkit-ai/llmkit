CREATE TABLE tool (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL UNIQUE,  -- unique identifier/name
    current_tool_version_id INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (current_tool_version_id) REFERENCES tool_version(id)
);

CREATE TABLE tool_version (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    tool_id INTEGER NOT NULL,
    version_number INTEGER NOT NULL,
    name TEXT NOT NULL,  -- function name for API
    description TEXT NOT NULL,
    parameters JSON NOT NULL,  -- JSON schema
    strict BOOLEAN NOT NULL DEFAULT TRUE, 
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tool_id) REFERENCES tool(id)
);

CREATE TABLE prompt_version_tool_access (
    prompt_version_id INTEGER NOT NULL,
    tool_version_id INTEGER NOT NULL,
    PRIMARY KEY (prompt_version_id, tool_version_id),
    FOREIGN KEY (prompt_version_id) REFERENCES prompt_version(id),
    FOREIGN KEY (tool_version_id) REFERENCES tool_version(id)
);
