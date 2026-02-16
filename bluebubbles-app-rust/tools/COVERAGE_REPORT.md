# BlueBubbles Rust Rewrite - Coverage Report

*Generated: 2026-02-16 23:14:32 UTC*

## Overall Progress

| Status | Count | Percentage |
|--------|-------|------------|
| Implemented | 338 | 88.7% |
| Stubbed | 2 | 0.5% |
| Missing | 41 | 10.8% |
| **Total** | **381** | **100%** |

---

## Per-Category Breakdown

| Category | Implemented | Stubbed | Missing | Total | Coverage |
|----------|-------------|---------|---------|-------|----------|
| API Endpoints | 71 | 0 | 0 | 71 | 100% |
| Socket Events | 13 | 0 | 0 | 13 | 100% |
| DB Models | 97 | 0 | 0 | 97 | 100% |
| DB Infrastructure | 10 | 0 | 0 | 10 | 100% |
| Services | 55 | 0 | 14 | 69 | 80% |
| CLI Commands | 35 | 2 | 0 | 37 | 95% |
| Core Infrastructure | 24 | 0 | 0 | 24 | 100% |
| Settings | 33 | 0 | 1 | 34 | 97% |
| UI Screens | 0 | 0 | 26 | 26 | 0% |

---

## Per-Crate Source Statistics

| Crate | Files | Lines | Structs | Functions | Tests |
|-------|-------|-------|---------|-----------|-------|
| bb-api | 15 | 2113 | 21 | 107 | 25 |
| bb-cli | 17 | 3837 | 0 | 19 | 0 |
| bb-core | 6 | 1195 | 12 | 24 | 12 |
| bb-models | 17 | 4948 | 23 | 176 | 71 |
| bb-services | 29 | 12150 | 34 | 310 | 292 |
| bb-socket | 4 | 1266 | 11 | 34 | 26 |
| bb-tauri | 5 | 662 | 4 | 23 | 0 |
| **Total** | | **26171** | | | **426** |

---

## API Endpoints

### Server

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| GET /api/v1/ping | OK | bb-api/src/endpoints/server.rs |  |
| GET /api/v1/server/info | OK | bb-api/src/endpoints/server.rs |  |
| POST /api/v1/server/restart/soft | OK | bb-api/src/endpoints/server.rs |  |
| POST /api/v1/server/restart/hard | OK | bb-api/src/endpoints/server.rs |  |
| GET /api/v1/server/update/check | OK | bb-api/src/endpoints/server.rs |  |
| POST /api/v1/server/update/install | OK | bb-api/src/endpoints/server.rs |  |
| GET /api/v1/server/statistics/totals | OK | bb-api/src/endpoints/server.rs |  |
| GET /api/v1/server/statistics/media | OK | bb-api/src/endpoints/server.rs |  |
| GET /api/v1/server/logs | OK | bb-api/src/endpoints/server.rs |  |

### Chat

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| GET /api/v1/chat/count | OK | bb-api/src/endpoints/chat.rs |  |
| POST /api/v1/chat/query | OK | bb-api/src/endpoints/chat.rs |  |
| GET /api/v1/chat/:guid | OK | bb-api/src/endpoints/chat.rs |  |
| PUT /api/v1/chat/:guid | OK | bb-api/src/endpoints/chat.rs |  |
| DELETE /api/v1/chat/:guid | OK | bb-api/src/endpoints/chat.rs |  |
| POST /api/v1/chat/new | OK | bb-api/src/endpoints/chat.rs |  |
| POST /api/v1/chat/:guid/read | OK | bb-api/src/endpoints/chat.rs |  |
| POST /api/v1/chat/:guid/unread | OK | bb-api/src/endpoints/chat.rs |  |
| POST /api/v1/chat/:guid/leave | OK | bb-api/src/endpoints/chat.rs |  |
| POST /api/v1/chat/:guid/participant/add | OK | bb-api/src/endpoints/chat.rs |  |
| POST /api/v1/chat/:guid/participant/remove | OK | bb-api/src/endpoints/chat.rs |  |
| GET /api/v1/chat/:guid/message | OK | bb-api/src/endpoints/chat.rs |  |
| GET /api/v1/chat/:guid/icon | OK | bb-api/src/endpoints/chat.rs |  |
| DELETE /api/v1/chat/:guid/:messageGuid | OK | bb-api/src/endpoints/chats.rs |  |

