PRAGMA foreign_keys = ON;

CREATE TABLE provider (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    base_url TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE model (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    provider_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    supports_json BOOLEAN NOT NULL DEFAULT 0,
    supports_tools BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(provider_id) REFERENCES provider(id),
    UNIQUE(provider_id, name)
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

CREATE INDEX idx_traces_prompt ON log(prompt_id);
CREATE INDEX idx_traces_model ON log(model_id);
CREATE INDEX idx_traces_created ON log(created_at);
CREATE INDEX idx_traces_status ON log(status_code);
