export interface CerebroRequest {
  query: string
}

export interface CerebroResponse {
  response: string
  status: string
}

export interface CerebroError {
  code: string
  message: string
}
