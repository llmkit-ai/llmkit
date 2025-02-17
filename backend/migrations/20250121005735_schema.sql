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
    prompt_version_id INTEGER NOT NULL,
    FOREIGN KEY (prompt_version_id) REFERENCES prompt_version(id)
);

CREATE TABLE prompt_version (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    version_number INTEGER NOT NULL,
    system_diff TEXT,
    user_diff TEXT,
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

CREATE TABLE prompt_version_eval (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    prompt_version_id INTEGER NOT NULL,
    evaluation_type TEXT NOT NULL CHECK(evaluation_type IN ('human', 'automated')) DEFAULT 'human',
    score INTEGER CHECK (score BETWEEN 1 AND 5),
    output TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (prompt_version_id) REFERENCES prompt_version(id)
);

CREATE TABLE prompt_sample (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    prompt_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    input_data TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (prompt_id) REFERENCES prompt(id)
);
CREATE UNIQUE INDEX idx_unique_prompt_sample_name ON prompt_sample(prompt_id, name);

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
