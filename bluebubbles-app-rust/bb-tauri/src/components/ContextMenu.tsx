/**
 * Context menu component for right-click actions.
 */
import { useEffect, useCallback, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";

export interface ContextMenuItem {
  label: string;
  icon?: string;
  onClick: () => void;
  destructive?: boolean;
  divider?: boolean;
}

interface ContextMenuProps {
  open: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

export function ContextMenu({ open, x, y, items, onClose }: ContextMenuProps) {
  const handleClick = useCallback(() => {
    onClose();
  }, [onClose]);

  useEffect(() => {
    if (open) {
      const handler = () => onClose();
      window.addEventListener("click", handler);
      window.addEventListener("contextmenu", handler);
      return () => {
        window.removeEventListener("click", handler);
        window.removeEventListener("contextmenu", handler);
      };
    }
  }, [open, onClose]);

  const menuStyle: CSSProperties = {
    position: "fixed",
    top: y,
    left: x,
    backgroundColor: "var(--color-surface)",
    borderRadius: 8,
    padding: "4px 0",
    boxShadow: "var(--elevation-3)",
    zIndex: "var(--z-dropdown)" as string,
    minWidth: 180,
    border: "1px solid var(--color-surface-variant)",
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          style={menuStyle}
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          transition={{ duration: 0.1 }}
          onClick={(e) => e.stopPropagation()}
        >
          {items.map((item, i) =>
            item.divider ? (
              <div
                key={`divider-${i}`}
                style={{
                  height: 1,
                  backgroundColor: "var(--color-surface-variant)",
                  margin: "4px 0",
                }}
              />
            ) : (
              <button
                key={item.label}
                onClick={() => {
                  item.onClick();
                  onClose();
                }}
                style={{
                  width: "100%",
                  textAlign: "left",
                  padding: "8px 16px",
                  fontSize: "var(--font-body-medium)",
                  color: item.destructive
                    ? "var(--color-error)"
                    : "var(--color-on-surface)",
                  display: "flex",
                  alignItems: "center",
                  gap: 10,
                  transition: "background-color 100ms",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor =
                    "var(--color-surface-variant)";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = "transparent";
                }}
              >
                {item.icon && <span>{item.icon}</span>}
                <span>{item.label}</span>
              </button>
            )
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
