/**
 * Message store - manages messages for the active conversation.
 */
import { create } from "zustand";
import type { Message } from "@/hooks/useTauri";
import { tauriGetMessages, tauriSendMessage } from "@/hooks/useTauri";
import { useChatStore } from "./chatStore";

interface MessageState {
  messages: Message[];
  loading: boolean;
  sending: boolean;
  error: string | null;
  chatGuid: string | null;
  hasMore: boolean;
  offset: number;

  loadMessages: (chatGuid: string) => Promise<void>;
  loadOlder: () => Promise<void>;
  sendMessage: (text: string, effect?: string) => Promise<void>;
  addMessage: (message: Message) => void;
  clear: () => void;
}

const MESSAGE_PAGE_SIZE = 25;

/** Counter to track the latest loadMessages call and ignore stale responses. */
let loadGeneration = 0;

export const useMessageStore = create<MessageState>((set, get) => ({
  messages: [],
  loading: false,
  sending: false,
  error: null,
  chatGuid: null,
  hasMore: true,
  offset: 0,

  loadMessages: async (chatGuid: string) => {
    // Increment generation so any in-flight load becomes stale
    const gen = ++loadGeneration;

    set({ loading: true, error: null, chatGuid, messages: [], offset: 0 });

    try {
      const fetched = await tauriGetMessages(chatGuid, null, MESSAGE_PAGE_SIZE);

      // If a newer loadMessages call was made while we were fetching,
      // discard this result to avoid overwriting fresher state
      if (gen !== loadGeneration) return;

      // Preserve any optimistic messages that were added while loading
      const { messages: currentMessages } = get();
      const optimistic = currentMessages.filter(
        (m) => m.guid != null && m.guid.startsWith("optimistic-")
      );

      // Merge: optimistic messages first, then fetched (deduped)
      const fetchedGuids = new Set(fetched.map((m) => m.guid));
      const kept = optimistic.filter((m) => !fetchedGuids.has(m.guid));

      set({
        messages: [...kept, ...fetched],
        loading: false,
        offset: fetched.length,
        hasMore: fetched.length >= MESSAGE_PAGE_SIZE,
      });
    } catch (err) {
      if (gen !== loadGeneration) return;
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
  },

  loadOlder: async () => {
    const { loading, hasMore, chatGuid, offset, messages } = get();
    if (loading || !hasMore || !chatGuid) return;

    set({ loading: true });
    try {
      const older = await tauriGetMessages(chatGuid, offset, MESSAGE_PAGE_SIZE);
      set({
        messages: [...messages, ...older],
        loading: false,
        offset: offset + older.length,
        hasMore: older.length >= MESSAGE_PAGE_SIZE,
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        loading: false,
      });
    }
  },

  sendMessage: async (text: string, effect?: string) => {
    const { chatGuid } = get();
    if (!chatGuid) return;

    // Create an optimistic message that appears immediately
    const optimisticGuid = `optimistic-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const optimisticMsg: Message = {
      id: null,
      guid: optimisticGuid,
      chat_id: null,
      handle_id: null,
      text,
      subject: null,
      error: 0,
      date_created: new Date().toISOString(),
      date_read: null,
      date_delivered: null,
      is_from_me: true,
      is_delivered: false,
      item_type: 0,
      group_title: null,
      associated_message_guid: null,
      associated_message_type: null,
      expressive_send_style_id: effect ?? null,
      has_attachments: false,
      has_reactions: false,
      thread_originator_guid: null,
      big_emoji: null,
      date_edited: null,
      attachments: [],
      associated_messages: [],
    };

    // Show the message immediately - always read fresh state
    set((state) => ({
      sending: true,
      error: null,
      messages: [optimisticMsg, ...state.messages],
    }));

    // Update sidebar chat preview optimistically
    useChatStore.getState().updateChatPreview(chatGuid, {
      text,
      date_created: optimisticMsg.date_created,
      is_from_me: true,
    });

    try {
      const msg = await tauriSendMessage(chatGuid, text, effect);

      // Replace the optimistic message with the real one from the server
      set((state) => ({
        messages: state.messages.map((m) =>
          m.guid === optimisticGuid ? msg : m
        ),
        sending: false,
      }));

      // Update sidebar with real server data
      useChatStore.getState().updateChatPreview(chatGuid, {
        text: msg.text,
        date_created: msg.date_created,
        is_from_me: true,
      });

      // Refresh messages immediately after sending to get latest from server
      setTimeout(() => {
        get().loadMessages(chatGuid);
      }, 500);
    } catch (err) {
      // Mark the optimistic message as failed instead of removing it
      set((state) => ({
        messages: state.messages.map((m) =>
          m.guid === optimisticGuid ? { ...m, error: 1 } : m
        ),
        error: err instanceof Error ? err.message : String(err),
        sending: false,
      }));
    }
  },

  addMessage: (message: Message) => {
    const { messages } = get();
    // Avoid duplicates
    if (messages.some((m) => m.guid === message.guid)) return;
    set({ messages: [message, ...messages] });
  },

  clear: () =>
    set({
      messages: [],
      chatGuid: null,
      hasMore: true,
      offset: 0,
      error: null,
    }),
}));
