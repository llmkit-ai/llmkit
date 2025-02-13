-- Providers
INSERT INTO provider (name, base_url) VALUES ('openai', 'https://api.openai.com/v1');
INSERT INTO provider (name, base_url) VALUES ('anthropic', 'https://api.anthropic.com/v1');
INSERT INTO provider (name, base_url) VALUES ('google', 'https://generativelanguage.googleapis.com/v1');
INSERT INTO provider (name, base_url) VALUES ('deepseek', 'https://api.deepseek.com/v1');
INSERT INTO provider (name, base_url) VALUES ('azure', 'https://your-instance.openai.azure.com/');

-- Models with provider relationships
INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gpt-4o-2024-11-20', 1, 1 FROM provider WHERE name = 'openai';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gpt-4o-mini-2024-07-18', 1, 1 FROM provider WHERE name = 'openai';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'o1-2024-12-17', 1, 1 FROM provider WHERE name = 'openai';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'o1-mini-2024-09-12', 1, 0 FROM provider WHERE name = 'openai';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'claude-3-5-sonnet-latest', 1, 1 FROM provider WHERE name = 'anthropic';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'claude-3-5-sonnet-20241022', 1, 1 FROM provider WHERE name = 'anthropic';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'claude-3-5-haiku-latest', 1, 1 FROM provider WHERE name = 'anthropic';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'claude-3-5-haiku-20241022', 1, 1 FROM provider WHERE name = 'anthropic';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gemini-2.0-flash-001', 1, 1 FROM provider WHERE name = 'google';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gemini-2.0-flash-lite-preview-02-05', 1, 1 FROM provider WHERE name = 'google';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gemini-2.0-flash-thinking-exp-01-21', 1, 0 FROM provider WHERE name = 'google';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gemini-2.0-pro-exp-02-05', 1, 1 FROM provider WHERE name = 'google';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gemini-1.5-flash', 1, 1 FROM provider WHERE name = 'google';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gemini-1.5-flash-8b', 1, 1 FROM provider WHERE name = 'google';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gemini-1.5-pro', 1, 1 FROM provider WHERE name = 'google';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'deepseek-chat', 1, 1 FROM provider WHERE name = 'deepseek';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'deepseek-reasoner', 1, 1 FROM provider WHERE name = 'deepseek';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'DeepSeek-R1', 1, 1 FROM provider WHERE name = 'azure';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'gpt-4o-mini', 1, 1 FROM provider WHERE name = 'azure';

-- Update sample prompt
INSERT INTO prompt (key, system, user, model_id, max_tokens, temperature, json_mode)
SELECT 'ANOTHER-TEST-PROMPT', 
       '{% if sarcastic %}You are sarcastic{% else %}Youre friendly{% endif %}', 
       'Tell me funny story about {{ name }}',
       m.id,
       250,
       0.7,
       0
FROM model m
WHERE m.name = 'gpt-4o-mini-2024-07-18';

