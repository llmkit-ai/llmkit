-- Create models table
PRAGMA foreign_keys = ON;

CREATE TABLE model (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    model_name TEXT NOT NULL,        -- Model identifier (e.g., 'gpt-4', 'claude-2')
    UNIQUE(model_name)     -- Prevent duplicate model entries
);

CREATE TABLE prompt (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL UNIQUE,
    system TEXT NOT NULL,       
    user TEXT NOT NULL,       
    model_id INTEGER NOT NULL,
    max_tokens INTEGER NOT NULL DEFAULT 256,
    temperature REAL NOT NULL DEFAULT 0.7,
    json_mode BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (model_id) REFERENCES model(id)
);

CREATE TABLE log (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    prompt_id INTEGER,
    model_id INTEGER NOT NULL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    reasoning_tokens INTEGER,
    status_code INTEGER,
    request_body TEXT,
    response_data TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(prompt_id) REFERENCES prompt(id),
    FOREIGN KEY(model_id) REFERENCES model(id)
);


-- Indexes remain similar but reference new FKs
CREATE INDEX idx_traces_prompt ON log(prompt_id);
CREATE INDEX idx_traces_model ON log(model_id);  -- New index for model queries
CREATE INDEX idx_traces_created ON log(created_at);
CREATE INDEX idx_traces_status ON log(status_code);
