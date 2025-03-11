-- Add sample getWeather tool
INSERT INTO tools (name, description, parameters, function_call)
VALUES ('getWeather', 
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
        'async function getWeather({ location }) {
            console.log(`Getting weather for ${location}`);
            // Simulated API call response
            return {
                location: location,
                temperature: `${Math.floor(10 + Math.random() * 25)}°C`,
                conditions: ["Sunny", "Cloudy", "Rainy", "Partly Cloudy", "Clear"][Math.floor(Math.random() * 5)],
                humidity: `${Math.floor(30 + Math.random() * 60)}%`,
                windSpeed: `${Math.floor(5 + Math.random() * 30)} km/h`
            };
        }');