### Message

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| GET /api/v1/message/count | OK | bb-api/src/endpoints/message.rs |  |
| POST /api/v1/message/query | OK | bb-api/src/endpoints/message.rs |  |
| GET /api/v1/message/:guid | OK | bb-api/src/endpoints/message.rs |  |
| GET /api/v1/message/:guid/embedded-media | OK | bb-api/src/endpoints/message.rs |  |
| POST /api/v1/message/text | OK | bb-api/src/endpoints/message.rs |  |
| POST /api/v1/message/multipart | OK | bb-api/src/endpoints/message.rs |  |
| POST /api/v1/message/react | OK | bb-api/src/endpoints/message.rs |  |
| POST /api/v1/message/:guid/unsend | OK | bb-api/src/endpoints/message.rs |  |
| POST /api/v1/message/:guid/edit | OK | bb-api/src/endpoints/message.rs |  |
| POST /api/v1/message/:guid/notify | OK | bb-api/src/endpoints/message.rs |  |
| GET /api/v1/message/schedule | OK | bb-api/src/endpoints/message.rs |  |
| POST /api/v1/message/schedule | OK | bb-api/src/endpoints/message.rs |  |
| PUT /api/v1/message/schedule/:id | OK | bb-api/src/endpoints/message.rs |  |
| DELETE /api/v1/message/schedule/:id | OK | bb-api/src/endpoints/message.rs |  |

### Attachment

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| GET /api/v1/attachment/count | OK | bb-api/src/endpoints/attachment.rs |  |
| GET /api/v1/attachment/:guid | OK | bb-api/src/endpoints/attachment.rs |  |
| GET /api/v1/attachment/:guid/download | OK | bb-api/src/endpoints/attachment.rs |  |
| GET /api/v1/attachment/:guid/live | OK | bb-api/src/endpoints/attachment.rs |  |
| GET /api/v1/attachment/:guid/blurhash | OK | bb-api/src/endpoints/attachment.rs |  |
| POST /api/v1/attachment/upload (multipart) | OK | bb-api/src/endpoints/attachment.rs |  |

### Handle

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| GET /api/v1/handle/count | OK | bb-api/src/endpoints/handle.rs |  |
| POST /api/v1/handle/query | OK | bb-api/src/endpoints/handle.rs |  |
| GET /api/v1/handle/:guid | OK | bb-api/src/endpoints/handles.rs |  |
| GET /api/v1/handle/:address/focus | OK | bb-api/src/endpoints/handle.rs |  |
| POST /api/v1/handle/availability/imessage | OK | bb-api/src/endpoints/handle.rs |  |
| POST /api/v1/handle/availability/facetime | OK | bb-api/src/endpoints/handle.rs |  |

### Contact

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| GET /api/v1/contact | OK | bb-api/src/endpoints/contact.rs |  |
| POST /api/v1/contact/query | OK | bb-api/src/endpoints/contact.rs |  |
| POST /api/v1/contact/upload | OK | bb-api/src/endpoints/contact.rs |  |

### FCM

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| POST /api/v1/fcm/device | OK | bb-api/src/endpoints/fcm.rs |  |
| GET /api/v1/fcm/client | OK | bb-api/src/endpoints/fcm.rs |  |

### Mac

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| POST /api/v1/mac/lock | OK | bb-api/src/endpoints/mac.rs |  |
| POST /api/v1/mac/imessage/restart | OK | bb-api/src/endpoints/mac.rs |  |

