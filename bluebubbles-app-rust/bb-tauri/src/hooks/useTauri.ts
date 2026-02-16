/**
 * Tauri IPC invoke wrapper with typed commands.
 *
 * Provides a type-safe interface for calling Rust backend commands
 * from the frontend via Tauri's IPC bridge.
 */
import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";

/**
 * Safe invoke wrapper that checks for Tauri context first.
 */
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error(
      "Not running in Tauri context. Please launch the app with 'npx tauri dev' instead of opening in a browser."
    );
  }
  return tauriInvoke<T>(cmd, args);
}

/** Server info returned from the connect command. */
export interface ServerInfo {
  os_version: string | null;
  server_version: string | null;
  private_api: boolean;
  proxy_service: string | null;
  helper_connected: boolean;
  detected_imessage: string | null;
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

// ─── Tauri command wrappers ──────────────────────────────────────────────────

export async function tauriConnect(
  address: string,
  password: string
): Promise<ServerInfo> {
  return invoke<ServerInfo>("connect", { address, password });
}

export async function tauriGetServerInfo(): Promise<ServerInfo> {
  return invoke<ServerInfo>("get_server_info");
}

export async function tauriGetChats(
  page: number,
  limit: number
): Promise<ChatWithPreview[]> {
  return invoke<ChatWithPreview[]>("get_chats", { page, limit });
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
