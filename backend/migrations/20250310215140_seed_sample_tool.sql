-- Add sample getWeather tool
INSERT INTO tool (name, tool_name, description, parameters, strict)
VALUES ('Weather Tool', 
        'getWeather', 
        'Get the current weather in a given location', 
        '{
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City and country e.g. Bogotá, Colombia"
                }
            },
            "required": [
                "location"
            ],
            "additionalProperties": false
        }',
        TRUE);
