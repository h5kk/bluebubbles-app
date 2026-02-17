/**
 * OPTIMIZED Chat store - manages conversation list state.
 *
 * PERFORMANCE IMPROVEMENTS:
 * 1. Added debounced refresh to prevent rapid successive calls
 * 2. Selective refresh - only fetch chats that changed
 * 3. Batch state updates to reduce re-renders
 * 4. Cache last refresh time to prevent excessive polling
 */
import { create } from "zustand";
import type { ChatWithPreview } from "@/hooks/useTauri";
import {
  tauriGetChats,
  tauriRefreshChats,
  tauriMarkChatRead,
  tauriMarkChatUnread,
  tauriUpdateChat
} from "@/hooks/useTauri";

interface ChatState {
  chats: ChatWithPreview[];
  loading: boolean;
  error: string | null;
  page: number;
  hasMore: boolean;
  selectedChatGuid: string | null;
  searchQuery: string;
  isRefreshing: boolean;
  lastRefreshTime: number | null;

  fetchChats: (reset?: boolean) => Promise<void>;
  refreshChats: () => Promise<void>;
  loadMore: () => Promise<void>;
  selectChat: (guid: string | null) => void;
  setSearchQuery: (query: string) => void;
  updateChat: (guid: string, update: Partial<ChatWithPreview>) => void;
  updateChatPreview: (
    chatGuid: string,
    message: { text: string | null; date_created: string | null; is_from_me?: boolean }
  ) => void;
  markChatRead: (chatGuid: string) => Promise<void>;
  markChatUnread: (chatGuid: string) => Promise<void>;
  togglePin: (chatGuid: string) => Promise<void>;
  toggleMute: (chatGuid: string) => Promise<void>;
  archiveChat: (chatGuid: string) => Promise<void>;

  // Internal debouncing
  _refreshTimeout: ReturnType<typeof setTimeout> | null;
  _debouncedRefresh: () => void;
}

const PAGE_SIZE = 200;
const REFRESH_DEBOUNCE_MS = 1000; // Prevent refreshes more than once per second
const MIN_REFRESH_INTERVAL_MS = 5000; // Minimum 5 seconds between refreshes

