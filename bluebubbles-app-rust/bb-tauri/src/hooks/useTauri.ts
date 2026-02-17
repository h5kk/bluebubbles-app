/**
 * Tauri IPC invoke wrapper with typed commands.
 *
 * Provides a type-safe interface for calling Rust backend commands
 * from the frontend via Tauri's IPC bridge.
 */
import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";
import { useConnectionStore } from "@/store/connectionStore";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * Safe invoke wrapper that checks for Tauri context first.
 */
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error(
      "Not running in Tauri context. Please launch the app with 'npx tauri dev' instead of opening in a browser."
    );
  }
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    const isConnectionError =
      /request timeout|error sending request|connection refused|timed out|timeout|network|failed to fetch/i.test(
        message
      );
    if (isConnectionError) {
      useConnectionStore.getState().setError(message);
    }
    throw err;
  }
}

/** Server info returned from the connect command. */
export interface ServerInfo {
  os_version: string | null;
  server_version: string | null;
  private_api: boolean;
  proxy_service: string | null;
  helper_connected: boolean;
  detected_imessage: string | null;
  /** Base API URL for constructing asset URLs (e.g. avatar URLs). */
  api_root: string | null;
  /** Auth key for constructing asset URLs. */
  auth_key: string | null;
}

/** Chat with preview data for the conversation list. */
export interface ChatWithPreview {
  chat: Chat;
  latest_message_text: string | null;
  latest_message_date: string | null;
  latest_message_is_from_me: boolean;
  participant_names: string[];
}

/** Chat model matching the Rust Chat struct. */
export interface Chat {
  id: number | null;
  guid: string;
  chat_identifier: string | null;
  display_name: string | null;
  is_archived: boolean;
  mute_type: string | null;
  is_pinned: boolean;
  has_unread_message: boolean;
  pin_index: number | null;
  latest_message_date: string | null;
  style: number | null;
  participants: Handle[];
  custom_avatar_path: string | null;
}

/** Handle model. */
export interface Handle {
  id: number | null;
  address: string;
  service: string;
  formatted_address: string | null;
  color: string | null;
  contact_id: number | null;
}

/** Message model matching the Rust Message struct. */
export interface Message {
  id: number | null;
  guid: string | null;
  chat_id: number | null;
  handle_id: number | null;
  text: string | null;
  subject: string | null;
  error: number;
  date_created: string | null;
  date_read: string | null;
  date_delivered: string | null;
  is_from_me: boolean;
  is_delivered: boolean;
  item_type: number;
  group_title: string | null;
  associated_message_guid: string | null;
  associated_message_type: string | null;
  expressive_send_style_id: string | null;
  has_attachments: boolean;
  has_reactions: boolean;
  thread_originator_guid: string | null;
  big_emoji: boolean | null;
  date_edited: string | null;
  attachments: Attachment[];
  associated_messages: Message[];
}

/** Attachment model. */
export interface Attachment {
  id: number | null;
  guid: string | null;
  mime_type: string | null;
  transfer_name: string | null;
  total_bytes: number | null;
  height: number | null;
  width: number | null;
  web_url: string | null;
}

/** Contact model. */
export interface Contact {
  id: number | null;
  external_id: string | null;
  display_name: string;
  phones: string;
  emails: string;
  structured_name: string | null;
}

/** Sync result from full sync. */
export interface SyncResult {
  chats_synced: number;
  messages_synced: number;
  handles_synced: number;
  contacts_synced: number;
}

/** Theme struct. */
export interface ThemeStruct {
  id: number | null;
  name: string;
  gradient_bg: boolean;
  google_font: string;
  theme_data: string;
}

// ─── Utility helpers ─────────────────────────────────────────────────────────

/** Get a contact's avatar as a data URI from the local database. */
export async function tauriGetContactAvatar(address: string): Promise<string | null> {
  return invoke<string | null>("get_contact_avatar", { address });
}

/** Get all contact avatars as a map of address -> data URI. */
export async function tauriGetAllContactAvatars(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>("get_all_contact_avatars");
}

/** Sync contact avatars from the server. Returns the number of avatars synced. */
export async function tauriSyncContactAvatars(): Promise<number> {
  return invoke<number>("sync_contact_avatars");
}

// ─── Tauri command wrappers ──────────────────────────────────────────────────

export async function tauriConnect(
  address: string,
  password: string
): Promise<ServerInfo> {
  return invoke<ServerInfo>("connect", { address, password });
}

export async function tauriTryAutoConnect(): Promise<ServerInfo | null> {
  return invoke<ServerInfo | null>("try_auto_connect");
}

export async function tauriGetServerInfo(): Promise<ServerInfo> {
  return invoke<ServerInfo>("get_server_info");
}

export async function tauriDetectLocalhost(): Promise<string | null> {
  return invoke<string | null>("detect_localhost");
}

export async function tauriGetChats(
  page: number,
  limit: number
): Promise<ChatWithPreview[]> {
  return invoke<ChatWithPreview[]>("get_chats", { page, limit });
}

