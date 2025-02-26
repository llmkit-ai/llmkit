import OpenAI from 'openai';

// Initialize the OpenAI client with our LLMKit server
const client = new OpenAI({
  apiKey: '',
  baseURL: 'http://localhost:8000/v1', // Point to the LLMKit server
});

// Utility function to print section headers
function printHeader(title) {
  console.log(`\n=== ${title} ===`);
}

// Example 1: Basic chat completion
async function basicChatCompletion() {
  printHeader('Example 1: Basic Chat Completion');

  try {
    const response = await client.chat.completions.create({
      model: 'STATIC-SYSTEM-CHAT', // Using our static system prompt for chat
      messages: [
        { role: 'user', content: 'What are three benefits of using TypeScript?' }
      ],
    });

    console.log(response.choices[0].message.content);
    return response.choices[0].message.content;
  } catch (error) {
    console.error('Error:', error.message);
    return null;
  }
}

// Example 2: Dynamic system prompt with JSON context
async function dynamicSystemWithJsonContext() {
  printHeader('Example 2: Dynamic System with JSON Context');

  // Context for template variables in the system prompt
  const context = {
    personality: 'professional',
    detail_level: 'thorough',
    tone: 'formal'
  };

  try {
    const response = await client.chat.completions.create({
      model: 'DYNAMIC-SYSTEM-CHAT',
      messages: [
        // Pass JSON as the system message for template variable substitution
        { role: 'system', content: JSON.stringify(context) },
        { role: 'user', content: 'What are the key principles of good API design?' }
      ],
    });

    console.log(response.choices[0].message.content);
    return response.choices[0].message.content;
  } catch (error) {
    console.error('Error:', error.message);
    return null;
  }
}

// Example 3: One-shot dynamic system prompt
async function oneshotDynamicSystem() {
  printHeader('Example 3: One-shot Dynamic System');

  const context = {
    content_type: 'blog post',
    word_count: 150,
    style: 'conversational',
    focus_on: 'practical tips'
  };

  try {
    const response = await client.chat.completions.create({
      model: 'DYNAMIC-SYSTEM-ONESHOT',
      messages: [
        { role: 'system', content: JSON.stringify(context) },
        { role: 'user', content: 'The JavaScript ecosystem continues to evolve at a rapid pace. New frameworks, libraries, and tools emerge frequently, making it challenging for developers to decide what to learn. React remains the dominant UI library, while Next.js has gained tremendous popularity for full-stack development. Meanwhile, Remix is challenging established patterns, and tools like Bun are redefining how we think about JavaScript runtimes. TypeScript adoption is at an all-time high, becoming the standard for enterprise applications.' }
      ],
    });

    console.log(response.choices[0].message.content);
    return response.choices[0].message.content;
  } catch (error) {
    console.error('Error:', error.message);
    return null;
  }
}

// Example 4: Dynamic system and user prompts
async function dynamicBothOneshot() {
  printHeader('Example 4: Dynamic System and User Prompt');

  const systemContext = {
    years_experience: 8,
    programming_language: 'JavaScript',
    additional_focus_areas: 'frontend architecture and performance'
  };

  const userContext = {
    programming_language: 'JavaScript',
    code: `function fetchUserData(userId) {
  fetch('/api/users/' + userId)
    .then(response => response.json())
    .then(data => {
      renderUserProfile(data);
    })
    .catch(error => {
      console.log('Failed to fetch user data');
    });
}`,
    specific_questions: 'How can I modernize this code with async/await? Are there any error handling improvements?'
  };

  try {
    const response = await client.chat.completions.create({
      model: 'DYNAMIC-BOTH-ONESHOT',
      messages: [
        { role: 'system', content: JSON.stringify(systemContext) },
        { role: 'user', content: JSON.stringify(userContext) }
      ],
    });

    console.log(response.choices[0].message.content);
    return response.choices[0].message.content;
  } catch (error) {
    console.error('Error:', error.message);
    return null;
  }
}

// Example 5: JSON response format with dynamic system
async function jsonResponseFormat() {
  printHeader('Example 5: JSON Response Format');

  const context = {
    criteria: 'technical specs, design, software experience, and value proposition',
    include_price_analysis: true
  };

  try {
    const response = await client.chat.completions.create({
      model: 'DYNAMIC-SYSTEM-JSON',
      messages: [
        { role: 'system', content: JSON.stringify(context) },
        { role: 'user', content: 'Analyze the MacBook Air M3 laptop.' }
      ],
      response_format: { type: 'json_object' } // Request JSON format
    });

    // Pretty print the JSON response
    const jsonResponse = JSON.parse(response.choices[0].message.content);
    console.log(JSON.stringify(jsonResponse, null, 2));
    return jsonResponse;
  } catch (error) {
    console.error('Error:', error.message);
    return null;
  }
}

