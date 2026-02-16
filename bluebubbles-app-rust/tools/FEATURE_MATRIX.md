# BlueBubbles Rust Rewrite - Feature Matrix

*Generated: 2026-02-16 23:14:32 UTC*

Legend: [x] = implemented, [~] = stubbed/partial, [ ] = missing

## API Endpoints (71/71 = 100%)

### Server

- [x] GET /api/v1/ping
- [x] GET /api/v1/server/info
- [x] POST /api/v1/server/restart/soft
- [x] POST /api/v1/server/restart/hard
- [x] GET /api/v1/server/update/check
- [x] POST /api/v1/server/update/install
- [x] GET /api/v1/server/statistics/totals
- [x] GET /api/v1/server/statistics/media
- [x] GET /api/v1/server/logs

### Chat

- [x] GET /api/v1/chat/count
- [x] POST /api/v1/chat/query
- [x] GET /api/v1/chat/:guid
- [x] PUT /api/v1/chat/:guid
- [x] DELETE /api/v1/chat/:guid
- [x] POST /api/v1/chat/new
- [x] POST /api/v1/chat/:guid/read
- [x] POST /api/v1/chat/:guid/unread
- [x] POST /api/v1/chat/:guid/leave
- [x] POST /api/v1/chat/:guid/participant/add
- [x] POST /api/v1/chat/:guid/participant/remove
- [x] GET /api/v1/chat/:guid/message
- [x] GET /api/v1/chat/:guid/icon
- [x] DELETE /api/v1/chat/:guid/:messageGuid

### Message

- [x] GET /api/v1/message/count
- [x] POST /api/v1/message/query
- [x] GET /api/v1/message/:guid
- [x] GET /api/v1/message/:guid/embedded-media
- [x] POST /api/v1/message/text
- [x] POST /api/v1/message/multipart
- [x] POST /api/v1/message/react
- [x] POST /api/v1/message/:guid/unsend
- [x] POST /api/v1/message/:guid/edit
- [x] POST /api/v1/message/:guid/notify
- [x] GET /api/v1/message/schedule
- [x] POST /api/v1/message/schedule
- [x] PUT /api/v1/message/schedule/:id
- [x] DELETE /api/v1/message/schedule/:id

### Attachment

- [x] GET /api/v1/attachment/count
- [x] GET /api/v1/attachment/:guid
- [x] GET /api/v1/attachment/:guid/download
- [x] GET /api/v1/attachment/:guid/live
- [x] GET /api/v1/attachment/:guid/blurhash
- [x] POST /api/v1/attachment/upload (multipart)

### Handle

- [x] GET /api/v1/handle/count
- [x] POST /api/v1/handle/query
- [x] GET /api/v1/handle/:guid
- [x] GET /api/v1/handle/:address/focus
- [x] POST /api/v1/handle/availability/imessage
- [x] POST /api/v1/handle/availability/facetime

### Contact

- [x] GET /api/v1/contact
- [x] POST /api/v1/contact/query
- [x] POST /api/v1/contact/upload

### FCM

- [x] POST /api/v1/fcm/device
- [x] GET /api/v1/fcm/client

### Mac

- [x] POST /api/v1/mac/lock
- [x] POST /api/v1/mac/imessage/restart

### Backup

- [x] GET /api/v1/backup/theme
- [x] POST /api/v1/backup/theme
- [x] DELETE /api/v1/backup/theme
- [x] GET /api/v1/backup/settings
- [x] POST /api/v1/backup/settings
- [x] DELETE /api/v1/backup/settings

### FaceTime

- [x] POST /api/v1/facetime/answer
- [x] POST /api/v1/facetime/leave

### iCloud

- [x] GET /api/v1/icloud/findmy/devices
- [x] POST /api/v1/icloud/findmy/devices/refresh
- [x] GET /api/v1/icloud/findmy/friends
- [x] POST /api/v1/icloud/findmy/friends/refresh
- [x] GET /api/v1/icloud/account
- [x] GET /api/v1/icloud/contact
- [x] POST /api/v1/icloud/account/alias

## Socket Events (13/13 = 100%)

### Events

