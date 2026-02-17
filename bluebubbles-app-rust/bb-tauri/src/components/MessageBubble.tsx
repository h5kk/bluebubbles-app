/**
 * MessageBubble component - macOS Messages / iOS style.
 * Features bubble tails on the last message in a group, delivered status.
 */
import { useMemo, useState, useEffect, memo, type CSSProperties } from "react";
import type { Message, Attachment } from "@/hooks/useTauri";
import { tauriDownloadAttachment } from "@/hooks/useTauri";
import { parseBBDate } from "@/utils/dateUtils";

// ─── Attachment rendering ─────────────────────────────────────────────────────

type AttachmentLoadState = "idle" | "loading" | "loaded" | "error";

/** Renders a single image attachment, fetching the data URI via Tauri IPC. */
const AttachmentImage = memo(function AttachmentImage({
  attachment,
  borderRadius,
}: {
  attachment: Attachment;
  borderRadius?: string;
}) {
  const [state, setState] = useState<AttachmentLoadState>("idle");
  const [dataUri, setDataUri] = useState<string | null>(null);

  useEffect(() => {
    if (!attachment.guid) return;
    let cancelled = false;

    setState("loading");
    tauriDownloadAttachment(attachment.guid)
      .then((uri) => {
        if (!cancelled) {
          setDataUri(uri);
          setState("loaded");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setState("error");
        }
      });

    return () => {
      cancelled = true;
    };
  }, [attachment.guid]);

  // Compute aspect ratio for the placeholder
  const aspectRatio =
    attachment.width && attachment.height
      ? attachment.width / attachment.height
      : 16 / 9;

  if (state === "loading" || state === "idle") {
    return (
      <div
        style={{
          width: "100%",
          maxWidth: "var(--bubble-max-width)",
          aspectRatio: String(aspectRatio),
          maxHeight: 320,
          backgroundColor: "var(--color-surface-variant, #e0e0e0)",
          borderRadius: borderRadius ?? "12px",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          overflow: "hidden",
        }}
      >
        <span
          style={{
            fontSize: 13,
            color: "var(--color-on-surface-variant, #666)",
            opacity: 0.6,
          }}
        >
          Loading...
        </span>
      </div>
    );
  }

  if (state === "error" || !dataUri) {
    return (
      <span style={{ opacity: 0.7, fontStyle: "italic" }}>
        {"\uD83D\uDCF7"} {attachment.transfer_name ?? "Image"}
      </span>
    );
  }

  return (
    <img
      src={dataUri}
      alt={attachment.transfer_name ?? "Image attachment"}
      style={{
        display: "block",
        width: "100%",
        maxWidth: "var(--bubble-max-width)",
        maxHeight: 400,
        borderRadius: borderRadius ?? "12px",
        objectFit: "cover",
      }}
      loading="lazy"
    />
  );
});

/** Renders all attachments for a message, stacked vertically. */
function AttachmentRenderer({
  attachments,
  borderRadius,
}: {
  attachments: Attachment[];
  borderRadius?: string;
}) {
  if (!attachments || attachments.length === 0) return null;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      {attachments.map((att, idx) => {
        const mime = att.mime_type ?? "";

        if (mime.startsWith("image/")) {
          return (
            <AttachmentImage
              key={att.guid ?? idx}
              attachment={att}
              borderRadius={borderRadius}
            />
          );
        }

        if (mime.startsWith("video/")) {
          return (
            <div
              key={att.guid ?? idx}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 6,
                opacity: 0.7,
                fontStyle: "italic",
              }}
            >
              <span>{"\uD83C\uDFA5"}</span>
              <span>{att.transfer_name ?? "Video"}</span>
            </div>
          );
        }

        if (mime.startsWith("audio/")) {
          return (
            <span
              key={att.guid ?? idx}
              style={{ opacity: 0.7, fontStyle: "italic" }}
            >
              {"\uD83C\uDFB5"} {att.transfer_name ?? "Audio"}
            </span>
          );
        }

        return (
          <span
            key={att.guid ?? idx}
            style={{ opacity: 0.7, fontStyle: "italic" }}
          >
            {"\uD83D\uDCCE"} {att.transfer_name ?? "Attachment"}
          </span>
        );
      })}
    </div>
  );
}

// ─── MessageBubble ────────────────────────────────────────────────────────────

interface MessageBubbleProps {
  message: Message;
  isGroupChat?: boolean;
  senderName?: string;
  isFirstInGroup?: boolean;
  isLastInGroup?: boolean;
  isImessage?: boolean;
  showTimestamp?: boolean;
}

