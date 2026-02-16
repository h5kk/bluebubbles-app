/**
 * Chat store - manages conversation list state.
 */
import { create } from "zustand";
import type { ChatWithPreview } from "@/hooks/useTauri";
import { tauriGetChats } from "@/hooks/useTauri";

interface ChatState {
  chats: ChatWithPreview[];
  loading: boolean;
  error: string | null;
  page: number;
  hasMore: boolean;
  selectedChatGuid: string | null;
  searchQuery: string;

  fetchChats: (reset?: boolean) => Promise<void>;
  loadMore: () => Promise<void>;
  selectChat: (guid: string | null) => void;
  setSearchQuery: (query: string) => void;
  updateChat: (guid: string, update: Partial<ChatWithPreview>) => void;
}

const PAGE_SIZE = 50;

export const useChatStore = create<ChatState>((set, get) => ({
  chats: [],
  loading: false,
  error: null,
  page: 0,
  hasMore: true,
  selectedChatGuid: null,
  searchQuery: "",

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
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
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
}));
