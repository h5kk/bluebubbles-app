/**
 * ConversationView page - macOS Messages style.
 * Centered header with avatar/name, FaceTime button, bubble tails.
 */
import {
  useEffect,
  useRef,
  useCallback,
  useMemo,
  useState,
  type CSSProperties,
} from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useMessageStore } from "@/store/messageStore";
import { useChatStore } from "@/store/chatStore";
import { useSettingsStore } from "@/store/settingsStore";
import { useConnectionStore } from "@/store/connectionStore";
import { MessageBubble } from "@/components/MessageBubble";
import { InputBar } from "@/components/InputBar";
import { Avatar, GroupAvatar } from "@/components/Avatar";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { LoadingLine } from "@/components/LoadingLine";
import { ContextMenu, type ContextMenuItem } from "@/components/ContextMenu";
import { DragDropZone } from "@/components/DragDropZone";
import { ImageLightbox } from "@/components/ImageLightbox";
import type { Message } from "@/hooks/useTauri";
import { tauriSendReaction, tauriSendTypingIndicator } from "@/hooks/useTauri";
import { parseBBDateMs } from "@/utils/dateUtils";
import { getDemoName, getDemoMessageSnippet, getDemoAvatarUrl } from "@/utils/demoData";
import { useAttachmentStore } from "@/store/attachmentStore";
import { useContactStore } from "@/store/contactStore";

