/**
 * Toast notification component.
 * Uses inverseSurface/onInverseSurface colors per spec 07 section 5.
 */
import { useEffect, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface ToastProps {
  message: string;
  visible: boolean;
  onDismiss: () => void;
  duration?: number;
  action?: { label: string; onClick: () => void };
}

export function Toast({
  message,
  visible,
  onDismiss,
  duration = 3500,
  action,
}: ToastProps) {
  useEffect(() => {
    if (!visible) return;
    const timer = setTimeout(onDismiss, duration);
    return () => clearTimeout(timer);
  }, [visible, duration, onDismiss]);

  const style: CSSProperties = {
    position: "fixed",
    bottom: 24,
    left: "50%",
    transform: "translateX(-50%)",
    backgroundColor: "var(--color-inverse-surface)",
    color: "var(--color-on-inverse-surface)",
    padding: "10px 20px",
    borderRadius: 8,
    fontSize: "var(--font-body-medium)",
    zIndex: "var(--z-toast)" as string,
    display: "flex",
    alignItems: "center",
    gap: 12,
    maxWidth: 400,
    boxShadow: "var(--elevation-3)",
  };

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          style={style}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 20 }}
          transition={{ duration: 0.2 }}
        >
          <span style={{ flex: 1 }}>{message}</span>
          {action && (
            <button
              onClick={action.onClick}
              style={{
                color: "var(--color-inverse-primary)",
                fontWeight: 600,
                fontSize: "var(--font-label-large)",
                whiteSpace: "nowrap",
              }}
            >
              {action.label}
            </button>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
