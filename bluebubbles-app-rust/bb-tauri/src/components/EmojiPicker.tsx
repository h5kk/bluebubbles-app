/**
 * Emoji picker component with popular emojis and search
 */
import { useState, useCallback, useRef, useEffect, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface EmojiPickerProps {
  onEmojiSelect: (emoji: string) => void;
  onClose: () => void;
  anchorRef?: React.RefObject<HTMLElement>;
}

const POPULAR_EMOJIS = [
  "😀", "😃", "😄", "😁", "😆", "😅", "😂", "🤣",
  "😊", "😇", "🙂", "🙃", "😉", "😌", "😍", "🥰",
  "😘", "😗", "😙", "😚", "😋", "😛", "😝", "😜",
  "🤪", "🤨", "🧐", "🤓", "😎", "🥸", "🤩", "🥳",
  "😏", "😒", "😞", "😔", "😟", "😕", "🙁", "☹️",
  "😣", "😖", "😫", "😩", "🥺", "😢", "😭", "😤",
  "😠", "😡", "🤬", "🤯", "😳", "🥵", "🥶", "😱",
  "😨", "😰", "😥", "😓", "🤗", "🤔", "🤭", "🤫",
  "🤥", "😶", "😐", "😑", "😬", "🙄", "😯", "😦",
  "😧", "😮", "😲", "🥱", "😴", "🤤", "😪", "😵",
  "🤐", "🥴", "🤢", "🤮", "🤧", "😷", "🤒", "🤕",
  "👍", "👎", "👌", "✌️", "🤞", "🤟", "🤘", "🤙",
  "👈", "👉", "👆", "👇", "☝️", "👏", "🙌", "👐",
  "🤲", "🤝", "🙏", "✍️", "💅", "🤳", "💪", "🦾",
  "❤️", "🧡", "💛", "💚", "💙", "💜", "🖤", "🤍",
  "🤎", "💔", "❣️", "💕", "💞", "💓", "💗", "💖",
  "💘", "💝", "💟", "☮️", "✝️", "☪️", "🕉️", "☸️",
  "✡️", "🔯", "🕎", "☯️", "☦️", "🛐", "⛎", "♈",
];

const EMOJI_CATEGORIES = [
  { name: "Smileys", icon: "😊", emojis: POPULAR_EMOJIS.slice(0, 32) },
  { name: "Gestures", icon: "👍", emojis: POPULAR_EMOJIS.slice(96, 112) },
  { name: "Hearts", icon: "❤️", emojis: POPULAR_EMOJIS.slice(112, 128) },
];

export function EmojiPicker({ onEmojiSelect, onClose, anchorRef }: EmojiPickerProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState(0);
  const pickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        pickerRef.current &&
        !pickerRef.current.contains(e.target as Node) &&
        anchorRef?.current &&
        !anchorRef.current.contains(e.target as Node)
      ) {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [onClose, anchorRef]);

  const handleEmojiClick = useCallback(
    (emoji: string) => {
      onEmojiSelect(emoji);
      onClose();
    },
    [onEmojiSelect, onClose]
  );

  const filteredEmojis = searchQuery
    ? POPULAR_EMOJIS.filter((emoji) => emoji.includes(searchQuery))
    : EMOJI_CATEGORIES[selectedCategory].emojis;

  const containerStyle: CSSProperties = {
    position: "absolute",
    bottom: "calc(100% + 8px)",
    right: 0,
    width: 320,
    maxHeight: 380,
    backgroundColor: "var(--color-surface)",
    borderRadius: 12,
    boxShadow: "0 4px 20px rgba(0,0,0,0.18)",
    border: "1px solid var(--color-surface-variant)",
    display: "flex",
    flexDirection: "column",
    overflow: "hidden",
    zIndex: 300,
  };

  return (
    <motion.div
      ref={pickerRef}
      style={containerStyle}
      initial={{ opacity: 0, scale: 0.95, y: 5 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.95, y: 5 }}
      transition={{ duration: 0.15 }}
    >
      {/* Search bar */}
      <div style={{ padding: 12, borderBottom: "1px solid var(--color-surface-variant)" }}>
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search emoji..."
          autoFocus
          style={{
            width: "100%",
            padding: "8px 12px",
            fontSize: "var(--font-body-medium)",
            color: "var(--color-on-surface)",
            backgroundColor: "var(--color-surface-variant)",
            borderRadius: 8,
            border: "none",
            outline: "none",
          }}
        />
      </div>

      {/* Category tabs */}
      {!searchQuery && (
        <div
          style={{
            display: "flex",
            gap: 4,
            padding: "8px 12px",
            borderBottom: "1px solid var(--color-surface-variant)",
          }}
        >
          {EMOJI_CATEGORIES.map((cat, idx) => (
            <button
              key={cat.name}
              onClick={() => setSelectedCategory(idx)}
              style={{
                flex: 1,
                padding: "6px 0",
                fontSize: 18,
                backgroundColor:
                  selectedCategory === idx
                    ? "var(--color-surface-variant)"
                    : "transparent",
                borderRadius: 6,
                cursor: "pointer",
                transition: "background-color 100ms",
              }}
              aria-label={cat.name}
            >
              {cat.icon}
            </button>
          ))}
        </div>
      )}

      {/* Emoji grid */}
      <div
        style={{
          flex: 1,
          overflow: "auto",
          padding: 8,
        }}
      >
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(8, 1fr)",
            gap: 4,
          }}
        >
          {filteredEmojis.map((emoji, idx) => (
            <button
              key={`${emoji}-${idx}`}
              onClick={() => handleEmojiClick(emoji)}
              style={{
                width: 36,
                height: 36,
                fontSize: 24,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                borderRadius: 6,
                cursor: "pointer",
                transition: "background-color 80ms",
                backgroundColor: "transparent",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = "var(--color-surface-variant)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = "transparent";
              }}
            >
              {emoji}
            </button>
          ))}
        </div>
      </div>
    </motion.div>
  );
}
