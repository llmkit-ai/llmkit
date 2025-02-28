export default defineEventHandler(async (event) => {
  // Clear the auth cookie
  setCookie(event, 'llmkit_auth_token', '', {
    httpOnly: true,
    secure: true,
    path: '/',
    maxAge: 0, // Setting maxAge to 0 will cause the cookie to be deleted
    sameSite: 'strict'
  })

  return { success: true }
})