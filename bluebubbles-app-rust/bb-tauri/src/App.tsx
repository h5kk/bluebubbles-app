/**
 * Main application component with routing.
 * Handles initial setup check, sync screen, and routes to appropriate pages.
 */
import { useEffect, useState, useCallback } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { useTheme } from "@/hooks/useTheme";
import { useSettingsStore } from "@/store/settingsStore";
import { tauriCheckSetupComplete, tauriTryAutoConnect, tauriCheckMessagesSynced } from "@/hooks/useTauri";
import { useConnectionStore } from "@/store/connectionStore";
import { useContactStore } from "@/store/contactStore";
import { AppLayout } from "@/layouts/AppLayout";
import { SetupWizard } from "@/pages/SetupWizard";
import { ConversationView } from "@/pages/ConversationView";
import { Settings } from "@/pages/Settings";
import { FindMy } from "@/pages/FindMy";
import { ChatDetails } from "@/pages/ChatDetails";
import { SyncScreen } from "@/pages/SyncScreen";
import { NewMessage } from "@/pages/NewMessage";
import { LoadingSpinner } from "@/components/LoadingSpinner";

export function App() {
  // Initialize theme system
  useTheme();

  const { loadSettings, loaded: settingsLoaded } = useSettingsStore();
  const { status, setStatus, setServerInfo } = useConnectionStore();
  const { syncAndLoadAvatars, loadAvatars } = useContactStore();
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);
  const [messagesSynced, setMessagesSynced] = useState<boolean | null>(null);

  const checkSetup = useCallback(async () => {
    try {
      const complete = await tauriCheckSetupComplete();
      setSetupComplete(complete);
    } catch {
      setSetupComplete(false);
    }
  }, []);

  // Auto-reconnect using saved credentials when setup is complete
  const autoConnect = useCallback(async () => {
    try {
      setStatus("connecting");
      const info = await tauriTryAutoConnect();
      if (info) {
        setServerInfo(info);
        setStatus("connected");
      } else {
        setStatus("disconnected");
      }
    } catch {
      setStatus("disconnected");
    }
  }, [setStatus, setServerInfo]);

  // Check if messages have been synced
  const checkSync = useCallback(async () => {
    try {
      const synced = await tauriCheckMessagesSynced();
      setMessagesSynced(synced);
    } catch {
      setMessagesSynced(false);
    }
  }, []);

  // Load settings and check setup on mount
  useEffect(() => {
    loadSettings();
    checkSetup();
  }, [loadSettings, checkSetup]);

  // Auto-reconnect when setup is complete
  useEffect(() => {
    if (setupComplete) {
      autoConnect();
    }
  }, [setupComplete, autoConnect]);

  // Check sync status after connecting
  useEffect(() => {
    if (status === "connected") {
      checkSync();
    }
  }, [status, checkSync]);

  // Load contact avatars after connection.
  // First load from local DB (instant), then sync from server in background.
  useEffect(() => {
    if (status === "connected") {
      // Load cached avatars from local DB immediately
      loadAvatars().then(() => {
        // Then sync fresh avatars from server in background
        syncAndLoadAvatars();
      });
    }
  }, [status, loadAvatars, syncAndLoadAvatars]);

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

  // Show sync screen if connected but messages not yet synced
  if (setupComplete && status === "connected" && messagesSynced === false) {
    return (
      <SyncScreen
        onComplete={() => setMessagesSynced(true)}
      />
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

            {/* New message */}
            <Route path="new" element={<NewMessage />} />

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
