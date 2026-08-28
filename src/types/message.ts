export type MessageRole = "user" | "jarvis" | "system"

export interface ChatMessage {
  id: string
  content: string
  role: MessageRole
  timestamp: Date
  status: "sending" | "sent" | "error"
}

export function createMessage(content: string, role: MessageRole): ChatMessage {
  return {
    id: crypto.randomUUID(),
    content,
    role,
    timestamp: new Date(),
    status: "sent",
  }
}