- [x] new-message
- [x] updated-message
- [x] typing-indicator
- [x] chat-read-status-changed
- [x] group-name-change
- [x] participant-removed
- [x] participant-added
- [x] participant-left
- [x] group-icon-changed
- [x] incoming-facetime
- [x] ft-call-status-changed
- [x] imessage-aliases-removed
- [x] server-update

## DB Models (97/97 = 100%)

### Chat

- [x] Chat.id
- [x] Chat.guid
- [x] Chat.chat_identifier
- [x] Chat.display_name
- [x] Chat.is_archived
- [x] Chat.mute_type
- [x] Chat.is_pinned
- [x] Chat.has_unread_message
- [x] Chat.style
- [x] Chat.latest_message_date
- [x] Chat.participants
- [x] Chat.from_server_map
- [x] Chat.save

### Message

- [x] Message.id
- [x] Message.guid
- [x] Message.chat_id
- [x] Message.handle_id
- [x] Message.text
- [x] Message.subject
- [x] Message.error
- [x] Message.date_created
- [x] Message.is_from_me
- [x] Message.item_type
- [x] Message.associated_message_guid
- [x] Message.has_attachments
- [x] Message.attributed_body
- [x] Message.payload_data
- [x] Message.from_server_map
- [x] Message.save

### Handle

- [x] Handle.id
- [x] Handle.address
- [x] Handle.service
- [x] Handle.formatted_address
- [x] Handle.contact
- [x] Handle.from_server_map
- [x] Handle.save
- [x] Handle.display_name

### Attachment

- [x] Attachment.id
- [x] Attachment.guid
- [x] Attachment.mime_type
- [x] Attachment.transfer_name
- [x] Attachment.total_bytes
- [x] Attachment.height
- [x] Attachment.width
- [x] Attachment.has_live_photo
- [x] Attachment.from_server_map
- [x] Attachment.save
- [x] Attachment.is_image
- [x] Attachment.is_video
- [x] Attachment.is_audio

### Contact

- [x] Contact.id
- [x] Contact.external_id
- [x] Contact.display_name
- [x] Contact.phones
- [x] Contact.emails
- [x] Contact.avatar
- [x] Contact.structured_name
- [x] Contact.from_server_map
- [x] Contact.save
- [x] Contact.matches_address

### FcmData

- [x] FcmData.project_id
- [x] FcmData.storage_bucket
- [x] FcmData.api_key
- [x] FcmData.firebase_url
- [x] FcmData.client_id
- [x] FcmData.application_id
- [x] FcmData.from_server_map
- [x] FcmData.save
- [x] FcmData.load

### ThemeStruct

- [x] ThemeStruct.id
- [x] ThemeStruct.name
- [x] ThemeStruct.gradient_bg
- [x] ThemeStruct.google_font
- [x] ThemeStruct.theme_data
- [x] ThemeStruct.parsed_data
- [x] ThemeStruct.is_dark
- [x] ThemeStruct.save
- [x] ThemeStruct.load_all
- [x] ThemeStruct.delete_by_name

### ScheduledMessage

- [x] ScheduledMessage.id
- [x] ScheduledMessage.type
- [x] ScheduledMessage.chat_guid
- [x] ScheduledMessage.message
- [x] ScheduledMessage.scheduled_for
- [x] ScheduledMessage.status
- [x] ScheduledMessage.from_server_map
- [x] ScheduledMessage.save
- [x] ScheduledMessage.load_all
- [x] ScheduledMessage.delete

### AttributedBody

- [x] AttributedBody.runs
- [x] AttributedBody.plain_text
- [x] AttributedBody.mentions
- [x] AttributedBody.links
- [x] AttributedBody.from_server_json

### PayloadData

- [x] PayloadData.url_preview
- [x] PayloadData.app_data
- [x] PayloadData.from_server_json

## DB Infrastructure (10/10 = 100%)

### Core

- [x] Connection pooling (r2d2)
- [x] WAL mode
- [x] Integrity check
- [x] Schema creation
- [x] Versioned migrations
- [x] Transaction support
- [x] Database reset
- [x] Database stats
- [x] Settings key-value store
- [x] Chat-Handle join table

## Services (55/69 = 80%)

### ChatService

- [x] ChatService.list_chats
- [x] ChatService.find_chat
- [x] ChatService.search_chats
- [x] ChatService.mark_read
- [x] ChatService.mark_unread
- [x] ChatService.create_chat
- [x] ChatService.rename_chat
- [x] ChatService.count
- [x] ChatService.toggle_pin
- [x] ChatService.toggle_archive

