/**
 * ConversationList page - rendered inside the Sidebar.
 * macOS Messages style with pinned avatars in horizontal row.
 */
import { useEffect, useCallback, useMemo, useRef, useState, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { useChatStore } from "@/store/chatStore";
import { useSettingsStore } from "@/store/settingsStore";
import { ConversationTile } from "@/components/ConversationTile";
import { Avatar, GroupAvatar } from "@/components/Avatar";
import { ContextMenu, type ContextMenuItem } from "@/components/ContextMenu";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { getDemoName, getDemoAvatarUrl } from "@/utils/demoData";

export function ConversationList() {
  const navigate = useNavigate();
  // Granular selectors: only re-render when the specific slice changes
  const chats = useChatStore((s) => s.chats);
  const loading = useChatStore((s) => s.loading);
  const hasMore = useChatStore((s) => s.hasMore);
  const selectedChatGuid = useChatStore((s) => s.selectedChatGuid);
  const searchQuery = useChatStore((s) => s.searchQuery);
  const fetchChats = useChatStore((s) => s.fetchChats);
  const refreshChats = useChatStore((s) => s.refreshChats);
  const loadMore = useChatStore((s) => s.loadMore);
  const selectChat = useChatStore((s) => s.selectChat);
  const markChatRead = useChatStore((s) => s.markChatRead);
  const markChatUnread = useChatStore((s) => s.markChatUnread);
  const togglePin = useChatStore((s) => s.togglePin);
  const toggleMute = useChatStore((s) => s.toggleMute);
  const archiveChat = useChatStore((s) => s.archiveChat);
  const { loaded: settingsLoaded } = useSettingsStore();
  const demoMode = useSettingsStore((s) => s.demoMode);

  const scrollRef = useRef<HTMLDivElement>(null);

  // Context menu state
  const [contextMenu, setContextMenu] = useState<{
    open: boolean;
    x: number;
    y: number;
    chatGuid: string;
  }>({ open: false, x: 0, y: 0, chatGuid: "" });

  // Load chats on mount
  useEffect(() => {
    if (settingsLoaded) {
      fetchChats();
    }
  }, [settingsLoaded, fetchChats]);

  // Background polling - refresh chat list every 20 seconds
  useEffect(() => {
    if (!settingsLoaded) return;

    const interval = setInterval(() => {
      refreshChats();
    }, 20_000);

    return () => clearInterval(interval);
  }, [settingsLoaded, refreshChats]);

  // Handle chat selection
  const handleSelectChat = useCallback(
    (guid: string) => {
      selectChat(guid);
      navigate(`/chat/${encodeURIComponent(guid)}`);
    },
    [selectChat, navigate]
  );

  // Handle context menu
  const handleContextMenu = useCallback(
    (guid: string, event: React.MouseEvent) => {
      setContextMenu({
        open: true,
        x: event.clientX,
        y: event.clientY,
        chatGuid: guid,
      });
    },
    []
  );

  const closeContextMenu = useCallback(() => {
    setContextMenu((prev) => ({ ...prev, open: false }));
  }, []);

  // Infinite scroll
  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el || loading || !hasMore) return;

    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 200;
    if (nearBottom) {
      loadMore();
    }
  }, [loading, hasMore, loadMore]);

  // Filter chats by search query - memoized to avoid recomputing on unrelated renders
  const filteredChats = useMemo(
    () =>
      searchQuery
        ? chats.filter((c) => {
            const q = searchQuery.toLowerCase();
            const title = (
              c.chat.display_name ||
              c.participant_names.join(", ") ||
              c.chat.chat_identifier ||
              ""
            ).toLowerCase();
            const preview = (c.latest_message_text ?? "").toLowerCase();
            return title.includes(q) || preview.includes(q);
          })
        : chats,
    [chats, searchQuery]
  );

  // Separate pinned and unpinned - memoized for stable references
  const pinned = useMemo(() => filteredChats.filter((c) => c.chat.is_pinned), [filteredChats]);
  const unpinned = useMemo(() => filteredChats.filter((c) => !c.chat.is_pinned), [filteredChats]);

  // Build context menu items based on the targeted chat's state
  const contextTargetChat = chats.find((c) => c.chat.guid === contextMenu.chatGuid);
  const isUnread = contextTargetChat?.chat.has_unread_message ?? false;
  const isPinned = contextTargetChat?.chat.is_pinned ?? false;
  const isMuted = contextTargetChat?.chat.mute_type != null;

  const contextMenuItems: ContextMenuItem[] = [
    {
      label: isUnread ? "Mark as Read" : "Mark as Unread",
      onClick: () => {
        if (isUnread) {
          markChatRead(contextMenu.chatGuid);
        } else {
          markChatUnread(contextMenu.chatGuid);
        }
      },
    },
    {
      label: isPinned ? "Unpin Conversation" : "Pin Conversation",
      onClick: () => {
        togglePin(contextMenu.chatGuid);
      },
    },
    { label: "", onClick: () => {}, divider: true },
    {
      label: isMuted ? "Unmute Conversation" : "Mute Conversation",
      onClick: () => {
        toggleMute(contextMenu.chatGuid);
      },
    },
    { label: "", onClick: () => {}, divider: true },
    {
      label: "Archive",
      onClick: () => {
        archiveChat(contextMenu.chatGuid);
      },
    },
  ];

  const emptyStyle: CSSProperties = {
    flex: 1,
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    padding: 32,
    gap: 12,
    color: "var(--color-on-surface-variant)",
  };

  return (
    <>
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        style={{ flex: 1, overflow: "auto" }}
      >
        {/* Pinned section - large circular avatars in horizontal row */}
        {pinned.length > 0 && (
          <div
            style={{
              padding: "8px 12px 12px",
              display: "flex",
              gap: 16,
              overflowX: "auto",
              overflowY: "hidden",
              flexShrink: 0,
              borderBottom: "1px solid var(--color-divider-subtle)",
            }}
          >
            {pinned.map((chat) => {
              const chatData = chat.chat;
              const isGroup = chatData.participants.length > 1;
              const rawName =
                chatData.display_name ||
                chat.participant_names.join(", ") ||
                chatData.chat_identifier ||
                "Unknown";
              const name = demoMode ? getDemoName(rawName, isGroup) : rawName;
              const isActive = chatData.guid === selectedChatGuid;
              const hasUnread = chatData.has_unread_message;
              const firstName = name.split(/[\s,]+/)[0] || name;

              return (
                <div
                  key={chatData.guid}
                  onClick={() => handleSelectChat(chatData.guid)}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    handleContextMenu(chatData.guid, e);
                  }}
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    gap: 4,
                    cursor: "pointer",
                    minWidth: 70,
                    maxWidth: 78,
                    position: "relative",
                    padding: "8px 6px 10px",
                    borderRadius: 14,
                    backgroundColor: isActive ? "#007AFF" : "transparent",
                    boxShadow: isActive ? "0 6px 14px rgba(0, 122, 255, 0.25)" : "none",
                  }}
                  role="button"
                  tabIndex={0}
                  aria-label={name}
                >
                  {/* Avatar with status indicator */}
                  <div style={{ position: "relative" }}>
                    {isGroup ? (
                      <GroupAvatar
                        participants={chatData.participants.map((p, i) => {
                          const participantName = chat.participant_names[i] ?? p.address;
                          const demoName = getDemoName(participantName);
                          return {
                            name: demoMode ? demoName : participantName,
                            address: p.address,
                            avatarUrl: demoMode ? getDemoAvatarUrl(demoName, p.address) : undefined,
                          };
                        })}
                        size={60}
                      />
                    ) : (
                      <Avatar
                        name={name}
                        address={chatData.participants[0]?.address ?? chatData.guid}
                        size={60}
                        avatarUrl={
                          demoMode
                            ? getDemoAvatarUrl(name, chatData.participants[0]?.address ?? chatData.guid)
                            : undefined
                        }
                      />
                    )}
                    {/* Unread indicator dot */}
                    {hasUnread && (
                      <div
                        style={{
                          position: "absolute",
                          bottom: 1,
                          right: 1,
                          width: 14,
                          height: 14,
                          borderRadius: "50%",
                          backgroundColor: "#007AFF",
                          border: "2px solid var(--color-background)",
                        }}
                      />
                    )}
                  </div>

                  {/* Contact name */}
                  <span
                    style={{
                      fontSize: 11,
                      fontWeight: isActive ? 600 : 400,
                      color: isActive ? "#FFFFFF" : "var(--color-on-surface)",
                      textAlign: "center",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                      width: "100%",
                      lineHeight: 1.2,
                    }}
                  >
                    {firstName}
                  </span>
                </div>
              );
            })}
          </div>
        )}

        {/* Regular conversation list */}
        <div style={{ padding: "4px 8px" }}>
          {unpinned.map((chat) => (
            <ConversationTile
              key={chat.chat.guid}
              chat={chat}
              isActive={chat.chat.guid === selectedChatGuid}
              onClick={handleSelectChat}
              onContextMenu={handleContextMenu}
            />
          ))}
        </div>

        {/* Loading indicator */}
        {loading && (
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

        {/* Empty state */}
        {!loading && filteredChats.length === 0 && (
          <div style={emptyStyle}>
            <span style={{ fontSize: 32, opacity: 0.5 }}>No conversations</span>
            <span style={{ fontSize: "var(--font-body-medium)", textAlign: "center" }}>
              {searchQuery
                ? "No results found"
                : "Connect to your BlueBubbles server to start messaging"}
            </span>
          </div>
        )}
      </div>

      {/* Context menu */}
      <ContextMenu
        open={contextMenu.open}
        x={contextMenu.x}
        y={contextMenu.y}
        items={contextMenuItems}
        onClose={closeContextMenu}
      />
    </>
  );
}
