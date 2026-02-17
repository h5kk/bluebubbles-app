/**
 * SyncScreen - Shows sync progress when pulling messages from the server.
 * Displays a blue progress bar and allows user to configure messages per conversation.
 */
import { useEffect, useState, useCallback, useRef, type CSSProperties } from "react";
import { motion } from "framer-motion";
import {
  tauriSyncMessages,
  onSyncProgress,
  onSyncComplete,
  type SyncProgress,
} from "@/hooks/useTauri";

interface SyncScreenProps {
  onComplete: () => void;
}

export function SyncScreen({ onComplete }: SyncScreenProps) {
  const [messagesPerChat, setMessagesPerChat] = useState(25);
  const [syncing, setSyncing] = useState(false);
  const [progress, setProgress] = useState<SyncProgress | null>(null);
  const [complete, setComplete] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const startedRef = useRef(false);

  // Listen for progress events
  useEffect(() => {
    let unlistenProgress: (() => void) | null = null;
    let unlistenComplete: (() => void) | null = null;

    (async () => {
      unlistenProgress = await onSyncProgress((p) => setProgress(p));
      unlistenComplete = await onSyncComplete(() => setComplete(true));
    })();

    return () => {
      unlistenProgress?.();
      unlistenComplete?.();
    };
  }, []);

  // Auto-navigate after completion
  useEffect(() => {
    if (complete) {
      const timer = setTimeout(onComplete, 1200);
      return () => clearTimeout(timer);
    }
  }, [complete, onComplete]);

  const startSync = useCallback(async () => {
    if (startedRef.current) return;
    startedRef.current = true;
    setSyncing(true);
    setError(null);
    try {
      await tauriSyncMessages(messagesPerChat);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setSyncing(false);
      startedRef.current = false;
    }
  }, [messagesPerChat]);

  const pct = progress ? (progress.current / progress.total) * 100 : 0;

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    height: "100vh",
    backgroundColor: "var(--color-background)",
    padding: 32,
    gap: 24,
  };

  const cardStyle: CSSProperties = {
    backgroundColor: "var(--color-surface)",
    borderRadius: 16,
    padding: 32,
    width: "100%",
    maxWidth: 480,
    display: "flex",
    flexDirection: "column",
    gap: 20,
    boxShadow: "0 4px 24px rgba(0,0,0,0.08)",
  };

  return (
    <div style={containerStyle}>
      <div style={cardStyle}>
        {/* Header */}
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 40, marginBottom: 8 }}>{"\uD83D\uDD04"}</div>
          <h1
            style={{
              fontSize: "var(--font-title-large)",
              fontWeight: 700,
              color: "var(--color-on-surface)",
              margin: 0,
            }}
          >
            {complete ? "Sync Complete" : "Syncing with Server"}
          </h1>
          <p
            style={{
              fontSize: "var(--font-body-medium)",
              color: "var(--color-on-surface-variant)",
              margin: "8px 0 0",
            }}
          >
            {complete
              ? `Synced ${progress?.messages_saved ?? 0} messages`
              : syncing
                ? `Syncing conversation ${progress?.current ?? 0} of ${progress?.total ?? "..."}`
                : "Configure how many messages to pull per conversation"}
          </p>
        </div>

        {/* Progress bar */}
        {syncing && (
          <div>
            <div
              style={{
                width: "100%",
                height: 8,
                borderRadius: 4,
                backgroundColor: "var(--color-surface-variant)",
                overflow: "hidden",
              }}
            >
              <motion.div
                initial={{ width: 0 }}
                animate={{ width: `${pct}%` }}
                transition={{ duration: 0.3, ease: "easeOut" }}
                style={{
                  height: "100%",
                  borderRadius: 4,
                  background: "linear-gradient(90deg, #3B82F6, #60A5FA)",
                }}
              />
            </div>
            {progress && (
              <p
                style={{
                  fontSize: "var(--font-label-small)",
                  color: "var(--color-outline)",
                  marginTop: 8,
                  textAlign: "center",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                }}
              >
                {progress.chat_name} &mdash; {progress.messages_saved} messages saved
              </p>
            )}
          </div>
        )}

        {/* Messages per chat input */}
        {!syncing && !complete && (
          <div>
            <label
              style={{
                display: "block",
                fontSize: "var(--font-label-large)",
                fontWeight: 500,
                color: "var(--color-on-surface)",
                marginBottom: 8,
              }}
            >
              Messages per conversation
            </label>
            <select
              value={messagesPerChat}
              onChange={(e) => setMessagesPerChat(Number(e.target.value))}
              style={{
                width: "100%",
                padding: "10px 12px",
                borderRadius: 8,
                border: "1px solid var(--color-outline)",
                backgroundColor: "var(--color-surface)",
                color: "var(--color-on-surface)",
                fontSize: "var(--font-body-large)",
                cursor: "pointer",
              }}
            >
              <option value={10}>10 messages</option>
              <option value={25}>25 messages (recommended)</option>
              <option value={50}>50 messages</option>
              <option value={100}>100 messages</option>
              <option value={250}>250 messages</option>
              <option value={500}>500 messages</option>
            </select>
          </div>
        )}

        {/* Error */}
        {error && (
          <p
            style={{
              fontSize: "var(--font-body-small)",
              color: "var(--color-error)",
              textAlign: "center",
              margin: 0,
            }}
          >
            {error}
          </p>
        )}

        {/* Buttons */}
        {!syncing && !complete && (
          <div style={{ display: "flex", gap: 12 }}>
            <button
              onClick={onComplete}
              style={{
                flex: 1,
                padding: "12px 16px",
                borderRadius: 12,
                fontSize: "var(--font-label-large)",
                fontWeight: 500,
                color: "var(--color-on-surface-variant)",
                backgroundColor: "var(--color-surface-variant)",
                cursor: "pointer",
                transition: "opacity 150ms",
              }}
            >
              Skip
            </button>
            <button
              onClick={startSync}
              style={{
                flex: 2,
                padding: "12px 16px",
                borderRadius: 12,
                fontSize: "var(--font-label-large)",
                fontWeight: 600,
                color: "#FFFFFF",
                background: "linear-gradient(135deg, #3B82F6, #2563EB)",
                cursor: "pointer",
                transition: "opacity 150ms",
              }}
            >
              Start Sync
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
