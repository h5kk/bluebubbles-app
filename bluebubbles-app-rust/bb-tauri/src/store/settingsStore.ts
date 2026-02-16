/**
 * Settings store - manages application settings.
 */
import { create } from "zustand";
import { tauriGetSettings, tauriUpdateSetting } from "@/hooks/useTauri";
import type { ThemeMode } from "@/hooks/useTheme";

interface SettingsState {
  settings: Record<string, string>;
  loaded: boolean;
  themeMode: ThemeMode;
  selectedLightTheme: string;
  selectedDarkTheme: string;
  skin: "ios" | "material" | "samsung";
  tabletMode: boolean;
  colorfulAvatars: boolean;
  colorfulBubbles: boolean;
  sendWithReturn: boolean;

  loadSettings: () => Promise<void>;
  updateSetting: (key: string, value: string) => Promise<void>;
  setThemeMode: (mode: ThemeMode) => void;
  setSkin: (skin: "ios" | "material" | "samsung") => void;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: {},
  loaded: false,
  themeMode: "system",
  selectedLightTheme: "Bright White",
  selectedDarkTheme: "OLED Dark",
  skin: "material",
  tabletMode: true,
  colorfulAvatars: true,
  colorfulBubbles: false,
  sendWithReturn: false,

  loadSettings: async () => {
    try {
      const settings = await tauriGetSettings();
      set({
        settings,
        loaded: true,
        selectedLightTheme: settings["selected-light"] ?? "Bright White",
        selectedDarkTheme: settings["selected-dark"] ?? "OLED Dark",
        skin: (settings["skin"] as "ios" | "material" | "samsung") ?? "material",
        tabletMode: settings["tabletMode"] === "true",
        colorfulAvatars: settings["colorfulAvatars"] !== "false",
        colorfulBubbles: settings["colorfulBubbles"] === "true",
        sendWithReturn: settings["sendWithReturn"] === "true",
      });
    } catch {
      set({ loaded: true });
    }
  },

  updateSetting: async (key: string, value: string) => {
    const { settings } = get();
    set({ settings: { ...settings, [key]: value } });

    try {
      await tauriUpdateSetting(key, value);
    } catch (err) {
      console.error("failed to save setting:", key, err);
    }
  },

  setThemeMode: (mode: ThemeMode) => {
    set({ themeMode: mode });
  },

  setSkin: (skin: "ios" | "material" | "samsung") => {
    set({ skin });
    get().updateSetting("skin", skin);
  },
}));