### Backup

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| GET /api/v1/backup/theme | OK | bb-api/src/endpoints/backup.rs |  |
| POST /api/v1/backup/theme | OK | bb-api/src/endpoints/backup.rs |  |
| DELETE /api/v1/backup/theme | OK | bb-api/src/endpoints/backup.rs |  |
| GET /api/v1/backup/settings | OK | bb-api/src/endpoints/backup.rs |  |
| POST /api/v1/backup/settings | OK | bb-api/src/endpoints/backup.rs |  |
| DELETE /api/v1/backup/settings | OK | bb-api/src/endpoints/backup.rs |  |

### FaceTime

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| POST /api/v1/facetime/answer | OK | bb-api/src/endpoints/facetime.rs |  |
| POST /api/v1/facetime/leave | OK | bb-api/src/endpoints/facetime.rs |  |

### iCloud

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| GET /api/v1/icloud/findmy/devices | OK | bb-api/src/endpoints/icloud.rs |  |
| POST /api/v1/icloud/findmy/devices/refresh | OK | bb-api/src/endpoints/icloud.rs |  |
| GET /api/v1/icloud/findmy/friends | OK | bb-api/src/endpoints/icloud.rs |  |
| POST /api/v1/icloud/findmy/friends/refresh | OK | bb-api/src/endpoints/icloud.rs |  |
| GET /api/v1/icloud/account | OK | bb-api/src/endpoints/icloud.rs |  |
| GET /api/v1/icloud/contact | OK | bb-api/src/endpoints/icloud.rs |  |
| POST /api/v1/icloud/account/alias | OK | bb-api/src/endpoints/icloud.rs |  |

## Socket Events

### Events

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| new-message | OK | bb-socket/src/events.rs |  |
| updated-message | OK | bb-socket/src/events.rs |  |
| typing-indicator | OK | bb-socket/src/events.rs |  |
| chat-read-status-changed | OK | bb-socket/src/events.rs |  |
| group-name-change | OK | bb-socket/src/events.rs |  |
| participant-removed | OK | bb-socket/src/events.rs |  |
| participant-added | OK | bb-socket/src/events.rs |  |
| participant-left | OK | bb-socket/src/events.rs |  |
| group-icon-changed | OK | bb-socket/src/events.rs |  |
| incoming-facetime | OK | bb-socket/src/events.rs |  |
| ft-call-status-changed | OK | bb-socket/src/events.rs |  |
| imessage-aliases-removed | OK | bb-socket/src/events.rs |  |
| server-update | OK | bb-socket/src/events.rs |  |

## DB Models

### Chat

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| Chat.id | OK | bb-models/src/models/chat.rs |  |
| Chat.guid | OK | bb-models/src/models/chat.rs |  |
| Chat.chat_identifier | OK | bb-models/src/models/chat.rs |  |
| Chat.display_name | OK | bb-models/src/models/chat.rs |  |
| Chat.is_archived | OK | bb-models/src/models/chat.rs |  |
| Chat.mute_type | OK | bb-models/src/models/chat.rs |  |
| Chat.is_pinned | OK | bb-models/src/models/chat.rs |  |
| Chat.has_unread_message | OK | bb-models/src/models/chat.rs |  |
| Chat.style | OK | bb-models/src/models/chat.rs |  |
| Chat.latest_message_date | OK | bb-models/src/models/chat.rs |  |
| Chat.participants | OK | bb-models/src/models/chat.rs |  |
| Chat.from_server_map | OK | bb-models/src/models/chat.rs |  |
| Chat.save | OK | bb-models/src/models/chat.rs |  |

### Message

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| Message.id | OK | bb-models/src/models/message.rs |  |
| Message.guid | OK | bb-models/src/models/message.rs |  |
| Message.chat_id | OK | bb-models/src/models/message.rs |  |
| Message.handle_id | OK | bb-models/src/models/message.rs |  |
| Message.text | OK | bb-models/src/models/message.rs |  |
| Message.subject | OK | bb-models/src/models/message.rs |  |
| Message.error | OK | bb-models/src/models/message.rs |  |
| Message.date_created | OK | bb-models/src/models/message.rs |  |
| Message.is_from_me | OK | bb-models/src/models/message.rs |  |
| Message.item_type | OK | bb-models/src/models/message.rs |  |
| Message.associated_message_guid | OK | bb-models/src/models/message.rs |  |
| Message.has_attachments | OK | bb-models/src/models/message.rs |  |
| Message.attributed_body | OK | bb-models/src/models/message.rs |  |
| Message.payload_data | OK | bb-models/src/models/message.rs |  |
| Message.from_server_map | OK | bb-models/src/models/message.rs |  |
| Message.save | OK | bb-models/src/models/message.rs |  |

