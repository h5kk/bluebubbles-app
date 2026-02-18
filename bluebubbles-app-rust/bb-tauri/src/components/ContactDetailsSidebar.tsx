/**
 * ContactDetailsSidebar - Right-side panel showing contact/chat details.
 * Slides in from the right when clicking a user's name at the top of a conversation.
 * Modeled after the macOS Messages contact details panel.
 */
import { useState, useMemo, useEffect, type CSSProperties } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { Chat, Message } from "@/hooks/useTauri";
import { Avatar, GroupAvatar } from "./Avatar";
import { useMessageStore } from "@/store/messageStore";
import { useAttachmentStore } from "@/store/attachmentStore";
import { LocationMapCard } from "./LocationMapCard";
import { LocationMapPopover } from "./LocationMapPopover";
import { useFindMyStore } from "@/store/findMyStore";

interface ContactDetailsSidebarProps {
  open: boolean;
  onClose: () => void;
  chatData: Chat | undefined;
  participantNames: string[];
  title: string;
}

type InfoTab = "info" | "backgrounds" | "photos";

function ActionButton({
  icon,
  label,
}: {
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <button
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 4,
        background: "none",
        border: "none",
        cursor: "pointer",
        color: "var(--color-on-surface-variant)",
        padding: 0,
      }}
      aria-label={label}
      title={label}
    >
      <div
        style={{
          width: 44,
          height: 44,
          borderRadius: "50%",
          background: "var(--color-surface-variant)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontSize: 18,
        }}
      >
        {icon}
      </div>
      <span style={{ fontSize: 11, opacity: 0.7 }}>{label}</span>
    </button>
  );
}

/** Simple SVG icons for the action buttons. */
function PhoneIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M22 16.92v3a2 2 0 01-2.18 2 19.79 19.79 0 01-8.63-3.07 19.5 19.5 0 01-6-6 19.79 19.79 0 01-3.07-8.67A2 2 0 014.11 2h3a2 2 0 012 1.72c.127.96.361 1.903.7 2.81a2 2 0 01-.45 2.11L8.09 9.91a16 16 0 006 6l1.27-1.27a2 2 0 012.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0122 16.92z" />
    </svg>
  );
}

function VideoIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="23 7 16 12 23 17 23 7" />
      <rect x="1" y="5" width="15" height="14" rx="2" ry="2" />
    </svg>
  );
}

function MailIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" />
      <polyline points="22,6 12,13 2,6" />
    </svg>
  );
}

function MessageIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  );
}

function CopyIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
    </svg>
  );
}

function DetailRow({
  icon,
  label,
  value,
  copyable,
}: {
  icon: string;
  label: string;
  value: string;
  copyable?: boolean;
}) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(value).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 10,
        padding: "6px 0",
      }}
    >
      <span style={{ fontSize: 16, width: 20, textAlign: "center" }}>{icon}</span>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            fontSize: 11,
            color: "var(--color-on-surface-variant)",
            marginBottom: 2,
          }}
        >
          {label}
        </div>
        <div
          style={{
            fontSize: 13,
            color: "var(--color-on-surface)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {value}
        </div>
      </div>
      {copyable && (
        <button
          onClick={handleCopy}
          title={copied ? "Copied!" : `Copy ${label.toLowerCase()}`}
          style={{
            background: "none",
            border: "none",
            cursor: "pointer",
            color: copied ? "var(--color-primary)" : "var(--color-on-surface-variant)",
            padding: 4,
            borderRadius: 4,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            transition: "color 150ms",
            flexShrink: 0,
          }}
        >
          {copied ? (
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          ) : (
            <CopyIcon />
          )}
        </button>
      )}
    </div>
  );
}

