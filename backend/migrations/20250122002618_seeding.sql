-- Add migration script here

-- OpenAI Models
INSERT INTO models (model_name) VALUES ('gpt-4o-2024-11-20');
INSERT INTO models (model_name) VALUES ('gpt-4o-mini-2024-07-18');
INSERT INTO models (model_name) VALUES ('o1-2024-12-17');
INSERT INTO models (model_name) VALUES ('o1-mini-2024-09-12');


-- Anthropic Models
INSERT INTO models (model_name) VALUES ('claude-3-5-sonnet-latest');
INSERT INTO models (model_name) VALUES ('claude-3-5-sonnet-20241022');
INSERT INTO models (model_name) VALUES ('claude-3-5-haiku-latest');
INSERT INTO models (model_name) VALUES ('claude-3-5-haiku-20241022');

-- Gemini Models
INSERT INTO models (model_name) VALUES ('gemini-2.0-flash-thinking-exp-01-21');
INSERT INTO models (model_name) VALUES ('gemini-2.0-flash-exp');
INSERT INTO models (model_name) VALUES ('gemini-1.5-flash');
INSERT INTO models (model_name) VALUES ('gemini-1.5-flash-8b');
INSERT INTO models (model_name) VALUES ('gemini-1.5-pro');

-- DeepSeek Models
INSERT INTO models (model_name) VALUES ('deepseek-chat');
INSERT INTO models (model_name) VALUES ('deepseek-reasoner');