export async function tauriRefreshChats(
  limit: number
): Promise<ChatWithPreview[]> {
  return invoke<ChatWithPreview[]>("refresh_chats", { limit });
}

export async function tauriMarkChatRead(
  chatGuid: string
): Promise<void> {
  return invoke<void>("mark_chat_read", { chatGuid });
}

export async function tauriMarkChatUnread(
  chatGuid: string
): Promise<void> {
  return invoke<void>("mark_chat_unread", { chatGuid });
}

export async function tauriUpdateChat(
  chatGuid: string,
  updates: Record<string, unknown>
): Promise<void> {
  return invoke<void>("update_chat", { chatGuid, updates });
}

export async function tauriGetMessages(
  chatGuid: string,
  offset: number | null,
  limit: number
): Promise<Message[]> {
  return invoke<Message[]>("get_messages", {
    chatGuid,
    offset,
    limit,
  });
}

export async function tauriSendMessage(
  chatGuid: string,
  text: string,
  effect?: string
): Promise<Message> {
  return invoke<Message>("send_message", { chatGuid, text, effect });
}

export async function tauriSearchMessages(
  query: string,
  chatGuid?: string
): Promise<Message[]> {
  return invoke<Message[]>("search_messages", { query, chatGuid });
}

export async function tauriGetContacts(): Promise<Contact[]> {
  return invoke<Contact[]>("get_contacts");
}

export async function tauriDownloadAttachment(guid: string): Promise<string> {
  return invoke<string>("download_attachment", { guid });
}

export async function tauriGetSettings(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>("get_settings");
}

export async function tauriUpdateSetting(
  key: string,
  value: string
): Promise<void> {
  return invoke<void>("update_setting", { key, value });
}

export async function tauriSyncFull(): Promise<SyncResult> {
  return invoke<SyncResult>("sync_full");
}

export async function tauriGetThemes(): Promise<ThemeStruct[]> {
  return invoke<ThemeStruct[]>("get_themes");
}

export async function tauriSetTheme(name: string): Promise<void> {
  return invoke<void>("set_theme", { name });
}

export async function tauriCheckSetupComplete(): Promise<boolean> {
  return invoke<boolean>("check_setup_complete");
}

export async function tauriCompleteSetup(): Promise<void> {
  return invoke<void>("complete_setup");
}

export async function tauriSyncMessages(messagesPerChat: number): Promise<SyncResult> {
  return invoke<SyncResult>("sync_messages", { messagesPerChat });
}

export async function tauriCheckMessagesSynced(): Promise<boolean> {
  return invoke<boolean>("check_messages_synced");
}

// ─── Private API command wrappers ────────────────────────────────────────────

/** Private API status returned from check_private_api_status. */
export interface PrivateApiStatus {
  enabled: boolean;
  helper_connected: boolean;
  server_version: string | null;
  os_version: string | null;
}

/** Check Private API status from the server. */
export async function tauriCheckPrivateApiStatus(): Promise<PrivateApiStatus> {
  return invoke<PrivateApiStatus>("check_private_api_status");
}

/** Send a typing indicator for a chat. status is "start" or "stop". */
export async function tauriSendTypingIndicator(
  chatGuid: string,
  status: "start" | "stop"
): Promise<void> {
  return invoke<void>("send_typing_indicator", { chatGuid, status });
}

/** Send a reaction (tapback) to a message. */
export async function tauriSendReaction(
  chatGuid: string,
  selectedMessageText: string,
  selectedMessageGuid: string,
  reaction: string,
  partIndex?: number
): Promise<unknown> {
  return invoke<unknown>("send_reaction", {
    chatGuid,
    selectedMessageText,
    selectedMessageGuid,
    reaction,
    partIndex: partIndex ?? null,
  });
}

/** Edit a sent message. */
export async function tauriEditMessage(
  messageGuid: string,
  editedMessage: string,
  backwardsCompatibilityMessage: string,
  partIndex: number
): Promise<unknown> {
  return invoke<unknown>("edit_message", {
    messageGuid,
    editedMessage,
    backwardsCompatibilityMessage,
    partIndex,
  });
}

/** Unsend a sent message. */
export async function tauriUnsendMessage(
  messageGuid: string,
  partIndex: number
): Promise<unknown> {
  return invoke<unknown>("unsend_message", { messageGuid, partIndex });
}

/** Sync progress event payload. */
export interface SyncProgress {
  current: number;
  total: number;
  chat_name: string;
  messages_saved: number;
}

/** Listen for sync progress events. Returns an unlisten function. */
export async function onSyncProgress(callback: (progress: SyncProgress) => void): Promise<UnlistenFn> {
  return listen<SyncProgress>("sync-progress", (event) => callback(event.payload));
}

/** Listen for sync complete event. Returns an unlisten function. */
export async function onSyncComplete(callback: (totalMessages: number) => void): Promise<UnlistenFn> {
  return listen<number>("sync-complete", (event) => callback(event.payload));
}
