export type BootStepStatus = "not_started" | "pending" | "running" | "ok" | "error";

export interface BootStep {
  id: string;
  label: string;
  status: BootStepStatus;
  message: string;
}