### Handle

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| Handle.id | OK | bb-models/src/models/handle.rs |  |
| Handle.address | OK | bb-models/src/models/handle.rs |  |
| Handle.service | OK | bb-models/src/models/handle.rs |  |
| Handle.formatted_address | OK | bb-models/src/models/handle.rs |  |
| Handle.contact | OK | bb-models/src/models/handle.rs |  |
| Handle.from_server_map | OK | bb-models/src/models/handle.rs |  |
| Handle.save | OK | bb-models/src/models/handle.rs |  |
| Handle.display_name | OK | bb-models/src/models/handle.rs |  |

### Attachment

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| Attachment.id | OK | bb-models/src/models/attachment.rs |  |
| Attachment.guid | OK | bb-models/src/models/attachment.rs |  |
| Attachment.mime_type | OK | bb-models/src/models/attachment.rs |  |
| Attachment.transfer_name | OK | bb-models/src/models/attachment.rs |  |
| Attachment.total_bytes | OK | bb-models/src/models/attachment.rs |  |
| Attachment.height | OK | bb-models/src/models/attachment.rs |  |
| Attachment.width | OK | bb-models/src/models/attachment.rs |  |
| Attachment.has_live_photo | OK | bb-models/src/models/attachment.rs |  |
| Attachment.from_server_map | OK | bb-models/src/models/attachment.rs |  |
| Attachment.save | OK | bb-models/src/models/attachment.rs |  |
| Attachment.is_image | OK | bb-models/src/models/attachment.rs |  |
| Attachment.is_video | OK | bb-models/src/models/attachment.rs |  |
| Attachment.is_audio | OK | bb-models/src/models/attachment.rs |  |

### Contact

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| Contact.id | OK | bb-models/src/models/contact.rs |  |
| Contact.external_id | OK | bb-models/src/models/contact.rs |  |
| Contact.display_name | OK | bb-models/src/models/contact.rs |  |
| Contact.phones | OK | bb-models/src/models/contact.rs |  |
| Contact.emails | OK | bb-models/src/models/contact.rs |  |
| Contact.avatar | OK | bb-models/src/models/contact.rs |  |
| Contact.structured_name | OK | bb-models/src/models/contact.rs |  |
| Contact.from_server_map | OK | bb-models/src/models/contact.rs |  |
| Contact.save | OK | bb-models/src/models/contact.rs |  |
| Contact.matches_address | OK | bb-models/src/models/contact.rs |  |

### FcmData

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| FcmData.project_id | OK | bb-models/src/models/fcm_data.rs |  |
| FcmData.storage_bucket | OK | bb-models/src/models/fcm_data.rs |  |
| FcmData.api_key | OK | bb-models/src/models/fcm_data.rs |  |
| FcmData.firebase_url | OK | bb-models/src/models/fcm_data.rs |  |
| FcmData.client_id | OK | bb-models/src/models/fcm_data.rs |  |
| FcmData.application_id | OK | bb-models/src/models/fcm_data.rs |  |
| FcmData.from_server_map | OK | bb-models/src/models/fcm_data.rs |  |
| FcmData.save | OK | bb-models/src/models/fcm_data.rs |  |
| FcmData.load | OK | bb-models/src/models/fcm_data.rs |  |

### ThemeStruct

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| ThemeStruct.id | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.name | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.gradient_bg | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.google_font | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.theme_data | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.parsed_data | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.is_dark | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.save | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.load_all | OK | bb-models/src/models/theme.rs |  |
| ThemeStruct.delete_by_name | OK | bb-models/src/models/theme.rs |  |

