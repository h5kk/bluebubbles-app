/**
 * MessageBubble component - macOS Messages / iOS style.
 * Features bubble tails on the last message in a group, delivered status.
 *
 * Performance optimizations:
 * - Shared ResizeObserver instead of per-bubble observers (100+ → 1)
 * - IntersectionObserver to only calculate visible bubbles
 * - Debounced resize calculations (~60fps limit)
 * - Memoized tail path generation with LRU cache
 */
import {
  useMemo,
  useState,
  useEffect,
  useLayoutEffect,
  useRef,
  useCallback,
  memo,
  type CSSProperties,
} from "react";
import type { Message, Attachment } from "@/hooks/useTauri";
import { tauriDownloadAttachment } from "@/hooks/useTauri";
import { parseBBDate } from "@/utils/dateUtils";
import { useAttachmentStore } from "@/store/attachmentStore";
import { ReactionPicker } from "./ReactionPicker";
import { ReactionBubble } from "./ReactionBubble";
import { ReactionDetailSheet } from "./ReactionDetailSheet";

// ─── Shared ResizeObserver for all bubbles ───────────────────────────────────

type ResizeCallback = (entry: ResizeObserverEntry) => void;

class SharedResizeObserver {
  private observer: ResizeObserver | null = null;
  private callbacks = new Map<Element, ResizeCallback>();
  private debounceTimers = new Map<Element, ReturnType<typeof setTimeout>>();
  private debounceDelay = 16; // ~60fps

  constructor() {
    if (typeof ResizeObserver !== "undefined") {
      this.observer = new ResizeObserver((entries) => {
        entries.forEach((entry) => {
          const callback = this.callbacks.get(entry.target);
          if (!callback) return;

          // Debounce the callback
          const existingTimer = this.debounceTimers.get(entry.target);
          if (existingTimer) clearTimeout(existingTimer);

          const timer = setTimeout(() => {
            callback(entry);
            this.debounceTimers.delete(entry.target);
          }, this.debounceDelay);

          this.debounceTimers.set(entry.target, timer);
        });
      });
    }
  }

  observe(element: Element, callback: ResizeCallback) {
    if (!this.observer) return;
    this.callbacks.set(element, callback);
    this.observer.observe(element);
  }

  unobserve(element: Element) {
    if (!this.observer) return;
    this.callbacks.delete(element);
    const timer = this.debounceTimers.get(element);
    if (timer) {
      clearTimeout(timer);
      this.debounceTimers.delete(element);
    }
    this.observer.unobserve(element);
  }

  disconnect() {
    this.debounceTimers.forEach((timer) => clearTimeout(timer));
    this.debounceTimers.clear();
    this.callbacks.clear();
    this.observer?.disconnect();
  }
}

const sharedResizeObserver = new SharedResizeObserver();

// ─── Tail path calculation cache ─────────────────────────────────────────────

class TailPathCache {
  private cache = new Map<string, string>();
  private maxSize = 100;

  private getKey(width: number, height: number, fromMe: boolean): string {
    // Round to reduce cache misses on tiny size changes
    const w = Math.round(width);
    const h = Math.round(height);
    return `${w}:${h}:${fromMe}`;
  }

  get(width: number, height: number, fromMe: boolean): string | null {
    return this.cache.get(this.getKey(width, height, fromMe)) ?? null;
  }

  set(width: number, height: number, fromMe: boolean, path: string): void {
    const key = this.getKey(width, height, fromMe);

    // LRU eviction when cache is full
    if (this.cache.size >= this.maxSize && !this.cache.has(key)) {
      const firstKey = this.cache.keys().next().value;
      if (firstKey) this.cache.delete(firstKey);
    }

    this.cache.set(key, path);
  }

  clear(): void {
    this.cache.clear();
  }
}

const tailPathCache = new TailPathCache();

// ─── Tail path builder (memoized) ────────────────────────────────────────────

const tailPadding = 8;

