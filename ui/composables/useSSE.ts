import { onUnmounted } from 'vue'
import { SSE } from 'sse.js'

type SSEOptions<T> = {
  onMessage: (data: string) => void
  onError?: (error: any) => void
}

export function useSSE() {
  let controller: AbortController | null = null

  const startStream = async <T = unknown>(
    message: T,
    endpoint: string,
    { onMessage, onError }: SSEOptions<T>
  ) => {
    try {
      var source = new SSE(endpoint, {
        headers: { 'Content-Type': 'application/json' },
        payload: JSON.stringify(message)
      });

      source.addEventListener('message', function(e: any) {
        onMessage(e.data)
      });
    } catch(e) {
      // @ts-ignore
      onError(e)
    }
  }

  const stopStream = () => {
    controller?.abort()
    controller = null
  }

  onUnmounted(stopStream)

  return { startStream, stopStream }
}