### ScheduledMessage

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| ScheduledMessage.id | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.type | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.chat_guid | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.message | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.scheduled_for | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.status | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.from_server_map | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.save | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.load_all | OK | bb-models/src/models/scheduled_message.rs |  |
| ScheduledMessage.delete | OK | bb-models/src/models/scheduled_message.rs |  |

### AttributedBody

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| AttributedBody.runs | OK | bb-models/src/models/attributed_body.rs |  |
| AttributedBody.plain_text | OK | bb-models/src/models/attributed_body.rs |  |
| AttributedBody.mentions | OK | bb-models/src/models/attributed_body.rs |  |
| AttributedBody.links | OK | bb-models/src/models/attributed_body.rs |  |
| AttributedBody.from_server_json | OK | bb-models/src/models/attributed_body.rs |  |

### PayloadData

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| PayloadData.url_preview | OK | bb-models/src/models/payload_data.rs |  |
| PayloadData.app_data | OK | bb-models/src/models/payload_data.rs |  |
| PayloadData.from_server_json | OK | bb-models/src/models/payload_data.rs |  |

## DB Infrastructure

### Core

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| Connection pooling (r2d2) | OK | bb-models/src/db.rs |  |
| WAL mode | OK | bb-models/src/db.rs |  |
| Integrity check | OK | bb-models/src/db.rs |  |
| Schema creation | OK | bb-models/src/db.rs |  |
| Versioned migrations | OK | bb-models/src/db.rs |  |
| Transaction support | OK | bb-models/src/db.rs |  |
| Database reset | OK | bb-models/src/db.rs |  |
| Database stats | OK | bb-models/src/db.rs |  |
| Settings key-value store | OK | bb-models/src/db.rs |  |
| Chat-Handle join table | OK | bb-models/src/db.rs |  |

## Services

### ChatService

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| ChatService.list_chats | OK | bb-services/src/chat.rs |  |
| ChatService.find_chat | OK | bb-services/src/chat.rs |  |
| ChatService.search_chats | OK | bb-services/src/chat.rs |  |
| ChatService.mark_read | OK | bb-services/src/chat.rs |  |
| ChatService.mark_unread | OK | bb-services/src/chat.rs |  |
| ChatService.create_chat | OK | bb-services/src/chat.rs |  |
| ChatService.rename_chat | OK | bb-services/src/chat.rs |  |
| ChatService.count | OK | bb-services/src/chat.rs |  |
| ChatService.toggle_pin | OK | bb-services/src/chat.rs |  |
| ChatService.toggle_archive | OK | bb-services/src/chat.rs |  |

### MessageService

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| MessageService.list_messages | OK | bb-services/src/message.rs |  |
| MessageService.find_message | OK | bb-services/src/message.rs |  |
| MessageService.search_messages | OK | bb-services/src/message.rs |  |
| MessageService.send_text | OK | bb-services/src/message.rs |  |
| MessageService.send_reaction | OK | bb-services/src/message.rs |  |
| MessageService.edit_message | OK | bb-services/src/message.rs |  |
| MessageService.unsend_message | OK | bb-services/src/message.rs |  |
| MessageService.handle_incoming_message | OK | bb-services/src/message.rs |  |
| MessageService.count_for_chat | OK | bb-services/src/message.rs |  |

### ContactService

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| ContactService.list_contacts | OK | bb-services/src/contact.rs |  |
| ContactService.search_contacts | OK | bb-services/src/contact.rs |  |
| ContactService.find_contact | OK | bb-services/src/contact.rs |  |
| ContactService.sync_contacts | OK | bb-services/src/contact.rs |  |
| ContactService.resolve_display_name | OK | bb-services/src/contact.rs |  |

### AttachmentService

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| AttachmentService.download | OK | bb-services/src/attachment.rs |  |
| AttachmentService.upload_and_send | OK | bb-services/src/attachment.rs |  |
| AttachmentService.attachments_for_message | OK | bb-services/src/attachment.rs |  |
| AttachmentService.cleanup_cache | OK | bb-services/src/attachment.rs |  |
| AttachmentService.cache_path | OK | bb-services/src/attachment.rs |  |
| AttachmentService.is_cached | OK | bb-services/src/attachment.rs |  |

