import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useBootStore } from "../stores/boot";
import type { BootStep } from "../types/boot";

export function useBootSequence() {
  const bootStore = useBootStore();

  async function runChecks(): Promise<boolean> {
    bootStore.reset();
    console.log("[boot] Starting boot sequence...");

    const unlisten = await listen<BootStep>("boot-step", (event) => {
      console.log("[boot] Event received:", event.payload);
      bootStore.setFromEvent(event.payload);
    });

    console.log("[boot] Listener ready, invoking run_boot_sequence...");

    try {
      await invoke("run_boot_sequence");
      console.log("[boot] run_boot_sequence completed successfully");
      bootStore.complete();
      return true;
    } catch (e) {
      console.error("[boot] run_boot_sequence failed:", e);
      return false;
    } finally {
      unlisten();
    }
  }

  async function retryStep(index: number): Promise<void> {
    const step = bootStore.getStepByIndex(index);
    if (!step) return;

    const stepId = step.id;
    console.log(`[boot] Retrying step: ${stepId}`);
    bootStore.retryStep(stepId);

    const unlisten = await listen<BootStep>("boot-step", (event) => {
      console.log("[boot] Event received:", event.payload);
      bootStore.setFromEvent(event.payload);
    });

    try {
      await invoke("run_boot_step", { stepId });
      console.log(`[boot] Step ${stepId} completed successfully`);
    } catch (e) {
      console.error(`[boot] Step ${stepId} failed:`, e);
    } finally {
      unlisten();
    }
  }

  return { runChecks, retryStep, bootStore };
}
