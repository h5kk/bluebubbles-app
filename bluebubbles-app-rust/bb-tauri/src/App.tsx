/**
 * Main application component with routing.
 * Handles initial setup check and routes to appropriate pages.
 */
import { useEffect, useState, useCallback } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { useTheme } from "@/hooks/useTheme";
import { useSettingsStore } from "@/store/settingsStore";
import { tauriCheckSetupComplete } from "@/hooks/useTauri";
import { AppLayout } from "@/layouts/AppLayout";
import { SetupWizard } from "@/pages/SetupWizard";
import { ConversationView } from "@/pages/ConversationView";
import { Settings } from "@/pages/Settings";
import { FindMy } from "@/pages/FindMy";
import { ChatDetails } from "@/pages/ChatDetails";
import { LoadingSpinner } from "@/components/LoadingSpinner";

export function App() {
  // Initialize theme system
  useTheme();

  const { loadSettings, loaded: settingsLoaded } = useSettingsStore();
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);

  const checkSetup = useCallback(async () => {
    try {
      const complete = await tauriCheckSetupComplete();
      setSetupComplete(complete);
    } catch {
      // If the command fails, assume setup needed
      setSetupComplete(false);
    }
  }, []);

  // Load settings and check setup on mount
  useEffect(() => {
    loadSettings();
    checkSetup();
  }, [loadSettings, checkSetup]);

  // Show loading while checking setup
  if (setupComplete === null || !settingsLoaded) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100vh",
          backgroundColor: "var(--color-background)",
        }}
      >
        <LoadingSpinner size={32} />
      </div>
    );
  }

  return (
    <BrowserRouter>
      <Routes>
        {/* Setup wizard - shown on first run */}
        <Route
          path="/setup"
          element={
            setupComplete ? <Navigate to="/" replace /> : <SetupWizard />
          }
        />

        {/* Main app layout - requires setup complete */}
        {setupComplete ? (
          <Route element={<AppLayout />}>
            {/* Default: empty conversation view */}
            <Route index element={<ConversationView />} />

            {/* Conversation view */}
            <Route path="chat/:chatGuid" element={<ConversationView />} />

            {/* Chat details */}
            <Route path="chat/:chatGuid/details" element={<ChatDetails />} />

            {/* Settings */}
            <Route path="settings" element={<Settings />} />

            {/* Find My */}
            <Route path="findmy" element={<FindMy />} />
          </Route>
        ) : (
          <Route path="*" element={<Navigate to="/setup" replace />} />
        )}
      </Routes>
    </BrowserRouter>
  );
}
