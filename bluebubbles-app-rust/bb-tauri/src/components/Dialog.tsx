/**
 * Dialog/Modal component.
 * Implements the dialog pattern from spec 07 section 4.
 */
import { useEffect, useCallback, type ReactNode, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface DialogProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  children: ReactNode;
  actions?: ReactNode;
  maxWidth?: number;
}

export function Dialog({
  open,
  onClose,
  title,
  children,
  actions,
  maxWidth = 400,
}: DialogProps) {
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    },
    [onClose]
  );

  useEffect(() => {
    if (open) {
      document.addEventListener("keydown", handleKeyDown);
      return () => document.removeEventListener("keydown", handleKeyDown);
    }
  }, [open, handleKeyDown]);

  const overlayStyle: CSSProperties = {
    position: "fixed",
    inset: 0,
    backgroundColor: "rgba(0, 0, 0, 0.5)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    zIndex: "var(--z-modal)" as string,
    padding: 24,
  };

  const dialogStyle: CSSProperties = {
    backgroundColor: "var(--color-surface)",
    borderRadius: 16,
    padding: 24,
    maxWidth,
    width: "100%",
    maxHeight: "80vh",
    overflow: "auto",
    boxShadow: "var(--elevation-3)",
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          style={overlayStyle}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
          onClick={onClose}
        >
          <motion.div
            style={dialogStyle}
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.9, opacity: 0 }}
            transition={{ duration: 0.15 }}
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-label={title}
          >
            {title && (
              <h2
                style={{
                  fontSize: "var(--font-title-large)",
                  fontWeight: 600,
                  color: "var(--color-on-surface)",
                  marginBottom: 16,
                }}
              >
                {title}
              </h2>
            )}

            <div style={{ color: "var(--color-on-surface-variant)" }}>
              {children}
            </div>

            {actions && (
              <div
                style={{
                  display: "flex",
                  justifyContent: "flex-end",
                  gap: 8,
                  marginTop: 24,
                }}
              >
                {actions}
              </div>
            )}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

/** Standard dialog button styles. */
interface DialogButtonProps {
  label: string;
  onClick: () => void;
  variant?: "text" | "filled";
  disabled?: boolean;
}

export function DialogButton({
  label,
  onClick,
  variant = "text",
  disabled = false,
}: DialogButtonProps) {
  const isFilled = variant === "filled";

  const style: CSSProperties = {
    padding: "8px 20px",
    borderRadius: 20,
    fontSize: "var(--font-label-large)",
    fontWeight: 500,
    backgroundColor: isFilled ? "var(--color-primary)" : "transparent",
    color: isFilled ? "var(--color-on-primary)" : "var(--color-primary)",
    opacity: disabled ? 0.5 : 1,
    cursor: disabled ? "default" : "pointer",
    transition: "opacity 150ms ease",
  };

  return (
    <button style={style} onClick={onClick} disabled={disabled}>
      {label}
    </button>
  );
}
