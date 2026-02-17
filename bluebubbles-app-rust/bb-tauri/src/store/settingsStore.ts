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
  selectedDarkTheme: "Blue Dark",
  skin: "ios",
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
        selectedDarkTheme: settings["selected-dark"] ?? "Blue Dark",
        themeMode: (settings["themeMode"] as ThemeMode) ?? "system",
        skin: (settings["skin"] as "ios" | "material" | "samsung") ?? "ios",
        tabletMode: settings["tabletMode"] !== "false",
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
    const updated: Partial<SettingsState> = { settings: { ...settings, [key]: value } };

    // Sync derived state for keys that have dedicated state fields
    if (key === "selected-light") updated.selectedLightTheme = value;
    if (key === "selected-dark") updated.selectedDarkTheme = value;
    if (key === "tabletMode") updated.tabletMode = value === "true";
    if (key === "colorfulAvatars") updated.colorfulAvatars = value !== "false";
    if (key === "colorfulBubbles") updated.colorfulBubbles = value === "true";
    if (key === "sendWithReturn") updated.sendWithReturn = value === "true";

    set(updated);

    try {
      await tauriUpdateSetting(key, value);
    } catch (err) {
      console.error("failed to save setting:", key, err);
    }
  },

  setThemeMode: (mode: ThemeMode) => {
    set({ themeMode: mode });
    get().updateSetting("themeMode", mode);
  },

  setSkin: (skin: "ios" | "material" | "samsung") => {
    set({ skin });
    get().updateSetting("skin", skin);
  },
}));
