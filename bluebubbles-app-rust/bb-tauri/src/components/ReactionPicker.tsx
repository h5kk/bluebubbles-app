/**
 * ReactionPicker - popup emoji picker for adding reactions to messages.
 * Shows quick reactions (❤️ 👍 😂 😮 😢 😡) with hover effects.
 */
import { useEffect, useRef, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface ReactionPickerProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectReaction: (reaction: string) => void;
  anchorElement: HTMLElement | null;
  position?: "above" | "below";
}

const QUICK_REACTIONS = [
  { emoji: "❤️", name: "love", label: "Love" },
  { emoji: "👍", name: "like", label: "Like" },
  { emoji: "😂", name: "laugh", label: "Laugh" },
  { emoji: "😮", name: "emphasize", label: "Emphasize" },
  { emoji: "😢", name: "dislike", label: "Dislike" },
  { emoji: "❓", name: "question", label: "Question" },
];

export function ReactionPicker({
  isOpen,
  onClose,
  onSelectReaction,
  anchorElement,
  position = "above",
}: ReactionPickerProps) {
  const pickerRef = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (
        pickerRef.current &&
        !pickerRef.current.contains(e.target as Node) &&
        anchorElement &&
        !anchorElement.contains(e.target as Node)
      ) {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen, onClose, anchorElement]);

  // Close on Escape
  useEffect(() => {
    if (!isOpen) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };

    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [isOpen, onClose]);

  if (!isOpen || !anchorElement) return null;

  // Calculate position relative to anchor
  const anchorRect = anchorElement.getBoundingClientRect();

  const containerStyle: CSSProperties = {
    position: "fixed",
    left: anchorRect.left + anchorRect.width / 2,
    top:
      position === "above"
        ? anchorRect.top - 10
        : anchorRect.bottom + 10,
    transform:
      position === "above"
        ? "translate(-50%, -100%)"
        : "translate(-50%, 0)",
    zIndex: 10000,
  };

  const pickerStyle: CSSProperties = {
    display: "flex",
    gap: 8,
    padding: "8px 12px",
    backgroundColor: "var(--color-surface)",
    borderRadius: 24,
    boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
    border: "1px solid var(--color-surface-variant)",
  };

  const reactionButtonStyle: CSSProperties = {
    width: 40,
    height: 40,
    borderRadius: "50%",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    fontSize: 24,
    cursor: "pointer",
    transition: "all 150ms cubic-bezier(0.4, 0, 0.2, 1)",
    backgroundColor: "transparent",
    border: "none",
  };

  return (
    <div style={containerStyle}>
      <AnimatePresence>
        <motion.div
          ref={pickerRef}
          initial={{ opacity: 0, scale: 0.8, y: position === "above" ? 10 : -10 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.8, y: position === "above" ? 10 : -10 }}
          transition={{
            duration: 0.2,
            ease: [0.4, 0, 0.2, 1],
          }}
          style={pickerStyle}
        >
          {QUICK_REACTIONS.map((reaction) => (
            <motion.button
              key={reaction.name}
              onClick={() => {
                onSelectReaction(reaction.name);
                onClose();
              }}
              style={reactionButtonStyle}
              whileHover={{
                scale: 1.3,
                backgroundColor: "var(--color-surface-variant)",
              }}
              whileTap={{ scale: 0.95 }}
              aria-label={`React with ${reaction.label}`}
              title={reaction.label}
            >
              {reaction.emoji}
            </motion.button>
          ))}
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
