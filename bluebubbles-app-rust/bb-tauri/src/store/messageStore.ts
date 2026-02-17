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

export const useMessageStore = create<MessageState>((set, get) => ({
  messages: [],
  loading: false,
  sending: false,
  error: null,
  chatGuid: null,
  hasMore: true,
  offset: 0,

  loadMessages: async (chatGuid: string) => {
    set({ loading: true, error: null, chatGuid, messages: [], offset: 0 });

    try {
      const messages = await tauriGetMessages(chatGuid, null, MESSAGE_PAGE_SIZE);
      set({
        messages,
        loading: false,
        offset: messages.length,
        hasMore: messages.length >= MESSAGE_PAGE_SIZE,
      });
    } catch (err) {
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
    const { chatGuid, messages } = get();
    if (!chatGuid) return;

    set({ sending: true, error: null });
    try {
      const msg = await tauriSendMessage(chatGuid, text, effect);
      set({
        messages: [msg, ...messages],
        sending: false,
      });

      // Update sidebar chat preview with the newly sent message
      useChatStore.getState().updateChatPreview(chatGuid, {
        text: msg.text,
        date_created: msg.date_created,
        is_from_me: true,
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        sending: false,
      });
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
