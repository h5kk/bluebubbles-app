/**
 * ConversationList page - rendered inside the Sidebar.
 * Fetches and displays the chat list using the chatStore.
 * Implements spec 02-conversation-list.md.
 */
import { useEffect, useCallback, useRef, useState, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import { useChatStore } from "@/store/chatStore";
import { useSettingsStore } from "@/store/settingsStore";
import { ConversationTile } from "@/components/ConversationTile";
import { ContextMenu, type ContextMenuItem } from "@/components/ContextMenu";
import { LoadingSpinner } from "@/components/LoadingSpinner";

export function ConversationList() {
  const navigate = useNavigate();
  const {
    chats,
    loading,
    hasMore,
    selectedChatGuid,
    searchQuery,
    fetchChats,
    loadMore,
    selectChat,
  } = useChatStore();
  const { loaded: settingsLoaded } = useSettingsStore();

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

  // Filter chats by search query
  const filteredChats = searchQuery
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
    : chats;

  // Separate pinned and unpinned
  const pinned = filteredChats.filter((c) => c.chat.is_pinned);
  const unpinned = filteredChats.filter((c) => !c.chat.is_pinned);

  // Build context menu items
  const contextMenuItems: ContextMenuItem[] = [
    {
      label: "Mark as Read",
      onClick: () => {
        // Will integrate with tauriUpdateSetting or dedicated command
      },
    },
    {
      label: "Pin Conversation",
      onClick: () => {
        // Will integrate with chat pin command
      },
    },
    { label: "", onClick: () => {}, divider: true },
    {
      label: "Mute Conversation",
      onClick: () => {
        // Will integrate with mute command
      },
    },
    { label: "", onClick: () => {}, divider: true },
    {
      label: "Archive",
      onClick: () => {
        // Will integrate with archive command
      },
    },
    {
      label: "Delete",
      onClick: () => {
        // Will integrate with delete command
      },
      destructive: true,
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
        {/* Pinned section */}
        {pinned.length > 0 && (
          <>
            <div
              style={{
                padding: "8px 16px 4px",
                fontSize: "var(--font-label-small)",
                fontWeight: 600,
                color: "var(--color-outline)",
                textTransform: "uppercase",
                letterSpacing: "0.8px",
              }}
            >
              Pinned
            </div>
            {pinned.map((chat) => (
              <ConversationTile
                key={chat.chat.guid}
                chat={chat}
                isActive={chat.chat.guid === selectedChatGuid}
                onClick={handleSelectChat}
                onContextMenu={handleContextMenu}
              />
            ))}
          </>
        )}

        {/* Main chat list */}
        {pinned.length > 0 && unpinned.length > 0 && (
          <div
            style={{
              padding: "8px 16px 4px",
              fontSize: "var(--font-label-small)",
              fontWeight: 600,
              color: "var(--color-outline)",
              textTransform: "uppercase",
              letterSpacing: "0.8px",
            }}
          >
            All Messages
          </div>
        )}

        {unpinned.map((chat) => (
          <ConversationTile
            key={chat.chat.guid}
            chat={chat}
            isActive={chat.chat.guid === selectedChatGuid}
            onClick={handleSelectChat}
            onContextMenu={handleContextMenu}
          />
        ))}

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
            <span style={{ fontSize: 32 }}>{"\uD83D\uDCAC"}</span>
            <span style={{ fontSize: "var(--font-body-large)", fontWeight: 500 }}>
              {searchQuery ? "No results found" : "No conversations yet"}
            </span>
            <span style={{ fontSize: "var(--font-body-medium)", textAlign: "center" }}>
              {searchQuery
                ? "Try a different search term"
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
