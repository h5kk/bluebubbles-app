/**
 * Message store - manages messages for the active conversation.
 */
import { create } from "zustand";
import type { Message } from "@/hooks/useTauri";
import { tauriGetMessages, tauriSendMessage, tauriSendAttachmentData, tauriSendAttachmentMessage, tauriSendNotification } from "@/hooks/useTauri";
import { useChatStore } from "./chatStore";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { playSentSound, playEffectSound, playNotificationSound, playReactionSound } from "@/utils/notificationSound";
import { useSettingsStore } from "./settingsStore";

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
  sendAttachment: (file: File) => Promise<void>;
  sendAttachmentFromPath: (filePath: string) => Promise<void>;
  addMessage: (message: Message) => void;
  addOptimisticReaction: (messageGuid: string, reaction: string) => void;
  removeOptimisticReaction: (messageGuid: string, tempGuid: string) => void;
  clear: () => void;

  // Event listener cleanup
  _unlisten: UnlistenFn | null;
  _startListening: (chatGuid: string) => Promise<void>;
  _stopListening: () => void;
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
  _unlisten: null,

  loadMessages: async (chatGuid: string) => {
    // Increment generation so any in-flight load becomes stale
    const gen = ++loadGeneration;

    // Stop listening to previous chat
    get()._stopListening();

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

      // Start listening for real-time updates for this chat
      get()._startListening(chatGuid);
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

      // Play send sound (or effect sound if an effect was used)
      const sendSoundOn = useSettingsStore.getState().settings["sendSoundEnabled"] !== "false";
      if (sendSoundOn) {
        if (effect) {
          playEffectSound(effect);
        } else {
          playSentSound();
        }
      }
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

  sendAttachment: async (file: File) => {
    const { chatGuid } = get();
    if (!chatGuid) return;

    // Create optimistic message
    const optimisticGuid = `optimistic-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const optimisticMsg: Message = {
      id: null,
      guid: optimisticGuid,
      chat_id: null,
      handle_id: null,
      text: null,
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
      expressive_send_style_id: null,
      has_attachments: true,
      has_reactions: false,
      thread_originator_guid: null,
      big_emoji: null,
      date_edited: null,
      attachments: [],
      associated_messages: [],
    };

    set((state) => ({
      sending: true,
      error: null,
      messages: [optimisticMsg, ...state.messages],
    }));

    useChatStore.getState().updateChatPreview(chatGuid, {
      text: `Sent ${file.name}`,
      date_created: optimisticMsg.date_created,
      is_from_me: true,
    });

    try {
      // Convert File to base64
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
      let binary = "";
      for (let i = 0; i < uint8Array.length; i++) {
        binary += String.fromCharCode(uint8Array[i]);
      }
      const base64Data = btoa(binary);

      const msg = await tauriSendAttachmentData(chatGuid, file.name, base64Data);

      set((state) => ({
        messages: state.messages.map((m) =>
          m.guid === optimisticGuid ? msg : m
        ),
        sending: false,
      }));

      useChatStore.getState().updateChatPreview(chatGuid, {
        text: msg.text ?? `Sent ${file.name}`,
        date_created: msg.date_created,
        is_from_me: true,
      });

      // Play send sound
      if (useSettingsStore.getState().settings["sendSoundEnabled"] !== "false") {
        playSentSound();
      }
    } catch (err) {
      set((state) => ({
        messages: state.messages.map((m) =>
          m.guid === optimisticGuid ? { ...m, error: 1 } : m
        ),
        error: err instanceof Error ? err.message : String(err),
        sending: false,
      }));
    }
  },

  sendAttachmentFromPath: async (filePath: string) => {
    const { chatGuid } = get();
    if (!chatGuid) return;

    set({ sending: true, error: null });

    try {
      const msg = await tauriSendAttachmentMessage(chatGuid, filePath);
      set((state) => ({
        messages: [msg, ...state.messages],
        sending: false,
      }));

      if (useSettingsStore.getState().settings["soundEnabled"] !== "false") {
        playSentSound();
      }
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : String(err),
        sending: false,
      });
    }
  },

  addMessage: (message: Message) => {
    const { messages, chatGuid } = get();

    // Only add if this message belongs to the current chat
    if (!chatGuid) return;

    // Avoid duplicates
    if (messages.some((m) => m.guid === message.guid)) return;

    // Add to top (newest first)
    set({ messages: [message, ...messages] });

    // Update chat preview
    useChatStore.getState().updateChatPreview(chatGuid, {
      text: message.text,
      date_created: message.date_created,
      is_from_me: message.is_from_me,
    });

    // Play notification sound + send desktop notification for incoming messages
    if (!message.is_from_me) {
      const s = useSettingsStore.getState().settings;
      const notifEnabled = s["notificationsEnabled"] !== "false";
      const soundOn = s["soundEnabled"] !== "false";
      const notifSound = s["notifSound"] || "default";

      // Check if it's a reaction (associated_message_type set)
      const isReaction = message.associated_message_type != null && message.associated_message_type !== "";

      if (isReaction) {
        if (soundOn && s["notifyReactions"] !== "false") {
          playReactionSound(false);
        }
      } else {
        if (soundOn && notifSound !== "none") {
          playNotificationSound(notifSound);
        }
        if (notifEnabled) {
          const showSender = s["notifShowSender"] !== "false";
          const showPreview = s["notifShowPreview"] !== "false";
          const title = showSender ? "New Message" : "BlueBubbles";
          const body = showPreview && message.text ? message.text : "New message received.";
          tauriSendNotification(title, body).catch(() => {});
        }
      }
    }
  },

  addOptimisticReaction: (messageGuid: string, reaction: string) => {
    const tempGuid = `temp-reaction-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const syntheticReaction: Message = {
      id: null,
      guid: tempGuid,
      chat_id: null,
      handle_id: null,
      text: null,
      subject: null,
      error: 0,
      date_created: new Date().toISOString(),
      date_read: null,
      date_delivered: null,
      is_from_me: true,
      is_delivered: false,
      item_type: 0,
      group_title: null,
      associated_message_guid: messageGuid,
      associated_message_type: reaction,
      expressive_send_style_id: null,
      has_attachments: false,
      has_reactions: false,
      thread_originator_guid: null,
      big_emoji: null,
      date_edited: null,
      attachments: [],
      associated_messages: [],
    };

    set((state) => ({
      messages: state.messages.map((m) => {
        if (m.guid !== messageGuid) return m;
        return {
          ...m,
          associated_messages: [...(m.associated_messages ?? []), syntheticReaction],
        };
      }),
    }));
  },

  removeOptimisticReaction: (messageGuid: string, tempGuid: string) => {
    set((state) => ({
      messages: state.messages.map((m) => {
        if (m.guid !== messageGuid) return m;
        return {
          ...m,
          associated_messages: (m.associated_messages ?? []).filter(
            (r) => r.guid !== tempGuid
          ),
        };
      }),
    }));
  },

  clear: () => {
    get()._stopListening();
    set({
      messages: [],
      chatGuid: null,
      hasMore: true,
      offset: 0,
      error: null,
    });
  },

  // Event listener management
  _startListening: async (chatGuid: string) => {
    // Clean up any existing listener
    get()._stopListening();

    try {
      // Listen for new messages in this chat
      const unlisten = await listen<Message>(`new-message:${chatGuid}`, (event) => {
        get().addMessage(event.payload);
      });

      // Store unlisten function
      set({ _unlisten: unlisten });
    } catch (err) {
      console.warn("Failed to set up message listeners:", err);
    }
  },

  _stopListening: () => {
    const { _unlisten } = get();
    if (_unlisten) {
      _unlisten();
      set({ _unlisten: null });
    }
  },
}));
