#!/usr/bin/env python3
"""
Simple runner script for the fallback test.

This script can be run directly to test the fallback functionality:
    python run_fallback_test.py

Or use pytest for more detailed output:
    pytest simple_fallback_test.py -v -s
"""

import sys
import subprocess
from pathlib import Path

def check_dependencies():
    """Check if required dependencies are installed."""
    try:
        import pytest
        import responses
        import openai
        print("All required dependencies are available")
        return True
    except ImportError as e:
        print(f"Missing dependency: {e}")
        print("\nTo install dependencies, run:")
        print("  uv pip install pytest responses openai")
        return False

def run_with_pytest():
    """Run tests using pytest for detailed output."""
    try:
        result = subprocess.run([
            sys.executable, "-m", "pytest", 
            "simple_fallback_test.py", 
            "-v", "-s", "--tb=short"
        ], cwd=Path(__file__).parent)
        return result.returncode == 0
    except Exception as e:
        print(f"Error running pytest: {e}")
        return False

def run_directly():
    """Run the test script directly."""
    try:
        from simple_fallback_test import TestSimpleFallback
        
        print("Running Simple Fallback Test Suite")
        print("=" * 60)
        
        test_instance = TestSimpleFallback()
        
        # Setup
        test_instance.setup_method()
        print("Test setup completed")
        
        # Run tests
        tests = [
            ("OpenRouter Rate Limit → OpenAI Fallback", 
             test_instance.test_openrouter_rate_limit_fallback_to_openai),
            ("Disabled Fallback", 
             test_instance.test_fallback_disabled_no_fallback_occurs),
            ("Fallback Exhausted", 
             test_instance.test_fallback_exhausted_all_providers_fail),
            ("Configuration Validation", 
             test_instance.test_fallback_config_validation),
        ]
        
        passed = 0
        total = len(tests)
        
        for test_name, test_func in tests:
            try:
                print(f"\nRunning: {test_name}")
                test_func()
                print(f"PASSED: {test_name}")
                passed += 1
            except Exception as e:
                print(f"FAILED: {test_name}")
                print(f"   Error: {e}")
        
        print("\n" + "=" * 60)
        print(f"Test Results: {passed}/{total} tests passed")
        
        if passed == total:
            print("ALL TESTS PASSED! Fallback functionality is working correctly.")
            return True
        else:
            print("Some tests failed. Check the fallback implementation.")
            return False
            
    except Exception as e:
        print(f"Failed to run tests directly: {e}")
        return False

def main():
    print("Fallback Test Runner")
    print("This tests the OpenRouter → OpenAI fallback functionality")
    print()
    
    # Check dependencies
    if not check_dependencies():
        return 1
    
    # Try pytest first, fall back to direct execution
    print("Attempting to run with pytest...")
    if run_with_pytest():
        print("\nTests completed successfully with pytest!")
        return 0
    
    print("\nPytest failed, trying direct execution...")
    if run_directly():
        print("\nTests completed successfully!")
        return 0
    else:
        print("\nTests failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())