// Example 6: Streaming chat completion
async function streamChatCompletion() {
  printHeader('Example 6: Streaming Chat Completion');

  try {
    const stream = await client.chat.completions.create({
      model: 'STATIC-SYSTEM-CHAT',
      messages: [
        { role: 'user', content: 'Write a short story about a programmer who discovers a magical JavaScript library.' }
      ],
      stream: true,
    });

    console.log('Streaming response:');
    let fullResponse = '';

    for await (const chunk of stream) {
      const content = chunk.choices[0]?.delta?.content || '';
      if (content) {
        process.stdout.write(content);
        fullResponse += content;
      }
    }

    console.log('\n');
    return fullResponse;
  } catch (error) {
    console.error('Error:', error.message);
    return null;
  }
}

// Example 7: Function calling with JSON response
async function functionCallingExample() {
  printHeader('Example 7: Function Calling');

  // Define the functions we want the model to potentially call
  const functions = [
    {
      name: 'get_weather',
      description: 'Get the current weather for a location',
      parameters: {
        type: 'object',
        properties: {
          location: {
            type: 'string',
            description: 'The city and state or country, e.g., "San Francisco, CA" or "Paris, France"'
          },
          unit: {
            type: 'string',
            enum: ['celsius', 'fahrenheit'],
            description: 'The unit of temperature'
          }
        },
        required: ['location']
      }
    }
  ];

  try {
    const response = await client.chat.completions.create({
      model: 'STATIC-SYSTEM-CHAT', // Models that support function calling
      messages: [
        { role: 'user', content: 'What\'s the weather like in San Francisco?' }
      ],
      tools: functions.map(func => ({
        type: 'function',
        function: func
      })),
      tool_choice: 'auto'
    });

    const responseMessage = response.choices[0].message;
    console.log('Response:', responseMessage);

    // Check if the model wanted to call a function
    if (responseMessage.tool_calls) {
      console.log('Function call requested:');
      responseMessage.tool_calls.forEach(toolCall => {
        if (toolCall.type === 'function') {
          console.log(`Function: ${toolCall.function.name}`);
          console.log(`Arguments: ${toolCall.function.arguments}`);

          // In a real app, you would call the actual function here
          // and then send the result back to the model
        }
      });
    } else {
      console.log('No function call requested');
    }

    return responseMessage;
  } catch (error) {
    console.error('Error:', error.message);
    return null;
  }
}

// Example 8: Multi-turn conversation with system context
async function multiTurnWithContext() {
  printHeader('Example 8: Multi-turn with Context');

  // Context will be applied to the dynamic system prompt
  const context = {
    personality: 'helpful',
    detail_level: 'moderate',
    tone: 'enthusiastic',
    include_examples: true,
    examples: "- Here's a clear explanation of that concept...\n- I'd be happy to explain that in more detail!\n- That's a great question about JavaScript!"
  };

  try {
    // First message in conversation
    const response1 = await client.chat.completions.create({
      model: 'DYNAMIC-SYSTEM-CHAT',
      messages: [
        { role: 'system', content: JSON.stringify(context) },
        { role: 'user', content: 'Can you explain what closures are in JavaScript?' }
      ],
    });

    const assistantResponse1 = response1.choices[0].message.content;
    console.log('User: Can you explain what closures are in JavaScript?');
    console.log(`Assistant: ${assistantResponse1}\n`);

    // Second message, maintaining the same system context
    const response2 = await client.chat.completions.create({
      model: 'DYNAMIC-SYSTEM-CHAT',
      messages: [
        { role: 'system', content: JSON.stringify(context) },
        { role: 'user', content: 'Can you explain what closures are in JavaScript?' },
        { role: 'assistant', content: assistantResponse1 },
        { role: 'user', content: 'Can you give me a practical example?' }
      ],
    });

    const assistantResponse2 = response2.choices[0].message.content;
    console.log('User: Can you give me a practical example?');
    console.log(`Assistant: ${assistantResponse2}`);

    return [assistantResponse1, assistantResponse2];
  } catch (error) {
    console.error('Error:', error.message);
    return null;
  }
}

// Verification tests to confirm template variable substitution
async function verifyTemplateSubstitution() {
  printHeader('Verification Test: Template Variable Substitution');

  const context = {
    personality: 'academic',
    detail_level: 'comprehensive',
    tone: 'informative'
  };

  try {
    const response = await client.chat.completions.create({
      model: 'DYNAMIC-SYSTEM-CHAT',
      messages: [
        { role: 'system', content: JSON.stringify(context) },
        { role: 'user', content: 'Describe your personality, detail level, and tone.' }
      ],
    });

    const result = response.choices[0].message.content;

    // Check if our context variables appear in the response
    const personalityPresent = result.toLowerCase().includes('academic');
    const detailLevelPresent = result.toLowerCase().includes('comprehensive') ||
      result.toLowerCase().includes('detail');
    const tonePresent = result.toLowerCase().includes('informative');

    console.log(`✓ Personality mentioned: ${personalityPresent}`);
    console.log(`✓ Detail level mentioned: ${detailLevelPresent}`);
    console.log(`✓ Tone mentioned: ${tonePresent}`);

    if (personalityPresent && detailLevelPresent && tonePresent) {
      console.log('✅ PASS: All context variables were included in the response');
    } else {
      console.log('❌ FAIL: One or more context variables missing from response');
    }

    console.log(`\nResponse excerpt: ${result.substring(0, 150)}...\n`);

    return result;
  } catch (error) {
    console.error('Error in verification test:', error.message);
    return null;
  }
}