### MessageService

- [x] MessageService.list_messages
- [x] MessageService.find_message
- [x] MessageService.search_messages
- [x] MessageService.send_text
- [x] MessageService.send_reaction
- [x] MessageService.edit_message
- [x] MessageService.unsend_message
- [x] MessageService.handle_incoming_message
- [x] MessageService.count_for_chat

### ContactService

- [x] ContactService.list_contacts
- [x] ContactService.search_contacts
- [x] ContactService.find_contact
- [x] ContactService.sync_contacts
- [x] ContactService.resolve_display_name

### AttachmentService

- [x] AttachmentService.download
- [x] AttachmentService.upload_and_send
- [x] AttachmentService.attachments_for_message
- [x] AttachmentService.cleanup_cache
- [x] AttachmentService.cache_path
- [x] AttachmentService.is_cached

### SyncService

- [x] SyncService.full_sync
- [x] SyncService.incremental_sync

### SettingsService

- [x] SettingsService.server_address
- [x] SettingsService.set_server_address
- [x] SettingsService.guid_auth_key
- [x] SettingsService.set_guid_auth_key
- [x] SettingsService.is_setup_complete
- [x] SettingsService.mark_setup_complete
- [x] SettingsService.save

### NotificationService

- [x] NotificationService.notify_message
- [x] NotificationService.notify_reaction
- [x] NotificationService.notify
- [x] NotificationService.set_enabled

### QueueService

- [x] QueueService.enqueue
- [x] QueueService.dequeue
- [x] QueueService.len
- [x] QueueService.is_empty
- [x] QueueService.remove
- [x] QueueService.clear

### ServiceRegistry

- [x] ServiceRegistry.register
- [x] ServiceRegistry.init_all
- [x] ServiceRegistry.shutdown_all
- [x] ServiceRegistry.set_api_client
- [x] ServiceRegistry.api_client
- [x] ServiceRegistry.health_check

### Flutter-only

- [ ] LifecycleService *(Flutter-specific service, not ported)*
- [ ] ActionHandler *(Flutter-specific service, not ported)*
- [ ] BackgroundIsolateService *(Flutter-specific service, not ported)*
- [ ] NotificationListenerService *(Flutter-specific service, not ported)*
- [ ] FCMService *(Flutter-specific service, not ported)*
- [ ] UnifiedPushService *(Flutter-specific service, not ported)*
- [ ] NetworkService *(Flutter-specific service, not ported)*
- [ ] ThemeService *(Flutter-specific service, not ported)*
- [ ] FindMyService *(Flutter-specific service, not ported)*
- [ ] FaceTimeService *(Flutter-specific service, not ported)*
- [ ] SocketService (high-level wrapper) *(Flutter-specific service, not ported)*
- [ ] MethodChannelService *(Flutter-specific service, not ported)*
- [ ] IntentService *(Flutter-specific service, not ported)*
- [ ] PrivateApiService *(Flutter-specific service, not ported)*

## CLI Commands (35/37 = 95%)

### connect

- [x] bb connect connect

### status

- [x] bb status status

### chats

- [x] bb chats list
- [x] bb chats get
- [x] bb chats search
- [x] bb chats read
- [x] bb chats unread

### messages

- [x] bb messages list
- [x] bb messages get
- [x] bb messages search
- [x] bb messages send
- [x] bb messages react
- [x] bb messages edit
- [x] bb messages unsend

### contacts

- [x] bb contacts list
- [x] bb contacts search
- [x] bb contacts sync

### attachments

- [x] bb attachments info
- [x] bb attachments download

### sync

- [x] bb sync full
- [x] bb sync incremental

### settings

- [x] bb settings show
- [~] bb settings set-address *(Module exists, checking for set-address)*
- [~] bb settings set-password *(Module exists, checking for set-password)*
- [x] bb settings export

### server

- [x] bb server ping
- [x] bb server info
- [x] bb server stats
- [x] bb server restart
- [x] bb server restart-hard
- [x] bb server check-update
- [x] bb server logs

### db

- [x] bb db stats
- [x] bb db check
- [x] bb db reset
- [x] bb db path

### logs

- [x] bb logs view

