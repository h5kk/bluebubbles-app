/**
 * NewMessage page - compose a new message to a contact.
 */
import { useState, useCallback, useRef, useEffect, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useChatStore } from "@/store/chatStore";
import { Avatar } from "@/components/Avatar";
import type { ChatWithPreview } from "@/hooks/useTauri";

export function NewMessage() {
  const navigate = useNavigate();
  const { chats } = useChatStore();
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Filter chats based on query
  const filtered = query.trim()
    ? chats.filter((c) => {
        const name =
          c.chat.display_name ||
          c.participant_names.join(", ") ||
          c.chat.chat_identifier ||
          "";
        return name.toLowerCase().includes(query.toLowerCase());
      })
    : chats.slice(0, 20);

  const handleSelect = useCallback(
    (chat: ChatWithPreview) => {
      navigate(`/chat/${encodeURIComponent(chat.chat.guid)}`);
    },
    [navigate]
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
    gap: 12,
    padding: "12px 16px",
    borderBottom: "1px solid var(--color-surface-variant)",
    backgroundColor: "var(--color-surface)",
    flexShrink: 0,
  };

  return (
    <div style={containerStyle}>
      <div style={headerStyle}>
        <button
          onClick={() => navigate(-1)}
          style={{
            width: 32,
            height: 32,
            borderRadius: "50%",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 16,
            cursor: "pointer",
            backgroundColor: "transparent",
          }}
          aria-label="Back"
        >
          {"\u2190"}
        </button>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontSize: "var(--font-body-large)",
              fontWeight: 600,
              color: "var(--color-on-surface)",
            }}
          >
            New Message
          </div>
        </div>
      </div>

      {/* To field */}
      <div
        style={{
          padding: "12px 16px",
          borderBottom: "1px solid var(--color-surface-variant)",
          display: "flex",
          alignItems: "center",
          gap: 8,
          flexShrink: 0,
        }}
      >
        <span
          style={{
            fontSize: "var(--font-body-medium)",
            color: "var(--color-on-surface-variant)",
            fontWeight: 500,
          }}
        >
          To:
        </span>
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search contacts or conversations"
          style={{
            flex: 1,
            padding: "6px 0",
            fontSize: "var(--font-body-medium)",
            color: "var(--color-on-surface)",
            backgroundColor: "transparent",
          }}
        />
      </div>

      {/* Results */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {filtered.length === 0 && query.trim() && (
          <div
            style={{
              padding: 24,
              textAlign: "center",
              color: "var(--color-on-surface-variant)",
              fontSize: "var(--font-body-medium)",
            }}
          >
            No conversations found
          </div>
        )}
        {filtered.map((chat) => {
          const isGroup = chat.chat.participants.length > 1;
          const title =
            chat.chat.display_name ||
            chat.participant_names.join(", ") ||
            chat.chat.chat_identifier ||
            "Unknown";

          return (
            <motion.div
              key={chat.chat.guid}
              onClick={() => handleSelect(chat)}
              whileTap={{ scale: 0.98 }}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                padding: "10px 16px",
                cursor: "pointer",
                borderBottom: "1px solid var(--color-surface-variant)",
              }}
            >
              <Avatar
                name={title}
                address={
                  isGroup
                    ? chat.chat.guid
                    : chat.chat.participants[0]?.address ?? chat.chat.guid
                }
                size={40}
              />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div
                  style={{
                    fontSize: "var(--font-body-medium)",
                    fontWeight: 500,
                    color: "var(--color-on-surface)",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  }}
                >
                  {title}
                </div>
                {chat.chat.chat_identifier && (
                  <div
                    style={{
                      fontSize: "var(--font-body-small)",
                      color: "var(--color-on-surface-variant)",
                    }}
                  >
                    {chat.chat.chat_identifier}
                  </div>
                )}
              </div>
            </motion.div>
          );
        })}
      </div>
    </div>
  );
}
