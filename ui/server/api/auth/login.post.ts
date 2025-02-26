import { readFileSync } from 'fs'
import { resolve } from 'path'
import { defineEventHandler, readBody, createError } from 'h3'
import { useSession } from '#imports'

export default defineEventHandler(async (event) => {
  // Get request body
  const { username, password } = await readBody(event)
  
  if (!username || !password) {
    throw createError({
      statusCode: 400,
      message: 'Username and password are required'
    })
  }
  
  // Read users from JSON file
  const usersPath = resolve('./server/users.json')
  const users = JSON.parse(readFileSync(usersPath, 'utf-8'))
  
  // Find user with matching credentials
  const user = users.find(u => u.username === username && u.password === password)
  
  if (!user) {
    throw createError({
      statusCode: 401,
      message: 'Invalid username or password'
    })
  }
  
  // Create session
  const session = await useSession(event)
  
  // Set session data
  await session.update({
    user: {
      id: response.user.id,
      username: response.user.username,
      name: response.user.name
    }
  })
  
  return { 
    success: true,
    user: {
      id: response.user.id,
      username: response.user.username,
      name: response.user.name
    }
  }
  } catch (error: any) {
    // Handle authentication errors
    throw createError({
      statusCode: error.response?.status || 401,
      message: error.response?.data?.message || 'Invalid username or password'
    })
  }
})