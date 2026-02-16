/**
 * MessageBubble component for individual messages.
 * Implements the bubble styling from spec 03-conversation-view.md section 6.
 */
import { useMemo, type CSSProperties } from "react";
import { motion } from "framer-motion";
import type { Message } from "@/hooks/useTauri";

interface MessageBubbleProps {
  message: Message;
  isGroupChat?: boolean;
  senderName?: string;
  isFirstInGroup?: boolean;
  isLastInGroup?: boolean;
  isImessage?: boolean;
  showTimestamp?: boolean;
}

export function MessageBubble({
  message,
  isGroupChat = false,
  senderName,
  isFirstInGroup = true,
  isLastInGroup = true,
  isImessage = true,
  showTimestamp = false,
}: MessageBubbleProps) {
  const isSent = message.is_from_me;
  const isBigEmoji = message.big_emoji === true;
  const hasError = message.error !== 0;

  const bubbleColor = useMemo(() => {
    if (!isSent) return "var(--bubble-received)";
    return isImessage
      ? "var(--bubble-imessage-sent)"
      : "var(--bubble-sms-sent)";
  }, [isSent, isImessage]);

  const textColor = useMemo(() => {
    if (!isSent) return "var(--bubble-on-received)";
    return isImessage
      ? "var(--bubble-imessage-on-sent)"
      : "var(--bubble-sms-on-sent)";
  }, [isSent, isImessage]);

  // Corner radius logic for grouped messages
  const borderRadius = useMemo(() => {
    const large = 20;
    const small = 5;
    if (isSent) {
      return `${isFirstInGroup ? large : small}px ${isFirstInGroup ? large : small}px ${isLastInGroup ? large : small}px ${large}px`;
    }
    return `${isFirstInGroup ? large : small}px ${isFirstInGroup ? large : small}px ${large}px ${isLastInGroup ? large : small}px`;
  }, [isSent, isFirstInGroup, isLastInGroup]);

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    alignItems: isSent ? "flex-end" : "flex-start",
    paddingLeft: isSent ? "20%" : "10px",
    paddingRight: isSent ? "10px" : "20%",
    marginBottom: isLastInGroup ? "8px" : "2px",
  };

  const bubbleStyle: CSSProperties = isBigEmoji
    ? {
        fontSize: "42px",
        lineHeight: 1.2,
        padding: "4px 0",
        background: "transparent",
      }
    : {
        backgroundColor: bubbleColor,
        color: textColor,
        borderRadius,
        padding: "var(--bubble-padding-v) var(--bubble-padding-h)",
        maxWidth: "var(--bubble-max-width)",
        minHeight: "var(--bubble-min-height)",
        fontSize: "var(--font-bubble-text)",
        lineHeight: 1.35,
        wordBreak: "break-word" as const,
        whiteSpace: "pre-wrap" as const,
        position: "relative" as const,
      };

  // Reactions display
  const reactions = message.associated_messages?.filter(
    (m) => m.associated_message_type != null
  );

  return (
    <div style={containerStyle}>
      {/* Sender name for group chats */}
      {isGroupChat && !isSent && isFirstInGroup && senderName && (
        <span
          style={{
            fontSize: "var(--font-body-small)",
            color: "var(--color-on-surface-variant)",
            marginBottom: 2,
            marginLeft: 4,
          }}
        >
          {senderName}
        </span>
      )}

      <motion.div
        style={bubbleStyle}
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.15 }}
      >
        {/* Subject line */}
        {message.subject && (
          <div
            style={{
              fontWeight: 600,
              marginBottom: 4,
              fontSize: "var(--font-body-medium)",
            }}
          >
            {message.subject}
          </div>
        )}

        {/* Message text */}
        {message.text ?? ""}

        {/* Error indicator */}
        {hasError && (
          <span
            style={{
              color: "var(--color-error)",
              fontSize: "var(--font-label-small)",
              display: "block",
              marginTop: 4,
            }}
          >
            Failed to send
          </span>
        )}
      </motion.div>

      {/* Reactions */}
      {reactions && reactions.length > 0 && (
        <div
          style={{
            display: "flex",
            gap: 2,
            marginTop: -6,
            marginBottom: 2,
            [isSent ? "marginRight" : "marginLeft"]: 12,
          }}
        >
          {reactions.map((r, i) => (
            <span
              key={i}
              style={{
                fontSize: 12,
                background: "var(--color-surface-variant)",
                borderRadius: "10px",
                padding: "1px 5px",
                border: "1px solid var(--color-outline)",
              }}
            >
              {getReactionEmoji(r.associated_message_type)}
            </span>
          ))}
        </div>
      )}

      {/* Timestamp */}
      {showTimestamp && message.date_created && (
        <span
          style={{
            fontSize: "var(--font-label-small)",
            color: "var(--color-outline)",
            marginTop: 2,
            [isSent ? "marginRight" : "marginLeft"]: 4,
          }}
        >
          {formatTimestamp(message.date_created)}
        </span>
      )}
    </div>
  );
}

function getReactionEmoji(type: string | null): string {
  const map: Record<string, string> = {
    love: "\u2764\uFE0F",
    like: "\uD83D\uDC4D",
    dislike: "\uD83D\uDC4E",
    laugh: "\uD83D\uDE02",
    emphasize: "\u203C\uFE0F",
    question: "\u2753",
  };
  if (!type) return "";
  // Associated message types may be formatted as "2000" (love) etc
  const normalized = type.toLowerCase().replace(/[^a-z]/g, "");
  return map[normalized] ?? type;
}

function formatTimestamp(dateStr: string): string {
  try {
    const ts = Number(dateStr);
    const date = isNaN(ts) ? new Date(dateStr) : new Date(ts);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return "Just now";
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;

    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  } catch {
    return dateStr;
  }
}