export function ConversationView() {
  const { chatGuid } = useParams<{ chatGuid: string }>();
  const navigate = useNavigate();
  const decodedGuid = chatGuid ? decodeURIComponent(chatGuid) : null;

  const {
    messages,
    loading,
    sending,
    hasMore,
    loadMessages,
    loadOlder,
    sendMessage,
  } = useMessageStore();
  const { chats, selectChat, markChatRead } = useChatStore();
  const { sendWithReturn, settings, demoMode, headerAvatarInline } = useSettingsStore();
  const { serverInfo } = useConnectionStore();
  const { addPendingAttachment } = useAttachmentStore();

  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<HTMLDivElement>(null);
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);

  // Context menu state
  const [contextMenu, setContextMenu] = useState<{
    open: boolean;
    x: number;
    y: number;
    message: Message | null;
  }>({ open: false, x: 0, y: 0, message: null });
  const [threadOriginGuid, setThreadOriginGuid] = useState<string | null>(null);

  // Check if typing indicators should be sent
  const shouldSendTyping =
    settings["privateSendTyping"] !== "false" &&
    settings["enablePrivateAPI"] === "true" &&
    serverInfo?.private_api === true;

  // Find the chat data for the header
  const chatPreview = useMemo(
    () => chats.find((c) => c.chat.guid === decodedGuid),
    [chats, decodedGuid]
  );
  const chatData = chatPreview?.chat;
  const isGroup = (chatData?.participants.length ?? 0) > 1;
  const isImessage = !decodedGuid?.startsWith("SMS");

  const title = useMemo(() => {
    if (!chatPreview) return decodedGuid ?? "Chat";
    const realTitle = chatData?.display_name ||
      chatPreview.participant_names.join(", ") ||
      chatData?.chat_identifier ||
      "Unknown";
    return demoMode ? getDemoName(realTitle, isGroup) : realTitle;
  }, [chatPreview, chatData, decodedGuid, demoMode, isGroup]);

  // Load messages when chat changes and mark as read
  useEffect(() => {
    if (decodedGuid) {
      selectChat(decodedGuid);
      loadMessages(decodedGuid);
      markChatRead(decodedGuid);
    }
  }, [decodedGuid, selectChat, loadMessages, markChatRead]);

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    const el = scrollContainerRef.current;
    if (el && !loading) {
      el.scrollTop = el.scrollHeight;
    }
  }, [messages.length, loading]);

  // Load older messages on scroll up
  const handleScroll = useCallback(() => {
    const el = scrollContainerRef.current;
    if (!el || loading || !hasMore) return;
    if (el.scrollTop < 100) {
      loadOlder();
    }
  }, [loading, hasMore, loadOlder]);

  // Typing indicator handler - sends start/stop typing to server
  const handleTypingChange = useCallback(
    (isTyping: boolean) => {
      if (!shouldSendTyping || !decodedGuid) return;

      if (isTyping && !isTypingRef.current) {
        isTypingRef.current = true;
        tauriSendTypingIndicator(decodedGuid, "start").catch(() => {});
      }

      // Clear any existing timeout
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }

      if (isTyping) {
        // Auto-stop typing after 5 seconds of no input
        typingTimeoutRef.current = setTimeout(() => {
          if (isTypingRef.current && decodedGuid) {
            isTypingRef.current = false;
            tauriSendTypingIndicator(decodedGuid, "stop").catch(() => {});
          }
        }, 5000);
      } else {
        isTypingRef.current = false;
        tauriSendTypingIndicator(decodedGuid, "stop").catch(() => {});
      }
    },
    [shouldSendTyping, decodedGuid]
  );

  // Stop typing indicator when navigating away
  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
      if (isTypingRef.current && decodedGuid) {
        tauriSendTypingIndicator(decodedGuid, "stop").catch(() => {});
        isTypingRef.current = false;
      }
    };
  }, [decodedGuid]);

  // Send handler
  const handleSend = useCallback(
    (text: string) => {
      // Stop typing indicator when sending
      if (isTypingRef.current && decodedGuid) {
        isTypingRef.current = false;
        tauriSendTypingIndicator(decodedGuid, "stop").catch(() => {});
      }
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
      sendMessage(text);
    },
    [sendMessage, decodedGuid]
  );

  // Attachment send handler
  const handleSendAttachment = useCallback(
    (file: File) => {
      console.log("Sending attachment:", file.name, file.type, file.size);
      // TODO: Implement actual attachment sending via Tauri command
      // For now, just log it
      alert(`Screenshot pasted! File: ${file.name} (${Math.round(file.size / 1024)}KB)\n\nAttachment sending not yet implemented in backend.`);
    },
    []
  );

  // Message context menu handler
  const handleMessageContextMenu = useCallback(
    (message: Message, event: React.MouseEvent) => {
      setContextMenu({
        open: true,
        x: event.clientX,
        y: event.clientY,
        message,
      });
    },
    []
  );

  const closeContextMenu = useCallback(() => {
    setContextMenu((prev) => ({ ...prev, open: false }));
  }, []);

  // Exit thread mode when clicking outside the conversation view
  useEffect(() => {
    if (!threadOriginGuid) return;
    const handlePointerDown = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      if (target?.closest?.("[data-context-menu]")) return;
      if (viewRef.current && target && !viewRef.current.contains(target)) {
        setThreadOriginGuid(null);
      }
    };
    window.addEventListener("mousedown", handlePointerDown);
    return () => window.removeEventListener("mousedown", handlePointerDown);
  }, [threadOriginGuid]);

  // Context menu actions
  const handleCopyMessage = useCallback(() => {
    if (contextMenu.message?.text) {
      navigator.clipboard.writeText(contextMenu.message.text);
    }
  }, [contextMenu.message]);

  const handleReplyToMessage = useCallback(() => {
    const origin =
      contextMenu.message?.thread_originator_guid ||
      contextMenu.message?.associated_message_guid ||
      contextMenu.message?.guid ||
      null;
    setThreadOriginGuid(origin);
  }, [contextMenu.message]);

  const handleReactToMessage = useCallback((reaction: string) => {
    if (!decodedGuid || !contextMenu.message?.guid) return;
    const messageText = contextMenu.message.text ?? "";
    tauriSendReaction(decodedGuid, messageText, contextMenu.message.guid, reaction).catch(() => {});
  }, [contextMenu.message, decodedGuid]);

  const handleReaction = useCallback(
    (messageGuid: string, reaction: string) => {
      if (!decodedGuid) return;
      const message = messages.find((m) => m.guid === messageGuid);
      if (!message) return;
      const messageText = message.text ?? "";
      tauriSendReaction(decodedGuid, messageText, messageGuid, reaction).catch(() => {});
    },
    [decodedGuid, messages]
  );

  const handleFileDrop = useCallback(
    (files: File[]) => {
      if (!decodedGuid) return;
      files.forEach((file) => {
        if (file.type.startsWith("image/")) {
          addPendingAttachment(file, decodedGuid);
        } else {
          handleSendAttachment(file);
        }
      });
    },
    [decodedGuid, addPendingAttachment, handleSendAttachment]
  );

  const getContactName = useCallback(
    (handleId: number | null): string => {
      if (!handleId) return "Unknown";
      const participant = chatData?.participants.find((p) => p.id === handleId);
      if (!participant) return "Unknown";

      // Use participant names from chatPreview if available
      const participantIndex = chatData?.participants.findIndex((p) => p.id === handleId);
      if (participantIndex !== undefined && participantIndex >= 0 && chatPreview?.participant_names[participantIndex]) {
        const name = chatPreview.participant_names[participantIndex];
        return demoMode ? getDemoName(name) : name;
      }

      return participant.formatted_address || participant.address || "Unknown";
    },
    [chatData, chatPreview, demoMode]
  );

  // Build message context menu items
  const messageContextMenuItems: ContextMenuItem[] = useMemo(() => {
    if (!contextMenu.message) return [];

    const items: ContextMenuItem[] = [];

    if (contextMenu.message.text) {
      items.push({
        label: "Copy",
        icon: "📋",
        onClick: handleCopyMessage,
      });
    }

    items.push({
      label: "Reply",
      icon: "↩️",
      onClick: handleReplyToMessage,
    });

    items.push({ label: "", onClick: () => {}, divider: true });

    items.push({
      label: "❤️ Love",
      onClick: () => handleReactToMessage("love"),
    });

    items.push({
      label: "👍 Like",
      onClick: () => handleReactToMessage("like"),
    });

    items.push({
      label: "😂 Laugh",
      onClick: () => handleReactToMessage("laugh"),
    });

    items.push({
      label: "❗ Emphasize",
      onClick: () => handleReactToMessage("emphasize"),
    });

    return items;
  }, [contextMenu.message, handleCopyMessage, handleReplyToMessage, handleReactToMessage]);

  const threadOriginMessage = useMemo(() => {
    if (!threadOriginGuid) return null;
    return messages.find((msg) => msg.guid === threadOriginGuid) ?? null;
  }, [messages, threadOriginGuid]);

  const threadMessageGuids = useMemo(() => {
    if (!threadOriginGuid) return new Set<string>();
    const set = new Set<string>();
    for (const msg of messages) {
      if (!msg.guid) continue;
      const inThread =
        msg.guid === threadOriginGuid ||
        msg.thread_originator_guid === threadOriginGuid ||
        msg.associated_message_guid === threadOriginGuid;
      if (inThread) {
        set.add(msg.guid);
      }
    }
    return set;
  }, [messages, threadOriginGuid]);

  // Group messages by sender for bubble grouping
  const groupedMessages = useMemo(() => {
    return messages.map((msg, idx) => {
      const prev = idx < messages.length - 1 ? messages[idx + 1] : null;
      const next = idx > 0 ? messages[idx - 1] : null;

      // Group events (item_type != 0) always break grouping
      const isGroupEvent = msg.item_type !== 0;
      const prevIsGroupEvent = prev != null && prev.item_type !== 0;
      const nextIsGroupEvent = next != null && next.item_type !== 0;

      const sameSenderAsPrev =
        !isGroupEvent && !prevIsGroupEvent &&
        prev != null && prev.is_from_me === msg.is_from_me && prev.handle_id === msg.handle_id;
      const sameSenderAsNext =
        !isGroupEvent && !nextIsGroupEvent &&
        next != null && next.is_from_me === msg.is_from_me && next.handle_id === msg.handle_id;

      let showTimestamp = false;
      if (prev && msg.date_created && prev.date_created) {
        const currTime = parseDate(msg.date_created);
        const prevTime = parseDate(prev.date_created);
        if (currTime && prevTime && currTime - prevTime > 15 * 60 * 1000) {
          showTimestamp = true;
        }
      }
      if (!prev) showTimestamp = true;

      // Check if the next (newer) message would show a timestamp break,
      // which means the current message is the last in its visual group
      let nextHasTimestampBreak = false;
      if (next && next.date_created && msg.date_created) {
        const nextTime = parseDate(next.date_created);
        const currTime = parseDate(msg.date_created);
        if (nextTime && currTime && nextTime - currTime > 15 * 60 * 1000) {
          nextHasTimestampBreak = true;
        }
      }

      const realSenderName = getSenderName(msg, chatPreview);
      return {
        message: msg,
        isFirstInGroup: !sameSenderAsPrev || showTimestamp,
        isLastInGroup: !sameSenderAsNext || nextHasTimestampBreak,
        showTimestamp,
        senderName: demoMode && realSenderName ? getDemoName(realSenderName) : realSenderName,
      };
    });
  }, [messages, chatPreview, demoMode]);

  // Memoize the reversed array to avoid allocating a new copy on every render
  const reversedMessages = useMemo(
    () => [...groupedMessages].reverse(),
    [groupedMessages]
  );

  const containerStyle: CSSProperties = {
    display: "grid",
    gridTemplateRows: "auto 1fr auto",
    flex: 1,
    height: "100%",
    minHeight: 0, // Critical for flex children with overflow
    overflow: "hidden",
    backgroundColor: "var(--color-surface)",
  };

  const headerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    padding: "10px 16px",
    borderBottom: "1px solid var(--color-surface-variant)",
    backgroundColor: "var(--color-surface)",
    position: "relative",
    minHeight: 56,
    gridRow: 1,
  };

  // Empty state when no chat is selected
  if (!decodedGuid) {
    return (
      <div
        style={{
          flex: 1,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexDirection: "column",
          gap: 12,
          color: "var(--color-on-surface-variant)",
        }}
      >
        <span style={{ fontSize: 48, opacity: 0.4 }}>
          {/* Chat bubble icon */}
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
            <path d="M8 10C8 7.79 9.79 6 12 6H36C38.21 6 40 7.79 40 10V30C40 32.21 38.21 34 36 34H16L8 42V10Z" fill="currentColor" opacity="0.3"/>
          </svg>
        </span>
        <span style={{ fontSize: "var(--font-title-large)", fontWeight: 500 }}>
          Select a conversation
        </span>
        <span style={{ fontSize: "var(--font-body-medium)" }}>
          Choose a chat from the sidebar to start messaging
        </span>
      </div>
    );
  }

  return (
    <div style={containerStyle} ref={viewRef} data-conversation-view>
      {/* Chat header - centered avatar and name */}
      <div
        style={headerStyle}
        onClick={() =>
          navigate(`/chat/${encodeURIComponent(decodedGuid)}/details`)
        }
      >
        {/* Center content: avatar + name */}
        <div
          style={{
            display: "flex",
            flexDirection: headerAvatarInline ? "row" : "column",
            alignItems: "center",
            gap: headerAvatarInline ? 10 : 2,
            cursor: "pointer",
          }}
        >
          {chatData && isGroup ? (
            <GroupAvatar
              participants={chatData.participants.map((p, i) => {
                const participantName = chatPreview?.participant_names[i] ?? p.address;
                return {
                  name: demoMode ? getDemoName(participantName) : participantName,
                  address: p.address,
                  avatarUrl: demoMode ? getDemoAvatarUrl(getDemoName(participantName), p.address) : undefined,
                };
              })}
              size={38}
            />
          ) : (
            <Avatar
              name={title}
              address={chatData?.participants[0]?.address ?? decodedGuid}
              size={38}
              avatarUrl={demoMode ? getDemoAvatarUrl(title, chatData?.participants[0]?.address ?? decodedGuid) : undefined}
            />
          )}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: headerAvatarInline ? "flex-start" : "center",
              gap: 2,
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 4,
              }}
            >
              <span
                style={{
                  fontSize: 13,
                  fontWeight: 600,
                  color: "var(--color-on-surface)",
                }}
              >
                {title}
              </span>
              {/* Chevron */}
              <svg width="8" height="12" viewBox="0 0 8 12" fill="none" style={{ opacity: 0.4 }}>
                <path d="M1.5 1L6.5 6L1.5 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </div>
            {chatData && (
              <span
                style={{
                  fontSize: 11,
                  color: isImessage && !isGroup ? "#007AFF" : "var(--color-outline)",
                  fontWeight: 500,
                }}
              >
                {isGroup
                  ? `${chatData.participants.length} participants`
                  : isImessage
                    ? "iMessage"
                    : "SMS"}
              </span>
            )}
            {/* Blue progress line under contact info when sending */}
            <LoadingLine
              visible={sending}
              height={2}
              style={{
                borderRadius: 1,
                marginTop: chatData ? 2 : 4,
                width: headerAvatarInline ? 56 : 40,
                alignSelf: headerAvatarInline ? "flex-start" : "center",
              }}
            />
          </div>
        </div>

        {/* FaceTime / Video call button - top right */}
        <button
          aria-label="FaceTime"
          style={{
            position: "absolute",
            right: 16,
            top: "50%",
            transform: "translateY(-50%)",
            width: 32,
            height: 32,
            borderRadius: 6,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "var(--color-on-surface-variant)",
            background: "transparent",
            cursor: "pointer",
          }}
        >
          {/* Video camera icon */}
          <svg width="20" height="16" viewBox="0 0 20 16" fill="none">
            <rect x="1" y="2" width="13" height="12" rx="2" stroke="currentColor" strokeWidth="1.5" fill="none" />
            <path d="M14 6L19 3V13L14 10V6Z" fill="currentColor" />
          </svg>
        </button>
      </div>

      {/* Messages area */}
      <DragDropZone onFileDrop={handleFileDrop} accept="image/*,video/*,audio/*,.pdf,.doc,.docx">
        <div
          ref={scrollContainerRef}
          onScroll={handleScroll}
          data-chat-scroll
          style={{
            overflowY: "auto",
            overflowX: "hidden",
            display: "flex",
            flexDirection: "column",
            padding: "8px 0",
            minHeight: 0,
            flex: 1,
          }}
        >
        {/* Load more spinner */}
        {loading && hasMore && (
          <div
            style={{
              display: "flex",
              justifyContent: "center",
              padding: 16,
            }}
          >
            <LoadingSpinner size={20} />
          </div>
        )}

        {threadOriginGuid && (
          <div
            style={{
              position: "sticky",
              top: 0,
              zIndex: 2,
              padding: "8px 12px",
              marginBottom: 6,
              background: "color-mix(in srgb, var(--color-surface) 92%, transparent)",
              borderBottom: "1px solid var(--color-surface-variant)",
              backdropFilter: "blur(6px)",
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              gap: 12,
            }}
          >
            <div style={{ minWidth: 0 }}>
              <div
                style={{
                  fontSize: "var(--font-body-medium)",
                  fontWeight: 600,
                  color: "var(--color-on-surface)",
                }}
              >
                Reply Thread
              </div>
              <div
                style={{
                  fontSize: "var(--font-body-small)",
                  color: "var(--color-on-surface-variant)",
                  whiteSpace: "nowrap",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  maxWidth: 360,
                }}
              >
                {threadOriginMessage?.text ?? "Original message"}
              </div>
            </div>
            <button
              onClick={() => setThreadOriginGuid(null)}
              style={{
                padding: "6px 10px",
                borderRadius: 8,
                border: "1px solid var(--color-outline)",
                background: "var(--color-surface-variant)",
                color: "var(--color-on-surface)",
                cursor: "pointer",
                fontSize: "var(--font-body-small)",
              }}
            >
              Show All
            </button>
          </div>
        )}

        {/* Messages rendered oldest-first (reversed view is memoized in groupedMessages) */}
        {reversedMessages.map((item) => {
          const guid = item.message.guid ?? "";
          const inThread = !threadOriginGuid || (guid && threadMessageGuids.has(guid));
          return (
            <div
              key={item.message.guid ?? item.message.id}
              style={{
                opacity: inThread ? 1 : 0.35,
                filter: inThread ? "none" : "grayscale(0.2)",
                transition: "opacity 150ms ease",
                cursor: threadOriginGuid && !inThread ? "pointer" : "default",
              }}
              onClick={() => {
                if (threadOriginGuid && !inThread) setThreadOriginGuid(null);
              }}
            >
              <MessageBubble
                message={item.message}
                isGroupChat={isGroup}
                senderName={item.senderName}
                isFirstInGroup={item.isFirstInGroup}
                isLastInGroup={item.isLastInGroup}
                isImessage={isImessage}
                showTimestamp={item.showTimestamp}
                onContextMenu={handleMessageContextMenu}
                onReaction={handleReaction}
                getContactName={getContactName}
              />
            </div>
          );
        })}

        {/* Loading initial messages */}
        {loading && messages.length === 0 && (
          <div
            style={{
              flex: 1,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <LoadingSpinner size={28} />
          </div>
        )}

        {/* No messages */}
        {!loading && messages.length === 0 && (
          <div
            className="animate-fade-in"
            style={{
              flex: 1,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: "var(--color-on-surface-variant)",
              fontSize: "var(--font-body-medium)",
            }}
          >
            No messages yet. Say hello!
          </div>
        )}
        </div>
      </DragDropZone>

      {/* Input bar */}
      <div style={{ gridRow: 3, minHeight: "var(--input-bar-min-height)" }}>
        <InputBar
          onSend={handleSend}
          onSendAttachment={handleSendAttachment}
          onTyping={handleTypingChange}
          sending={sending}
          sendWithReturn={sendWithReturn}
          placeholder={isImessage ? "iMessage" : "Text Message"}
          chatGuid={decodedGuid}
        />
      </div>

      {/* Message context menu */}
      <ContextMenu
        open={contextMenu.open}
        x={contextMenu.x}
        y={contextMenu.y}
        items={messageContextMenuItems}
        onClose={closeContextMenu}
      />

      {/* Image Lightbox */}
      <ImageLightbox />
    </div>
  );
}

function parseDate(dateStr: string): number | null {
  return parseBBDateMs(dateStr);
}

function getSenderName(
  msg: Message,
  chatPreview:
    | { participant_names: string[]; chat: { participants: Array<{ id: number | null }> } }
    | undefined
): string | undefined {
  if (msg.is_from_me || !chatPreview) return undefined;
  const handleId = msg.handle_id;
  const idx = chatPreview.chat.participants.findIndex((p) => p.id === handleId);
  if (idx >= 0 && chatPreview.participant_names[idx]) {
    return chatPreview.participant_names[idx];
  }
  return undefined;
}
