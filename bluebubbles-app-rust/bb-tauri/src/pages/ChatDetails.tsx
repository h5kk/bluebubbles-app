/**
 * ChatDetails page - shows conversation details, participants, and actions.
 * Accessible by clicking the header in ConversationView.
 */
import { useMemo, useState, useCallback, type CSSProperties } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { useChatStore } from "@/store/chatStore";
import { Avatar } from "@/components/Avatar";
import type { Handle } from "@/hooks/useTauri";

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
          onClick={() => navigate(`/chat/${encodeURIComponent(decodedGuid)}`)}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 4,
            padding: "4px 8px 4px 2px",
            borderRadius: 8,
            color: "var(--color-primary)",
            cursor: "pointer",
            backgroundColor: "transparent",
            fontSize: "var(--font-body-medium)",
            fontWeight: 400,
          }}
          aria-label="Go back"
        >
          <svg width="10" height="18" viewBox="0 0 10 18" fill="none">
            <path d="M9 1L1.5 9L9 17" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          Back
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
          <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
            {chatData.participants.map((participant, idx) => (
              <ParticipantRow
                key={participant.address}
                participant={participant}
                displayName={chatPreview?.participant_names[idx] ?? participant.address}
                chats={chats}
                navigate={navigate}
              />
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

/**
 * Find the guid of a 1-on-1 chat with the given address, searching all loaded chats.
 */
function findDirectChatGuid(
  chats: Array<{ chat: { guid: string; participants: Handle[] } }>,
  address: string
): string | null {
  for (const c of chats) {
    if (
      c.chat.participants.length === 1 &&
      c.chat.participants[0].address === address
    ) {
      return c.chat.guid;
    }
  }
  return null;
}

function ParticipantRow({
  participant,
  displayName,
  chats,
  navigate,
}: {
  participant: Handle;
  displayName: string;
  chats: Array<{ chat: { guid: string; participants: Handle[] } }>;
  navigate: ReturnType<typeof useNavigate>;
}) {
  const [hovered, setHovered] = useState(false);
  const [copiedAddress, setCopiedAddress] = useState(false);

  const directChatGuid = useMemo(
    () => findDirectChatGuid(chats, participant.address),
    [chats, participant.address]
  );

  const handleNavigateToChat = useCallback(() => {
    if (directChatGuid) {
      navigate(`/chat/${encodeURIComponent(directChatGuid)}`);
    }
  }, [directChatGuid, navigate]);

  const handleCopyAddress = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      navigator.clipboard.writeText(participant.address).then(() => {
        setCopiedAddress(true);
        setTimeout(() => setCopiedAddress(false), 1500);
      });
    },
    [participant.address]
  );

  const handleMessageClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      handleNavigateToChat();
    },
    [handleNavigateToChat]
  );

  // The address is the phone number or email
  const addressDisplay = participant.formatted_address ?? participant.address;

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={handleNavigateToChat}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        padding: "10px 8px",
        borderRadius: 10,
        cursor: directChatGuid ? "pointer" : "default",
        backgroundColor: hovered ? "var(--color-surface-variant)" : "transparent",
        transition: "background-color 100ms ease",
      }}
    >
      <Avatar
        name={displayName}
        address={participant.address}
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
          {displayName}
        </div>
        <div
          style={{
            fontSize: "var(--font-body-small)",
            color: "var(--color-on-surface-variant)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {addressDisplay}
        </div>
      </div>

      {/* Action buttons */}
      <div
        style={{
          display: "flex",
          gap: 4,
          opacity: hovered ? 1 : 0,
          transition: "opacity 100ms ease",
        }}
      >
        {/* Message button */}
        {directChatGuid && (
          <button
            onClick={handleMessageClick}
            title="Send message"
            aria-label={`Message ${displayName}`}
            style={{
              width: 30,
              height: 30,
              borderRadius: "50%",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              background: "var(--color-surface)",
              border: "1px solid var(--color-surface-variant)",
              cursor: "pointer",
              color: "var(--color-primary)",
              padding: 0,
            }}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
            </svg>
          </button>
        )}

        {/* Copy address button */}
        <button
          onClick={handleCopyAddress}
          title={copiedAddress ? "Copied!" : `Copy ${addressDisplay}`}
          aria-label={`Copy address for ${displayName}`}
          style={{
            width: 30,
            height: 30,
            borderRadius: "50%",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: "var(--color-surface)",
            border: "1px solid var(--color-surface-variant)",
            cursor: "pointer",
            color: copiedAddress ? "var(--color-primary)" : "var(--color-on-surface-variant)",
            padding: 0,
          }}
        >
          {copiedAddress ? (
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          ) : (
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
              <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
            </svg>
          )}
        </button>
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
