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
  onClick?: () => void;
}

export const ReactionBubble = memo(function ReactionBubble({
  emoji,
  count,
  isOwnReaction,
  onClick,
}: ReactionBubbleProps) {
  const bubbleStyle: CSSProperties = {
    display: "inline-flex",
    alignItems: "center",
    gap: 4,
    padding: "2px 8px",
    borderRadius: 12,
    fontSize: 14,
    fontWeight: 500,
    backgroundColor: isOwnReaction
      ? "rgba(0, 122, 255, 0.15)"
      : "var(--color-surface-variant)",
    border: isOwnReaction
      ? "1.5px solid #007AFF"
      : "1px solid var(--color-outline)",
    color: isOwnReaction ? "#007AFF" : "var(--color-on-surface)",
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
              scale: 1.1,
              backgroundColor: isOwnReaction
                ? "rgba(0, 122, 255, 0.25)"
                : "var(--color-surface)",
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