### SyncService

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| SyncService.full_sync | OK | bb-services/src/sync.rs |  |
| SyncService.incremental_sync | OK | bb-services/src/sync.rs |  |

### SettingsService

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| SettingsService.server_address | OK | bb-services/src/settings.rs |  |
| SettingsService.set_server_address | OK | bb-services/src/settings.rs |  |
| SettingsService.guid_auth_key | OK | bb-services/src/settings.rs |  |
| SettingsService.set_guid_auth_key | OK | bb-services/src/settings.rs |  |
| SettingsService.is_setup_complete | OK | bb-services/src/settings.rs |  |
| SettingsService.mark_setup_complete | OK | bb-services/src/settings.rs |  |
| SettingsService.save | OK | bb-services/src/settings.rs |  |

### NotificationService

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| NotificationService.notify_message | OK | bb-services/src/notification.rs |  |
| NotificationService.notify_reaction | OK | bb-services/src/notification.rs |  |
| NotificationService.notify | OK | bb-services/src/notification.rs |  |
| NotificationService.set_enabled | OK | bb-services/src/notification.rs |  |

### QueueService

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| QueueService.enqueue | OK | bb-services/src/queue.rs |  |
| QueueService.dequeue | OK | bb-services/src/queue.rs |  |
| QueueService.len | OK | bb-services/src/queue.rs |  |
| QueueService.is_empty | OK | bb-services/src/queue.rs |  |
| QueueService.remove | OK | bb-services/src/queue.rs |  |
| QueueService.clear | OK | bb-services/src/queue.rs |  |

### ServiceRegistry

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| ServiceRegistry.register | OK | bb-services/src/registry.rs |  |
| ServiceRegistry.init_all | OK | bb-services/src/registry.rs |  |
| ServiceRegistry.shutdown_all | OK | bb-services/src/registry.rs |  |
| ServiceRegistry.set_api_client | OK | bb-services/src/registry.rs |  |
| ServiceRegistry.api_client | OK | bb-services/src/registry.rs |  |
| ServiceRegistry.health_check | OK | bb-services/src/registry.rs |  |

### Flutter-only

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| LifecycleService | MISSING |  | Flutter-specific service, not ported |
| ActionHandler | MISSING |  | Flutter-specific service, not ported |
| BackgroundIsolateService | MISSING |  | Flutter-specific service, not ported |
| NotificationListenerService | MISSING |  | Flutter-specific service, not ported |
| FCMService | MISSING |  | Flutter-specific service, not ported |
| UnifiedPushService | MISSING |  | Flutter-specific service, not ported |
| NetworkService | MISSING |  | Flutter-specific service, not ported |
| ThemeService | MISSING |  | Flutter-specific service, not ported |
| FindMyService | MISSING |  | Flutter-specific service, not ported |
| FaceTimeService | MISSING |  | Flutter-specific service, not ported |
| SocketService (high-level wrapper) | MISSING |  | Flutter-specific service, not ported |
| MethodChannelService | MISSING |  | Flutter-specific service, not ported |
| IntentService | MISSING |  | Flutter-specific service, not ported |
| PrivateApiService | MISSING |  | Flutter-specific service, not ported |

## CLI Commands

### connect

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb connect connect | OK | bb-cli/src/commands/connect.rs |  |

### status

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb status status | OK | bb-cli/src/commands/status.rs |  |

### chats

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb chats list | OK | bb-cli/src/commands/chats.rs |  |
| bb chats get | OK | bb-cli/src/commands/chats.rs |  |
| bb chats search | OK | bb-cli/src/commands/chats.rs |  |
| bb chats read | OK | bb-cli/src/commands/chats.rs |  |
| bb chats unread | OK | bb-cli/src/commands/chats.rs |  |

