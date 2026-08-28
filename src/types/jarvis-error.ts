export type JarvisErrorType =
  | "ConnectionTimeout"
  | "ServiceUnavailable"
  | "ParseError"
  | "Unknown"

export interface JarvisError {
  type: JarvisErrorType
  detail: string
}
