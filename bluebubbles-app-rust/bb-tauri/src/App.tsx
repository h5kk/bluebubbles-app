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
import { useChatStore } from "@/store/chatStore";
import { AppLayout } from "@/layouts/AppLayout";
import { SetupWizard } from "@/pages/SetupWizard";
import { ConversationView } from "@/pages/ConversationView";
import { Settings } from "@/pages/Settings";
import { FindMy } from "@/pages/FindMy";
import { ChatDetails } from "@/pages/ChatDetails";
import { SyncScreen } from "@/pages/SyncScreen";
import { NewMessage } from "@/pages/NewMessage";
import { OtpDemo } from "@/pages/OtpDemo";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { OtpToastProvider } from "@/contexts/OtpToastContext";
import { OtpToast } from "@/components/OtpToast";
import { useOtpDetection } from "@/hooks/useOtpDetection";
import { useOtpToast } from "@/contexts/OtpToastContext";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Image } from "@tauri-apps/api/image";
import { resolveResource } from "@tauri-apps/api/path";

function AppContent() {
  // Initialize theme system
  useTheme();

  // Initialize OTP detection
  useOtpDetection();

  // Ensure window icon is set (avoids stale/default icon in dev)
  useEffect(() => {
    if (!(window as unknown as { __TAURI__?: unknown }).__TAURI__) return;
    const setWindowIcon = async () => {
      try {
        const iconPath = await resolveResource("icons/icon.png");
        const iconImage = await Image.fromPath(iconPath);
        await getCurrentWindow().setIcon(iconImage);
      } catch {
        // ignore icon errors (fallback to default)
      }
    };
    setWindowIcon();
  }, []);

  const { loadSettings, loaded: settingsLoaded } = useSettingsStore();
  const { status, setStatus, setServerInfo } = useConnectionStore();
  const { syncAndLoadAvatars, loadAvatars } = useContactStore();
  const refreshChats = useChatStore((s) => s.refreshChats);
  const { otpData, dismissOtp } = useOtpToast();
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
  // After sync completes, refresh chats so participant names get re-resolved
  // with the now-populated contacts table (contacts are linked to handles
  // during sync, enabling name resolution via contact_id JOIN).
  useEffect(() => {
    if (status === "connected") {
      // Load cached avatars from local DB immediately
      loadAvatars().then(() => {
        // Then sync fresh avatars from server in background
        syncAndLoadAvatars().then(() => {
          // Re-fetch chats so participant_names are re-resolved
          // now that contacts are saved and linked to handles
          refreshChats();
        });
      });
    }
  }, [status, loadAvatars, syncAndLoadAvatars, refreshChats]);

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
    <>
      {/* OTP Toast Notification */}
      <OtpToast data={otpData} onDismiss={dismissOtp} />

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

              {/* OTP Demo */}
              <Route path="otp-demo" element={<OtpDemo />} />
            </Route>
          ) : (
            <Route path="*" element={<Navigate to="/setup" replace />} />
          )}
        </Routes>
      </BrowserRouter>
    </>
  );
}

export function App() {
  return (
    <OtpToastProvider>
      <AppContent />
    </OtpToastProvider>
  );
}
