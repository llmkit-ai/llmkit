-- Add migration script here
CREATE TABLE llm_prompts (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL,
    prompt TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER UpdateTimestamps AFTER UPDATE ON llm_prompts
BEGIN
    UPDATE llm_prompts SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;


CREATE TABLE llm_api_traces (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    prompt_id INTEGER,
    provider TEXT NOT NULL,  -- e.g., 'openai', 'anthropic', 'cohere'
    model TEXT NOT NULL,
    request_data TEXT NOT NULL,  -- JSON string of input parameters
    response_data TEXT,  -- JSON string of API response
    status_code INTEGER,
    latency_ms INTEGER,  -- API call duration in milliseconds
    input_tokens INTEGER,
    output_tokens INTEGER,
    error_code TEXT,
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY(prompt_id) REFERENCES llm_prompts(id)
);

-- Indexes for common query patterns
CREATE INDEX idx_traces_prompt ON llm_api_traces(prompt_id);
CREATE INDEX idx_traces_created ON llm_api_traces(created_at);
CREATE INDEX idx_traces_status ON llm_api_traces(status_code);
