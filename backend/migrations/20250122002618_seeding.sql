INSERT INTO provider (name, base_url)
VALUES ('openrouter', 'https://openrouter.ai/api/v1');


INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'google/gemini-2.0-flash-001', 1, 1
FROM provider
WHERE name = 'openrouter';

INSERT INTO model (provider_id, name, supports_json, supports_tools)
SELECT id, 'openai/o1-mini-2024-09-12', 1, 1
FROM provider
WHERE name = 'openrouter';
