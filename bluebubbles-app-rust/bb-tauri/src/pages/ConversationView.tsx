/**
 * ConversationView page - displays messages for a specific chat.
 * Implements spec 03-conversation-view.md.
 */
import {
  useEffect,
  useRef,
  useCallback,
  useMemo,
  type CSSProperties,
} from "react";
import { useParams, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useMessageStore } from "@/store/messageStore";
import { useChatStore } from "@/store/chatStore";
import { useSettingsStore } from "@/store/settingsStore";
import { MessageBubble } from "@/components/MessageBubble";
import { InputBar } from "@/components/InputBar";
import { Avatar, GroupAvatar } from "@/components/Avatar";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import type { Message } from "@/hooks/useTauri";

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
  const { chats, selectChat } = useChatStore();
  const { sendWithReturn } = useSettingsStore();

  const scrollContainerRef = useRef<HTMLDivElement>(null);

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

  // Load messages when chat changes
  useEffect(() => {
    if (decodedGuid) {
      selectChat(decodedGuid);
      loadMessages(decodedGuid);
    }
  }, [decodedGuid, selectChat, loadMessages]);

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

  // Send handler
  const handleSend = useCallback(
    (text: string) => {
      sendMessage(text);
    },
    [sendMessage]
  );

  // Group messages by sender for bubble grouping
  const groupedMessages = useMemo(() => {
    return messages.map((msg, idx) => {
      const prev = idx < messages.length - 1 ? messages[idx + 1] : null;
      const next = idx > 0 ? messages[idx - 1] : null;

      // Messages are newest-first in the store, but we render oldest-first
      // so "previous" in display order is messages[idx+1]
      const sameSenderAsPrev =
        prev != null && prev.is_from_me === msg.is_from_me && prev.handle_id === msg.handle_id;
      const sameSenderAsNext =
        next != null && next.is_from_me === msg.is_from_me && next.handle_id === msg.handle_id;

      // Show timestamp if gap > 15 minutes from previous
      let showTimestamp = false;
      if (prev && msg.date_created && prev.date_created) {
        const currTime = parseDate(msg.date_created);
        const prevTime = parseDate(prev.date_created);
        if (currTime && prevTime && currTime - prevTime > 15 * 60 * 1000) {
          showTimestamp = true;
        }
      }
      if (!prev) showTimestamp = true;

      return {
        message: msg,
        isFirstInGroup: !sameSenderAsPrev || showTimestamp,
        isLastInGroup: !sameSenderAsNext,
        showTimestamp,
        senderName: getSenderName(msg, chatPreview),
      };
    });
  }, [messages, chatPreview]);

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100%",
    overflow: "hidden",
  };

  const headerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 12,
    padding: "10px 16px",
    borderBottom: "1px solid var(--color-surface-variant)",
    backgroundColor: "var(--color-surface)",
    flexShrink: 0,
    cursor: "pointer",
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
        <span style={{ fontSize: 48 }}>{"\uD83D\uDCAC"}</span>
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
      {/* Chat header */}
      <div
        style={headerStyle}
        onClick={() =>
          navigate(`/chat/${encodeURIComponent(decodedGuid)}/details`)
        }
      >
        {chatData && isGroup ? (
          <GroupAvatar
            participants={chatData.participants.map((p, i) => ({
              name: chatPreview?.participant_names[i] ?? p.address,
              address: p.address,
            }))}
            size={36}
          />
        ) : (
          <Avatar
            name={title}
            address={chatData?.participants[0]?.address ?? decodedGuid}
            size={36}
          />
        )}
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontSize: "var(--font-body-large)",
              fontWeight: 600,
              color: "var(--color-on-surface)",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {title}
          </div>
          {chatData && (
            <div
              style={{
                fontSize: "var(--font-label-small)",
                color: "var(--color-outline)",
              }}
            >
              {isGroup
                ? `${chatData.participants.length} participants`
                : isImessage
                  ? "iMessage"
                  : "SMS"}
            </div>
          )}
        </div>
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

        {/* Messages rendered oldest-first (reversed from store order) */}
        {[...groupedMessages].reverse().map((item) => (
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
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
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
          </motion.div>
        )}
      </div>

      {/* Input bar */}
      <InputBar
        onSend={handleSend}
        sending={sending}
        sendWithReturn={sendWithReturn}
        placeholder={isImessage ? "iMessage" : "Text Message"}
      />
    </div>
  );
}

function parseDate(dateStr: string): number | null {
  try {
    const ts = Number(dateStr);
    if (!isNaN(ts)) return ts;
    return new Date(dateStr).getTime();
  } catch {
    return null;
  }
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