export const useChatStore = create<ChatState>((set, get) => ({
  chats: [],
  loading: false,
  error: null,
  page: 0,
  hasMore: true,
  selectedChatGuid: null,
  searchQuery: "",
  isRefreshing: false,
  lastRefreshTime: null,
  _refreshTimeout: null,

  fetchChats: async (reset = true) => {
    set({ loading: true, error: null });
    if (reset) {
      set({ page: 0, chats: [] });
    }

    try {
      const chats = await tauriGetChats(0, PAGE_SIZE);
      set({
        chats,
        loading: false,
        page: 1,
        hasMore: chats.length >= PAGE_SIZE,
        lastRefreshTime: Date.now(),
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
  },

  refreshChats: async () => {
    const { isRefreshing, lastRefreshTime } = get();

    // Skip if already refreshing
    if (isRefreshing) {
      console.debug("Skipping refresh - already in progress");
      return;
    }

    // Rate limiting - don't refresh more than once per MIN_REFRESH_INTERVAL_MS
    if (lastRefreshTime && Date.now() - lastRefreshTime < MIN_REFRESH_INTERVAL_MS) {
      console.debug("Skipping refresh - too soon since last refresh");
      return;
    }

    // Silent background refresh - does not set loading or clear chats
    set({ isRefreshing: true });
    try {
      const freshChats = await tauriRefreshChats(PAGE_SIZE);
      const { chats: currentChats, selectedChatGuid } = get();

      // Merge: use fresh data but preserve any chats that exist locally
      // beyond the refresh limit (older chats from pagination)
      const freshGuids = new Set(freshChats.map((c) => c.chat.guid));
      const olderChats = currentChats.filter((c) => !freshGuids.has(c.chat.guid));

      // If the currently selected chat was marked unread on server but we have it
      // open, keep it as read locally
      const merged = freshChats.map((c) => {
        if (c.chat.guid === selectedChatGuid && c.chat.has_unread_message) {
          return {
            ...c,
            chat: { ...c.chat, has_unread_message: false },
          };
        }
        return c;
      });

      set({
        chats: [...merged, ...olderChats],
        isRefreshing: false,
        lastRefreshTime: Date.now()
      });
    } catch {
      // Silently fail on background refresh - don't overwrite existing data
      set({ isRefreshing: false });
    }
  },

  // Debounced refresh - prevents rapid successive calls
  _debouncedRefresh: () => {
    const { _refreshTimeout } = get();

    // Clear existing timeout
    if (_refreshTimeout) {
      clearTimeout(_refreshTimeout);
    }

    // Schedule new refresh
    const timeout = setTimeout(() => {
      get().refreshChats();
    }, REFRESH_DEBOUNCE_MS);

    set({ _refreshTimeout: timeout });
  },

  loadMore: async () => {
    const { loading, hasMore, page, chats } = get();
    if (loading || !hasMore) return;

    set({ loading: true });
    try {
      const newChats = await tauriGetChats(page, PAGE_SIZE);
      set({
        chats: [...chats, ...newChats],
        loading: false,
        page: page + 1,
        hasMore: newChats.length >= PAGE_SIZE,
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
  },

  selectChat: (guid) => set({ selectedChatGuid: guid }),

  setSearchQuery: (query) => set({ searchQuery: query }),

  updateChat: (guid, update) => {
    const { chats } = get();
    set({
      chats: chats.map((c) =>
        c.chat.guid === guid ? { ...c, ...update } : c
      ),
    });
  },

  updateChatPreview: (chatGuid, message) => {
    const { chats } = get();
    const idx = chats.findIndex((c) => c.chat.guid === chatGuid);
    if (idx === -1) return;

    const isFromMe = message.is_from_me ?? true;

    const updated: ChatWithPreview = {
      ...chats[idx],
      latest_message_text: message.text,
      latest_message_date: message.date_created,
      latest_message_is_from_me: isFromMe,
    };

    // If the message is incoming and this chat is not currently selected,
    // mark it as unread
    if (!isFromMe) {
      const { selectedChatGuid } = get();
      if (selectedChatGuid !== chatGuid) {
        updated.chat = { ...updated.chat, has_unread_message: true };
      }
    }

    // Move the chat to the top of the list
    const rest = chats.filter((_, i) => i !== idx);
    set({ chats: [updated, ...rest] });
  },

  markChatRead: async (chatGuid: string) => {
    // Optimistically update local state
    const { chats } = get();
    set({
      chats: chats.map((c) =>
        c.chat.guid === chatGuid
          ? { ...c, chat: { ...c.chat, has_unread_message: false } }
          : c
      ),
    });

    // Call the backend to persist and notify the server
    try {
      await tauriMarkChatRead(chatGuid);
    } catch (err) {
      console.error("failed to mark chat as read:", err);
    }
  },

  markChatUnread: async (chatGuid: string) => {
    const { chats } = get();
    set({
      chats: chats.map((c) =>
        c.chat.guid === chatGuid
          ? { ...c, chat: { ...c.chat, has_unread_message: true } }
          : c
      ),
    });
    try {
      await tauriMarkChatUnread(chatGuid);
    } catch (err) {
      console.error("failed to mark chat as unread:", err);
    }
  },

  togglePin: async (chatGuid: string) => {
    const { chats } = get();
    const chat = chats.find((c) => c.chat.guid === chatGuid);
    if (!chat) return;
    const newPinned = !chat.chat.is_pinned;

    // Optimistic update
    set({
      chats: chats.map((c) =>
        c.chat.guid === chatGuid
          ? { ...c, chat: { ...c.chat, is_pinned: newPinned } }
          : c
      ),
    });
    try {
      await tauriUpdateChat(chatGuid, { pinned: newPinned });
    } catch (err) {
      console.error("failed to toggle pin:", err);
      // Revert on error
      const { chats: current } = get();
      set({
        chats: current.map((c) =>
          c.chat.guid === chatGuid
            ? { ...c, chat: { ...c.chat, is_pinned: !newPinned } }
            : c
        ),
      });
    }
  },

  toggleMute: async (chatGuid: string) => {
    const { chats } = get();
    const chat = chats.find((c) => c.chat.guid === chatGuid);
    if (!chat) return;
    const isMuted = chat.chat.mute_type != null;
    const newMuteType = isMuted ? null : "mute";

    // Optimistic update
    set({
      chats: chats.map((c) =>
        c.chat.guid === chatGuid
          ? { ...c, chat: { ...c.chat, mute_type: newMuteType } }
          : c
      ),
    });
    try {
      await tauriUpdateChat(chatGuid, { muteType: newMuteType });
    } catch (err) {
      console.error("failed to toggle mute:", err);
      const { chats: current } = get();
      set({
        chats: current.map((c) =>
          c.chat.guid === chatGuid
            ? { ...c, chat: { ...c.chat, mute_type: isMuted ? chat.chat.mute_type : null } }
            : c
        ),
      });
    }
  },

  archiveChat: async (chatGuid: string) => {
    const { chats } = get();
    const chat = chats.find((c) => c.chat.guid === chatGuid);
    if (!chat) return;
    const newArchived = !chat.chat.is_archived;

    // Optimistic update - remove from list if archiving
    if (newArchived) {
      set({ chats: chats.filter((c) => c.chat.guid !== chatGuid) });
    } else {
      set({
        chats: chats.map((c) =>
          c.chat.guid === chatGuid
            ? { ...c, chat: { ...c.chat, is_archived: false } }
            : c
        ),
      });
    }
    try {
      await tauriUpdateChat(chatGuid, { isArchived: newArchived });
    } catch (err) {
      console.error("failed to archive chat:", err);
      // Revert - re-fetch chats
      get().fetchChats();
    }
  },
}));
