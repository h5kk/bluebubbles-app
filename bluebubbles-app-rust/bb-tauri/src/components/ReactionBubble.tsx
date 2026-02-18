/**
 * ReactionBubble - displays a reaction with count and highlights user's own reactions.
 * Clickable to show reaction details.
 */
import { memo, type CSSProperties } from "react";
import { motion } from "framer-motion";

interface ReactionBubbleProps {
  emoji: string;
  count: number;
  isOwnReaction: boolean;
  isSent: boolean;
  accentColor?: string;
  onClick?: () => void;
}

export const ReactionBubble = memo(function ReactionBubble({
  emoji,
  count,
  isOwnReaction,
  isSent,
  accentColor,
  onClick,
}: ReactionBubbleProps) {
  const baseAccent = accentColor ?? (isSent
    ? "var(--bubble-imessage-sent)"
    : "var(--color-on-surface)");
  const backgroundColor = isSent
    ? `color-mix(in srgb, ${baseAccent} 14%, white)`
    : "color-mix(in srgb, var(--color-surface) 92%, white)";
  const borderColor = isSent
    ? `color-mix(in srgb, ${baseAccent} 60%, transparent)`
    : "color-mix(in srgb, var(--color-outline) 80%, transparent)";

  const bubbleStyle: CSSProperties = {
    display: "inline-flex",
    alignItems: "center",
    gap: 4,
    padding: "2px 9px",
    borderRadius: 999,
    fontSize: 13,
    fontWeight: 600,
    backgroundColor,
    border: `${isOwnReaction ? 1.5 : 1}px solid ${borderColor}`,
    color: baseAccent,
    boxShadow:
      "0 1px 3px rgba(0,0,0,0.12), 0 0 0 1px rgba(255,255,255,0.6) inset",
    cursor: onClick ? "pointer" : "default",
    transition: "all 150ms ease",
    userSelect: "none",
  };

  const emojiStyle: CSSProperties = {
    fontSize: 16,
    lineHeight: 1,
  };

  const countStyle: CSSProperties = {
    fontSize: 12,
    fontWeight: 600,
    lineHeight: 1,
  };

  return (
    <motion.div
      initial={{ scale: 0, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      exit={{ scale: 0, opacity: 0 }}
      transition={{
        duration: 0.2,
        ease: [0.4, 0, 0.2, 1],
      }}
      whileHover={
        onClick
          ? {
              scale: 1.08,
              backgroundColor: backgroundColor,
            }
          : undefined
      }
      whileTap={onClick ? { scale: 0.95 } : undefined}
      style={bubbleStyle}
      onClick={onClick}
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
    >
      <span style={emojiStyle}>{emoji}</span>
      {count > 1 && <span style={countStyle}>{count}</span>}
    </motion.div>
  );
});
