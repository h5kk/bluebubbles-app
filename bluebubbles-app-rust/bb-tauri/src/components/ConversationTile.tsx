/**
 * ConversationTile component - macOS Messages style.
 * Clean tile with rounded corners, blue selection highlight, no dividers.
 */
import { useCallback, useState, memo, type CSSProperties } from "react";
import { Avatar, GroupAvatar } from "./Avatar";
import type { ChatWithPreview } from "@/hooks/useTauri";
import { parseBBDate } from "@/utils/dateUtils";

interface ConversationTileProps {
  chat: ChatWithPreview;
  isActive?: boolean;
  onClick?: (guid: string) => void;
  onContextMenu?: (guid: string, event: React.MouseEvent) => void;
}

export const ConversationTile = memo(function ConversationTile({
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
  const preview = chat.latest_message_text
    || (chat.latest_message_date ? "\uD83D\uDCCE Attachment" : "");
  const isUnread = chatData.has_unread_message;
  const isMuted = chatData.mute_type != null;

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
    gap: 10,
    padding: "8px 10px",
    cursor: "pointer",
    borderRadius: 10,
    transition: "background-color 100ms ease, transform 80ms ease",
    backgroundColor: isActive
      ? "#007AFF"
      : isHovered
        ? "var(--color-surface-variant)"
        : "transparent",
    position: "relative",
    marginBottom: 1,
  };

  const titleColor = isActive ? "#FFFFFF" : "var(--color-on-surface)";
  const subtitleColor = isActive ? "rgba(255,255,255,0.75)" : "var(--color-on-surface-variant)";
  const timeColor = isActive ? "rgba(255,255,255,0.6)" : "var(--color-outline)";

  return (
    <div
      className="conversation-tile"
      style={tileStyle}
      onClick={handleClick}
      onContextMenu={handleContextMenu}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      role="button"
      tabIndex={0}
      aria-label={`${title}, ${preview}`}
    >
      {/* Unread dot - left of avatar */}
      {isUnread && (
        <div
          style={{
            width: 8,
            height: 8,
            borderRadius: "50%",
            backgroundColor: isActive ? "#FFFFFF" : "#007AFF",
            flexShrink: 0,
          }}
        />
      )}

      {/* Avatar */}
      {isGroup ? (
        <GroupAvatar
          participants={chatData.participants.map((p, i) => ({
            name: chat.participant_names[i] ?? p.address,
            address: p.address,
          }))}
          size={44}
        />
      ) : (
        <Avatar
          name={title}
          address={chatData.participants[0]?.address ?? chatData.guid}
          size={44}
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
              fontWeight: isUnread ? 600 : 400,
              color: titleColor,
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
              color: timeColor,
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
              color: subtitleColor,
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
              flex: 1,
            }}
          >
            {chat.latest_message_is_from_me ? "You: " : ""}
            {preview}
          </span>

          {/* Muted indicator */}
          {isMuted && !isActive && (
            <span style={{ fontSize: 11, color: "var(--color-outline)", flexShrink: 0 }}>
              {"\uD83D\uDD15"}
            </span>
          )}
        </div>
      </div>
    </div>
  );
});

function formatRelativeTime(dateStr: string | null): string {
  if (!dateStr) return "";
  try {
    const date = parseBBDate(dateStr);
    if (!date) return "";

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
