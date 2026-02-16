# API and Networking

This document describes the complete networking layer of the BlueBubbles Flutter client. It covers the REST API surface, Socket.IO real-time protocol, authentication, file transfer, sync mechanisms, SSL handling, and error recovery.

---

## Table of Contents

1. [Server Connection](#1-server-connection)
2. [REST API Endpoints](#2-rest-api-endpoints)
3. [Socket.IO Protocol](#3-socketio-protocol)
4. [Authentication Flow](#4-authentication-flow)
5. [Error Handling](#5-error-handling)
6. [SSL and Certificate Handling](#6-ssl-and-certificate-handling)
7. [File Transfer Protocol](#7-file-transfer-protocol)
8. [Sync Protocol](#8-sync-protocol)

---

## 1. Server Connection

### Architecture Overview

The BlueBubbles client communicates with a self-hosted BlueBubbles macOS server over two channels:

- **HTTP (REST API)** -- Request/response operations (fetching data, sending messages, uploading attachments). Implemented with the Dio HTTP client in `HttpService`.
- **Socket.IO (WebSocket)** -- Real-time event streaming (new messages, typing indicators, status changes). Implemented in `SocketService`.

### URL Configuration

The server address is stored in `ss.settings.serverAddress` and persisted across sessions. The `HttpService` derives two values from it:

| Property   | Derivation                                           | Example                          |
|------------|------------------------------------------------------|----------------------------------|
| `origin`   | `Uri.parse(serverAddress).origin` (scheme + host)    | `https://abc123.trycloudflare.com` |
| `apiRoot`  | `"$origin/api/v1"`                                   | `https://abc123.trycloudflare.com/api/v1` |

The address is sanitized by `sanitizeServerAddress()` in `lib/helpers/network/network_helpers.dart`. Sanitization rules:

- Strips surrounding quotes and whitespace.
- If no scheme is present, applies `https://` for ngrok.io, trycloudflare.com, and zrok.io domains; `http://` for all others.

### Localhost Detection

When the user has configured a `localhostPort`, the client attempts local network connection on each socket connect event via `NetworkTasks.detectLocalhost()`:

1. Fetches `serverInfo` from the remote server to obtain `local_ipv4s` and `local_ipv6s` arrays.
2. Iterates over those IPs with both `https` and `http` schemes, sending a `/api/v1/ping` to each.
3. The first address that responds with `"pong"` becomes `http.originOverride`, routing all subsequent HTTP traffic locally.
4. If no direct IP match is found, falls back to a port scan of the local subnet using `network_tools`.
5. On network switch away from WiFi/Ethernet, `originOverride` is cleared.

### Custom Headers

`HttpService.headers` automatically injects tunnel-specific headers:

| Server Tunnel   | Header Added                          |
|-----------------|---------------------------------------|
| ngrok           | `ngrok-skip-browser-warning: true`    |
| zrok            | `skip_zrok_interstitial: true`        |

Additional custom headers come from `ss.settings.customHeaders`.

---

## 2. REST API Endpoints

All endpoints are relative to `{apiRoot}` which resolves to `{origin}/api/v1`. Every request includes the query parameter `guid={guidAuthKey}` for authentication.

### Server Management

| Method | Path                            | Query Params        | Request Body | Response Data                                                                 | Description                              |
|--------|---------------------------------|---------------------|--------------|-------------------------------------------------------------------------------|------------------------------------------|
| GET    | `/ping`                         | `guid`              | --           | `{ "message": "pong" }`                                                      | Health check; measures latency           |
| GET    | `/server/info`                  | `guid`              | --           | `{ "data": { "os_version", "server_version", "private_api", "detected_icloud", "local_ipv4s", "local_ipv6s", ... } }` | Server metadata (cached 1 min client-side) |
| GET    | `/server/restart/soft`          | `guid`              | --           | `{ "status": 200 }`                                                          | Restart server app services              |
| GET    | `/server/restart/hard`          | `guid`              | --           | `{ "status": 200 }`                                                          | Restart entire server application        |
| GET    | `/server/update/check`          | `guid`              | --           | `{ "data": { ... } }`                                                        | Check for new server versions            |
| POST   | `/server/update/install`        | `guid`              | --           | `{ "status": 200 }`                                                          | Install a server update                  |
| GET    | `/server/statistics/totals`     | `guid`              | --           | `{ "data": { "handles", "messages", "chats", "attachments" } }`              | Database totals                          |
| GET    | `/server/statistics/media`      | `guid`              | --           | `{ "data": { "images", "videos", "locations" } }`                            | Media totals                             |
| GET    | `/server/statistics/media/chat` | `guid`              | --           | `{ "data": { ... } }`                                                        | Media totals split by chat               |
| GET    | `/server/logs`                  | `guid`, `count`     | --           | `{ "data": [ ...log entries ] }`                                             | Server logs (default count: 10000)       |

### Mac Control

| Method | Path                       | Query Params | Request Body | Response Data        | Description            |
|--------|----------------------------|--------------|--------------|----------------------|------------------------|
| POST   | `/mac/lock`                | `guid`       | --           | `{ "status": 200 }`  | Lock the Mac           |
| POST   | `/mac/imessage/restart`    | `guid`       | --           | `{ "status": 200 }`  | Restart iMessage app   |

### FCM (Firebase Cloud Messaging)

| Method | Path           | Query Params | Request Body                              | Response Data                  | Description                       |
|--------|----------------|--------------|-------------------------------------------|--------------------------------|-----------------------------------|
| POST   | `/fcm/device`  | `guid`       | `{ "name": string, "identifier": string }`| `{ "status": 200 }`           | Register FCM device with server   |
| GET    | `/fcm/client`  | `guid`       | --                                        | `{ "data": { ...fcmConfig } }` | Get FCM configuration data        |

### Chats

| Method | Path                                   | Query Params                 | Request Body                                                                                           | Response Data                         | Description                              |
|--------|----------------------------------------|------------------------------|--------------------------------------------------------------------------------------------------------|---------------------------------------|------------------------------------------|
| POST   | `/chat/query`                          | `guid`                       | `{ "with": ["participants","lastmessage","sms","archived"], "offset": int, "limit": int, "sort": string }` | `{ "data": [ ...chats ] }`           | Query chats with optional includes       |
| GET    | `/chat/count`                          | `guid`                       | --                                                                                                     | `{ "data": { "total": int } }`       | Total chat count                         |
| GET    | `/chat/{guid}`                         | `guid`, `with`               | --                                                                                                     | `{ "data": { ...chat } }`            | Get single chat. `with` options: `"participants"`, `"lastmessage"` |
| PUT    | `/chat/{guid}`                         | `guid`                       | `{ "displayName": string }`                                                                            | `{ "data": { ...chat } }`            | Update chat display name                 |
| DELETE | `/chat/{guid}`                         | `guid`                       | --                                                                                                     | `{ "status": 200 }`                  | Delete a chat                            |
| POST   | `/chat/new`                            | `guid`                       | `{ "addresses": [string], "message": string, "service": string, "method": "private-api"\|"apple-script" }` | `{ "data": { ...chat } }`     | Create a new chat                        |
| POST   | `/chat/{guid}/read`                    | `guid`                       | --                                                                                                     | `{ "status": 200 }`                  | Mark chat as read                        |
| POST   | `/chat/{guid}/unread`                  | `guid`                       | --                                                                                                     | `{ "status": 200 }`                  | Mark chat as unread                      |
| POST   | `/chat/{guid}/leave`                   | `guid`                       | --                                                                                                     | `{ "status": 200 }`                  | Leave a group chat                       |
| POST   | `/chat/{guid}/participant/{method}`    | `guid`                       | `{ "address": string }`                                                                                | `{ "data": { ...chat } }`            | Add or remove participant (`method` = `"add"` or `"remove"`) |
| GET    | `/chat/{guid}/message`                 | `guid`, `with`, `sort`, `before`, `after`, `offset`, `limit` | --                                                        | `{ "data": [ ...messages ] }`        | Get messages for a chat. `with` options: `"attachment"`, `"handle"`, `"sms"`, `"message.attributedbody"` |
| GET    | `/chat/{guid}/icon`                    | `guid`                       | --                                                                                                     | Binary image data                     | Get group chat icon (response type: bytes) |
| POST   | `/chat/{guid}/icon`                    | `guid`                       | Multipart: `{ "icon": File }`                                                                          | `{ "status": 200 }`                  | Set group chat icon                      |
| DELETE | `/chat/{guid}/icon`                    | `guid`                       | --                                                                                                     | `{ "status": 200 }`                  | Delete group chat icon                   |
| DELETE | `/chat/{guid}/{messageGuid}`           | `guid`                       | --                                                                                                     | `{ "status": 200 }`                  | Delete a specific message from a chat    |

### Messages

| Method | Path                                    | Query Params                          | Request Body                                                                                                                                | Response Data                              | Description                                       |
|--------|-----------------------------------------|---------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------|----------------------------------------------------|
| GET    | `/message/count`                        | `guid`, `after`, `before`             | --                                                                                                                                          | `{ "data": { "total": int } }`            | Total message count                                |
| GET    | `/message/count/updated`                | `guid`, `after`, `before`             | --                                                                                                                                          | `{ "data": { "total": int } }`            | Count of updated messages                          |
| GET    | `/message/count/me`                     | `guid`, `after`, `before`             | --                                                                                                                                          | `{ "data": { "total": int } }`            | Count of messages sent by user                     |
| POST   | `/message/query`                        | `guid`                                | `{ "with": [string], "where": [object], "sort": "ASC"\|"DESC", "before": int, "after": int, "chatGuid": string, "offset": int, "limit": int, "convertAttachments": bool }` | `{ "data": [ ...messages ], "metadata": { "total": int } }` | Query messages with filters |
| GET    | `/message/{guid}`                       | `guid`, `with`                        | --                                                                                                                                          | `{ "data": { ...message } }`              | Get single message. `with` options: `"chats"`, `"attachment"`, `"chats.participants"`, `"attributedBody"` |
| GET    | `/message/{guid}/embedded-media`        | `guid`                                | --                                                                                                                                          | Binary data                                | Get embedded media for digital touch/handwritten messages |
| POST   | `/message/text`                         | `guid`                                | `{ "chatGuid": string, "tempGuid": string, "message": string, "method": string, "effectId": string?, "subject": string?, "selectedMessageGuid": string?, "partIndex": int?, "ddScan": bool? }` | `{ "data": { ...message } }` | Send a text message |
| POST   | `/message/attachment`                   | `guid`                                | Multipart: `{ "attachment": File, "chatGuid": string, "tempGuid": string, "name": string, "method": string, "effectId"?: string, "subject"?: string, "selectedMessageGuid"?: string, "partIndex"?: int, "isAudioMessage"?: bool }` | `{ "data": { ...message } }` | Send an attachment |
| POST   | `/message/multipart`                    | `guid`                                | `{ "chatGuid": string, "tempGuid": string, "parts": [{ "text": string, "mention": string?, "partIndex": int? }], "effectId"?: string, "subject"?: string, "selectedMessageGuid"?: string, "partIndex"?: int, "ddScan"?: bool }` | `{ "data": { ...message } }` | Send a multipart message (mentions) |
| POST   | `/message/react`                        | `guid`                                | `{ "chatGuid": string, "selectedMessageText": string, "selectedMessageGuid": string, "reaction": string, "partIndex": int? }`              | `{ "data": { ...message } }`              | Send a tapback/reaction                            |
| POST   | `/message/{guid}/unsend`                | `guid`                                | `{ "partIndex": int }`                                                                                                                     | `{ "data": { ...message } }`              | Unsend a message                                   |
| POST   | `/message/{guid}/edit`                  | `guid`                                | `{ "editedMessage": string, "backwardsCompatibilityMessage": string, "partIndex": int }`                                                   | `{ "data": { ...message } }`              | Edit a sent message                                |
| POST   | `/message/{guid}/notify`                | `guid`                                | --                                                                                                                                          | `{ "data": { ...message } }`              | Send a notify-anyway for message                   |
| GET    | `/message/schedule`                     | `guid`                                | --                                                                                                                                          | `{ "data": [ ...scheduled ] }`            | Get all scheduled messages                         |
| POST   | `/message/schedule`                     | `guid`                                | `{ "type": "send-message", "payload": { "chatGuid": string, "message": string, "method": string }, "scheduledFor": int (ms epoch), "schedule": object }` | `{ "data": { ...scheduled } }` | Create a scheduled message |
| PUT    | `/message/schedule/{id}`                | `guid`                                | Same as POST body above                                                                                                                    | `{ "data": { ...scheduled } }`            | Update a scheduled message                         |
| DELETE | `/message/schedule/{id}`                | `guid`                                | --                                                                                                                                          | `{ "status": 200 }`                       | Delete a scheduled message                         |

#### Message Query `where` Clause Format

The `where` parameter in `/message/query` accepts an array of condition objects used for row-based filtering (server v1.6.0+):

```json
[
  {
    "statement": "message.ROWID > :startRowId",
    "args": { "startRowId": 12345 }
  },
  {
    "statement": "message.ROWID <= :endRowId",
    "args": { "endRowId": 67890 }
  }
]
```

#### Message Query `with` Options

Valid values for message query and single message retrieval:

- `"chats"` / `"chat"` -- Include associated chat data
- `"attachment"` / `"attachments"` -- Include attachment metadata
- `"handle"` -- Include sender handle data
- `"chats.participants"` / `"chat.participants"` -- Include chat participant handles
- `"attachment.metadata"` -- Include attachment metadata details
- `"attributedBody"` -- Include rich text body information
- `"messageSummaryInfo"` -- Include message summary data
- `"payloadData"` -- Include payload data

### Attachments

| Method | Path                              | Query Params             | Request Body | Response Data            | Description                                |
|--------|-----------------------------------|--------------------------|--------------|--------------------------|--------------------------------------------|
| GET    | `/attachment/count`               | `guid`                   | --           | `{ "data": { "total": int } }` | Total attachment count                |
| GET    | `/attachment/{guid}`              | `guid`                   | --           | `{ "data": { ...attachment } }` | Get attachment metadata              |
| GET    | `/attachment/{guid}/download`     | `guid`, `original`       | --           | Binary file data         | Download attachment bytes. `original=true` skips server-side conversion. Receive timeout is 12x normal. |
| GET    | `/attachment/{guid}/live`         | `guid`                   | --           | Binary file data         | Download live photo video component        |
| GET    | `/attachment/{guid}/blurhash`     | `guid`                   | --           | Binary data              | Get attachment blurhash placeholder        |

### Handles (Contacts in iMessage DB)

| Method | Path                                    | Query Params          | Request Body                                                        | Response Data                        | Description                                |
|--------|-----------------------------------------|-----------------------|---------------------------------------------------------------------|--------------------------------------|--------------------------------------------|
| GET    | `/handle/count`                         | `guid`                | --                                                                  | `{ "data": { "total": int } }`      | Total handle count                         |
| POST   | `/handle/query`                         | `guid`                | `{ "with": [string], "address": string?, "offset": int, "limit": int }` | `{ "data": [ ...handles ] }`   | Query handles. `with` options: `"chats"`, `"chats.participants"` |
| GET    | `/handle/{guid}`                        | `guid`                | --                                                                  | `{ "data": { ...handle } }`         | Get single handle by GUID                  |
| GET    | `/handle/{address}/focus`               | `guid`                | --                                                                  | `{ "data": { ...focusState } }`     | Get handle focus/DND state                 |
| GET    | `/handle/availability/imessage`         | `guid`, `address`     | --                                                                  | `{ "data": { ...state } }`          | Check iMessage availability for address    |
| GET    | `/handle/availability/facetime`         | `guid`, `address`     | --                                                                  | `{ "data": { ...state } }`          | Check FaceTime availability for address    |

### Contacts (iCloud)

| Method | Path                   | Query Params                 | Request Body                         | Response Data                  | Description                                |
|--------|------------------------|------------------------------|--------------------------------------|--------------------------------|--------------------------------------------|
| GET    | `/contact`             | `guid`, `extraProperties`    | --                                   | `{ "data": [ ...contacts ] }` | Get all iCloud contacts. `extraProperties=avatar` to include avatars. |
| POST   | `/contact/query`       | `guid`                       | `{ "addresses": [string] }`          | `{ "data": [ ...contacts ] }` | Get contacts by phone numbers or emails    |
| POST   | `/contact`             | `guid`                       | `[ { ...contactMap }, ... ]`         | `{ "status": 200 }`           | Create/upload contacts to server. Extended timeout (12x). |

### Backups (Theme and Settings)

| Method | Path                | Query Params | Request Body                                    | Response Data                     | Description                    |
|--------|---------------------|--------------|-------------------------------------------------|-----------------------------------|--------------------------------|
| GET    | `/backup/theme`     | `guid`       | --                                              | `{ "data": { ...themes } }`      | Get theme backup JSON          |
| POST   | `/backup/theme`     | `guid`       | `{ "name": string, "data": { ...themeJson } }`  | `{ "status": 200 }`              | Save theme backup              |
| DELETE | `/backup/theme`     | `guid`       | `{ "name": string }`                            | `{ "status": 200 }`              | Delete theme backup            |
| GET    | `/backup/settings`  | `guid`       | --                                              | `{ "data": { ...settings } }`    | Get settings backup            |
| POST   | `/backup/settings`  | `guid`       | `{ "name": string, "data": { ...settingsJson } }` | `{ "status": 200 }`           | Save settings backup           |
| DELETE | `/backup/settings`  | `guid`       | `{ "name": string }`                            | `{ "status": 200 }`              | Delete settings backup         |

### FaceTime

| Method | Path                              | Query Params | Request Body | Response Data                                   | Description                          |
|--------|-----------------------------------|--------------|--------------|-------------------------------------------------|--------------------------------------|
| POST   | `/facetime/answer/{callUuid}`     | `guid`       | `{}`         | `{ "data": { "link": string } }`               | Answer FaceTime call; returns join link |
| POST   | `/facetime/leave/{callUuid}`      | `guid`       | `{}`         | `{ "status": 200 }`                             | Leave a FaceTime call                |

### iCloud Services

| Method | Path                                   | Query Params | Request Body              | Response Data                       | Description                         |
|--------|----------------------------------------|--------------|---------------------------|-------------------------------------|-------------------------------------|
| GET    | `/icloud/findmy/devices`               | `guid`       | --                        | `{ "data": [ ...devices ] }`       | Get FindMy devices                  |
| POST   | `/icloud/findmy/devices/refresh`       | `guid`       | --                        | `{ "data": [ ...devices ] }`       | Refresh FindMy device locations (12x timeout) |
| GET    | `/icloud/findmy/friends`               | `guid`       | --                        | `{ "data": [ ...friends ] }`       | Get FindMy friends                  |
| POST   | `/icloud/findmy/friends/refresh`       | `guid`       | --                        | `{ "data": [ ...friends ] }`       | Refresh FindMy friend locations     |
| GET    | `/icloud/account`                      | `guid`       | --                        | `{ "data": { ...accountInfo } }`   | Get iCloud account info             |
| GET    | `/icloud/contact`                      | `guid`       | --                        | `{ "data": { ...contactCard } }`   | Get iCloud account contact card     |
| POST   | `/icloud/account/alias`                | `guid`       | `{ "alias": string }`    | `{ "status": 200 }`                | Set iCloud account alias            |

### Landing Page

| Method | Path   | Query Params | Request Body | Response Data | Description                     |
|--------|--------|--------------|--------------|--------------|---------------------------------|
| GET    | `/`    | `guid`       | --           | HTML page    | Server landing/status page      |

### Firebase External API Calls

These calls are made directly to Google/Firebase APIs (not the BlueBubbles server). They bypass the `origin` check.

| Method | URL                                                                        | Auth Parameter                   | Description                              |
|--------|----------------------------------------------------------------------------|----------------------------------|------------------------------------------|
| GET    | `https://firebase.googleapis.com/v1beta1/projects`                         | `access_token` query param       | List Firebase projects                   |
| GET    | `https://www.googleapis.com/oauth2/v1/userinfo`                            | `access_token` query param       | Get Google user info                     |
| GET    | `https://{rtdb}.firebaseio.com/config.json`                                | `token` query param              | Get server URL from Realtime Database    |
| GET    | `https://firestore.googleapis.com/v1/projects/{project}/databases/(default)/documents/server/config` | `access_token` query param | Get server URL from Cloud Firestore |
| PATCH  | `https://firestore.googleapis.com/v1/projects/{project}/databases/(default)/documents/server/commands?updateMask.fieldPaths=nextRestart` | -- | Set restart date in Cloud Firestore |

---

## 3. Socket.IO Protocol

### Connection Setup

The Socket.IO connection is configured in `SocketService.startSocket()`:

- **Server URL**: Same `origin` used by the HTTP service.
- **Authentication**: Passed as a query parameter: `{ "guid": password }` where `password` is `ss.settings.guidAuthKey.value`.
- **Transports**: `['websocket', 'polling']` (WebSocket preferred, HTTP long-polling fallback).
- **Extra Headers**: Same custom headers as the HTTP service (ngrok/zrok skip headers, user custom headers).
- **Reconnection**: Enabled by default via the Socket.IO client library.

### Connection State Machine

```
disconnected --> connecting --> connected
      ^              |              |
      |              v              v
      +---------- error <----------+
```

States are tracked as `SocketState` enum: `connected`, `disconnected`, `error`, `connecting`.

### Socket.IO Events -- Server to Client

| Event Name                   | Payload Format                                   | Platform Filter          | Description                                              |
|------------------------------|--------------------------------------------------|--------------------------|----------------------------------------------------------|
| `new-message`                | `ServerPayload` JSON (see below)                 | All                      | New message received in any chat                         |
| `updated-message`            | `ServerPayload` JSON                             | All                      | Existing message was updated (delivered, read, edited)   |
| `typing-indicator`           | `{ "guid": string, "display": bool }`            | All                      | Someone started or stopped typing in a chat              |
| `chat-read-status-changed`   | `{ "chatGuid": string, "read": bool }`           | All                      | Chat read status changed on another device               |
| `group-name-change`          | Message data map                                 | Web and Desktop only     | Group chat name was changed                              |
| `participant-removed`        | Message data map                                 | Web and Desktop only     | Participant removed from group chat                      |
| `participant-added`          | Message data map                                 | Web and Desktop only     | Participant added to group chat                          |
| `participant-left`           | Message data map                                 | Web and Desktop only     | Participant left a group chat                            |
| `incoming-facetime`          | JSON-encoded string (decoded before handling)    | Web and Desktop only     | Incoming FaceTime call (legacy format)                   |
| `ft-call-status-changed`     | `{ "uuid": string, "status_id": int, "handle": { "address": string }, "address": string, "is_audio": bool }` | All | FaceTime call status changed. `status_id` 4 = incoming, 6 = ended. |
| `imessage-aliases-removed`   | `{ "aliases": [string] }`                        | All                      | iMessage aliases were removed from the account           |

**Platform filtering note**: On Android, FCM handles `group-name-change`, `participant-removed`, `participant-added`, `participant-left`, and `incoming-facetime` events instead of the socket. The socket only listens for these on web and desktop platforms.

### Socket.IO Events -- Client to Server

The client sends messages to the server via `SocketService.sendMessage()`, which uses `socket.emitWithAck()`:

| Usage Context     | Event (dynamic) | Payload                        | Ack Response                                | Description                        |
|-------------------|-----------------|--------------------------------|---------------------------------------------|------------------------------------|
| General emit      | Varies          | `Map<String, dynamic>`         | `Map<String, dynamic>` (may be encrypted)   | Generic emit-with-ack pattern      |

The response may include `{ "encrypted": true, "data": "base64-encoded-AES-string" }`, in which case the data is decrypted using `decryptAESCryptoJS(data, password)` before being returned to the caller.

### ServerPayload Format

All socket event data is wrapped in a `ServerPayload` object parsed from the incoming JSON:

```json
{
  "type": "NEW_MESSAGE" | "UPDATED_MESSAGE" | "MESSAGE" | "CHAT" | "ATTACHMENT" | "HANDLE" | "OTHER",
  "subtype": "string (optional)",
  "encrypted": false,
  "partial": false,
  "encoding": "JSON_OBJECT" | "BASE64" | "JSON_STRING",
  "encryptionType": "AES_PB",
  "data": { ... }
}
```

If `encrypted` is `true`, the `data` field is decrypted using AES-CBC with PKCS7 padding, where the key and IV are derived from the `guidAuthKey` password using MD5-based key derivation (OpenSSL-compatible `Salted__` prefix format).

### Reconnection Behavior

On connection error:

1. State transitions to `SocketState.error`.
2. A 5-second timer starts.
3. After 5 seconds, if still not connected:
   - Calls `fdb.fetchNewUrl()` to get a potentially updated server URL from Firebase.
   - Calls `restartSocket()` to close and re-establish the connection.
   - If `keepAppAlive` is disabled, creates a socket error notification.

On connectivity change (WiFi/Ethernet lost):

- `http.originOverride` is cleared (disabling localhost routing).

---

## 4. Authentication Flow

### Initial Setup Authentication

The `guidAuthKey` serves as the primary authentication credential. It is a password/token string set during initial server configuration.

1. **User enters server URL and password** during first-time setup.
2. The password is stored as `ss.settings.guidAuthKey`.
3. Every HTTP request appends `?guid={guidAuthKey}` via `buildQueryParams()`.
4. The Socket.IO connection passes `{ "guid": password }` as a query parameter during handshake.

### Token Management

There is no token refresh or expiration mechanism for the primary `guidAuthKey`. The same key persists until the user changes it or calls `forgetConnection()`, which:

1. Closes the socket.
2. Clears `guidAuthKey` to empty string.
3. Clears the server URL.

### Firebase Cloud Messaging Authentication (Android)

FCM registration follows this sequence:

1. Check preconditions: setup complete, `keepAppAlive` disabled, FCM data available.
2. If an FCM token already exists, call `POST /fcm/device` to register it with the server.
3. If no token exists:
   a. Invoke native `firebase-auth` method with `fcmData.toMap()`.
   b. If authentication fails, fetch fresh FCM config from `GET /fcm/client`.
   c. Parse the new `FCMData`, save it, and retry `firebase-auth`.
   d. On success, register the resulting token with `POST /fcm/device`.
4. Device name is generated from platform info (brand + model + unique timestamp ID on Android).

### Firebase Database Authentication (Web/Desktop)

On web and desktop, the app uses `firebase_dart` to connect directly to Firebase:

1. FCM config is fetched from the server via `GET /fcm/client`.
2. A `FirebaseApp` is initialized with `FirebaseOptions` containing `apiKey`, `appId`, `projectId`, and `databaseURL`.
3. If `firebaseURL` is set, Realtime Database is used: reads `config/serverUrl`.
4. If `firebaseURL` is not set, Cloud Firestore is used: reads `server/config` document's `serverUrl` field.
5. The retrieved URL replaces the current server address.

### OAuth Client IDs

Different OAuth client IDs are used per platform:

| Platform                     | Client ID                                                |
|------------------------------|----------------------------------------------------------|
| Web (debug / tneotia.github.io) | `500464701389-5u2eogcqls1eljhu3hq825oed6iue1f0.apps.googleusercontent.com` |
| Web (production)             | `500464701389-8trcdkcj7ni5l4dn6n7l795rhb1asnh3.apps.googleusercontent.com` |
| Desktop                      | `500464701389-18rfq995s6dqo3e5d3n2e7i3ljr0uc9i.apps.googleusercontent.com` |

---

## 5. Error Handling

### HTTP Error Interception

The `ApiInterceptor` class intercepts all Dio errors and normalizes them:

| Condition                    | Behavior                                                         |
|------------------------------|------------------------------------------------------------------|
| Response is a `Map`          | Resolves with the original response (allows caller to inspect)   |
| Response exists but not Map  | Wraps in `{ "status": statusCode, "error": { "type": "Error", "error": responseData } }` |
| Timeout error                | Returns `{ "status": 500, "error": { "type": "timeout", "error": "Failed to receive response from server." } }` |
| Other errors                 | Propagated to the default Dio error handler                      |

### Retry Logic

- **Cloudflare 502 retry**: If a request returns HTTP 502 and the URL contains `trycloudflare`, the request is retried once automatically in `runApiGuarded()`.
- **Socket reconnect**: On socket error, a 5-second timer triggers URL refresh and socket restart.
- **No general HTTP retry**: Regular HTTP failures are not automatically retried beyond the Cloudflare case.

### Timeout Configuration

| Timeout Type      | Default Value                         | Notes                                     |
|-------------------|---------------------------------------|-------------------------------------------|
| Connect timeout   | 15,000 ms                             | Fixed at Dio initialization               |
| Receive timeout   | `ss.settings.apiTimeout` ms           | User-configurable                         |
| Send timeout      | `ss.settings.apiTimeout` ms           | User-configurable                         |
| Extended timeout  | Base timeout x 12                     | Used for attachment download/upload, contact upload, FindMy refresh, chat icon upload |

### Send Error Handling

When a message fails to send, `handleSendError()` in `lib/helpers/network/network_error_handler.dart` processes the error:

| Error Type             | GUID Transform                                           | Error Code                    |
|------------------------|----------------------------------------------------------|-------------------------------|
| `Response` error       | `temp` replaced with `error-{server error message}`      | Response status code          |
| `DioException` timeout | `temp` replaced with `error-{timeout description}`       | Response status code or 400   |
| Connection timeout     | Specific connect/send/receive timeout messages           | 400 (BAD_REQUEST)             |
| Other                  | `temp` replaced with generic connection timeout message  | 400 (BAD_REQUEST)             |

The GUID mutation from `temp-*` to `error-*` is used by the UI to display error states on messages.

### Server Info Caching

`serverInfo()` responses are cached for 1 minute to avoid redundant calls. The cache is a simple in-memory `Response` object with a timestamp check.

---

## 6. SSL and Certificate Handling

### BadCertOverride

The class `BadCertOverride` in `lib/services/network/http_overrides.dart` extends `HttpOverrides` to handle self-signed or invalid SSL certificates.

**Behavior**:

1. Overrides the global `HttpClient` factory.
2. Sets a `badCertificateCallback` that checks if the certificate's host matches the configured server URL.
3. **Wildcard host matching**: If the host starts with `*`, a regex is built from the last two domain segments and matched against the server URL.
4. **Direct host matching**: If the host does not start with `*`, checks if the sanitized server URL ends with the certificate host.
5. Returns `true` (accept certificate) only if the host matches the server URL.
6. Sets the global `hasBadCert` flag for UI indication.

**Registration**: Applied via `HttpOverrides.global = BadCertOverride()` early in the application lifecycle and in isolates used for incremental sync.

This means the app accepts self-signed certificates only when they belong to the configured BlueBubbles server, not for arbitrary HTTPS connections.

---

## 7. File Transfer Protocol

### Attachment Download

Downloads are managed by `AttachmentDownloadService` and individual `AttachmentDownloadController` instances.

**Queue system**:

- Maximum 2 concurrent downloads (`maxDownloads = 2`).
- Downloads are organized by chat GUID.
- Active chat downloads are prioritized over background chat downloads.
- When a download completes or fails, the next queued download starts automatically.

**Download flow**:

1. `AttachmentDownloadController` is created and registered via `Get.put()` with the attachment GUID as tag.
2. Controller is added to the download queue.
3. When a slot opens, `fetchAttachment()` is called.
4. Calls `GET /attachment/{guid}/download` with progress tracking.
5. On success:
   - GIF files are processed through `fixSpeedyGifs()`.
   - On mobile: file is written to `{appDocDir}/attachments/{guid}/{transferName}`.
   - On desktop: file is written after all processing.
   - Attachment properties are loaded and saved to database.
   - If `autoSave` is enabled on mobile, the file is saved to the device Downloads folder.
6. On failure: error state is set, error callbacks fire, controller is removed from queue.

**Progress tracking**: Progress is reported as a 0-1 double via `RxnNum progress`. On web, progress is `count / total` from the HTTP response. On native, progress is `count / attachment.totalBytes`.

### Attachment Upload

Uploads are handled by `ActionHandler.sendAttachment()`:

1. Attachment bytes are prepared and stored locally.
2. A `Tuple2<guid, RxDouble>` progress tracker is added to `attachmentProgress`.
3. `POST /message/attachment` is called with multipart form data.
4. Extended timeout (12x) is used for both send and receive.
5. Progress is tracked via `onSendProgress` callback.
6. On success: the temp message/attachment GUIDs are matched to the server-assigned GUIDs.
7. A `CancelToken` (`latestCancelToken`) allows the user to cancel an in-progress upload.

### Live Photo Download

Live photos (video component) are downloaded separately via `GET /attachment/{guid}/live` with byte response type and 12x timeout.

---

## 8. Sync Protocol

### Full Sync

Full sync is triggered during initial setup. It downloads all chats and their messages from the server.

**Flow**:

1. Set `lastIncrementalSync` timestamp to now.
2. Refresh contacts from server.
3. Fetch total chat count via `GET /chat/count`.
4. Stream chat pages in batches of 200 via `POST /chat/query` with `offset` and `limit`.
5. Bulk-save chats to local database.
6. For each chat, stream messages via `GET /chat/{guid}/message`:
   - Uses `withQuery`: `"attachments,message.attributedBody,message.messageSummaryInfo,message.payloadData"`.
   - `before` is set to the sync start timestamp; `after` is 0.
   - Batch size matches `numberOfMessagesPerPage` (default 25).
7. Chats with no participants are soft-deleted.
8. If `skipEmptyChats` is true (default), chats with no messages are soft-deleted.
9. Messages are bulk-saved per chat.
10. On completion: contacts are refreshed and the chat list is reloaded.

### Incremental Sync

Incremental sync runs automatically when the socket connects (via `NetworkTasks.onConnect()`) if the app is resuming from a paused/hidden state. It fetches only messages created since the last sync.

**Sync markers**: Two markers determine the sync range:

- `lastIncrementalSync` -- Timestamp (milliseconds since epoch) of the last sync.
- `lastIncrementalSyncRowId` -- The ROWID of the last synced message in the server's iMessage database.

**Server version-dependent behavior**:

| Server Version | Strategy                                                                     |
|----------------|------------------------------------------------------------------------------|
| >= 1.6.0       | Uses `message.ROWID` based `where` clauses in `POST /message/query`. The API returns `metadata.total` for accurate pagination. |
| >= 1.2.0       | Uses `GET /message/count` with `after` timestamp, then pages through `POST /message/query` with `after`/`before` timestamps. |
| < 1.2.0        | Same as 1.2.0 but doubles the count to compensate for a server bug where messages with null text were excluded from counts. |

**Message processing**:

1. Messages are fetched with `withQuery`: `["chats", "chats.participants", "attachments", "attributedBody", "messageSummaryInfo", "payloadData"]`.
2. Chats are extracted from message payloads and bulk-synced.
3. Messages are grouped by chat GUID and bulk-synced per chat.
4. `lastSyncedRowId` and `lastSyncedTimestamp` are tracked and saved on completion.
5. After message sync, `Chat.syncLatestMessages()` updates the latest message for each affected chat.

**Isolate execution**: On Android, incremental sync runs in a `FlutterIsolate` to avoid blocking the UI thread. On web/desktop, it runs in the main thread.

**Contact auto-upload**: If contacts changed during the sync and `syncContactsAutomatically` is enabled, the updated contacts are uploaded to the server via `POST /contact`.

### Real-Time Sync

Between sync intervals, the socket provides real-time updates:

- `new-message` events create new messages in the local database.
- `updated-message` events update existing messages (delivery receipts, read receipts, edits).
- Out-of-order handling: When a `new-message` arrives for a sent message (from-me) without a `tempGuid`, it is delayed 500ms to allow the API response to arrive first, preventing duplicates.

**Incoming message queue**: New and updated messages are processed through `IncomingQueue` (`inq.queue()`) to serialize processing and avoid race conditions.

---

## Source Files Reference

| File Path | Purpose |
|-----------|---------|
| `lib/services/network/http_service.dart` | REST API client with all endpoint methods |
| `lib/services/network/socket_service.dart` | Socket.IO connection and event handling |
| `lib/services/network/downloads_service.dart` | Attachment download queue and controller |
| `lib/services/network/http_overrides.dart` | SSL certificate override for self-signed certs |
| `lib/services/network/firebase/cloud_messaging_service.dart` | FCM device registration (Android) |
| `lib/services/network/firebase/firebase_database_service.dart` | Firebase Database/Firestore URL fetching |
| `lib/services/backend/action_handler.dart` | Socket event dispatch and message send orchestration |
| `lib/helpers/network/network_tasks.dart` | Post-connect tasks (incremental sync, localhost detection) |
| `lib/helpers/network/network_helpers.dart` | URL sanitization, device name generation |
| `lib/helpers/network/network_error_handler.dart` | Send error classification and GUID mutation |
| `lib/helpers/network/metadata_helper.dart` | URL preview metadata fetching |
| `lib/services/backend/sync/sync_service.dart` | Sync orchestration (full and incremental) |
| `lib/services/backend/sync/full_sync_manager.dart` | Full sync implementation |
| `lib/services/backend/sync/incremental_sync_manager.dart` | Incremental sync implementation |
| `lib/services/backend/sync/sync_manager_impl.dart` | Base sync manager with status and progress tracking |
| `lib/database/global/server_payload.dart` | Socket payload parsing and decryption |
| `lib/utils/crypto_utils.dart` | AES-CBC encryption/decryption (OpenSSL-compatible) |
| `lib/database/global/queue_items.dart` | Queue item types for incoming/outgoing messages |
| `lib/helpers/backend/settings_helpers.dart` | Server URL persistence and socket restart |
