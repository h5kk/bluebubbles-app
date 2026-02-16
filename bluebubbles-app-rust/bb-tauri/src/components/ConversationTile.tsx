/**
 * ConversationTile component for the chat list.
 * Implements the tile layout from spec 02-conversation-list.md section 4.
 */
import { useCallback, useState, type CSSProperties } from "react";
import { motion } from "framer-motion";
import { Avatar, GroupAvatar } from "./Avatar";
import type { ChatWithPreview } from "@/hooks/useTauri";

interface ConversationTileProps {
  chat: ChatWithPreview;
  isActive?: boolean;
  onClick?: (guid: string) => void;
  onContextMenu?: (guid: string, event: React.MouseEvent) => void;
}

export function ConversationTile({
  chat,
  isActive = false,
  onClick,
  onContextMenu,
}: ConversationTileProps) {
  const [isHovered, setIsHovered] = useState(false);

  const chatData = chat.chat;
  const isGroup = chatData.participants.length > 1;
  const title =
    chatData.display_name ||
    chat.participant_names.join(", ") ||
    chatData.chat_identifier ||
    "Unknown";
  const preview = chat.latest_message_text ?? "";
  const isUnread = chatData.has_unread_message;
  const isPinned = chatData.is_pinned;
  const isMuted = chatData.mute_type != null;
  const isImessage = !chatData.guid.startsWith("SMS");

  const handleClick = useCallback(() => {
    onClick?.(chatData.guid);
  }, [onClick, chatData.guid]);

  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      onContextMenu?.(chatData.guid, e);
    },
    [onContextMenu, chatData.guid]
  );

  const tileStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 12,
    padding: "10px 16px",
    cursor: "pointer",
    borderRadius: 0,
    transition: "background-color 100ms ease",
    backgroundColor: isActive
      ? "var(--color-primary-container)"
      : isHovered
        ? "var(--color-surface-variant)"
        : "transparent",
    borderBottom: "1px solid var(--color-surface-variant)",
    position: "relative",
  };

  return (
    <motion.div
      style={tileStyle}
      onClick={handleClick}
      onContextMenu={handleContextMenu}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      whileTap={{ scale: 0.98 }}
      role="button"
      tabIndex={0}
      aria-label={`${title}, ${preview}`}
    >
      {/* Avatar */}
      {isGroup ? (
        <GroupAvatar
          participants={chatData.participants.map((p, i) => ({
            name: chat.participant_names[i] ?? p.address,
            address: p.address,
          }))}
          size={48}
        />
      ) : (
        <Avatar
          name={title}
          address={chatData.participants[0]?.address ?? chatData.guid}
          size={48}
        />
      )}

      {/* Content */}
      <div
        style={{
          flex: 1,
          minWidth: 0,
          display: "flex",
          flexDirection: "column",
          gap: 2,
        }}
      >
        {/* Title row */}
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            gap: 8,
          }}
        >
          <span
            style={{
              fontSize: "var(--font-body-large)",
              fontWeight: isUnread ? 700 : 400,
              color: "var(--color-on-surface)",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
              flex: 1,
            }}
          >
            {title}
          </span>
          <span
            style={{
              fontSize: "var(--font-body-small)",
              color: "var(--color-outline)",
              whiteSpace: "nowrap",
              flexShrink: 0,
            }}
          >
            {formatRelativeTime(chat.latest_message_date)}
          </span>
        </div>

        {/* Preview row */}
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            gap: 8,
          }}
        >
          <span
            style={{
              fontSize: "var(--font-body-medium)",
              color: "var(--color-on-surface-variant)",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
              flex: 1,
            }}
          >
            {chat.latest_message_is_from_me ? "You: " : ""}
            {preview}
          </span>

          {/* Status indicators */}
          <div style={{ display: "flex", gap: 6, alignItems: "center", flexShrink: 0 }}>
            {isUnread && (
              <div
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: "50%",
                  backgroundColor: "var(--color-primary)",
                }}
              />
            )}
            {isPinned && (
              <span style={{ fontSize: 12, color: "var(--color-outline)" }}>
                \uD83D\uDCCC
              </span>
            )}
            {isMuted && (
              <span style={{ fontSize: 12, color: "var(--color-outline)" }}>
                \uD83D\uDD15
              </span>
            )}
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function formatRelativeTime(dateStr: string | null): string {
  if (!dateStr) return "";
  try {
    const ts = Number(dateStr);
    const date = isNaN(ts) ? new Date(dateStr) : new Date(ts);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return "now";
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h`;
    if (diff < 604800000) {
      return date.toLocaleDateString(undefined, { weekday: "short" });
    }
    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
    });
  } catch {
    return "";
  }
}