export const MessageBubble = memo(function MessageBubble({
  message,
  isGroupChat = false,
  senderName,
  isFirstInGroup = true,
  isLastInGroup = true,
  isImessage = true,
  showTimestamp = false,
}: MessageBubbleProps) {
  const isSent = message.is_from_me;
  const isBigEmoji = message.big_emoji === true || detectBigEmoji(message.text);
  const hasError = message.error !== 0;
  const isGroupEvent = message.item_type !== 0;

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

  // iOS-style corner radius: 18px default, 4px on the inner continuation side
  // Sent messages: right side is the "tail side" (inner). top-right is sm when
  // continuing from a previous message (not first). bottom-right is sm when
  // continuing into the next message (not last).
  // Received messages: left side is the "tail side" (inner). Same logic mirrored.
  const borderRadius = useMemo(() => {
    const lg = 18;
    const sm = 4;
    if (isSent) {
      // order: top-left top-right bottom-right bottom-left
      const topRight = isFirstInGroup ? lg : sm;
      const bottomRight = isLastInGroup ? lg : sm;
      return `${lg}px ${topRight}px ${bottomRight}px ${lg}px`;
    }
    // received
    const topLeft = isFirstInGroup ? lg : sm;
    const bottomLeft = isLastInGroup ? lg : sm;
    return `${topLeft}px ${lg}px ${lg}px ${bottomLeft}px`;
  }, [isSent, isFirstInGroup, isLastInGroup]);

  // Detect image attachments for edge-to-edge image rendering
  const hasImageAttachments =
    message.has_attachments &&
    message.attachments?.some((a) => a.mime_type?.startsWith("image/"));
  // When images are present, use zero padding on bubble and wrap text in its own padded div
  const useImageLayout = !!hasImageAttachments;

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    alignItems: isSent ? "flex-end" : "flex-start",
    paddingLeft: isSent ? 0 : "10px",
    paddingRight: isSent ? "10px" : 0,
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
        padding: useImageLayout ? "0" : "var(--bubble-padding-v) var(--bubble-padding-h)",
        minWidth: 44,
        minHeight: useImageLayout ? 0 : "var(--bubble-min-height)",
        fontSize: "var(--font-bubble-text)",
        lineHeight: 1.35,
        wordBreak: "break-word" as const,
        whiteSpace: "pre-wrap" as const,
        position: "relative" as const,
        overflow: useImageLayout ? "hidden" : undefined,
      };

  // Reactions display
  const reactions = message.associated_messages?.filter(
    (m) => m.associated_message_type != null
  );

  // Group events (name changes, participant added/removed) render as centered system messages
  if (isGroupEvent) {
    const eventText = message.group_title
      ? `Group name changed to "${message.group_title}"`
      : message.text || "Group event";
    return (
      <div
        style={{
          textAlign: "center",
          fontSize: "var(--font-body-small)",
          color: "var(--color-on-surface-variant)",
          padding: "4px 16px",
          fontStyle: "italic",
        }}
      >
        {showTimestamp && message.date_created && (
          <div
            style={{
              fontSize: "var(--font-label-small)",
              color: "var(--color-outline)",
              padding: "8px 0 4px",
            }}
          >
            {formatTimestamp(message.date_created)}
          </div>
        )}
        {eventText}
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      {/* Timestamp separator */}
      {showTimestamp && message.date_created && (
        <div
          style={{
            width: "100%",
            textAlign: "center",
            fontSize: "var(--font-label-small)",
            color: "var(--color-outline)",
            padding: "8px 0 4px",
          }}
        >
          {formatTimestamp(message.date_created)}
        </div>
      )}

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

      {/* Bubble with optional tail */}
      <div style={{ position: "relative", maxWidth: "var(--bubble-max-width)" }}>
        <div style={bubbleStyle}>
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

          {/* Attachments rendered above text */}
          {message.has_attachments &&
            message.attachments &&
            message.attachments.length > 0 && (
              <AttachmentRenderer
                attachments={message.attachments}
                borderRadius={borderRadius}
              />
            )}

          {/* Message text */}
          {message.text ? (
            hasImageAttachments ? (
              <div
                style={{
                  padding: "var(--bubble-padding-v) var(--bubble-padding-h)",
                }}
              >
                {message.text}
              </div>
            ) : (
              message.text
            )
          ) : !message.has_attachments ||
              !message.attachments ||
              message.attachments.length === 0 ? (
            ""
          ) : null}

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
        </div>

        {/* Bubble tail - only on last message in group */}
        {isLastInGroup && !isBigEmoji && (
          <svg
            width="10"
            height="16"
            viewBox="0 0 10 16"
            fill="none"
            style={{
              position: "absolute",
              bottom: 0,
              ...(isSent
                ? { right: -6 }
                : { left: -6, transform: "scaleX(-1)" }),
              pointerEvents: "none",
            }}
          >
            <path
              d="M0 16C0 16 0 8 0 0C0 0 0 12 5 15C7 16 10 16 10 16L0 16Z"
              fill={bubbleColor}
            />
          </svg>
        )}
      </div>

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

      {/* Delivered status - only on last sent message in group */}
      {isSent && isLastInGroup && !hasError && (
        <span
          style={{
            fontSize: 13,
            color: "var(--color-on-surface-variant, #888)",
            marginTop: 4,
            marginRight: 4,
          }}
        >
          {message.date_delivered
            ? `Delivered ${formatDeliveredTime(message.date_delivered)}`
            : message.is_delivered
              ? "Delivered"
              : "Sent"}
        </span>
      )}
    </div>
  );
});

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
  const normalized = type.toLowerCase().replace(/[^a-z]/g, "");
  return map[normalized] ?? type;
}

function formatTimestamp(dateStr: string): string {
  try {
    const date = parseBBDate(dateStr);
    if (!date) return dateStr;

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

/**
 * Client-side fallback for detecting messages that are only 1-3 emoji.
 * Used when the server doesn't set big_emoji.
 */
function detectBigEmoji(text: string | null): boolean {
  if (!text) return false;
  const trimmed = text.trim();
  if (trimmed.length === 0 || trimmed.length > 20) return false;
  // Match emoji sequences (including modifiers, ZWJ sequences, keycap, flags)
  const emojiRegex =
    /^(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F)(?:\u200D(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F))*(?:\s*(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F)(?:\u200D(?:\p{Emoji_Presentation}|\p{Emoji}\uFE0F))*){0,2}$/u;
  return emojiRegex.test(trimmed);
}

function formatDeliveredTime(dateStr: string): string {
  try {
    const date = parseBBDate(dateStr);
    if (!date) return "";

    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return "just now";
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;

    return date.toLocaleTimeString(undefined, {
      hour: "numeric",
      minute: "2-digit",
    });
  } catch {
    return "";
  }
}
