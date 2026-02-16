/**
 * Setup wizard - multi-step first-run configuration.
 * Guides the user through connecting to their BlueBubbles server.
 */
import { useState, useCallback, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { useConnection } from "@/hooks/useConnection";
import { useChatStore } from "@/store/chatStore";
import { useSettingsStore } from "@/store/settingsStore";
import { tauriSyncFull, tauriCompleteSetup } from "@/hooks/useTauri";
import { LoadingSpinner, ProgressBar } from "@/components/LoadingSpinner";

type WizardStep = "welcome" | "connect" | "sync" | "theme" | "done";

export function SetupWizard() {
  const navigate = useNavigate();
  const { connect, isConnecting, error: connectError } = useConnection();
  const { fetchChats } = useChatStore();
  const { setThemeMode } = useSettingsStore();

  const [step, setStep] = useState<WizardStep>("welcome");
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  const [syncProgress, setSyncProgress] = useState(0);
  const [syncError, setSyncError] = useState<string | null>(null);

  // Step: Connect
  const handleConnect = useCallback(async () => {
    if (!address.trim() || !password.trim()) return;
    try {
      await connect(address.trim(), password.trim());
      setStep("sync");
      handleSync();
    } catch {
      // Error is handled by useConnection
    }
  }, [address, password, connect]);

  // Step: Sync
  const handleSync = useCallback(async () => {
    setSyncProgress(10);
    setSyncError(null);
    try {
      setSyncProgress(30);
      await tauriSyncFull();
      setSyncProgress(80);
      await fetchChats();
      setSyncProgress(100);
      setTimeout(() => setStep("theme"), 500);
    } catch (err) {
      setSyncError(err instanceof Error ? err.message : String(err));
    }
  }, [fetchChats]);

  // Step: Done
  const handleFinish = useCallback(async () => {
    try {
      await tauriCompleteSetup();
    } catch {
      // Non-critical
    }
    navigate("/");
  }, [navigate]);

  const pageStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    height: "100vh",
    backgroundColor: "var(--color-background)",
    padding: 32,
  };

  const cardStyle: CSSProperties = {
    backgroundColor: "var(--color-surface)",
    borderRadius: 20,
    padding: 40,
    maxWidth: 480,
    width: "100%",
    boxShadow: "var(--elevation-2)",
    display: "flex",
    flexDirection: "column",
    gap: 24,
  };

  const inputStyle: CSSProperties = {
    width: "100%",
    padding: "12px 16px",
    borderRadius: 12,
    border: "1px solid var(--color-outline)",
    fontSize: "var(--font-body-large)",
    color: "var(--color-on-surface)",
    backgroundColor: "var(--color-surface-variant)",
  };

  const primaryBtnStyle: CSSProperties = {
    width: "100%",
    padding: "12px 24px",
    borderRadius: 24,
    fontSize: "var(--font-label-large)",
    fontWeight: 600,
    backgroundColor: "var(--color-primary)",
    color: "var(--color-on-primary)",
    cursor: "pointer",
    transition: "opacity 150ms ease",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: 8,
  };

  return (
    <div style={pageStyle}>
      <AnimatePresence mode="wait">
        <motion.div
          key={step}
          style={cardStyle}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -20 }}
          transition={{ duration: 0.25 }}
        >
          {/* Welcome */}
          {step === "welcome" && (
            <>
              <div style={{ textAlign: "center" }}>
                <div style={{ fontSize: 48, marginBottom: 8 }}>{"\uD83D\uDCAC"}</div>
                <h1
                  style={{
                    fontSize: "var(--font-title-large)",
                    fontWeight: 700,
                    color: "var(--color-on-surface)",
                    marginBottom: 8,
                  }}
                >
                  Welcome to BlueBubbles
                </h1>
                <p
                  style={{
                    fontSize: "var(--font-body-medium)",
                    color: "var(--color-on-surface-variant)",
                  }}
                >
                  Send and receive iMessages from your desktop.
                  Connect to your BlueBubbles server to get started.
                </p>
              </div>
              <button
                style={primaryBtnStyle}
                onClick={() => setStep("connect")}
              >
                Get Started
              </button>
            </>
          )}

          {/* Connect */}
          {step === "connect" && (
            <>
              <div>
                <h2
                  style={{
                    fontSize: "var(--font-title-large)",
                    fontWeight: 600,
                    color: "var(--color-on-surface)",
                    marginBottom: 4,
                  }}
                >
                  Connect to Server
                </h2>
                <p
                  style={{
                    fontSize: "var(--font-body-medium)",
                    color: "var(--color-on-surface-variant)",
                  }}
                >
                  Enter your BlueBubbles server address and password.
                </p>
              </div>

              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                <input
                  style={inputStyle}
                  type="text"
                  placeholder="Server address (e.g., https://your-server.ngrok.io)"
                  value={address}
                  onChange={(e) => setAddress(e.target.value)}
                  autoFocus
                />
                <input
                  style={inputStyle}
                  type="password"
                  placeholder="Password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleConnect();
                  }}
                />
              </div>

              {connectError && (
                <div
                  style={{
                    padding: "10px 14px",
                    borderRadius: 10,
                    backgroundColor: "var(--color-error-container)",
                    color: "var(--color-error)",
                    fontSize: "var(--font-body-medium)",
                  }}
                >
                  {connectError}
                </div>
              )}

              <button
                style={{
                  ...primaryBtnStyle,
                  opacity: isConnecting || !address.trim() || !password.trim() ? 0.6 : 1,
                  pointerEvents:
                    isConnecting || !address.trim() || !password.trim() ? "none" : "auto",
                }}
                onClick={handleConnect}
              >
                {isConnecting ? (
                  <>
                    <LoadingSpinner size={18} color="var(--color-on-primary)" />
                    Connecting...
                  </>
                ) : (
                  "Connect"
                )}
              </button>
            </>
          )}

          {/* Sync */}
          {step === "sync" && (
            <>
              <div style={{ textAlign: "center" }}>
                <h2
                  style={{
                    fontSize: "var(--font-title-large)",
                    fontWeight: 600,
                    color: "var(--color-on-surface)",
                    marginBottom: 8,
                  }}
                >
                  Syncing Data
                </h2>
                <p
                  style={{
                    fontSize: "var(--font-body-medium)",
                    color: "var(--color-on-surface-variant)",
                  }}
                >
                  Downloading your messages and contacts from the server.
                  This may take a moment.
                </p>
              </div>

              <ProgressBar progress={syncProgress} height={6} />

              {syncError && (
                <div
                  style={{
                    padding: "10px 14px",
                    borderRadius: 10,
                    backgroundColor: "var(--color-error-container)",
                    color: "var(--color-error)",
                    fontSize: "var(--font-body-medium)",
                  }}
                >
                  {syncError}
                  <button
                    style={{
                      display: "block",
                      marginTop: 8,
                      color: "var(--color-primary)",
                      fontWeight: 600,
                      fontSize: "var(--font-label-large)",
                    }}
                    onClick={handleSync}
                  >
                    Retry
                  </button>
                </div>
              )}
            </>
          )}

          {/* Theme selection */}
          {step === "theme" && (
            <>
              <div>
                <h2
                  style={{
                    fontSize: "var(--font-title-large)",
                    fontWeight: 600,
                    color: "var(--color-on-surface)",
                    marginBottom: 4,
                  }}
                >
                  Choose Your Look
                </h2>
                <p
                  style={{
                    fontSize: "var(--font-body-medium)",
                    color: "var(--color-on-surface-variant)",
                  }}
                >
                  Select a theme mode. You can change this later in settings.
                </p>
              </div>

              <div style={{ display: "flex", gap: 12 }}>
                <ThemeOption
                  label="Light"
                  selected={false}
                  onClick={() => setThemeMode("light")}
                  previewBg="#FFFFFF"
                  previewFg="#1C1B1F"
                />
                <ThemeOption
                  label="Dark"
                  selected={false}
                  onClick={() => setThemeMode("dark")}
                  previewBg="#1C1B1F"
                  previewFg="#E6E1E5"
                />
                <ThemeOption
                  label="System"
                  selected={true}
                  onClick={() => setThemeMode("system")}
                  previewBg="linear-gradient(135deg, #FFFFFF 50%, #1C1B1F 50%)"
                  previewFg="#1C1B1F"
                />
              </div>

              <button
                style={primaryBtnStyle}
                onClick={() => setStep("done")}
              >
                Continue
              </button>
            </>
          )}

          {/* Done */}
          {step === "done" && (
            <>
              <div style={{ textAlign: "center" }}>
                <div style={{ fontSize: 48, marginBottom: 8 }}>{"\u2705"}</div>
                <h2
                  style={{
                    fontSize: "var(--font-title-large)",
                    fontWeight: 600,
                    color: "var(--color-on-surface)",
                    marginBottom: 8,
                  }}
                >
                  All Set!
                </h2>
                <p
                  style={{
                    fontSize: "var(--font-body-medium)",
                    color: "var(--color-on-surface-variant)",
                  }}
                >
                  BlueBubbles is ready to use. Enjoy messaging from your desktop!
                </p>
              </div>

              <button
                style={primaryBtnStyle}
                onClick={handleFinish}
              >
                Start Messaging
              </button>
            </>
          )}
        </motion.div>
      </AnimatePresence>
    </div>
  );
}

interface ThemeOptionProps {
  label: string;
  selected: boolean;
  onClick: () => void;
  previewBg: string;
  previewFg: string;
}

function ThemeOption({ label, selected, onClick, previewBg, previewFg }: ThemeOptionProps) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        flex: 1,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 8,
        padding: 12,
        borderRadius: 16,
        border: selected
          ? "2px solid var(--color-primary)"
          : "2px solid var(--color-outline)",
        backgroundColor: hovered ? "var(--color-surface-variant)" : "transparent",
        cursor: "pointer",
        transition: "all 150ms ease",
      }}
    >
      <div
        style={{
          width: 48,
          height: 48,
          borderRadius: 12,
          background: previewBg,
          border: "1px solid var(--color-outline)",
        }}
      />
      <span
        style={{
          fontSize: "var(--font-label-large)",
          fontWeight: selected ? 600 : 400,
          color: "var(--color-on-surface)",
        }}
      >
        {label}
      </span>
    </button>
  );
}
