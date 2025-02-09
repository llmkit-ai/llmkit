import { onUnmounted } from 'vue'

type SSEOptions<T> = {
  onMessage: (data: string) => void
  onError?: (error: Error) => void
  onComplete?: () => void
}

export function useSSE() {
  let controller: AbortController | null = null

  const startStream = async <T = unknown>(
    message: T,
    endpoint: string,
    { onMessage, onError, onComplete }: SSEOptions<T>
  ) => {
    controller = new AbortController()

    try {
      const res = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(message),
        signal: controller.signal
      })

      if (!res.body) throw new Error('No response body')

      const reader = res.body.getReader()
      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        if (done) {
          onComplete?.()
          break
        }
        
        buffer += decoder.decode(value, { stream: true })
        const events = buffer.split('\n\n')
        buffer = events.pop() || ''

        for (const event of events) {
          const lines = event.split("\n")

          if (lines.length === 1) {
            const data = lines[0].startsWith('data:') ? event.slice(6):''
            if (data) onMessage(data)

          } else {
            lines.forEach(l => {
              const data = l.replace('data: ', "")
              if (data === "") {
                onMessage("\n")
              } else {
                onMessage(data)
              }
            })
          }
        }
      }
    } catch (e) {
      if (e instanceof DOMException) return
      const error = e instanceof Error ? e : new Error(String(e))
      onError?.(error)
    }
  }

  const stopStream = () => {
    controller?.abort()
    controller = null
  }

  onUnmounted(stopStream)

  return { startStream, stopStream }
}
