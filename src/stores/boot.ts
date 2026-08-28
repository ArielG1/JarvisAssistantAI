import { defineStore } from "pinia";
import { ref } from "vue";
import type { BootStep } from "../types/boot";

export const useBootStore = defineStore("boot", () => {
  const steps = ref<BootStep[]>([
    { id: "llamacpp", label: "LLAMACPP", status: "pending", message: "" },
    { id: "modelo", label: "MODELO", status: "pending", message: "" },
  ]);

  const isComplete = ref(false);

  function updateStep(id: string, patch: Partial<Omit<BootStep, "id">>) {
    const step = steps.value.find((s) => s.id === id);
    if (step) {
      Object.assign(step, patch);
    }
  }

  function setFromEvent(step: BootStep) {
    const idx = steps.value.findIndex((s) => s.id === step.id);
    if (idx === -1) {
      console.debug(`[boot] Ignoring event for unknown step: "${step.id}"`);
      return;
    }
    steps.value[idx] = { ...step };
  }

  function complete() {
    isComplete.value = true;
  }

  function reset() {
    steps.value.forEach((s) => {
      s.status = "pending";
      s.message = "";
    });
    isComplete.value = false;
  }

  function getStepByIndex(index: number): BootStep | undefined {
    return steps.value[index];
  }

  function retryStep(id: string) {
    const step = steps.value.find((s) => s.id === id);
    if (step) {
      step.status = "pending";
      step.message = "";
    }
  }

  return { steps, isComplete, updateStep, setFromEvent, complete, reset, getStepByIndex, retryStep };
});
