export default defineNuxtRouteMiddleware(async () => {
  // Check authentication by making a request to the backend
  const { checkAuth, isAuthenticated } = useAuth();
  
  // Try to authenticate with backend
  await checkAuth();
  
  // If not authenticated, redirect to login
  if (!isAuthenticated.value) {
    return navigateTo('/login');
  }
});