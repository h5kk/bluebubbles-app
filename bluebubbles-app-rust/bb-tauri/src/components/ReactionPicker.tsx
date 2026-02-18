/**
 * ReactionPicker - popup emoji picker for adding reactions to messages.
 * Uses emoji-picker-react in reactions mode to match supported tapbacks.
 */
import { useEffect, useMemo, useRef, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";
import EmojiPicker, { EmojiStyle, Theme, type EmojiClickData } from "emoji-picker-react";

interface ReactionPickerProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectReaction: (reaction: string) => void;
  anchorElement: HTMLElement | null;
  position?: "above" | "below";
}

const TAPBACK_REACTIONS = [
  { emoji: "\u2764\uFE0F", name: "love", label: "Love", unified: "2764-fe0f" },
  { emoji: "\uD83D\uDC4D", name: "like", label: "Like", unified: "1f44d" },
  { emoji: "\uD83D\uDC4E", name: "dislike", label: "Dislike", unified: "1f44e" },
  { emoji: "\uD83D\uDE02", name: "laugh", label: "Laugh", unified: "1f602" },
  { emoji: "\u203C\uFE0F", name: "emphasize", label: "Emphasize", unified: "203c-fe0f" },
  { emoji: "\u2753", name: "question", label: "Question", unified: "2753" },
];

function isDarkTheme(): boolean {
  const theme = document.documentElement.getAttribute("data-theme") ?? "";
  return theme.includes("dark") || theme.includes("oled") || theme.includes("nord");
}

export function ReactionPicker({
  isOpen,
  onClose,
  onSelectReaction,
  anchorElement,
  position = "above",
}: ReactionPickerProps) {
  const pickerRef = useRef<HTMLDivElement>(null);
  const theme = useMemo(() => (isDarkTheme() ? Theme.DARK : Theme.LIGHT), []);

  const reactionByUnified = useMemo(() => {
    const map = new Map<string, string>();
    TAPBACK_REACTIONS.forEach((r) => map.set(r.unified, r.name));
    return map;
  }, []);

  const reactionByEmoji = useMemo(() => {
    const map = new Map<string, string>();
    TAPBACK_REACTIONS.forEach((r) => map.set(r.emoji, r.name));
    return map;
  }, []);

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
    padding: "6px 8px",
    backgroundColor: "var(--color-surface)",
    borderRadius: 24,
    boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
    border: "1px solid var(--color-surface-variant)",
  };

  const handleReactionClick = (emojiData: EmojiClickData) => {
    const unified = emojiData.unified?.toLowerCase();
    const emoji = emojiData.emoji;
    const name =
      (unified && reactionByUnified.get(unified)) ||
      (emoji && reactionByEmoji.get(emoji));
    if (!name) return;
    onSelectReaction(name);
    onClose();
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
          <EmojiPicker
            theme={theme}
            emojiStyle={EmojiStyle.APPLE}
            reactionsDefaultOpen
            allowExpandReactions={false}
            reactions={TAPBACK_REACTIONS.map((r) => r.unified)}
            onReactionClick={handleReactionClick}
            previewConfig={{ showPreview: false }}
            searchDisabled
            skinTonesDisabled
            height={56}
            width={320}
            lazyLoadEmojis
            style={{
              "--epr-bg-color": "var(--color-surface)",
              "--epr-hover-bg-color": "var(--color-surface-variant)",
              "--epr-border-radius": "20px",
              "--epr-category-label-bg-color": "transparent",
              "--epr-highlight-color": "var(--color-primary)",
              "--epr-search-border-color": "var(--color-surface-variant)",
              "--epr-picker-border-color": "transparent",
              "--epr-text-color": "var(--color-on-surface)",
              "--epr-emoji-size": "22px",
            } as CSSProperties}
          />
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