function buildTailPath(width: number, height: number, fromMe: boolean): string {
  // Check cache first
  const cached = tailPathCache.get(width, height, fromMe);
  if (cached !== null) return cached;

  // iOS-style proportions: 18px radius, ~6-8px tail
  const baseRadius = Math.min(18, height * 0.4);
  const tailSize = Math.min(8, baseRadius * 0.45);

  const h = height;
  const w = width;

  // Don't create tail for very small bubbles
  if (h < baseRadius * 2 || w < baseRadius * 2) return "";

  let path: string;

  if (fromMe) {
    // Sent message - tail on bottom-right, pointing RIGHT
    path = [
      // Start at top-left corner
      `M 0 ${baseRadius}`,
      `A ${baseRadius} ${baseRadius} 0 0 1 ${baseRadius} 0`,
      // Top edge
      `L ${w - baseRadius} 0`,
      `A ${baseRadius} ${baseRadius} 0 0 1 ${w} ${baseRadius}`,
      // Right edge down to just above tail
      `L ${w} ${h - tailSize - baseRadius * 0.3}`,
      // Tail curves OUT to the right
      `C ${w} ${h - tailSize} ${w + tailSize * 0.8} ${h - tailSize * 0.6} ${w + tailSize} ${h - tailSize * 0.2}`,
      `C ${w + tailSize} ${h - tailSize * 0.1} ${w + tailSize * 0.3} ${h} ${w} ${h - 2}`,
      // Bottom-right corner
      `A ${baseRadius} ${baseRadius} 0 0 1 ${w - baseRadius} ${h}`,
      // Bottom edge
      `L ${baseRadius} ${h}`,
      `A ${baseRadius} ${baseRadius} 0 0 1 0 ${h - baseRadius}`,
      // Left edge back to start
      `L 0 ${baseRadius}`,
      "Z",
    ].join(" ");
  } else {
    // Received message - tail on left
    path = [
      // Start at top-left corner (after radius)
      `M ${baseRadius} 0`,
      // Top edge
      `L ${w - baseRadius} 0`,
      `A ${baseRadius} ${baseRadius} 0 0 1 ${w} ${baseRadius}`,
      // Right edge
      `L ${w} ${h - baseRadius}`,
      `A ${baseRadius} ${baseRadius} 0 0 1 ${w - baseRadius} ${h}`,
      // Bottom edge
      `L ${baseRadius} ${h}`,
      // Bottom-left corner and tail start
      `Q ${tailSize * 0.5} ${h} 0 ${h - tailSize * 0.5}`,
      `Q ${-tailSize * 0.5} ${h} 0 ${h - tailSize}`,
      // Continue up left edge
      `L 0 ${baseRadius}`,
      `A ${baseRadius} ${baseRadius} 0 0 1 ${baseRadius} 0`,
      "Z",
    ].join(" ");
  }

  // Cache the result
  tailPathCache.set(width, height, fromMe, path);
  return path;
}

// ─── Attachment rendering ─────────────────────────────────────────────────────

type AttachmentLoadState = "idle" | "loading" | "loaded" | "error";

/** Renders a single image attachment, fetching the data URI via Tauri IPC. */
const AttachmentImage = memo(function AttachmentImage({
  attachment,
  borderRadius,
  onClick,
}: {
  attachment: Attachment;
  borderRadius?: string;
  onClick?: () => void;
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
        cursor: onClick ? "pointer" : "default",
      }}
      loading="lazy"
      onClick={onClick}
    />
  );
});

