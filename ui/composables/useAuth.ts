import { ref } from 'vue';

export interface User {
  id: number;
  email: string;
  name: string;
  role: string;
  registration_state: string;
}

export const useAuth = () => {
  const user = ref<User | null>(null);
  const isAuthenticated = ref(false);
  const isLoading = ref(false);
  const error = ref('');

  // Login function
  const login = async (email: string, password: string) => {
    isLoading.value = true;
    error.value = '';
    
    try {
      const response = await fetch('/v1/ui/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email,
          password
        }),
        credentials: 'include' // Important for cookies
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Login failed');
      }
      
      const data = await response.json();
      
      // Store user data
      user.value = {
        id: data.id,
        email: data.email,
        name: data.name,
        role: data.role,
        registration_state: data.registration_state
      };
      
      isAuthenticated.value = true;
      
      return data;
    } catch (err) {
      console.error('Login error:', err);
      error.value = err instanceof Error ? err.message : 'An unexpected error occurred';
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  // Register function
  const register = async (name: string, email: string, password: string, confirmPassword: string) => {
    // Validate inputs client-side
    if (password !== confirmPassword) {
      error.value = 'Passwords do not match';
      return null;
    }
    
    if (password.length < 8) {
      error.value = 'Password must be at least 8 characters long';
      return null;
    }
    
    isLoading.value = true;
    error.value = '';
    
    try {
      const response = await fetch('/v1/ui/auth/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name,
          email,
          password
        }),
        credentials: 'include'
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Registration failed');
      }
      
      const data = await response.json();
      return data;
    } catch (err) {
      console.error('Registration error:', err);
      error.value = err instanceof Error ? err.message : 'An unexpected error occurred';
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  // Logout function
  const logout = async () => {
    // Clear the auth cookie
    document.cookie = "llmkit-auth=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
    
    user.value = null;
    isAuthenticated.value = false;
    
    // Redirect to login page
    navigateTo('/login');
  };

  // Check if user is authenticated
  const checkAuth = async () => {
    isLoading.value = true;
    
    try {
      // Make a request to an endpoint that returns the current user
      const response = await fetch('/v1/ui/auth/current-user', {
        credentials: 'include'
      });
      
      if (response.ok) {
        const userData = await response.json();
        user.value = userData;
        isAuthenticated.value = true;
        return true;
      } else {
        user.value = null;
        isAuthenticated.value = false;
        return false;
      }
    } catch (err) {
      console.error('Auth check error:', err);
      user.value = null;
      isAuthenticated.value = false;
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  return {
    user,
    isAuthenticated,
    isLoading,
    error,
    login,
    register,
    logout,
    checkAuth
  };
};