// Verify that conditional logic in templates works
async function verifyConditionalLogic() {
  printHeader('Verification Test: Conditional Logic');

  // Use a simpler condition to test - formal vs informal
  const contextFormal = {
    personality: 'helpful',
    formal: true
  };

  const contextInformal = {
    personality: 'helpful',
    formal: false
  };

  try {
    console.log('Testing simplified conditional logic...');

    // First request - formal tone
    const responseFormal = await client.chat.completions.create({
      model: 'DYNAMIC-SYSTEM-CHAT',
      messages: [
        { role: 'system', content: JSON.stringify(contextFormal) },
        { role: 'user', content: 'Say hello and introduce yourself.' }
      ],
    });

    // Second request - informal tone
    const responseInformal = await client.chat.completions.create({
      model: 'DYNAMIC-SYSTEM-CHAT',
      messages: [
        { role: 'system', content: JSON.stringify(contextInformal) },
        { role: 'user', content: 'Say hello and introduce yourself.' }
      ],
    });

    const resultFormal = responseFormal.choices[0].message.content;
    const resultInformal = responseInformal.choices[0].message.content;

    // The responses should differ in formality
    const responsesDifferent = resultFormal !== resultInformal;

    console.log(`✓ Responses differ: ${responsesDifferent}`);

    if (responsesDifferent) {
      console.log('✅ PASS: Conditional logic produced different responses');
    } else {
      console.log('❌ FAIL: Responses don\'t appear to differ based on condition');
    }

    console.log(`\nFormal response (len=${resultFormal.length}): ${resultFormal.substring(0, 100)}...`);
    console.log(`\nInformal response (len=${resultInformal.length}): ${resultInformal.substring(0, 100)}...`);

    return responsesDifferent;
  } catch (error) {
    console.error('Error in verification test:', error.message);
    // Print more detailed error info if available
    if (error.response) {
      console.error('Response status:', error.response.status);
      console.error('Response data:', error.response.data);
    }
    return null;
  }
}

// Verify user prompt template variables
async function verifyUserPromptTemplate() {
  printHeader('Verification Test: User Prompt Template Variables');

  const systemContext = {
    years_experience: 8,
    programming_language: 'JavaScript',
    additional_focus_areas: 'performance optimization'
  };

  const userContext = {
    programming_language: 'JavaScript',
    code: 'function sum(a, b) {\n  return a + b;\n}',
    specific_questions: 'Is this function optimized? How would you improve it?'
  };

  try {
    const response = await client.chat.completions.create({
      model: 'DYNAMIC-BOTH-ONESHOT',
      messages: [
        { role: 'system', content: JSON.stringify(systemContext) },
        { role: 'user', content: JSON.stringify(userContext) }
      ],
    });

    const result = response.choices[0].message.content;

    // Check if the response references elements from both contexts
    const hasCodeReference = result.includes('sum') || result.toLowerCase().includes('function');
    const hasQuestionReference = result.toLowerCase().includes('optimize') ||
      result.toLowerCase().includes('improve');
    const mentionsExperience = result.toLowerCase().includes('experience') ||
      result.toLowerCase().includes('years');

    console.log(`✓ References code: ${hasCodeReference}`);
    console.log(`✓ Addresses optimization question: ${hasQuestionReference}`);
    console.log(`✓ Mentions experience level: ${mentionsExperience}`);

    const success = hasCodeReference && hasQuestionReference;

    if (success) {
      console.log('✅ PASS: User template variables successfully applied');
    } else {
      console.log('❌ FAIL: User template variables not properly reflected in response');
    }

    console.log(`\nResponse excerpt: ${result.substring(0, 150)}...\n`);

    return result;
  } catch (error) {
    console.error('Error in verification test:', error.message);
    return null;
  }
}

// Run all examples
async function runAllExamples() {
  console.log('Running Node.js OpenAI Client Examples against LLMKit');

  // Run basic examples
  console.log('\n--- BASIC EXAMPLES ---');
  await basicChatCompletion();
  await dynamicSystemWithJsonContext();
  await oneshotDynamicSystem();
  await dynamicBothOneshot();
  await jsonResponseFormat();
  await streamChatCompletion();
  await functionCallingExample();
  await multiTurnWithContext();

  // Run verification tests
  console.log('\n--- VERIFICATION TESTS ---');
  await verifyTemplateSubstitution();
  await verifyConditionalLogic();
  await verifyUserPromptTemplate();

  console.log('\nAll examples and tests completed!');
}

// Main execution
runAllExamples().catch(error => {
  console.error('Error running examples:', error);
});