export function ContactDetailsSidebar({
  open,
  onClose,
  chatData,
  participantNames,
  title,
}: ContactDetailsSidebarProps) {
  const [activeTab, setActiveTab] = useState<InfoTab>("info");
  const [mapPopoverOpen, setMapPopoverOpen] = useState(false);
  const isGroup =
    chatData?.participants && chatData.participants.length > 1;

  // Get all messages for this chat to extract photos
  const { messages } = useMessageStore();
  const { openLightbox } = useAttachmentStore();

  // Find My integration
  const { fetchFriends, getFriendByHandle } = useFindMyStore();
  const contactAddress = chatData?.participants?.[0]?.address ?? "";
  const friendLocation = useMemo(
    () => (!isGroup && contactAddress ? getFriendByHandle(contactAddress) : null),
    [isGroup, contactAddress, getFriendByHandle]
  );

  // Load Find My friends when sidebar opens
  useEffect(() => {
    if (open && !isGroup) {
      fetchFriends();
    }
  }, [open, isGroup, fetchFriends]);

  // Filter messages with image/video attachments
  const mediaAttachments = useMemo(() => {
    const media: Array<{ url: string; type: string; message: Message }> = [];

    messages.forEach((msg) => {
      if (msg.attachments && msg.attachments.length > 0) {
        msg.attachments.forEach((att) => {
          if (
            att.mime_type &&
            (att.mime_type.startsWith("image/") || att.mime_type.startsWith("video/")) &&
            att.web_url
          ) {
            media.push({
              url: att.web_url,
              type: att.mime_type,
              message: msg,
            });
          }
        });
      }
    });

    // Return most recent first
    return media.reverse();
  }, [messages]);

  const tabStyle = (active: boolean): CSSProperties => ({
    flex: 1,
    padding: "8px 0",
    background: "none",
    border: "none",
    borderBottom: active
      ? "2px solid var(--color-primary)"
      : "2px solid transparent",
    color: active
      ? "var(--color-primary)"
      : "var(--color-on-surface-variant)",
    fontSize: 13,
    fontWeight: active ? 600 : 400,
    cursor: "pointer",
    transition: "color 0.15s, border-color 0.15s",
  });

  return (
    <AnimatePresence>
      {open && (
        <motion.aside
          initial={{ x: 320, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          exit={{ x: 320, opacity: 0 }}
          transition={{ type: "spring", damping: 28, stiffness: 300 }}
          style={{
            width: 320,
            minWidth: 320,
            height: "100%",
            background: "var(--color-surface)",
            borderLeft: "1px solid var(--color-surface-variant)",
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
            position: "relative",
            flexShrink: 0,
          }}
          role="complementary"
          aria-label="Contact details"
        >
          {/* Header bar */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "12px 16px",
              flexShrink: 0,
            }}
          >
            <button
              onClick={onClose}
              aria-label="Close contact details"
              style={{
                width: 28,
                height: 28,
                borderRadius: "50%",
                border: "none",
                background: "var(--color-surface-variant)",
                color: "var(--color-on-surface-variant)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                cursor: "pointer",
                padding: 0,
              }}
            >
              <CloseIcon />
            </button>
            <button
              style={{
                background: "none",
                border: "none",
                color: "var(--color-primary)",
                fontSize: 14,
                fontWeight: 500,
                cursor: "pointer",
                padding: "4px 8px",
              }}
            >
              Edit
            </button>
          </div>

          {/* Scrollable content */}
          <div
            style={{
              flex: 1,
              display: "flex",
              flexDirection: "column",
              minHeight: 0,
            }}
          >
            <div
              style={{
                flexShrink: 0,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                padding: "8px 16px 16px",
              }}
            >
            {/* Avatar */}
            <div style={{ marginBottom: 12 }}>
              {isGroup ? (
                <GroupAvatar
                  participants={chatData.participants.map((h, i) => ({
                    name: participantNames[i] ?? h.address,
                    address: h.address,
                  }))}
                  size={80}
                />
              ) : (
                <Avatar
                  name={title}
                  address={
                    chatData?.participants?.[0]?.address ?? ""
                  }
                  size={80}
                />
              )}
            </div>

            {/* Name */}
            <h2
              style={{
                fontSize: 20,
                fontWeight: 600,
                color: "var(--color-on-surface)",
                margin: "0 0 16px",
                textAlign: "center",
                lineHeight: 1.3,
                wordBreak: "break-word",
              }}
            >
              {title}
            </h2>

            {/* Action buttons row */}
            <div
              style={{
                display: "flex",
                gap: 20,
                justifyContent: "center",
                marginBottom: 20,
              }}
            >
              <ActionButton icon={<PhoneIcon />} label="Phone" />
              <ActionButton icon={<VideoIcon />} label="Video" />
              <ActionButton icon={<MailIcon />} label="Mail" />
              <ActionButton icon={<MessageIcon />} label="Message" />
            </div>

            {/* Tab row */}
            <div
              style={{
                display: "flex",
                width: "100%",
                borderBottom: "1px solid var(--color-surface-variant)",
                marginBottom: 16,
              }}
              role="tablist"
            >
              <button
                style={tabStyle(activeTab === "info")}
                role="tab"
                aria-selected={activeTab === "info"}
                onClick={() => setActiveTab("info")}
              >
                Info
              </button>
              <button
                style={tabStyle(activeTab === "backgrounds")}
                role="tab"
                aria-selected={activeTab === "backgrounds"}
                onClick={() => setActiveTab("backgrounds")}
              >
                Backgrounds
              </button>
              <button
                style={tabStyle(activeTab === "photos")}
                role="tab"
                aria-selected={activeTab === "photos"}
                onClick={() => setActiveTab("photos")}
              >
                Photos
              </button>
            </div>

            </div>

            {/* Tab content - scrollable area */}
            <div
              style={{
                flex: 1,
                overflowY: "auto",
                padding: "0 16px 24px",
                minHeight: 0,
              }}
            >
              {activeTab === "info" && (
                <motion.div
                  key="info"
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 10 }}
                  transition={{ duration: 0.2 }}
                  style={{ width: "100%" }}
                >
                  {/* Find My Location Map */}
                  {!isGroup && (
                    <LocationMapCard
                      friend={friendLocation}
                      contactName={title}
                      contactAddress={contactAddress}
                      onExpand={() => friendLocation && setMapPopoverOpen(true)}
                    />
                  )}

                  {/* Contact Details Section */}
                  <div
                    style={{
                      background: "var(--color-surface-variant)",
                      borderRadius: 12,
                      padding: 16,
                      marginBottom: 16,
                    }}
                  >
                    <div
                      style={{
                        fontSize: 13,
                        fontWeight: 600,
                        color: "var(--color-on-surface)",
                        marginBottom: 12,
                      }}
                    >
                      Contact Details
                    </div>
                    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
                      {chatData?.participants && chatData.participants.length === 1 && (() => {
                        const phone = chatData.participants[0].formatted_address || chatData.participants[0].address;
                        const email = chatData.chat_identifier || "";
                        // Deduplicate: if phone and email are the same, only show one
                        const phoneIsEmail = phone && email && phone.replace(/\s/g, "").toLowerCase() === email.replace(/\s/g, "").toLowerCase();
                        const isEmailAddress = (v: string) => v.includes("@");

                        return (
                          <>
                            {phone && !isEmailAddress(phone) && (
                              <DetailRow icon="📱" label="Phone" value={phone} copyable />
                            )}
                            {email && !phoneIsEmail && (
                              <DetailRow
                                icon={isEmailAddress(email) ? "✉️" : "📱"}
                                label={isEmailAddress(email) ? "Email" : "Phone"}
                                value={email}
                                copyable
                              />
                            )}
                            {phone && isEmailAddress(phone) && (
                              <DetailRow icon="✉️" label="Email" value={phone} copyable />
                            )}
                          </>
                        );
                      })()}
                      {isGroup && (
                        <div style={{ fontSize: 12, color: "var(--color-on-surface-variant)" }}>
                          Group chat with {chatData?.participants.length} participants
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Location action links */}
                  <div
                    style={{
                      display: "flex",
                      flexDirection: "column",
                      gap: 12,
                      paddingLeft: 4,
                    }}
                  >
                    <button
                      style={{
                        background: "none",
                        border: "none",
                        color: "#FF453A",
                        fontSize: 13,
                        cursor: "pointer",
                        textAlign: "left",
                        padding: 0,
                      }}
                    >
                      Stop Sharing My Location
                    </button>
                    <button
                      style={{
                        background: "none",
                        border: "none",
                        color: "var(--color-primary)",
                        fontSize: 13,
                        cursor: "pointer",
                        textAlign: "left",
                        padding: 0,
                      }}
                    >
                      Send My Current Location
                    </button>
                  </div>
                </motion.div>
              )}

              {activeTab === "backgrounds" && (
                <motion.div
                  key="backgrounds"
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 10 }}
                  transition={{ duration: 0.2 }}
                  style={{
                    width: "100%",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    padding: 32,
                    color: "var(--color-on-surface-variant)",
                    fontSize: 13,
                  }}
                >
                  Custom backgrounds coming soon
                </motion.div>
              )}

              {activeTab === "photos" && (
                <motion.div
                  key="photos"
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 10 }}
                  transition={{ duration: 0.2 }}
                  style={{ width: "100%", height: "100%", overflowY: "auto" }}
                >
                  {mediaAttachments.length > 0 ? (
                    <div
                      style={{
                        display: "grid",
                        gridTemplateColumns: "repeat(3, 1fr)",
                        gap: 4,
                        padding: "0 0 16px",
                      }}
                    >
                      {mediaAttachments.map((media, idx) => (
                        <button
                          key={`${media.url}-${idx}`}
                          onClick={() => openLightbox(mediaAttachments.map(m => ({ src: m.url })), idx)}
                          style={{
                            aspectRatio: "1",
                            borderRadius: 8,
                            overflow: "hidden",
                            background: "var(--color-surface-variant)",
                            border: "none",
                            padding: 0,
                            cursor: "pointer",
                            position: "relative",
                          }}
                        >
                          <img
                            src={media.url}
                            alt=""
                            style={{
                              width: "100%",
                              height: "100%",
                              objectFit: "cover",
                            }}
                            loading="lazy"
                          />
                          {media.type.startsWith("video/") && (
                            <div
                              style={{
                                position: "absolute",
                                top: 4,
                                right: 4,
                                width: 24,
                                height: 24,
                                borderRadius: "50%",
                                background: "rgba(0, 0, 0, 0.6)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                                color: "white",
                              }}
                            >
                              <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M8 5v14l11-7z" />
                              </svg>
                            </div>
                          )}
                        </button>
                      ))}
                    </div>
                  ) : (
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        padding: 32,
                        color: "var(--color-on-surface-variant)",
                        fontSize: 13,
                      }}
                    >
                      No photos or videos yet
                    </div>
                  )}
                </motion.div>
              )}
            </div>
          </div>
        </motion.aside>
      )}

      {/* Location Map Popover */}
      {friendLocation && (
        <LocationMapPopover
          open={mapPopoverOpen}
          onClose={() => setMapPopoverOpen(false)}
          friend={friendLocation}
          contactName={title}
          contactAddress={contactAddress}
        />
      )}
    </AnimatePresence>
  );
}
