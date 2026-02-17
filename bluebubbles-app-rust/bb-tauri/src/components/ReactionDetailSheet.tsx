/**
 * ReactionDetailSheet - bottom sheet showing who reacted with what emoji.
 * Groups reactions by type and shows user names.
 */
import { useEffect, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { Message } from "@/hooks/useTauri";

interface ReactionDetailSheetProps {
  isOpen: boolean;
  onClose: () => void;
  reactions: Message[];
  getContactName?: (handleId: number | null) => string;
}

interface GroupedReaction {
  emoji: string;
  reactionType: string;
  users: Array<{
    name: string;
    handleId: number | null;
  }>;
}

const REACTION_MAP: Record<string, string> = {
  love: "❤️",
  like: "👍",
  dislike: "👎",
  laugh: "😂",
  emphasize: "😮",
  question: "❓",
};

export function ReactionDetailSheet({
  isOpen,
  onClose,
  reactions,
  getContactName,
}: ReactionDetailSheetProps) {
  // Close on Escape
  useEffect(() => {
    if (!isOpen) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };

    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [isOpen, onClose]);

  if (!isOpen || reactions.length === 0) return null;

  // Group reactions by type
  const groupedReactions: GroupedReaction[] = [];
  const reactionGroups = new Map<string, GroupedReaction>();

  reactions.forEach((reaction) => {
    const type = reaction.associated_message_type?.toLowerCase() || "unknown";
    const normalizedType = type.replace(/[^a-z]/g, "");

    if (!reactionGroups.has(normalizedType)) {
      reactionGroups.set(normalizedType, {
        emoji: REACTION_MAP[normalizedType] || type,
        reactionType: normalizedType,
        users: [],
      });
    }

    const group = reactionGroups.get(normalizedType)!;
    const userName = getContactName
      ? getContactName(reaction.handle_id)
      : reaction.is_from_me
        ? "You"
        : `User ${reaction.handle_id}`;

    group.users.push({
      name: userName,
      handleId: reaction.handle_id,
    });
  });

  groupedReactions.push(...reactionGroups.values());

  const overlayStyle: CSSProperties = {
    position: "fixed",
    inset: 0,
    backgroundColor: "rgba(0, 0, 0, 0.5)",
    zIndex: 9998,
    display: "flex",
    alignItems: "flex-end",
    justifyContent: "center",
  };

  const sheetStyle: CSSProperties = {
    width: "100%",
    maxWidth: 600,
    maxHeight: "70vh",
    backgroundColor: "var(--color-surface)",
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    boxShadow: "0 -4px 20px rgba(0,0,0,0.2)",
    display: "flex",
    flexDirection: "column",
    overflow: "hidden",
  };

  const headerStyle: CSSProperties = {
    padding: "20px 24px",
    borderBottom: "1px solid var(--color-surface-variant)",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
  };

  const titleStyle: CSSProperties = {
    fontSize: "var(--font-body-large)",
    fontWeight: 600,
    color: "var(--color-on-surface)",
  };

  const closeButtonStyle: CSSProperties = {
    width: 32,
    height: 32,
    borderRadius: "50%",
    backgroundColor: "var(--color-surface-variant)",
    color: "var(--color-on-surface-variant)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    cursor: "pointer",
    border: "none",
    transition: "background-color 150ms ease",
  };

  const contentStyle: CSSProperties = {
    flex: 1,
    overflowY: "auto",
    padding: "16px 24px",
  };

  const reactionGroupStyle: CSSProperties = {
    marginBottom: 20,
  };

  const reactionHeaderStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 8,
    marginBottom: 12,
    fontSize: "var(--font-body-medium)",
    fontWeight: 600,
    color: "var(--color-on-surface)",
  };

  const emojiStyle: CSSProperties = {
    fontSize: 24,
  };

  const userListStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    gap: 8,
  };

  const userItemStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 12,
    padding: "8px 12px",
    backgroundColor: "var(--color-surface-variant)",
    borderRadius: 12,
    fontSize: "var(--font-body-medium)",
    color: "var(--color-on-surface)",
  };

  const avatarStyle: CSSProperties = {
    width: 32,
    height: 32,
    borderRadius: "50%",
    backgroundColor: "var(--color-primary)",
    color: "#FFFFFF",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    fontSize: 14,
    fontWeight: 600,
    flexShrink: 0,
  };

  const getInitials = (name: string): string => {
    const parts = name.split(" ");
    if (parts.length >= 2) {
      return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
    }
    return name.slice(0, 2).toUpperCase();
  };

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.2 }}
        style={overlayStyle}
        onClick={onClose}
      >
        <motion.div
          initial={{ y: "100%" }}
          animate={{ y: 0 }}
          exit={{ y: "100%" }}
          transition={{
            duration: 0.3,
            ease: [0.4, 0, 0.2, 1],
          }}
          style={sheetStyle}
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div style={headerStyle}>
            <div style={titleStyle}>Reactions</div>
            <button
              onClick={onClose}
              style={closeButtonStyle}
              aria-label="Close"
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor =
                  "var(--color-surface)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor =
                  "var(--color-surface-variant)";
              }}
            >
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                <path
                  d="M1 1L13 13M13 1L1 13"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                />
              </svg>
            </button>
          </div>

          {/* Content */}
          <div style={contentStyle}>
            {groupedReactions.map((group, idx) => (
              <div key={idx} style={reactionGroupStyle}>
                <div style={reactionHeaderStyle}>
                  <span style={emojiStyle}>{group.emoji}</span>
                  <span>
                    {group.reactionType.charAt(0).toUpperCase() +
                      group.reactionType.slice(1)}{" "}
                    ({group.users.length})
                  </span>
                </div>
                <div style={userListStyle}>
                  {group.users.map((user, userIdx) => (
                    <div key={userIdx} style={userItemStyle}>
                      <div style={avatarStyle}>{getInitials(user.name)}</div>
                      <span>{user.name}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
