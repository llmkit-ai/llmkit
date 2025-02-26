import json
import os
from openai import OpenAI

# Initialize OpenAI client to point to our local LLMKit instance
client = OpenAI(
    api_key="",
    base_url="http://localhost:8000/v1",  # Point to the LLMKit server
)

# Example 1: Basic chat completion with system and user message
def basic_chat_completion():
    print("\n=== Example 1: Basic Chat Completion ===")
    try:
        response = client.chat.completions.create(
            model="STATIC-SYSTEM-CHAT",  # Using our static system prompt for chat
            messages=[
                {"role": "user", "content": "What's the capital of France?"},
            ],
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        print(f"An error occurred: {e}")
        return None

# Example 2: Dynamic system prompt with JSON context
def dynamic_system_with_json_context():
    print("\n=== Example 2: Dynamic System with JSON Context ===")
    # The context will be substituted into the template variables in the system prompt
    context = {
        "personality": "friendly",
        "detail_level": "concise",
        "tone": "casual"
    }
    
    try:
        response = client.chat.completions.create(
            model="DYNAMIC-SYSTEM-CHAT",
            messages=[
                # Pass JSON as the system message for template variable substitution
                {"role": "system", "content": json.dumps(context)},
                {"role": "user", "content": "Tell me about artificial intelligence."}
            ],
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        print(f"An error occurred: {e}")
        return None

# Example 3: One-shot prompt with dynamic system using JSON context
def oneshot_dynamic_system():
    print("\n=== Example 3: One-shot Dynamic System ===")
    context = {
        "content_type": "research paper",
        "word_count": 100,
        "style": "academic",
        "focus_on": "key findings"
    }
    
    try:
        response = client.chat.completions.create(
            model="DYNAMIC-SYSTEM-ONESHOT",
            messages=[
                {"role": "system", "content": json.dumps(context)},
                {"role": "user", "content": "Quantum computing has seen significant advances in recent years. Researchers at IBM have demonstrated quantum supremacy, showing that their quantum computer can solve specific problems faster than classical supercomputers. Google has also made strides with their Sycamore processor. The field continues to advance with improvements in qubit stability and error correction techniques, bringing practical quantum computing applications closer to reality."}
            ],
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        print(f"An error occurred: {e}")
        return None

# Example 4: Dynamic system and user prompt with complex template variables
def dynamic_both_oneshot():
    print("\n=== Example 4: Dynamic System and User Prompt ===")
    context = {
        "years_experience": 10,
        "programming_language": "Python",
        "additional_focus_areas": "API design and documentation"
    }
    
    # The user message will have its own template variables that need values
    user_context = {
        "programming_language": "Python",
        "code": "def fetch_data(url, timeout=30):\n    import requests\n    try:\n        response = requests.get(url, timeout=timeout)\n        response.raise_for_status()\n        return response.json()\n    except requests.RequestException as e:\n        print(f\"Error fetching data: {e}\")\n        return None",
        "specific_questions": "Is this function handling errors properly? How could we improve its reusability?"
    }
    
    try:
        response = client.chat.completions.create(
            model="DYNAMIC-BOTH-ONESHOT",
            messages=[
                {"role": "system", "content": json.dumps(context)},
                {"role": "user", "content": json.dumps(user_context)}
            ],
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        print(f"An error occurred: {e}")
        return None

# Example 5: JSON response format with dynamic system prompt
def json_response_format():
    print("\n=== Example 5: JSON Response Format ===")
    context = {
        "criteria": "price, features, market share, and user satisfaction",
        "include_price_analysis": True
    }
    
    try:
        response = client.chat.completions.create(
            model="DYNAMIC-SYSTEM-JSON",
            messages=[
                {"role": "system", "content": json.dumps(context)},
                {"role": "user", "content": "Analyze the iPhone 15 Pro smartphone."}
            ],
            response_format={"type": "json_object"}  # Request JSON response format
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        print(f"An error occurred: {e}")
        return None

# Example 6: Stream chat completion for incremental responses
def stream_chat_completion():
    print("\n=== Example 6: Streaming Chat Completion ===")
    try:
        response_stream = client.chat.completions.create(
            model="STATIC-SYSTEM-CHAT",
            messages=[
                {"role": "user", "content": "Write a short poem about coding."}
            ],
            stream=True  # Enable streaming
        )
        
        # Print each chunk as it arrives
        collected_content = []
        print("Streaming response:")
        for chunk in response_stream:
            if chunk.choices[0].delta.content:
                content = chunk.choices[0].delta.content
                print(content, end="", flush=True)
                collected_content.append(content)
        print("\n")
        return "".join(collected_content)
    except Exception as e:
        print(f"An error occurred: {e}")
        return None

# Example 7: Multi-turn conversation with chat history
def multi_turn_conversation():
    print("\n=== Example 7: Multi-turn Conversation ===")
    try:
        # First message
        response1 = client.chat.completions.create(
            model="STATIC-SYSTEM-CHAT",
            messages=[
                {"role": "user", "content": "Hi, I'm learning about machine learning. Can you explain what supervised learning is?"}
            ]
        )
        
        assistant_response1 = response1.choices[0].message.content.strip()
        print(f"User: Hi, I'm learning about machine learning. Can you explain what supervised learning is?")
        print(f"Assistant: {assistant_response1}\n")
        
        # Second message, including conversation history
        response2 = client.chat.completions.create(
            model="STATIC-SYSTEM-CHAT",
            messages=[
                {"role": "user", "content": "Hi, I'm learning about machine learning. Can you explain what supervised learning is?"},
                {"role": "assistant", "content": assistant_response1},
                {"role": "user", "content": "What's the difference between that and unsupervised learning?"}
            ]
        )
        
        assistant_response2 = response2.choices[0].message.content.strip()
        print(f"User: What's the difference between that and unsupervised learning?")
        print(f"Assistant: {assistant_response2}")
        
        return [assistant_response1, assistant_response2]
    except Exception as e:
        print(f"An error occurred: {e}")
        return None

# Add verification tests to confirm template substitution
def verify_template_substitution():
    print("\n=== Verification Test: Template Variable Substitution ===")
    
    # Test specific variables are included in the response
    context = {
        "personality": "technical",
        "detail_level": "detailed",
        "tone": "professional"
    }
    
    try:
        response = client.chat.completions.create(
            model="DYNAMIC-SYSTEM-CHAT",
            messages=[
                {"role": "system", "content": json.dumps(context)},
                {"role": "user", "content": "Describe your personality, detail level, and tone."}
            ],
        )
        
        result = response.choices[0].message.content.strip()
        
        # Check if our context variables appear in the response
        personality_present = "technical" in result.lower()
        detail_level_present = "detailed" in result.lower() or "detail" in result.lower()
        tone_present = "professional" in result.lower()
        
        print(f"✓ Personality mentioned: {personality_present}")
        print(f"✓ Detail level mentioned: {detail_level_present}")
        print(f"✓ Tone mentioned: {tone_present}")
        
        if personality_present and detail_level_present and tone_present:
            print("✅ PASS: All context variables were included in the response")
        else:
            print("❌ FAIL: One or more context variables missing from response")
        
        print(f"\nResponse excerpt: {result[:150]}...\n")
        
        return result
    except Exception as e:
        print(f"Error in verification test: {e}")
        return None

# Verify that conditional logic in templates works
def verify_conditional_logic():
    print("\n=== Verification Test: Conditional Logic ===")
    
    # Use a simpler condition - formal vs casual tone
    context_formal = {
        "personality": "helpful",
        "formal": True
    }
    
    # With formal = false
    context_casual = {
        "personality": "helpful",
        "formal": False
    }
    
    try:
        print("Testing simplified conditional logic...")
        
        # First request with formal tone
        response_formal = client.chat.completions.create(
            model="DYNAMIC-SYSTEM-CHAT",
            messages=[
                {"role": "system", "content": json.dumps(context_formal)},
                {"role": "user", "content": "Say hello and introduce yourself."}
            ],
        )
        
        # Second request with casual tone
        response_casual = client.chat.completions.create(
            model="DYNAMIC-SYSTEM-CHAT",
            messages=[
                {"role": "system", "content": json.dumps(context_casual)},
                {"role": "user", "content": "Say hello and introduce yourself."}
            ],
        )
        
        result_formal = response_formal.choices[0].message.content.strip()
        result_casual = response_casual.choices[0].message.content.strip()
        
        # The responses should differ in formality
        responses_different = result_formal != result_casual
        
        print(f"✓ Responses differ: {responses_different}")
        
        if responses_different:
            print("✅ PASS: Conditional logic produced different responses")
        else:
            print("❌ FAIL: Responses don't appear to differ based on condition")
        
        print(f"\nFormal response (len={len(result_formal)}): {result_formal[:100]}...")
        print(f"\nCasual response (len={len(result_casual)}): {result_casual[:100]}...")
        
        return responses_different
    except Exception as e:
        print(f"Error in verification test: {e}")
        return None

# Verify user prompt template variables
def verify_user_prompt_template():
    print("\n=== Verification Test: User Prompt Template Variables ===")
    
    system_context = {
        "years_experience": 10,
        "programming_language": "Python",
        "additional_focus_areas": "API design"
    }
    
    user_context = {
        "programming_language": "Python",
        "code": "def add(a, b):\n    return a + b",
        "specific_questions": "Is this function well-named?"
    }
    
    try:
        response = client.chat.completions.create(
            model="DYNAMIC-BOTH-ONESHOT",
            messages=[
                {"role": "system", "content": json.dumps(system_context)},
                {"role": "user", "content": json.dumps(user_context)}
            ],
        )
        
        result = response.choices[0].message.content.strip()
        
        # Check if the response references elements from both system and user context
        has_code_reference = "add" in result or "function" in result.lower()
        has_question_reference = "name" in result.lower() or "well-named" in result.lower()
        mentions_experience = "experience" in result.lower() or "years" in result.lower()
        
        print(f"✓ References code: {has_code_reference}")
        print(f"✓ Addresses naming question: {has_question_reference}")
        print(f"✓ Mentions experience level: {mentions_experience}")
        
        success = has_code_reference and has_question_reference
        
        if success:
            print("✅ PASS: User template variables successfully applied")
        else:
            print("❌ FAIL: User template variables not properly reflected in response")
        
        print(f"\nResponse excerpt: {result[:150]}...\n")
        
        return result
    except Exception as e:
        print(f"Error in verification test: {e}")
        return None

if __name__ == "__main__":
    print("Running Python OpenAI Client Examples against LLMKit")
    
    # Run examples
    print("\n--- BASIC EXAMPLES ---")
    basic_chat_completion()
    dynamic_system_with_json_context()
    oneshot_dynamic_system()
    dynamic_both_oneshot()
    json_response_format()
    stream_chat_completion()
    multi_turn_conversation()
    
    # Run verification tests
    print("\n--- VERIFICATION TESTS ---")
    verify_template_substitution()
    verify_conditional_logic()
    verify_user_prompt_template()
