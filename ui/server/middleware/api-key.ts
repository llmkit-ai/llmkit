import { defineEventHandler, getRequestURL, proxyRequest } from 'h3'
import { useRuntimeConfig } from '#imports'

export default defineEventHandler(async (event) => {
  const url = getRequestURL(event)
  
  // Only intercept requests to the backend API
  if (url.pathname.startsWith('/v1/')) {
    const config = useRuntimeConfig()
    const session = await useSession(event)
    
    // Check if user is authenticated
    if (!session || !session.user) {
      throw createError({
        statusCode: 401,
        statusMessage: 'Unauthorized: Authentication required'
      })
    }
    
    // Create a new headers object with the API key
    const headers = new Headers(event.node.req.headers as HeadersInit)
    headers.set('Authorization', `Bearer ${config.apiKey}`)
    
    // Add user information to headers
    headers.set('X-User-ID', session.user.id)
    
    // Proxy the request to the backend with the API key
    return proxyRequest(event, `http://127.0.0.1:8000${url.pathname}${url.search || ''}`, {
      headers
    })
  }
})