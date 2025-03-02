-- This migration adds 5 sample prompts to the database:
-- 1. Standard prompt with static system prompt, enabled for chat
-- 2. Dynamic system prompt enabled for chat
-- 3. Dynamic system prompt for one-shot prompts (not chat)
-- 4. Dynamic system and user prompt for one-shot prompts
-- 5. Dynamic system prompt with JSON response

-- Sample Prompt 1: Standard static system prompt for chat
INSERT INTO prompt (key, current_prompt_version_id)
VALUES ('STATIC-SYSTEM-CHAT', NULL);

INSERT INTO prompt_version (
    prompt_id,
    version_number,
    system_diff,
    user_diff,
    system,
    user,
    model_id,
    max_tokens,
    temperature,
    json_mode,
    json_schema,
    prompt_type,
    is_chat
)
SELECT
    p.id,
    1,
    NULL,
    NULL,
    'You are a helpful, friendly assistant who provides clear and concise answers to questions. Be polite, accurate, and straightforward.',
    '',
    m.id,
    2048,
    0.7,
    0,
    NULL,
    'static',
    1
FROM prompt p 
JOIN model m ON m.name = 'google/gemini-2.0-flash-001'
WHERE p.key = 'STATIC-SYSTEM-CHAT'
ORDER BY p.id DESC
LIMIT 1;

UPDATE prompt
SET current_prompt_version_id = (
    SELECT id FROM prompt_version
    WHERE prompt_id = (SELECT id FROM prompt WHERE key = 'STATIC-SYSTEM-CHAT')
    ORDER BY id DESC
    LIMIT 1
)
WHERE key = 'STATIC-SYSTEM-CHAT';

-- Sample Prompt 2: Dynamic system prompt for chat
INSERT INTO prompt (key, current_prompt_version_id)
VALUES ('DYNAMIC-SYSTEM-CHAT', NULL);

INSERT INTO prompt_version (
    prompt_id,
    version_number,
    system_diff,
    user_diff,
    system,
    user,
    model_id,
    max_tokens,
    temperature,
    json_mode,
    json_schema,
    prompt_type,
    is_chat
)
SELECT
    p.id,
    1,
    NULL,
    NULL,
    'You are a helpful assistant with a {{ personality }} personality. Provide answers that are {{ detail_level }} in detail. Remember to be {{ tone }} in your responses.

{% if include_examples %}
Here are some examples of good responses:
{{ examples }}
{% endif %}',
    '',
    m.id,
    2048,
    0.7,
    0,
    NULL,
    'dynamic_system',
    1
FROM prompt p 
JOIN model m ON m.name = 'google/gemini-2.0-flash-001'
WHERE p.key = 'DYNAMIC-SYSTEM-CHAT'
ORDER BY p.id DESC
LIMIT 1;

UPDATE prompt
SET current_prompt_version_id = (
    SELECT id FROM prompt_version
    WHERE prompt_id = (SELECT id FROM prompt WHERE key = 'DYNAMIC-SYSTEM-CHAT')
    ORDER BY id DESC
    LIMIT 1
)
WHERE key = 'DYNAMIC-SYSTEM-CHAT';

-- Sample Prompt 3: Dynamic system prompt for one-shot (not chat)
INSERT INTO prompt (key, current_prompt_version_id)
VALUES ('DYNAMIC-SYSTEM-ONESHOT', NULL);

INSERT INTO prompt_version (
    prompt_id,
    version_number,
    system_diff,
    user_diff,
    system,
    user,
    model_id,
    max_tokens,
    temperature,
    json_mode,
    json_schema,
    prompt_type,
    is_chat
)
SELECT
    p.id,
    1,
    NULL,
    NULL,
    'You are a summarization expert. Your task is to summarize the following {{ content_type }} in {{ word_count }} words or less.
{% if style %}
Use a {{ style }} writing style.
{% endif %}
{% if focus_on %}
Focus particularly on {{ focus_on }} in your summary.
{% endif %}',
    '',
    m.id,
    1024,
    0.5,
    0,
    NULL,
    'dynamic_system',
    0
