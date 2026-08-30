import { inject, provide } from "vue";
import { useConfigStore } from "../stores/config";

export type ConfigStore = ReturnType<typeof useConfigStore>;

const CONFIG_KEY = Symbol("config");

export function provideConfig() {
  const store = useConfigStore();
  provide(CONFIG_KEY, store);
  return store;
}

export function injectConfig(): ConfigStore {
  const store = inject<ConfigStore>(CONFIG_KEY);
  if (!store) {
    throw new Error("Config store not provided. Wrap component with provideConfig().");
  }
  return store;
}

export function getCerebroUrl(): string {
  const store = useConfigStore();
  return store.config.cerebro.base_url;
}

export function getOllamaModel(): string {
  const store = useConfigStore();
  return store.config.llm.model_path;
}

export function getActivePanels(): string[] {
  const store = useConfigStore();
  return store.config.ui.panels;
}
