/**
 * OTP Toast Notification with iOS 26 Liquid Glass Design
 *
 * Displays one-time password codes detected from incoming messages
 * with automatic clipboard copy and iOS-style liquid glass frosted effect.
 */
import { motion, AnimatePresence } from "framer-motion";
import { useEffect, useState, type CSSProperties } from "react";

export interface OtpToastData {
  code: string;
  snippet: string;
  timestamp: number;
}

interface OtpToastProps {
  data: OtpToastData | null;
  onDismiss: () => void;
  autoDismissMs?: number;
}

export function OtpToast({ data, onDismiss, autoDismissMs = 5000 }: OtpToastProps) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!data) return;

    // Auto-copy to clipboard when OTP appears
    navigator.clipboard.writeText(data.code).then(() => {
      setCopied(true);
    }).catch(() => {
      // Silent fail if clipboard access denied
    });

    // Auto-dismiss timer
    const timer = setTimeout(() => {
      onDismiss();
    }, autoDismissMs);

    return () => clearTimeout(timer);
  }, [data, onDismiss, autoDismissMs]);

  // Reset copied state when toast disappears
  useEffect(() => {
    if (!data) {
      setCopied(false);
    }
  }, [data]);

  const containerStyle: CSSProperties = {
    position: "fixed",
    top: 20,
    left: 20,
    zIndex: 1000,
    pointerEvents: "none",
  };

  const toastStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    gap: 8,
    padding: "16px 20px",
    minWidth: 320,
    maxWidth: 400,
    borderRadius: 20,
    pointerEvents: "auto",
    // Liquid glass effect
    background: "var(--glass-bg-elevated, rgba(255, 255, 255, 0.82))",
    WebkitBackdropFilter: "blur(var(--glass-blur, 20px)) saturate(180%)",
    backdropFilter: "blur(var(--glass-blur, 20px)) saturate(180%)",
    border: "1px solid var(--glass-border, rgba(255, 255, 255, 0.18))",
    boxShadow: "var(--glass-shadow-elevated, 0 8px 32px rgba(0, 0, 0, 0.10)), inset 0 1px 0 rgba(255, 255, 255, 0.2)",
    // Fallback for non-glass themes
    backgroundColor: data ? "var(--color-surface-variant)" : "transparent",
  };

  const headerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  };

  const titleStyle: CSSProperties = {
    fontSize: "var(--font-body-large)",
    fontWeight: 600,
    color: "var(--color-on-surface)",
    letterSpacing: "-0.02em",
  };

  const closeButtonStyle: CSSProperties = {
    width: 24,
    height: 24,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    borderRadius: "50%",
    backgroundColor: "var(--glass-border-subtle, rgba(0, 0, 0, 0.06))",
    color: "var(--color-on-surface-variant)",
    cursor: "pointer",
    transition: "all 0.15s ease",
    fontSize: 16,
    fontWeight: 500,
    opacity: 0.8,
  };

  const codeContainerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 12,
    padding: "12px 16px",
    borderRadius: 12,
    background: "var(--glass-bg, rgba(255, 255, 255, 0.5))",
    border: "1px solid var(--glass-border-subtle, rgba(0, 0, 0, 0.06))",
  };

  const codeStyle: CSSProperties = {
    flex: 1,
    fontSize: 28,
    fontWeight: 700,
    fontFamily: "ui-monospace, 'SF Mono', 'Cascadia Code', monospace",
    color: "var(--color-primary)",
    letterSpacing: "0.05em",
  };

  const copiedBadgeStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 4,
    padding: "4px 10px",
    borderRadius: 8,
    backgroundColor: "var(--color-primary)",
    color: "var(--color-on-primary)",
    fontSize: "var(--font-body-small)",
    fontWeight: 600,
    whiteSpace: "nowrap",
  };

  const snippetStyle: CSSProperties = {
    fontSize: "var(--font-body-small)",
    color: "var(--color-on-surface-variant)",
    lineHeight: "1.4",
    maxHeight: 48,
    overflow: "hidden",
    textOverflow: "ellipsis",
    display: "-webkit-box",
    WebkitLineClamp: 2,
    WebkitBoxOrient: "vertical",
  };

  return (
    <div style={containerStyle}>
      <AnimatePresence>
        {data && (
          <motion.div
            initial={{ opacity: 0, x: -30, scale: 0.95 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: -30, scale: 0.95 }}
            transition={{
              type: "spring",
              stiffness: 400,
              damping: 30,
              mass: 0.8,
            }}
            style={toastStyle}
          >
            {/* Header */}
            <div style={headerStyle}>
              <div style={titleStyle}>Verification Code</div>
              <button
                onClick={onDismiss}
                style={closeButtonStyle}
                onMouseEnter={(e) => {
                  e.currentTarget.style.opacity = "1";
                  e.currentTarget.style.transform = "scale(1.1)";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.opacity = "0.8";
                  e.currentTarget.style.transform = "scale(1)";
                }}
                aria-label="Dismiss"
              >
                ×
              </button>
            </div>

            {/* Code Display */}
            <div style={codeContainerStyle}>
              <div style={codeStyle}>{data.code}</div>
              <AnimatePresence>
                {copied && (
                  <motion.div
                    initial={{ opacity: 0, scale: 0.8 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.8 }}
                    transition={{ duration: 0.2 }}
                    style={copiedBadgeStyle}
                  >
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                      <path
                        d="M10 3L4.5 8.5L2 6"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      />
                    </svg>
                    Copied
                  </motion.div>
                )}
              </AnimatePresence>
            </div>

            {/* Message Snippet */}
            {data.snippet && (
              <div style={snippetStyle}>
                {data.snippet}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
