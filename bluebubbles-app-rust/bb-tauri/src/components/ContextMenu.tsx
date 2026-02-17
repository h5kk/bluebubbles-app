/**
 * Context menu component for right-click actions.
 */
import { useEffect, useMemo, type CSSProperties } from "react";
import { createPortal } from "react-dom";
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
  useEffect(() => {
    if (open) {
      const handler = () => onClose();
      window.addEventListener("click", handler);
      return () => {
        window.removeEventListener("click", handler);
      };
    }
  }, [open, onClose]);

  const dividerCount = useMemo(() => items.filter((i) => i.divider).length, [items]);
  const actionCount = items.length - dividerCount;
  const estimatedHeight = useMemo(
    () => actionCount * 34 + dividerCount * 9 + 8,
    [actionCount, dividerCount]
  );
  const estimatedWidth = 200;

  const clamped = useMemo(() => {
    if (typeof window === "undefined") {
      return { left: x, top: y };
    }
    const padding = 8;
    const maxX = window.innerWidth - estimatedWidth - padding;
    const maxY = window.innerHeight - estimatedHeight - padding;
    return {
      left: Math.max(padding, Math.min(x, maxX)),
      top: Math.max(padding, Math.min(y, maxY)),
    };
  }, [x, y, estimatedHeight]);

  const menuStyle: CSSProperties = {
    position: "fixed",
    top: clamped.top,
    left: clamped.left,
    backgroundColor: "var(--color-surface)",
    borderRadius: 8,
    padding: "4px 0",
    boxShadow: "var(--elevation-3)",
    zIndex: 1000,
    minWidth: 180,
    border: "1px solid var(--color-surface-variant)",
  };

  if (typeof document === "undefined") return null;

  return createPortal(
    <AnimatePresence>
      {open && (
        <motion.div
          style={menuStyle}
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          transition={{ duration: 0.1 }}
          onClick={(e) => e.stopPropagation()}
          onContextMenu={(e) => e.stopPropagation()}
          data-context-menu
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
    </AnimatePresence>,
    document.body
  );
}
