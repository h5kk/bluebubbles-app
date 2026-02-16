/**
 * ChatDetails page - shows conversation details, participants, and actions.
 * Accessible by clicking the header in ConversationView.
 */
import { useMemo, useState, type CSSProperties } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useChatStore } from "@/store/chatStore";
import { Avatar } from "@/components/Avatar";

export function ChatDetails() {
  const { chatGuid } = useParams<{ chatGuid: string }>();
  const navigate = useNavigate();
  const decodedGuid = chatGuid ? decodeURIComponent(chatGuid) : null;
  const { chats } = useChatStore();

  const chatPreview = useMemo(
    () => chats.find((c) => c.chat.guid === decodedGuid),
    [chats, decodedGuid]
  );
  const chatData = chatPreview?.chat;
  const isGroup = (chatData?.participants.length ?? 0) > 1;

  const title = useMemo(() => {
    if (!chatPreview) return decodedGuid ?? "Chat";
    return (
      chatData?.display_name ||
      chatPreview.participant_names.join(", ") ||
      chatData?.chat_identifier ||
      "Unknown"
    );
  }, [chatPreview, chatData, decodedGuid]);

  const containerStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    height: "100%",
    overflow: "auto",
  };

  const headerStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: 12,
    padding: "12px 16px",
    borderBottom: "1px solid var(--color-surface-variant)",
    flexShrink: 0,
  };

  if (!decodedGuid || !chatData) {
    return (
      <div
        style={{
          flex: 1,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "var(--color-on-surface-variant)",
        }}
      >
        Chat not found
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      {/* Back header */}
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
            fontSize: 18,
            color: "var(--color-primary)",
            cursor: "pointer",
          }}
          aria-label="Go back"
        >
          {"\u2190"}
        </button>
        <span
          style={{
            fontSize: "var(--font-body-large)",
            fontWeight: 600,
            color: "var(--color-on-surface)",
          }}
        >
          Details
        </span>
      </div>

      {/* Profile section */}
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          padding: "32px 24px",
          gap: 12,
        }}
      >
        <Avatar
          name={title}
          address={chatData.participants[0]?.address ?? decodedGuid}
          size={80}
        />
        <h2
          style={{
            fontSize: "var(--font-title-large)",
            fontWeight: 600,
            color: "var(--color-on-surface)",
            textAlign: "center",
          }}
        >
          {title}
        </h2>
        {!isGroup && chatData.chat_identifier && (
          <span
            style={{
              fontSize: "var(--font-body-medium)",
              color: "var(--color-on-surface-variant)",
            }}
          >
            {chatData.chat_identifier}
          </span>
        )}
      </motion.div>

      {/* Quick actions */}
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          gap: 16,
          padding: "0 24px 24px",
        }}
      >
        <QuickAction icon={"\uD83D\uDD14"} label={chatData.mute_type ? "Unmute" : "Mute"} />
        <QuickAction icon={"\uD83D\uDCCC"} label={chatData.is_pinned ? "Unpin" : "Pin"} />
        <QuickAction icon={"\uD83D\uDCE6"} label={chatData.is_archived ? "Unarchive" : "Archive"} />
      </div>

      {/* Participants */}
      {isGroup && (
        <div style={{ padding: "0 24px 24px" }}>
          <h3
            style={{
              fontSize: "var(--font-body-large)",
              fontWeight: 600,
              color: "var(--color-on-surface)",
              marginBottom: 12,
              paddingBottom: 8,
              borderBottom: "1px solid var(--color-surface-variant)",
            }}
          >
            {chatData.participants.length} Participants
          </h3>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {chatData.participants.map((participant, idx) => (
              <div
                key={participant.address}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                  padding: "8px 0",
                }}
              >
                <Avatar
                  name={
                    chatPreview?.participant_names[idx] ?? participant.address
                  }
                  address={participant.address}
                  size={36}
                />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontSize: "var(--font-body-medium)",
                      color: "var(--color-on-surface)",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {chatPreview?.participant_names[idx] ?? participant.address}
                  </div>
                  <div
                    style={{
                      fontSize: "var(--font-body-small)",
                      color: "var(--color-on-surface-variant)",
                    }}
                  >
                    {participant.service === "iMessage" ? "iMessage" : "SMS"}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Chat info */}
      <div style={{ padding: "0 24px 32px" }}>
        <h3
          style={{
            fontSize: "var(--font-body-large)",
            fontWeight: 600,
            color: "var(--color-on-surface)",
            marginBottom: 12,
            paddingBottom: 8,
            borderBottom: "1px solid var(--color-surface-variant)",
          }}
        >
          Info
        </h3>
        <InfoRow label="Chat GUID" value={chatData.guid} />
        <InfoRow
          label="Service"
          value={chatData.guid.startsWith("SMS") ? "SMS" : "iMessage"}
        />
        {chatData.display_name && (
          <InfoRow label="Group Name" value={chatData.display_name} />
        )}
      </div>
    </div>
  );
}

function QuickAction({ icon, label }: { icon: string; label: string }) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 6,
        padding: "10px 16px",
        borderRadius: 16,
        backgroundColor: hovered
          ? "var(--color-surface-variant)"
          : "var(--color-surface)",
        border: "1px solid var(--color-surface-variant)",
        cursor: "pointer",
        transition: "background-color 100ms ease",
        minWidth: 72,
      }}
    >
      <span style={{ fontSize: 20 }}>{icon}</span>
      <span
        style={{
          fontSize: "var(--font-label-small)",
          color: "var(--color-on-surface-variant)",
        }}
      >
        {label}
      </span>
    </button>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "8px 0",
        gap: 16,
      }}
    >
      <span
        style={{
          fontSize: "var(--font-body-medium)",
          color: "var(--color-on-surface)",
        }}
      >
        {label}
      </span>
      <span
        style={{
          fontSize: "var(--font-body-medium)",
          color: "var(--color-on-surface-variant)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          maxWidth: 250,
          textAlign: "right",
        }}
      >
        {value}
      </span>
    </div>
  );
}
