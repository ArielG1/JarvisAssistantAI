export type JarvisStatus = "escuchando" | "pensando" | "trabajando" | "respondiendo"

export const STATUS_COLORS: Record<JarvisStatus, string> = {
  escuchando: "#00d4ff",
  pensando: "#8b5cf6",
  trabajando: "#f59e0b",
  respondiendo: "#10b981",
}

export const STATUS_LABELS: Record<JarvisStatus, string> = {
  escuchando: "Escuchando",
  pensando: "Pensando",
  trabajando: "Trabajando",
  respondiendo: "Respondiendo",
}
