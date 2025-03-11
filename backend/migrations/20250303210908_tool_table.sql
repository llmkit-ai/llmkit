CREATE TABLE tool (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    tool_name TEXT NOT NULL,  -- function name for API
    description TEXT NOT NULL,
    parameters TEXT NOT NULL,
    strict BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE prompt_version_tool_access (
    prompt_version_id INTEGER NOT NULL,
    tool_id INTEGER NOT NULL,
    PRIMARY KEY (prompt_version_id, tool_id),
    FOREIGN KEY (prompt_version_id) REFERENCES prompt_version(id),
    FOREIGN KEY (tool_id) REFERENCES tool(id)
);
