import { defineEventHandler, getRequestURL, proxyRequest } from 'h3'
import { useRuntimeConfig } from '#imports'

export default defineEventHandler(async (event) => {
  const url = getRequestURL(event)
  
  // Only intercept requests to the backend API
  if (url.pathname.startsWith('/v1/')) {
    const config = useRuntimeConfig()
    
    // Create a new headers object with the API key
    const headers = new Headers(event.node.req.headers as HeadersInit)
    headers.set('Authorization', `Bearer ${config.apiKey}`)
    
    // Proxy the request to the backend with the API key
    return proxyRequest(event, `http://127.0.0.1:8000${url.pathname}${url.search || ''}`, {
      headers
    })
  }
})