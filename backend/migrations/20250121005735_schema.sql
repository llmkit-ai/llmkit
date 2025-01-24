-- Create models table
CREATE TABLE models (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    model_name TEXT NOT NULL,        -- Model identifier (e.g., 'gpt-4', 'claude-2')
    UNIQUE(model_name)     -- Prevent duplicate model entries
);


-- Updated prompts table
CREATE TABLE llm_prompts (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    prompt TEXT NOT NULL,
    model_id INTEGER NOT NULL,       -- Foreign key to models
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (model_id) REFERENCES models(id)
);

-- Updated api traces table
CREATE TABLE llm_api_traces (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    prompt_id INTEGER,
    model_id INTEGER NOT NULL,       -- Foreign key to models
    request_data TEXT NOT NULL,
    response_data TEXT,
    status_code INTEGER,
    latency_ms INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    error_code TEXT,
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(prompt_id) REFERENCES llm_prompts(id),
    FOREIGN KEY(model_id) REFERENCES models(id)
);

-- Indexes remain similar but reference new FKs
CREATE INDEX idx_traces_prompt ON llm_api_traces(prompt_id);
CREATE INDEX idx_traces_model ON llm_api_traces(model_id);  -- New index for model queries
CREATE INDEX idx_traces_created ON llm_api_traces(created_at);
CREATE INDEX idx_traces_status ON llm_api_traces(status_code);
