-- SQLite workaround: drop and recreate column to make it nullable
ALTER TABLE provider DROP COLUMN base_url;
ALTER TABLE provider ADD COLUMN base_url TEXT;

-- Update existing providers with their base URLs
UPDATE provider SET base_url = 'https://openrouter.ai/api/v1' WHERE name = 'openrouter';
UPDATE provider SET base_url = 'https://api.openai.com/v1' WHERE name = 'openai';

-- Add Azure provider with null base_url (requires user configuration)
INSERT INTO provider (name, base_url) VALUES ('azure', NULL);