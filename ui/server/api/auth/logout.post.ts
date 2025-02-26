import { defineEventHandler } from 'h3'
import { useSession } from '#imports'

export default defineEventHandler(async (event) => {
  const session = await useSession(event)
  
  // Clear session
  await session.clear()
  
  return { success: true }
})