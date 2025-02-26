import { defineEventHandler, createError } from 'h3'
import { useSession } from '#imports'

export default defineEventHandler(async (event) => {
  const session = await useSession(event)
  const user = session.data.user
  
  if (!user) {
    throw createError({
      statusCode: 401,
      message: 'Not authenticated'
    })
  }
  
  return { user }
})