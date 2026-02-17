/**
 * ConversationView page - macOS Messages style.
 * Centered header with avatar/name, FaceTime button, bubble tails.
 */
import {
  useEffect,
  useRef,
  useCallback,
  useMemo,
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
import type { Message } from "@/hooks/useTauri";
import { tauriSendTypingIndicator } from "@/hooks/useTauri";
import { parseBBDateMs } from "@/utils/dateUtils";

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
  const { sendWithReturn, settings } = useSettingsStore();
  const { serverInfo } = useConnectionStore();

  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);

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
    return (
      chatData?.display_name ||
      chatPreview.participant_names.join(", ") ||
      chatData?.chat_identifier ||
      "Unknown"
    );
  }, [chatPreview, chatData, decodedGuid]);

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

      return {
        message: msg,
        isFirstInGroup: !sameSenderAsPrev || showTimestamp,
        isLastInGroup: !sameSenderAsNext || nextHasTimestampBreak,
        showTimestamp,
        senderName: getSenderName(msg, chatPreview),
      };
    });
  }, [messages, chatPreview]);

  // Memoize the reversed array to avoid allocating a new copy on every render
  const reversedMessages = useMemo(
    () => [...groupedMessages].reverse(),
    [groupedMessages]
  );

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100%",
    overflow: "hidden",
  };

  const headerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    padding: "10px 16px",
    borderBottom: "1px solid var(--color-surface-variant)",
    backgroundColor: "var(--color-surface)",
    flexShrink: 0,
    position: "relative",
    minHeight: 56,
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
    <div style={containerStyle}>
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
            flexDirection: "column",
            alignItems: "center",
            gap: 2,
            cursor: "pointer",
          }}
        >
          {chatData && isGroup ? (
            <GroupAvatar
              participants={chatData.participants.map((p, i) => ({
                name: chatPreview?.participant_names[i] ?? p.address,
                address: p.address,
              }))}
              size={32}
            />
          ) : (
            <Avatar
              name={title}
              address={chatData?.participants[0]?.address ?? decodedGuid}
              size={32}
            />
          )}
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
                color: "var(--color-outline)",
              }}
            >
              {isGroup
                ? `${chatData.participants.length} participants`
                : isImessage
                  ? "iMessage"
                  : "SMS"}
            </span>
          )}
          {/* Blue progress line under contact photo when sending */}
          <LoadingLine
            visible={sending}
            height={2}
            style={{ borderRadius: 1, marginTop: 2, width: 40 }}
          />
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
      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        style={{
          flex: 1,
          overflow: "auto",
          display: "flex",
          flexDirection: "column",
          padding: "8px 0",
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

        {/* Messages rendered oldest-first (reversed view is memoized in groupedMessages) */}
        {reversedMessages.map((item) => (
          <MessageBubble
            key={item.message.guid ?? item.message.id}
            message={item.message}
            isGroupChat={isGroup}
            senderName={item.senderName}
            isFirstInGroup={item.isFirstInGroup}
            isLastInGroup={item.isLastInGroup}
            isImessage={isImessage}
            showTimestamp={item.showTimestamp}
          />
        ))}

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

      {/* Input bar */}
      <InputBar
        onSend={handleSend}
        onTyping={handleTypingChange}
        sending={sending}
        sendWithReturn={sendWithReturn}
        placeholder={isImessage ? "iMessage" : "Text Message"}
      />
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