/** Renders all attachments for a message, stacked vertically. */
function AttachmentRenderer({
  attachments,
  borderRadius,
  onImageClick,
}: {
  attachments: Attachment[];
  borderRadius?: string;
  onImageClick?: (index: number) => void;
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
              onClick={() => onImageClick?.(idx)}
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
  onContextMenu?: (message: Message, event: React.MouseEvent) => void;
  onReaction?: (messageGuid: string, reaction: string) => void;
  getContactName?: (handleId: number | null) => string;
}

export const MessageBubble = memo(function MessageBubble({
  message,
  isGroupChat = false,
  senderName,
  isFirstInGroup = true,
  isLastInGroup = true,
  isImessage = true,
  showTimestamp = false,
  onContextMenu,
  onReaction,
  getContactName,
}: MessageBubbleProps) {
  const isSent = message.is_from_me;
  const isBigEmoji = message.big_emoji === true || detectBigEmoji(message.text);
  const hasError = message.error !== 0;
  const isGroupEvent = message.item_type !== 0;

  const bubbleRef = useRef<HTMLDivElement>(null);
  const [tailClipPath, setTailClipPath] = useState<string | null>(null);
  const [isVisible, setIsVisible] = useState(true);
  const longPressTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const longPressStartRef = useRef<{ x: number; y: number } | null>(null);
  const [showReactionPicker, setShowReactionPicker] = useState(false);
  const [showReactionDetails, setShowReactionDetails] = useState(false);

  const { openLightbox } = useAttachmentStore();

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

  // iOS-style corner radius
  const borderRadius = useMemo(() => {
    const lg = "var(--bubble-radius-large)";
    const sm = "var(--bubble-radius-small)";
    if (isSent) {
      const topRight = isFirstInGroup ? lg : sm;
      const bottomRight = isLastInGroup ? lg : sm;
      return `${lg} ${topRight} ${bottomRight} ${lg}`;
    }
    const topLeft = isFirstInGroup ? lg : sm;
    const bottomLeft = isLastInGroup ? lg : sm;
    return `${topLeft} ${lg} ${lg} ${bottomLeft}`;
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

  // Optimized tail calculation with shared observer and visibility check
  useLayoutEffect(() => {
    if (!isLastInGroup || isBigEmoji) {
      setTailClipPath(null);
      return;
    }

    const el = bubbleRef.current;
    if (!el) return;

    const update = () => {
      // Only calculate if visible (performance optimization)
      if (!isVisible) return;

      const rect = el.getBoundingClientRect();
      const path = buildTailPath(rect.width, rect.height, isSent);
      setTailClipPath(path ? `path("${path}")` : null);
    };

    // Initial calculation
    update();

    // Use shared ResizeObserver instead of creating new one
    sharedResizeObserver.observe(el, update);

    // IntersectionObserver to track visibility
    let intersectionObserver: IntersectionObserver | null = null;
    if (typeof IntersectionObserver !== "undefined") {
      intersectionObserver = new IntersectionObserver(
        (entries) => {
          entries.forEach((entry) => {
            const visible = entry.isIntersecting;
            setIsVisible(visible);
            // Recalculate when becoming visible
            if (visible) update();
          });
        },
        {
          rootMargin: "100px", // Start calculating slightly before visible
          threshold: 0,
        }
      );
      intersectionObserver.observe(el);
    }

    return () => {
      sharedResizeObserver.unobserve(el);
      intersectionObserver?.disconnect();
    };
  }, [isLastInGroup, isBigEmoji, isSent, isVisible]);

  const handlePointerDown = useCallback(
    (event: React.PointerEvent) => {
      if (event.pointerType !== "touch") return;
      if (!onContextMenu) return;
      longPressStartRef.current = { x: event.clientX, y: event.clientY };
      if (longPressTimerRef.current) {
        clearTimeout(longPressTimerRef.current);
      }
      longPressTimerRef.current = setTimeout(() => {
        const start = longPressStartRef.current;
        if (!start) return;
        onContextMenu(
          message,
          {
            clientX: start.x,
            clientY: start.y,
            preventDefault: () => {},
            stopPropagation: () => {},
          } as React.MouseEvent
        );
      }, 450);
    },
    [message, onContextMenu]
  );

  const clearLongPress = useCallback(() => {
    if (longPressTimerRef.current) {
      clearTimeout(longPressTimerRef.current);
      longPressTimerRef.current = null;
    }
    longPressStartRef.current = null;
  }, []);

  const handleImageClick = useCallback(
    async (index: number) => {
      if (!message.attachments) return;

      // Get all image attachments and their data URIs
      const imageAttachments = message.attachments.filter((a) =>
        a.mime_type?.startsWith("image/")
      );

      const images = await Promise.all(
        imageAttachments.map(async (att) => {
          if (!att.guid) return null;
          try {
            const dataUri = await tauriDownloadAttachment(att.guid);
            return {
              src: dataUri,
              alt: att.transfer_name ?? "Image",
              messageGuid: message.guid ?? undefined,
              attachmentGuid: att.guid,
            };
          } catch {
            return null;
          }
        })
      );

      const validImages = images.filter((img) => img !== null);
      if (validImages.length > 0) {
        openLightbox(validImages, index);
      }
    },
    [message.attachments, message.guid, openLightbox]
  );

  const handleReactionSelect = useCallback(
    (reaction: string) => {
      if (message.guid && onReaction) {
        onReaction(message.guid, reaction);
      }
    },
    [message.guid, onReaction]
  );

  const handleReactionClick = useCallback(() => {
    setShowReactionDetails(true);
  }, []);

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
        borderRadius: tailClipPath ? undefined : borderRadius,
        clipPath: tailClipPath ?? undefined,
        WebkitClipPath: tailClipPath ?? undefined,
        padding: useImageLayout ? "0" : "var(--bubble-padding-v) var(--bubble-padding-h)",
        minWidth: 44,
        minHeight: useImageLayout ? 0 : "var(--bubble-min-height)",
        fontSize: "var(--font-bubble-text)",
        lineHeight: 1.32,
        wordBreak: "break-word" as const,
        whiteSpace: "pre-wrap" as const,
        position: "relative" as const,
        overflow: useImageLayout ? "hidden" : undefined,
      };

  // Tail padding adjustment
  if (!isBigEmoji && !useImageLayout && tailClipPath) {
    if (isSent) {
      bubbleStyle.paddingRight = `calc(var(--bubble-padding-h) + ${tailPadding}px)`;
    } else {
      bubbleStyle.paddingLeft = `calc(var(--bubble-padding-h) + ${tailPadding}px)`;
    }
  }

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
      <div
        style={{ position: "relative", maxWidth: "var(--bubble-max-width)" }}
        onContextMenu={(e) => {
          e.preventDefault();
          onContextMenu?.(message, e);
        }}
        onPointerDown={handlePointerDown}
        onPointerUp={clearLongPress}
        onPointerCancel={clearLongPress}
        onPointerLeave={clearLongPress}
        onPointerMove={(event) => {
          if (event.pointerType !== "touch" || !longPressStartRef.current) return;
          const dx = event.clientX - longPressStartRef.current.x;
          const dy = event.clientY - longPressStartRef.current.y;
          if (Math.hypot(dx, dy) > 8) clearLongPress();
        }}
      >
        <div style={bubbleStyle} ref={bubbleRef}>
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
                onImageClick={handleImageClick}
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
            gap: 4,
            marginTop: -6,
            marginBottom: 2,
            [isSent ? "marginRight" : "marginLeft"]: 12,
          }}
        >
          {(() => {
            // Group reactions by type
            const reactionGroups = new Map<string, { emoji: string; count: number; isOwn: boolean }>();
            reactions.forEach((r) => {
              const type = r.associated_message_type?.toLowerCase() || "unknown";
              const normalized = type.replace(/[^a-z]/g, "");
              const emoji = getReactionEmoji(r.associated_message_type);

              if (!reactionGroups.has(normalized)) {
                reactionGroups.set(normalized, { emoji, count: 0, isOwn: r.is_from_me });
              }
              const group = reactionGroups.get(normalized)!;
              group.count++;
              if (r.is_from_me) group.isOwn = true;
            });

            return Array.from(reactionGroups.values()).map((group, i) => (
              <ReactionBubble
                key={i}
                emoji={group.emoji}
                count={group.count}
                isOwnReaction={group.isOwn}
                onClick={handleReactionClick}
              />
            ));
          })()}
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

      {/* Reaction Picker */}
      {onReaction && (
        <ReactionPicker
          isOpen={showReactionPicker}
          onClose={() => setShowReactionPicker(false)}
          onSelectReaction={handleReactionSelect}
          anchorElement={bubbleRef.current}
          position={isSent ? "above" : "above"}
        />
      )}

      {/* Reaction Details Sheet */}
      {reactions && reactions.length > 0 && (
        <ReactionDetailSheet
          isOpen={showReactionDetails}
          onClose={() => setShowReactionDetails(false)}
          reactions={reactions}
          getContactName={getContactName}
        />
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