### messages

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb messages list | OK | bb-cli/src/commands/messages.rs |  |
| bb messages get | OK | bb-cli/src/commands/messages.rs |  |
| bb messages search | OK | bb-cli/src/commands/messages.rs |  |
| bb messages send | OK | bb-cli/src/commands/messages.rs |  |
| bb messages react | OK | bb-cli/src/commands/messages.rs |  |
| bb messages edit | OK | bb-cli/src/commands/messages.rs |  |
| bb messages unsend | OK | bb-cli/src/commands/messages.rs |  |

### contacts

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb contacts list | OK | bb-cli/src/commands/contacts.rs |  |
| bb contacts search | OK | bb-cli/src/commands/contacts.rs |  |
| bb contacts sync | OK | bb-cli/src/commands/contacts.rs |  |

### attachments

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb attachments info | OK | bb-cli/src/commands/attachments.rs |  |
| bb attachments download | OK | bb-cli/src/commands/attachments.rs |  |

### sync

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb sync full | OK | bb-cli/src/commands/sync.rs |  |
| bb sync incremental | OK | bb-cli/src/commands/sync.rs |  |

### settings

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb settings show | OK | bb-cli/src/commands/settings.rs |  |
| bb settings set-address | PARTIAL |  | Module exists, checking for set-address |
| bb settings set-password | PARTIAL |  | Module exists, checking for set-password |
| bb settings export | OK | bb-cli/src/commands/settings.rs |  |

### server

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb server ping | OK | bb-cli/src/commands/server.rs |  |
| bb server info | OK | bb-cli/src/commands/server.rs |  |
| bb server stats | OK | bb-cli/src/commands/server.rs |  |
| bb server restart | OK | bb-cli/src/commands/server.rs |  |
| bb server restart-hard | OK | bb-cli/src/commands/server.rs |  |
| bb server check-update | OK | bb-cli/src/commands/server.rs |  |
| bb server logs | OK | bb-cli/src/commands/server.rs |  |

### db

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb db stats | OK | bb-cli/src/commands/db.rs |  |
| bb db check | OK | bb-cli/src/commands/db.rs |  |
| bb db reset | OK | bb-cli/src/commands/db.rs |  |
| bb db path | OK | bb-cli/src/commands/db.rs |  |

### logs

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| bb logs view | OK | bb-cli/src/commands/logs.rs |  |

## Core Infrastructure

### Core

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| AppConfig (TOML) | OK | bb-core/src/config.rs |  |
| ServerConfig | OK | bb-core/src/config.rs |  |
| DatabaseConfig | OK | bb-core/src/config.rs |  |
| LoggingConfig | OK | bb-core/src/config.rs |  |
| SyncConfig | OK | bb-core/src/config.rs |  |
| NotificationConfig | OK | bb-core/src/config.rs |  |
| DisplayConfig | OK | bb-core/src/config.rs |  |
| ConfigHandle (thread-safe) | OK | bb-core/src/config.rs |  |
| BbError unified error type | OK | bb-core/src/error.rs |  |
| MessageError codes | OK | bb-core/src/error.rs |  |
| init_logging (tracing + file rotation) | OK | bb-core/src/logging.rs |  |
| init_console_logging | OK | bb-core/src/logging.rs |  |
| Platform detection (Win/Mac/Linux) | OK | bb-core/src/platform.rs |  |
| Platform data_dir / config_dir / cache_dir | OK | bb-core/src/platform.rs |  |
| Constants (reactions, effects, balloon bundles) | OK | bb-core/src/constants.rs |  |
| AES-256-CBC crypto (CryptoJS compatible) | OK | bb-socket/src/crypto.rs |  |
| SocketEventType enum | OK | bb-socket/src/events.rs |  |
| EventDispatcher (broadcast channels) | OK | bb-socket/src/events.rs |  |
| ConnectionState | OK | bb-socket/src/events.rs |  |
| SocketManager | OK | bb-socket/src/manager.rs |  |
| Reconnect with exponential backoff + jitter | OK | bb-socket/src/manager.rs |  |
| Event deduplication | OK | bb-socket/src/manager.rs |  |
| Service trait | OK | bb-services/src/service.rs |  |
| ServiceState lifecycle | OK | bb-services/src/service.rs |  |

## Settings

