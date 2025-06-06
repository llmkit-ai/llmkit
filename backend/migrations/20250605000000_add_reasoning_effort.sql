ALTER TABLE prompt_version
ADD COLUMN reasoning_effort TEXT CHECK(reasoning_effort IN ('low', 'medium', 'high'));