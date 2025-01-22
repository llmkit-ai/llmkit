-- Add migration script here

-- OpenAI Models
INSERT INTO models (provider, model_name) VALUES ('openai', 'gpt-4o-2024-11-20	');
INSERT INTO models (provider, model_name) VALUES ('openai', 'gpt-4o-mini-2024-07-18');
INSERT INTO models (provider, model_name) VALUES ('openai', 'o1-2024-12-17');
INSERT INTO models (provider, model_name) VALUES ('openai', 'o1-mini-2024-09-12');


-- Anthropic Models
INSERT INTO models (provider, model_name) VALUES ('anthropic', 'claude-3-5-sonnet-latest');
INSERT INTO models (provider, model_name) VALUES ('anthropic', 'claude-3-5-sonnet-20241022');
INSERT INTO models (provider, model_name) VALUES ('anthropic', 'claude-3-5-haiku-latest');
INSERT INTO models (provider, model_name) VALUES ('anthropic', 'claude-3-5-haiku-20241022');

-- Gemini Models
INSERT INTO models (provider, model_name) VALUES ('gemini', 'gemini-2.0-flash-exp');
INSERT INTO models (provider, model_name) VALUES ('gemini', 'gemini-1.5-flash');
INSERT INTO models (provider, model_name) VALUES ('gemini', 'gemini-1.5-flash-8b');
INSERT INTO models (provider, model_name) VALUES ('gemini', 'gemini-1.5-pro');

-- DeepSeek Models
INSERT INTO models (provider, model_name) VALUES ('deepseek', 'deepseek-chat');
INSERT INTO models (provider, model_name) VALUES ('deepseek', 'deepseek-reasoner');
