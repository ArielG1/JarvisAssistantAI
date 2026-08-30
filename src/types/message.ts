export type MessageRole = "user" | "jarvis" | "system"

export interface MessageSource {
  type: "cerebro" | "web" | "ollama"
  url?: string
  domain?: string
}

export interface ChatMessage {
  id: string
  content: string
  role: MessageRole
  timestamp: Date
  status: "sending" | "sent" | "error"
  source?: MessageSource
}

export function createMessage(
  content: string,
  role: MessageRole,
  source?: MessageSource
): ChatMessage {
  return {
    id: crypto.randomUUID(),
    content,
    role,
    timestamp: new Date(),
    status: "sent",
    source,
  }
}
