import { useErrorsStore, type AppError } from "@/stores/errors"
import type { JarvisError } from "@/types/jarvis-error"

const ERROR_MESSAGES: Record<string, (ctx?: string) => string> = {
  ConnectionTimeout: (service) =>
    `No se pudo conectar a ${service ?? "el servicio"}. Verifica que esté corriendo.`,
  ServiceUnavailable: (service) =>
    `${service ?? "El servicio"} no está disponible temporalmente.`,
  ParseError: () => "Error al procesar la respuesta.",
  Unknown: () => "Error inesperado. Consulta el log para más detalles.",
}

export function useErrorHandler() {
  const store = useErrorsStore()

  function handleError(error: unknown, context?: string): string {
    let userMessage: string

    if (
      typeof error === "object" &&
      error !== null &&
      "type" in error &&
      typeof (error as JarvisError).type === "string"
    ) {
      const jarvisErr = error as JarvisError
      const msgFn = ERROR_MESSAGES[jarvisErr.type] ?? ERROR_MESSAGES.Unknown
      userMessage = msgFn(jarvisErr.detail)
    } else if (error instanceof Error) {
      userMessage = classifyErrorMessage(error.message)
    } else {
      userMessage = ERROR_MESSAGES.Unknown()
    }

    const type = classifyErrorType(error)
    store.addError(userMessage, type, context, type !== "error")

    return userMessage
  }

  function classifyErrorMessage(msg: string): string {
    if (msg.includes("timeout") || msg.includes("Timeout")) {
      return ERROR_MESSAGES.ConnectionTimeout()
    }
    if (msg.includes("no disponible") || msg.includes("unavailable")) {
      return ERROR_MESSAGES.ServiceUnavailable()
    }
    if (msg.includes("parse") || msg.includes("Parse")) {
      return ERROR_MESSAGES.ParseError()
    }
    return msg || ERROR_MESSAGES.Unknown()
  }

  function classifyErrorType(error: unknown): AppError["type"] {
    if (
      typeof error === "object" &&
      error !== null &&
      "type" in error
    ) {
      const type = (error as { type: string }).type
      if (type === "ServiceUnavailable" || type === "ParseError") {
        return "warning"
      }
    }
    if (error instanceof Error) {
      const msg = error.message.toLowerCase()
      if (msg.includes("timeout") || msg.includes("connect")) {
        return "warning"
      }
    }
    return "error"
  }

  async function wrapAsync<T>(
    fn: () => Promise<T>,
    context?: string,
  ): Promise<T | undefined> {
    try {
      return await fn()
    } catch (e) {
      handleError(e, context)
      return undefined
    }
  }

  async function retry<T>(
    fn: () => Promise<T>,
    maxRetries = 3,
    delayMs = 1000,
  ): Promise<T> {
    let lastError: unknown
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await fn()
      } catch (e) {
        lastError = e
        if (attempt < maxRetries) {
          const delay = delayMs * Math.pow(2, attempt)
          await new Promise((r) => setTimeout(r, delay))
        }
      }
    }
    throw lastError
  }

  return { handleError, wrapAsync, retry }
}