FROM prompt p 
JOIN model m ON m.name = 'google/gemini-2.0-flash-001'
WHERE p.key = 'DYNAMIC-SYSTEM-ONESHOT'
ORDER BY p.id DESC
LIMIT 1;

UPDATE prompt
SET current_prompt_version_id = (
    SELECT id FROM prompt_version
    WHERE prompt_id = (SELECT id FROM prompt WHERE key = 'DYNAMIC-SYSTEM-ONESHOT')
    ORDER BY id DESC
    LIMIT 1
)
WHERE key = 'DYNAMIC-SYSTEM-ONESHOT';

-- Sample Prompt 4: Dynamic system and user prompts for one-shot
INSERT INTO prompt (key, current_prompt_version_id)
VALUES ('DYNAMIC-BOTH-ONESHOT', NULL);

INSERT INTO prompt_version (
    prompt_id,
    version_number,
    system_diff,
    user_diff,
    system,
    user,
    model_id,
    max_tokens,
    temperature,
    json_mode,
    json_schema,
    prompt_type,
    is_chat
)
SELECT
    p.id,
    1,
    NULL,
    NULL,
    'You are an expert code reviewer with {{ years_experience }} years of experience in {{ programming_language }}. Focus your review on:
- Code quality
- Performance
- Security
- Maintainability
{% if additional_focus_areas %}
- {{ additional_focus_areas }}
{% endif %}
Provide specific, actionable feedback that will help improve the code.',
    'Review the following code:

```{{ programming_language }}
{{ code }}
```

{{ specific_questions }}',
    m.id,
    2048,
    0.3,
    0,
    NULL,
    'dynamic_both',
    0
FROM prompt p 
JOIN model m ON m.name = 'google/gemini-2.0-flash-001'
WHERE p.key = 'DYNAMIC-BOTH-ONESHOT'
ORDER BY p.id DESC
LIMIT 1;

UPDATE prompt
SET current_prompt_version_id = (
    SELECT id FROM prompt_version
    WHERE prompt_id = (SELECT id FROM prompt WHERE key = 'DYNAMIC-BOTH-ONESHOT')
    ORDER BY id DESC
    LIMIT 1
)
WHERE key = 'DYNAMIC-BOTH-ONESHOT';

-- Sample Prompt 5: Dynamic system prompt with JSON response
INSERT INTO prompt (key, current_prompt_version_id)
VALUES ('DYNAMIC-SYSTEM-JSON', NULL);

INSERT INTO prompt_version (
    prompt_id,
    version_number,
    system_diff,
    user_diff,
    system,
    user,
    model_id,
    max_tokens,
    temperature,
    json_mode,
    json_schema,
    prompt_type,
    is_chat
)
SELECT
    p.id,
    1,
    NULL,
    NULL,
    'You are a product analysis expert. Analyze the given product based on the following criteria: {{ criteria }}.

Your task is to extract key information and provide a structured analysis of the product.

Return the result in JSON format with the following structure:
{
  "product_name": string,
  "category": string,
  "key_features": string[],
  "strengths": string[],
  "weaknesses": string[],
  "target_audience": string[],
  "competitive_analysis": {
    "advantages": string[],
    "disadvantages": string[]
  },
  "overall_rating": number (1-10),
  "recommendation": string
}

{% if include_price_analysis %}
Also include a "price_analysis" field with a price_to_value assessment.
{% endif %}',
    '',
    m.id,
    1536,
    0.2,
    1,
    NULL,
    'dynamic_system',
    0
FROM prompt p 
JOIN model m ON m.name = 'google/gemini-2.0-flash-001'
WHERE p.key = 'DYNAMIC-SYSTEM-JSON'
ORDER BY p.id DESC
LIMIT 1;

UPDATE prompt
SET current_prompt_version_id = (
    SELECT id FROM prompt_version
    WHERE prompt_id = (SELECT id FROM prompt WHERE key = 'DYNAMIC-SYSTEM-JSON')
    ORDER BY id DESC
    LIMIT 1
)
WHERE key = 'DYNAMIC-SYSTEM-JSON';
