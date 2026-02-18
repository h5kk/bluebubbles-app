/**
 * SendOptionsPopover - Frosted glass popover for send with effect and schedule send.
 * Anchored above the send button, triggered by right-click/context menu.
 */
import React, { useEffect, useRef, useState, useCallback, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";

export const BUBBLE_EFFECTS = [
  { id: "com.apple.MobileSMS.expressivesend.impact", label: "Slam", icon: "slam" },
  { id: "com.apple.MobileSMS.expressivesend.loud", label: "Loud", icon: "loud" },
  { id: "com.apple.MobileSMS.expressivesend.gentle", label: "Gentle", icon: "gentle" },
  { id: "com.apple.MobileSMS.expressivesend.invisibleink", label: "Invisible Ink", icon: "ink" },
] as const;

interface SendOptionsPopoverProps {
  open: boolean;
  anchorRef: React.RefObject<HTMLButtonElement | null>;
  onClose: () => void;
  onSendWithEffect: (effectId: string) => void;
  onScheduleSend: (scheduledFor: number) => void;
  privateApiEnabled: boolean;
}

/** Compute next occurrence of a given hour (today or tomorrow). */
function getNextTime(hour: number, minute = 0): number {
  const now = new Date();
  const target = new Date(now);
  target.setHours(hour, minute, 0, 0);
  if (target.getTime() <= now.getTime()) {
    target.setDate(target.getDate() + 1);
  }
  return target.getTime();
}

/** Tomorrow at a specific hour. */
function getTomorrowAt(hour: number, minute = 0): number {
  const tomorrow = new Date();
  tomorrow.setDate(tomorrow.getDate() + 1);
  tomorrow.setHours(hour, minute, 0, 0);
  return tomorrow.getTime();
}

/** Format epoch ms to local datetime-local input value. */
function toDatetimeLocalValue(epochMs?: number): string {
  const d = epochMs ? new Date(epochMs) : new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** Effect icon SVGs */
function EffectIcon({ icon }: { icon: string }) {
  const size = 16;
  const style: CSSProperties = { width: size, height: size, flexShrink: 0 };

  switch (icon) {
    case "slam":
      return (
        <svg style={style} viewBox="0 0 16 16" fill="none">
          <path d="M8 2L3 14H13L8 2Z" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round" />
          <path d="M8 6V10" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          <circle cx="8" cy="12" r="0.8" fill="currentColor" />
        </svg>
      );
    case "loud":
      return (
        <svg style={style} viewBox="0 0 16 16" fill="none">
          <path d="M3 6H1V10H3L7 13V3L3 6Z" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round" />
          <path d="M10 5.5C10.8 6.3 11.2 7.3 11.2 8.2C11.2 9.1 10.8 10 10 10.8" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          <path d="M12 3.5C13.3 4.8 14 6.5 14 8.2C14 9.9 13.3 11.5 12 12.8" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
        </svg>
      );
    case "gentle":
      return (
        <svg style={style} viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="1.3" strokeDasharray="2 2" />
          <circle cx="8" cy="8" r="2.5" fill="currentColor" opacity="0.4" />
        </svg>
      );
    case "ink":
      return (
        <svg style={style} viewBox="0 0 16 16" fill="none">
          <path d="M8 2C8 2 3 7 3 10C3 12.8 5.2 15 8 15C10.8 15 13 12.8 13 10C13 7 8 2 8 2Z" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round" />
          <path d="M6 10C6 10 7 9 8 10C9 11 10 10 10 10" stroke="currentColor" strokeWidth="1" strokeLinecap="round" opacity="0.5" />
        </svg>
      );
    default:
      return null;
  }
}

export function SendOptionsPopover({
  open,
  anchorRef,
  onClose,
  onSendWithEffect,
  onScheduleSend,
  privateApiEnabled,
}: SendOptionsPopoverProps) {
  const popoverRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<{ top: number; left: number }>({ top: 0, left: 0 });
  const [showDatePicker, setShowDatePicker] = useState(false);
  const [customDate, setCustomDate] = useState("");
  const itemsRef = useRef<(HTMLButtonElement | null)[]>([]);
  const [focusIndex, setFocusIndex] = useState(-1);

  // Calculate position relative to anchor
  useEffect(() => {
    if (open && anchorRef.current) {
      const rect = anchorRef.current.getBoundingClientRect();
      setPosition({
        top: rect.top - 8, // 8px gap above anchor
        left: rect.left + rect.width / 2,
      });
      setShowDatePicker(false);
      setCustomDate("");
      setFocusIndex(-1);
    }
  }, [open, anchorRef]);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handleMouseDown = (e: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    window.addEventListener("mousedown", handleMouseDown);
    return () => window.removeEventListener("mousedown", handleMouseDown);
  }, [open, onClose]);

  // Close on Escape, navigate with arrow keys
  useEffect(() => {
    if (!open) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
        anchorRef.current?.focus();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setFocusIndex((prev) => {
          const next = prev + 1;
          const total = itemsRef.current.filter(Boolean).length;
          return next >= total ? 0 : next;
        });
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setFocusIndex((prev) => {
          const total = itemsRef.current.filter(Boolean).length;
          return prev <= 0 ? total - 1 : prev - 1;
        });
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose, anchorRef]);

  // Focus the active item when focusIndex changes
  useEffect(() => {
    if (focusIndex >= 0 && itemsRef.current[focusIndex]) {
      itemsRef.current[focusIndex]?.focus();
    }
  }, [focusIndex]);

  const handleEffectClick = useCallback(
    (effectId: string) => {
      onSendWithEffect(effectId);
      onClose();
    },
    [onSendWithEffect, onClose]
  );

  const handlePresetClick = useCallback(
    (epochMs: number) => {
      onScheduleSend(epochMs);
      onClose();
    },
    [onScheduleSend, onClose]
  );

  const handleCustomDateConfirm = useCallback(() => {
    if (!customDate) return;
    const epochMs = new Date(customDate).getTime();
    if (epochMs > Date.now()) {
      onScheduleSend(epochMs);
      onClose();
    }
  }, [customDate, onScheduleSend, onClose]);

  // Build list of items for ref tracking
  let itemIndex = 0;
  const getItemRef = () => {
    const idx = itemIndex++;
    return (el: HTMLButtonElement | null) => {
      itemsRef.current[idx] = el;
    };
  };

  // Reset item index on each render
  itemIndex = 0;
  itemsRef.current = [];

  const schedulePresets = [
    { label: "In 1 hour", getTime: () => Date.now() + 3600000 },
    { label: "Tonight 9 PM", getTime: () => getNextTime(21) },
    { label: "Tomorrow 8 AM", getTime: () => getTomorrowAt(8) },
    { label: "Tomorrow 12 PM", getTime: () => getTomorrowAt(12) },
  ];

  const popoverStyle: CSSProperties = {
    position: "fixed",
    top: position.top,
    left: position.left,
    transform: "translate(-50%, -100%)",
    width: 240,
    background: "color-mix(in srgb, var(--color-surface) 80%, transparent)",
    backdropFilter: "blur(20px)",
    WebkitBackdropFilter: "blur(20px)",
    border: "1px solid var(--color-surface-variant)",
    borderRadius: 16,
    boxShadow: "0 8px 32px rgba(0, 0, 0, 0.12)",
    padding: "8px 0",
    zIndex: 9999,
    overflow: "hidden",
  };

  const sectionHeaderStyle: CSSProperties = {
    fontSize: 11,
    fontWeight: 600,
    textTransform: "uppercase" as const,
    letterSpacing: "0.05em",
    color: "var(--color-on-surface-variant)",
    padding: "8px 14px 4px",
  };

  const menuItemStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 10,
    width: "100%",
    padding: "8px 14px",
    fontSize: "var(--font-body-medium)",
    color: "var(--color-on-surface)",
    background: "transparent",
    cursor: "pointer",
    textAlign: "left" as const,
    transition: "background-color 100ms ease",
  };

  const dividerStyle: CSSProperties = {
    height: 1,
    background: "var(--color-surface-variant)",
    margin: "4px 0",
  };

  const pillButtonStyle: CSSProperties = {
    padding: "5px 12px",
    borderRadius: 14,
    fontSize: 12,
    fontWeight: 500,
    color: "var(--color-on-surface)",
    background: "var(--color-surface-variant)",
    cursor: "pointer",
    border: "none",
    transition: "background-color 100ms ease",
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          ref={popoverRef}
          role="menu"
          aria-label="Send options menu"
          style={popoverStyle}
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          transition={{ duration: 0.15, ease: "easeOut" }}
        >
          {/* Send with Effect section */}
          {privateApiEnabled && (
            <>
              <div style={sectionHeaderStyle}>Send with Effect</div>
              {BUBBLE_EFFECTS.map((effect) => (
                <button
                  key={effect.id}
                  ref={getItemRef()}
                  role="menuitem"
                  aria-label={`Send with ${effect.label} effect`}
                  style={menuItemStyle}
                  onClick={() => handleEffectClick(effect.id)}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLElement).style.backgroundColor =
                      "var(--color-surface-variant)";
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLElement).style.backgroundColor = "transparent";
                  }}
                >
                  <EffectIcon icon={effect.icon} />
                  <span>{effect.label}</span>
                </button>
              ))}
              <div style={dividerStyle} />
            </>
          )}

          {/* Schedule Send section */}
          <div style={sectionHeaderStyle}>Schedule Send</div>
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              gap: 6,
              padding: "4px 14px 6px",
            }}
          >
            {schedulePresets.map((preset) => (
              <button
                key={preset.label}
                ref={getItemRef()}
                role="menuitem"
                aria-label={`Schedule send ${preset.label}`}
                style={pillButtonStyle}
                onClick={() => handlePresetClick(preset.getTime())}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLElement).style.backgroundColor =
                    "var(--color-primary)";
                  (e.currentTarget as HTMLElement).style.color = "#fff";
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLElement).style.backgroundColor =
                    "var(--color-surface-variant)";
                  (e.currentTarget as HTMLElement).style.color =
                    "var(--color-on-surface)";
                }}
              >
                {preset.label}
              </button>
            ))}
          </div>

          {/* Custom date/time picker */}
          {!showDatePicker ? (
            <button
              ref={getItemRef()}
              role="menuitem"
              aria-label="Pick custom date and time"
              style={menuItemStyle}
              onClick={() => setShowDatePicker(true)}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLElement).style.backgroundColor =
                  "var(--color-surface-variant)";
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLElement).style.backgroundColor = "transparent";
              }}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" style={{ flexShrink: 0 }}>
                <rect x="1.5" y="2.5" width="13" height="11" rx="2" stroke="currentColor" strokeWidth="1.3" />
                <path d="M1.5 6H14.5" stroke="currentColor" strokeWidth="1.3" />
                <path d="M5 1V4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
                <path d="M11 1V4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
              </svg>
              <span>Pick date & time...</span>
            </button>
          ) : (
            <div style={{ padding: "6px 14px 8px", display: "flex", flexDirection: "column", gap: 6 }}>
              <label
                htmlFor="send-options-datetime"
                style={{
                  fontSize: 11,
                  color: "var(--color-on-surface-variant)",
                  fontWeight: 500,
                }}
              >
                Send at:
              </label>
              <input
                id="send-options-datetime"
                type="datetime-local"
                min={toDatetimeLocalValue()}
                value={customDate}
                onChange={(e) => setCustomDate(e.target.value)}
                style={{
                  padding: "6px 8px",
                  borderRadius: 8,
                  border: "1px solid var(--color-surface-variant)",
                  background: "var(--color-surface)",
                  color: "var(--color-on-surface)",
                  fontSize: 13,
                  width: "100%",
                  boxSizing: "border-box",
                }}
              />
              <button
                onClick={handleCustomDateConfirm}
                disabled={!customDate || new Date(customDate).getTime() <= Date.now()}
                style={{
                  padding: "6px 12px",
                  borderRadius: 10,
                  fontSize: 13,
                  fontWeight: 600,
                  color: "#fff",
                  background:
                    customDate && new Date(customDate).getTime() > Date.now()
                      ? "#007AFF"
                      : "var(--color-outline)",
                  cursor:
                    customDate && new Date(customDate).getTime() > Date.now()
                      ? "pointer"
                      : "default",
                  border: "none",
                  transition: "background-color 100ms ease",
                }}
              >
                Schedule
              </button>
            </div>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