### Connection

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| serverAddress | OK | bb-core/src/config.rs |  |
| guidAuthKey | OK | bb-core/src/config.rs |  |
| customHeaders | OK | bb-core/src/config.rs |  |
| apiTimeout | OK | bb-core/src/config.rs |  |
| acceptSelfSignedCerts | OK | bb-core/src/config.rs |  |

### Sync

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| finishedSetup | OK | bb-core/src/config.rs |  |
| lastIncrementalSync | OK | bb-core/src/config.rs |  |
| lastIncrementalSyncRowId | OK | bb-core/src/config.rs |  |
| messagesPerPage | OK | bb-core/src/config.rs |  |
| skipEmptyChats | OK | bb-core/src/config.rs |  |
| syncContactsAutomatically | OK | bb-core/src/config.rs |  |

### Notifications

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| notifyReactions | OK | bb-core/src/config.rs |  |
| notifyOnChatList | OK | bb-core/src/config.rs |  |
| filterUnknownSenders | OK | bb-core/src/config.rs |  |
| globalTextDetection | OK | bb-core/src/config.rs |  |

### Display

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| userName | OK | bb-core/src/config.rs |  |
| use24HrFormat | MISSING |  | Not in TOML config yet |
| redactedMode | OK | bb-core/src/config.rs |  |

### Theme

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| selectedTheme | OK | bb-core/src/config.rs |  |
| skin | OK | bb-core/src/config.rs |  |
| colorfulAvatars | OK | bb-core/src/config.rs |  |
| colorfulBubbles | OK | bb-core/src/config.rs |  |
| monetTheming | OK | bb-core/src/config.rs |  |

### Privacy

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| incognitoMode | OK | bb-core/src/config.rs |  |
| hideMessagePreview | OK | bb-core/src/config.rs |  |
| generateFakeContactNames | OK | bb-core/src/config.rs |  |
| generateFakeMessageContent | OK | bb-core/src/config.rs |  |

### Conversation

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| autoOpenKeyboard | OK | bb-core/src/config.rs |  |
| swipeToReply | OK | bb-core/src/config.rs |  |
| swipeToArchive | OK | bb-core/src/config.rs |  |
| moveChatCreatorToHeader | OK | bb-core/src/config.rs |  |
| doubleTapForDetails | OK | bb-core/src/config.rs |  |
| autoPlayGifs | OK | bb-core/src/config.rs |  |
| showDeliveryTimestamps | OK | bb-core/src/config.rs |  |

## UI Screens

### Tauri UI

| Feature | Status | Rust File | Notes |
|---------|--------|-----------|-------|
| SetupView (initial setup wizard) | MISSING |  | Tauri UI not yet started |
| ConversationList | MISSING |  | Tauri UI not yet started |
| ConversationView (message thread) | MISSING |  | Tauri UI not yet started |
| MessageWidget | MISSING |  | Tauri UI not yet started |
| SearchView (global search) | MISSING |  | Tauri UI not yet started |
| CreateChat | MISSING |  | Tauri UI not yet started |
| ConversationDetails | MISSING |  | Tauri UI not yet started |
| AttachmentFullscreenViewer | MISSING |  | Tauri UI not yet started |
| CameraWidget | MISSING |  | Tauri UI not yet started |
| SettingsPanel (main) | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Connection & Server | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Message & Notification | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Theme & Style | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Desktop & Tray | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Chat List | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Conversation | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Misc & Advanced | MISSING |  | Tauri UI not yet started |
| SettingsPanel > About & Links | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Backup & Restore | MISSING |  | Tauri UI not yet started |
| SettingsPanel > FindMy | MISSING |  | Tauri UI not yet started |
| SettingsPanel > Troubleshooting | MISSING |  | Tauri UI not yet started |
| ThemePicker | MISSING |  | Tauri UI not yet started |
| FontPicker | MISSING |  | Tauri UI not yet started |
| FindMyDevicesList | MISSING |  | Tauri UI not yet started |
| FindMyFriendsList | MISSING |  | Tauri UI not yet started |
| ScheduledMessagesView | MISSING |  | Tauri UI not yet started |
