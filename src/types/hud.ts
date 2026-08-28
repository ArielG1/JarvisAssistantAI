import type { JarvisStatus } from "./status"
import { STATUS_COLORS, STATUS_LABELS } from "./status"

export interface LazyProcessStatus {
  name: string
  running: boolean
  healthy: boolean
  idle_secs: number
}

// Panel configuration for the HUD
// Values are placeholder examples - real data connects in later phases
export interface PanelConfig {
  id: string
  label: string
  icon: string
  metrics?: { key: string; label: string; value: string }[]
}

export const PANEL_CONFIG: PanelConfig[] = [
  {
    id: "sistema",
    label: "SISTEMA",
    icon: "🖥",
    metrics: [
      { key: "cpu", label: "CPU", value: "12%" },
      { key: "ram", label: "RAM", value: "2.1 GB" },
      { key: "gpu", label: "GPU", value: "0%" },
    ],
  },
  {
    id: "cerebro",
    label: "CEREBRO · MCP",
    icon: "🧠",
    metrics: [
      { key: "status", label: "Estado", value: "Conectado" },
      { key: "model", label: "Modelo", value: "llama3" },
      { key: "latency", label: "Latencia", value: "45ms" },
    ],
  },
  {
    id: "modulos",
    label: "MÓDULOS",
    icon: "📦",
    metrics: [
      { key: "active", label: "Activos", value: "3" },
      { key: "pending", label: "Pendientes", value: "0" },
    ],
  },
]

export function cerebroStatusLabel(status: LazyProcessStatus): string {
  if (status.running && status.healthy) return "Conectado"
  if (status.running && !status.healthy) return "Iniciando..."
  return "Apagado (a discreción)"
}

// State definitions - all 4 states with their visual properties
export interface StateConfig {
  key: JarvisStatus
  label: string
  color: string
  icon: string
  description: string
}

export const STATES: Record<JarvisStatus, StateConfig> = {
  escuchando: {
    key: "escuchando",
    label: STATUS_LABELS.escuchando,
    color: STATUS_COLORS.escuchando,
    icon: "🎤",
    description: "Esperando input del usuario",
  },
  pensando: {
    key: "pensando",
    label: STATUS_LABELS.pensando,
    color: STATUS_COLORS.pensando,
    icon: "🧠",
    description: "Procesando consulta en Cerebro",
  },
  trabajando: {
    key: "trabajando",
    label: STATUS_LABELS.trabajando,
    color: STATUS_COLORS.trabajando,
    icon: "⚡",
    description: "Ejecutando acciones",
  },
  respondiendo: {
    key: "respondiendo",
    label: STATUS_LABELS.respondiendo,
    color: STATUS_COLORS.respondiendo,
    icon: "💬",
    description: "Mostrando respuesta al usuario",
  },
}

// Ordered list of all states for iteration
export const STATE_ORDER: JarvisStatus[] = [
  "escuchando",
  "pensando",
  "trabajando",
  "respondiendo",
]
