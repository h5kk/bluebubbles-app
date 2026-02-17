/**
 * Theme management hook.
 * Handles theme switching and persistence via the Tauri backend.
 */
import { useCallback, useEffect } from "react";
import { useSettingsStore } from "@/store/settingsStore";

export type ThemeMode = "light" | "dark" | "system";

const THEME_MAP: Record<string, string> = {
  "OLED Dark": "oled-dark",
  "Bright White": "bright-white",
  Nord: "nord",
  "Blue Light": "blue-light",
  "Blue Dark": "blue-dark",
  "Indigo Dark": "indigo-dark",
  "Pink Light": "pink-light",
  "Green Dark": "green-dark",
  "Purple Dark": "purple-dark",
  "Liquid Glass Light": "liquid-glass-light",
  "Liquid Glass Dark": "liquid-glass-dark",
};

function getSystemDarkMode(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function useTheme() {
  const { themeMode, selectedLightTheme, selectedDarkTheme, setThemeMode } =
    useSettingsStore();

  const applyTheme = useCallback(() => {
    let isDark: boolean;

    if (themeMode === "system") {
      isDark = getSystemDarkMode();
    } else {
      isDark = themeMode === "dark";
    }

    const themeName = isDark ? selectedDarkTheme : selectedLightTheme;
    const cssTheme = THEME_MAP[themeName] ?? (isDark ? "oled-dark" : "bright-white");

    document.documentElement.setAttribute("data-theme", cssTheme);
  }, [themeMode, selectedLightTheme, selectedDarkTheme]);

  useEffect(() => {
    applyTheme();

    // Listen for system theme changes
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      if (themeMode === "system") {
        applyTheme();
      }
    };
    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  }, [applyTheme, themeMode]);

  return {
    themeMode,
    setThemeMode,
    applyTheme,
  };
}
