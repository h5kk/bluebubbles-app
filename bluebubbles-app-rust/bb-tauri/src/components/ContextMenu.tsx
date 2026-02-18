/**
 * Context menu component for right-click actions.
 * Supports an optional tapback row at the top (iMessage-style reactions).
 */
import { useState, useEffect, useMemo, type CSSProperties } from "react";
import { createPortal } from "react-dom";
import { motion, AnimatePresence } from "framer-motion";

export interface ContextMenuItem {
  label: string;
  icon?: string;
  onClick: () => void;
  destructive?: boolean;
  divider?: boolean;
}

export interface TapbackOption {
  emoji: string;
  name: string;
  active?: boolean;
}

interface ContextMenuProps {
  open: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
  tapbacks?: TapbackOption[];
  onTapback?: (name: string) => void;
}

export function ContextMenu({ open, x, y, items, onClose, tapbacks, onTapback }: ContextMenuProps) {
  const [hoveredTapback, setHoveredTapback] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      const handler = () => onClose();
      window.addEventListener("click", handler);
      return () => {
        window.removeEventListener("click", handler);
      };
    }
  }, [open, onClose]);

  // Reset hover state when menu closes
  useEffect(() => {
    if (!open) setHoveredTapback(null);
  }, [open]);

  const dividerCount = useMemo(() => items.filter((i) => i.divider).length, [items]);
  const actionCount = items.length - dividerCount;
  const tapbackRowHeight = tapbacks?.length ? 52 : 0;
  const estimatedHeight = useMemo(
    () => actionCount * 34 + dividerCount * 9 + 8 + tapbackRowHeight,
    [actionCount, dividerCount, tapbackRowHeight]
  );
  const estimatedWidth = 220;

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
    borderRadius: 12,
    padding: "4px 0",
    boxShadow: "0 8px 32px rgba(0,0,0,0.18), 0 2px 8px rgba(0,0,0,0.08)",
    zIndex: 1000,
    minWidth: 200,
    border: "1px solid var(--color-surface-variant)",
    backdropFilter: "blur(20px)",
    WebkitBackdropFilter: "blur(20px)",
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
          transition={{ duration: 0.12 }}
          onClick={(e) => e.stopPropagation()}
          onContextMenu={(e) => e.stopPropagation()}
          data-context-menu
        >
          {/* Tapback row */}
          {tapbacks && tapbacks.length > 0 && onTapback && (
            <>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: 2,
                  padding: "6px 8px",
                }}
              >
                {tapbacks.map((tb) => (
                  <button
                    key={tb.name}
                    onClick={() => {
                      onTapback(tb.name);
                      onClose();
                    }}
                    onMouseEnter={() => setHoveredTapback(tb.name)}
                    onMouseLeave={() => setHoveredTapback(null)}
                    style={{
                      width: 36,
                      height: 36,
                      borderRadius: "50%",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      fontSize: 22,
                      cursor: "pointer",
                      backgroundColor: tb.active
                        ? "var(--color-primary-container)"
                        : hoveredTapback === tb.name
                          ? "var(--color-surface-variant)"
                          : "transparent",
                      border: "none",
                      transition: "all 120ms ease",
                      transform: hoveredTapback === tb.name ? "scale(1.25)" : "scale(1)",
                    }}
                    title={tb.name.charAt(0).toUpperCase() + tb.name.slice(1)}
                  >
                    {tb.emoji}
                  </button>
                ))}
              </div>
              <div
                style={{
                  height: 1,
                  backgroundColor: "var(--color-surface-variant)",
                  margin: "2px 0",
                }}
              />
            </>
          )}

          {/* Menu items */}
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
                  padding: "7px 16px",
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
