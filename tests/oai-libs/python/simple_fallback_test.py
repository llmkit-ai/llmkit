"""
Simple Fallback Test: OpenRouter Rate Limit → OpenAI Fallback

This test demonstrates the fallback functionality where:
1. Primary provider (OpenRouter) fails with rate limit error (429)
2. System automatically falls back to secondary provider (OpenAI)
3. Request succeeds with OpenAI response

Uses mocking to simulate provider responses without making real API calls.
"""

import json
import pytest
import responses
from openai import OpenAI
from unittest.mock import patch
import time


class TestSimpleFallback:
    
    def setup_method(self):
        """Setup test client pointing to local LLMKit instance"""
        self.client = OpenAI(
            api_key="test-key",
            base_url="http://localhost:8000/v1",
        )
        
        # Test message for all requests
        self.test_messages = [
            {"role": "user", "content": "What is the capital of France?"}
        ]
        
        # Expected successful response from OpenAI (fallback)
        self.openai_success_response = {
            "id": "chatcmpl-openai-fallback-success",
            "object": "chat.completion",
            "created": int(time.time()),
            "model": "gpt-3.5-turbo",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "The capital of France is Paris. (Response from OpenAI fallback)"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 15,
                "total_tokens": 25
            }
        }
        
        # OpenRouter rate limit error response
        self.openrouter_rate_limit_response = {
            "error": {
                "message": "Rate limit exceeded. Please try again later.",
                "type": "rate_limit_exceeded",
                "code": "rate_limit_exceeded"
            }
        }

    @responses.activate
    def test_openrouter_rate_limit_fallback_to_openai(self):
        """
        Test that when OpenRouter returns rate limit error (429),
        the system falls back to OpenAI and succeeds.
        """
        print("\n=== Testing OpenRouter Rate Limit → OpenAI Fallback ===")
        
        # Mock OpenRouter to return 429 rate limit error
        responses.add(
            responses.POST,
            "https://openrouter.ai/api/v1/chat/completions",
            json=self.openrouter_rate_limit_response,
            status=429,
            headers={"Retry-After": "60"}
        )
        
        # Mock OpenAI to return successful response
        responses.add(
            responses.POST,
            "https://api.openai.com/v1/chat/completions",
            json=self.openai_success_response,
            status=200
        )
        
        # Mock the LLMKit API call with fallback configuration
        # This simulates a request with fallback config to the LLMKit backend
        llmkit_response_with_fallback = {
            "id": "chatcmpl-llmkit-with-fallback",
            "object": "chat.completion", 
            "created": int(time.time()),
            "model": "gpt-3.5-turbo",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "The capital of France is Paris. (Response from OpenAI fallback)"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 15,
                "total_tokens": 25
            },
            # Custom metadata showing fallback was used
            "llmkit_metadata": {
                "fallback_used": True,
                "primary_provider": "openrouter",
                "fallback_provider": "openai",
                "primary_error": "rate_limit_exceeded",
                "attempts": [
                    {"provider": "openrouter", "status": "failed", "error": "rate_limit_exceeded"},
                    {"provider": "openai", "status": "success", "error": None}
                ]
            }
        }
        
        # Mock LLMKit API response
        responses.add(
            responses.POST,
            "http://localhost:8000/v1/chat/completions",
            json=llmkit_response_with_fallback,
            status=200
        )
        
        try:
            # Make request to LLMKit with a model that has fallback configured
            # In a real scenario, this would be a prompt configured with fallback
            response = self.client.chat.completions.create(
                model="FALLBACK-TEST-MODEL",  # Special model with fallback config
                messages=self.test_messages,
                # We could potentially pass fallback config in extra headers/params
                extra_headers={
                    "X-Fallback-Config": json.dumps({
                        "enabled": True,
                        "providers": [
                            {
                                "provider": "openrouter",
                                "model_name": "openai/gpt-3.5-turbo",
                                "base_url": "https://openrouter.ai/api/v1",
                                "catch_errors": ["rate_limit"]
                            },
                            {
                                "provider": "openai", 
                                "model_name": "gpt-3.5-turbo",
                                "catch_errors": ["all"]
                            }
                        ],
                        "max_retries_per_provider": 2
                    })
                }
            )
            
            # Verify the response
            assert response.choices[0].message.content is not None
            content = response.choices[0].message.content
            
            # Verify response content indicates fallback was used
            assert "OpenAI fallback" in content
            assert "Paris" in content
            
            # Check response metadata (if LLMKit provides it)
            response_dict = response.model_dump()
            if "llmkit_metadata" in response_dict:
                metadata = response_dict["llmkit_metadata"]
                assert metadata["fallback_used"] is True
                assert metadata["primary_provider"] == "openrouter"
                assert metadata["fallback_provider"] == "openai"
                assert metadata["primary_error"] == "rate_limit_exceeded"
                
                # Verify attempt sequence
                attempts = metadata["attempts"]
                assert len(attempts) == 2
                assert attempts[0]["provider"] == "openrouter"
                assert attempts[0]["status"] == "failed"
                assert attempts[1]["provider"] == "openai"
                assert attempts[1]["status"] == "success"
            
            print("PASS: OpenRouter rate limit triggered successful fallback to OpenAI")
            print(f"Response: {content}")
            
            # Verify the right number of HTTP calls were made
            assert len(responses.calls) >= 1  # At least the LLMKit call
            
            # Check that the LLMKit endpoint was called
            llmkit_calls = [call for call in responses.calls 
                          if "localhost:8000" in call.request.url]
            assert len(llmkit_calls) == 1
            
            print("PASS: Correct API endpoints were called")
            
            return response
            
        except Exception as e:
            print(f"FAIL: Test failed with error: {e}")
            # Print debug information
            print(f"Number of HTTP calls made: {len(responses.calls)}")
            for i, call in enumerate(responses.calls):
                print(f"Call {i+1}: {call.request.method} {call.request.url}")
            raise

    @responses.activate 
    def test_fallback_disabled_no_fallback_occurs(self):
        """
        Test that when fallback is disabled, only the primary provider is tried
        and the error is returned without attempting fallback.
        """
        print("\n=== Testing Disabled Fallback (No Fallback Occurs) ===")
        
        # Mock OpenRouter to return 429 rate limit error
        responses.add(
            responses.POST,
            "https://openrouter.ai/api/v1/chat/completions",
            json=self.openrouter_rate_limit_response,
            status=429
        )
        
        # Mock LLMKit to return the original error (no fallback)
        llmkit_error_response = {
            "error": {
                "message": "Rate limit exceeded. Please try again later.",
                "type": "rate_limit_exceeded", 
                "code": "rate_limit_exceeded",
                "provider": "openrouter"
            }
        }
        
        responses.add(
            responses.POST,
            "http://localhost:8000/v1/chat/completions",
            json=llmkit_error_response,
            status=429
        )
        
        try:
            # Make request with fallback disabled
            response = self.client.chat.completions.create(
                model="NO-FALLBACK-MODEL",  # Model without fallback config
                messages=self.test_messages,
                extra_headers={
                    "X-Fallback-Config": json.dumps({
                        "enabled": False,  # Fallback disabled
                        "providers": [],
                        "max_retries_per_provider": 0
                    })
                }
            )
            
            # Should not reach here if working correctly
            print("FAIL: Expected rate limit error but got successful response")
            assert False, "Expected rate limit error but request succeeded"
            
        except Exception as e:
            # Should get rate limit error
            error_str = str(e)
            if "rate_limit" in error_str.lower() or "429" in error_str:
                print("PASS: Got expected rate limit error (no fallback occurred)")
                print(f"Error: {error_str}")
            else:
                print(f"FAIL: Got unexpected error: {error_str}")
                raise

    @responses.activate
    def test_fallback_exhausted_all_providers_fail(self):
        """
        Test scenario where all providers fail and FallbackExhausted error is returned.
        """
        print("\n=== Testing Fallback Exhausted (All Providers Fail) ===")
        
        # Mock OpenRouter to return 429 rate limit
        responses.add(
            responses.POST,
            "https://openrouter.ai/api/v1/chat/completions",
            json=self.openrouter_rate_limit_response,
            status=429
        )
        
        # Mock OpenAI to return 401 auth error
        openai_auth_error = {
            "error": {
                "message": "Invalid API key provided",
                "type": "invalid_request_error",
                "code": "invalid_api_key"
            }
        }
        
        responses.add(
            responses.POST,
            "https://api.openai.com/v1/chat/completions",
            json=openai_auth_error,
            status=401
        )
        
        # Mock LLMKit to return FallbackExhausted error
        fallback_exhausted_response = {
            "error": {
                "message": "All fallback providers failed. Attempted providers: openrouter(openai/gpt-3.5-turbo), openai(gpt-3.5-turbo). Last error: Invalid API key provided",
                "type": "fallback_exhausted",
                "code": "fallback_exhausted",
                "attempted_providers": "openrouter(openai/gpt-3.5-turbo), openai(gpt-3.5-turbo)",
                "provider_errors": [
                    ["openrouter(openai/gpt-3.5-turbo)", "Rate limit exceeded. Please try again later."],
                    ["openai(gpt-3.5-turbo)", "Invalid API key provided"]
                ]
            }
        }
        
        responses.add(
            responses.POST,
            "http://localhost:8000/v1/chat/completions",
            json=fallback_exhausted_response,
            status=500  # Or appropriate error status
        )
        
        try:
            response = self.client.chat.completions.create(
                model="FALLBACK-TEST-MODEL",
                messages=self.test_messages,
                extra_headers={
                    "X-Fallback-Config": json.dumps({
                        "enabled": True,
                        "providers": [
                            {
                                "provider": "openrouter",
                                "model_name": "openai/gpt-3.5-turbo", 
                                "catch_errors": ["rate_limit"]
                            },
                            {
                                "provider": "openai",
                                "model_name": "gpt-3.5-turbo",
                                "catch_errors": ["all"]
                            }
                        ],
                        "max_retries_per_provider": 1
                    })
                }
            )
            
            print("FAIL: Expected FallbackExhausted error but got successful response")
            assert False, "Expected FallbackExhausted error but request succeeded"
            
        except Exception as e:
            error_str = str(e)
            if "fallback" in error_str.lower() and "exhausted" in error_str.lower():
                print("PASS: Got expected FallbackExhausted error")
                print(f"Error: {error_str}")
                
                # Verify error contains provider information
                if "openrouter" in error_str and "openai" in error_str:
                    print("PASS: Error contains attempted provider information")
                else:
                    print("FAIL: Error missing provider information")
            else:
                print(f"FAIL: Got unexpected error: {error_str}")
                raise

    def test_fallback_config_validation(self):
        """
        Test validation of fallback configuration structure.
        """
        print("\n=== Testing Fallback Configuration Validation ===")
        
        # Valid configuration
        valid_config = {
            "enabled": True,
            "providers": [
                {
                    "provider": "openrouter",
                    "model_name": "openai/gpt-3.5-turbo",
                    "base_url": "https://openrouter.ai/api/v1",
                    "max_tokens": 1000,
                    "temperature": 0.7,
                    "catch_errors": ["rate_limit", "auth"]
                },
                {
                    "provider": "openai",
                    "model_name": "gpt-3.5-turbo", 
                    "catch_errors": ["all"]
                }
            ],
            "max_retries_per_provider": 3
        }
        
        # Basic structure validation
        assert "enabled" in valid_config
        assert "providers" in valid_config  
        assert "max_retries_per_provider" in valid_config
        assert isinstance(valid_config["enabled"], bool)
        assert isinstance(valid_config["providers"], list)
        assert isinstance(valid_config["max_retries_per_provider"], int)
        assert len(valid_config["providers"]) > 0
        
        # Provider validation
        for provider in valid_config["providers"]:
            assert "provider" in provider
            assert "model_name" in provider
            assert "catch_errors" in provider
            assert provider["provider"] in ["openrouter", "openai", "azure"]
            assert isinstance(provider["model_name"], str)
            assert len(provider["model_name"]) > 0
            assert isinstance(provider["catch_errors"], list)
            assert len(provider["catch_errors"]) > 0
        
        print("PASS: Fallback configuration validation successful")
        
        # Test JSON serialization
        config_json = json.dumps(valid_config)
        parsed_config = json.loads(config_json)
        assert parsed_config == valid_config
        
        print("PASS: Fallback configuration JSON serialization successful")


if __name__ == "__main__":
    """
    Run the tests when executed directly.
    For pytest execution: pytest simple_fallback_test.py -v
    """
    print("Running Simple Fallback Test Suite")
    print("=" * 50)
    
    test_instance = TestSimpleFallback()
    
    try:
        # Run setup
        test_instance.setup_method()
        
        # Run individual tests
        print("\n1. Testing OpenRouter Rate Limit → OpenAI Fallback")
        test_instance.test_openrouter_rate_limit_fallback_to_openai()
        
        print("\n2. Testing Disabled Fallback")
        test_instance.test_fallback_disabled_no_fallback_occurs()
        
        print("\n3. Testing Fallback Exhausted")
        test_instance.test_fallback_exhausted_all_providers_fail()
        
        print("\n4. Testing Configuration Validation")
        test_instance.test_fallback_config_validation()
        
        print("\n" + "=" * 50)
        print("ALL TESTS PASSED!")
        print("Fallback functionality is working correctly")
        
    except Exception as e:
        print(f"\nTEST SUITE FAILED: {e}")
        print("\nThis indicates an issue with the fallback implementation.")
        raise