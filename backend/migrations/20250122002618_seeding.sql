-- Add migration script here

-- OpenAI model
INSERT INTO model (model_name) VALUES ('gpt-4o-2024-11-20');
INSERT INTO model (model_name) VALUES ('gpt-4o-mini-2024-07-18');
INSERT INTO model (model_name) VALUES ('o1-2024-12-17');
INSERT INTO model (model_name) VALUES ('o1-mini-2024-09-12');


-- Anthropic model
INSERT INTO model (model_name) VALUES ('claude-3-5-sonnet-latest');
INSERT INTO model (model_name) VALUES ('claude-3-5-sonnet-20241022');
INSERT INTO model (model_name) VALUES ('claude-3-5-haiku-latest');
INSERT INTO model (model_name) VALUES ('claude-3-5-haiku-20241022');

-- Gemini model
INSERT INTO model (model_name) VALUES ('gemini-2.0-flash-001');
INSERT INTO model (model_name) VALUES ('gemini-2.0-flash-lite-preview-02-05');
INSERT INTO model (model_name) VALUES ('gemini-2.0-flash-thinking-exp-01-21');
INSERT INTO model (model_name) VALUES ('gemini-1.5-flash');
INSERT INTO model (model_name) VALUES ('gemini-1.5-flash-8b');
INSERT INTO model (model_name) VALUES ('gemini-1.5-pro');

-- DeepSeek model
INSERT INTO model (model_name) VALUES ('deepseek-chat');
INSERT INTO model (model_name) VALUES ('deepseek-reasoner');

-- Sample Prompt
INSERT INTO prompt (key, prompt, model_id, max_tokens, temperature, json_mode)
VALUES (
    'ANOTHER-TEST-PROMPT',
    '<!-- role:system -->
    you are a sarcastic assistant

    <!-- role:user -->
    what is the meaning of life?',
    9,
    250,
    0.7,
    0
);