## Core Infrastructure (24/24 = 100%)

### Core

- [x] AppConfig (TOML)
- [x] ServerConfig
- [x] DatabaseConfig
- [x] LoggingConfig
- [x] SyncConfig
- [x] NotificationConfig
- [x] DisplayConfig
- [x] ConfigHandle (thread-safe)
- [x] BbError unified error type
- [x] MessageError codes
- [x] init_logging (tracing + file rotation)
- [x] init_console_logging
- [x] Platform detection (Win/Mac/Linux)
- [x] Platform data_dir / config_dir / cache_dir
- [x] Constants (reactions, effects, balloon bundles)
- [x] AES-256-CBC crypto (CryptoJS compatible)
- [x] SocketEventType enum
- [x] EventDispatcher (broadcast channels)
- [x] ConnectionState
- [x] SocketManager
- [x] Reconnect with exponential backoff + jitter
- [x] Event deduplication
- [x] Service trait
- [x] ServiceState lifecycle

## Settings (33/34 = 97%)

### Connection

- [x] serverAddress
- [x] guidAuthKey
- [x] customHeaders
- [x] apiTimeout
- [x] acceptSelfSignedCerts

### Sync

- [x] finishedSetup
- [x] lastIncrementalSync
- [x] lastIncrementalSyncRowId
- [x] messagesPerPage
- [x] skipEmptyChats
- [x] syncContactsAutomatically

### Notifications

- [x] notifyReactions
- [x] notifyOnChatList
- [x] filterUnknownSenders
- [x] globalTextDetection

### Display

- [x] userName
- [ ] use24HrFormat *(Not in TOML config yet)*
- [x] redactedMode

### Theme

- [x] selectedTheme
- [x] skin
- [x] colorfulAvatars
- [x] colorfulBubbles
- [x] monetTheming

### Privacy

- [x] incognitoMode
- [x] hideMessagePreview
- [x] generateFakeContactNames
- [x] generateFakeMessageContent

### Conversation

- [x] autoOpenKeyboard
- [x] swipeToReply
- [x] swipeToArchive
- [x] moveChatCreatorToHeader
- [x] doubleTapForDetails
- [x] autoPlayGifs
- [x] showDeliveryTimestamps

## UI Screens (0/26 = 0%)

### Tauri UI

- [ ] SetupView (initial setup wizard) *(Tauri UI not yet started)*
- [ ] ConversationList *(Tauri UI not yet started)*
- [ ] ConversationView (message thread) *(Tauri UI not yet started)*
- [ ] MessageWidget *(Tauri UI not yet started)*
- [ ] SearchView (global search) *(Tauri UI not yet started)*
- [ ] CreateChat *(Tauri UI not yet started)*
- [ ] ConversationDetails *(Tauri UI not yet started)*
- [ ] AttachmentFullscreenViewer *(Tauri UI not yet started)*
- [ ] CameraWidget *(Tauri UI not yet started)*
- [ ] SettingsPanel (main) *(Tauri UI not yet started)*
- [ ] SettingsPanel > Connection & Server *(Tauri UI not yet started)*
- [ ] SettingsPanel > Message & Notification *(Tauri UI not yet started)*
- [ ] SettingsPanel > Theme & Style *(Tauri UI not yet started)*
- [ ] SettingsPanel > Desktop & Tray *(Tauri UI not yet started)*
- [ ] SettingsPanel > Chat List *(Tauri UI not yet started)*
- [ ] SettingsPanel > Conversation *(Tauri UI not yet started)*
- [ ] SettingsPanel > Misc & Advanced *(Tauri UI not yet started)*
- [ ] SettingsPanel > About & Links *(Tauri UI not yet started)*
- [ ] SettingsPanel > Backup & Restore *(Tauri UI not yet started)*
- [ ] SettingsPanel > FindMy *(Tauri UI not yet started)*
- [ ] SettingsPanel > Troubleshooting *(Tauri UI not yet started)*
- [ ] ThemePicker *(Tauri UI not yet started)*
- [ ] FontPicker *(Tauri UI not yet started)*
- [ ] FindMyDevicesList *(Tauri UI not yet started)*
- [ ] FindMyFriendsList *(Tauri UI not yet started)*
- [ ] ScheduledMessagesView *(Tauri UI not yet started)*
