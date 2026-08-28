import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { JarvisConfig } from "../types/config";

const defaultConfig: JarvisConfig = {
  cerebro: { base_url: "http://localhost:8080", timeout_secs: 15 },
  ollama: { model: "llama3:8b", base_url: "http://localhost:11434" },
  ui: { panels: ["chat"] },
};

export const useConfigStore = defineStore("config", () => {
  const config = ref<JarvisConfig>({ ...defaultConfig });
  const loaded = ref(false);

  async function loadConfig() {
    try {
      const result = await invoke<JarvisConfig>("load_config");
      config.value = result;
      loaded.value = true;
    } catch (e) {
      console.error("Failed to load config:", e);
      config.value = { ...defaultConfig };
      loaded.value = true;
    }
  }

  async function saveConfig() {
    try {
      await invoke("save_config", { config: config.value });
    } catch (e) {
      console.error("Failed to save config:", e);
      throw e;
    }
  }

  async function getConfigValue(key: string): Promise<string> {
    try {
      return await invoke<string>("get_config_value", { key });
    } catch (e) {
      console.error("Failed to get config value:", e);
      throw e;
    }
  }

  loadConfig();

  return { config, loaded, loadConfig, saveConfig, getConfigValue };
});
