#!/usr/bin/env python3
"""
BlueBubbles Rust Rewrite - Implementation Coverage Tracker

Scans the Rust source files and compares against the feature matrix derived
from the Flutter app documentation to determine implementation coverage.

Outputs:
  - Colored terminal summary
  - tools/COVERAGE_REPORT.md
  - tools/coverage_data.json
"""

import json
import os
import re
import sys
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional


# ---------------------------------------------------------------------------
# ANSI helpers
# ---------------------------------------------------------------------------
class C:
    """ANSI color codes for terminal output."""
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    CYAN = "\033[96m"
    BOLD = "\033[1m"
    DIM = "\033[2m"
    RESET = "\033[0m"

    @staticmethod
    def supports_color() -> bool:
        if os.getenv("NO_COLOR"):
            return False
        if sys.platform == "win32":
            return os.getenv("ANSICON") is not None or "WT_SESSION" in os.environ or os.getenv("TERM_PROGRAM") == "vscode"
        return hasattr(sys.stdout, "isatty") and sys.stdout.isatty()


USE_COLOR = C.supports_color()


def c(code: str, text: str) -> str:
    return f"{code}{text}{C.RESET}" if USE_COLOR else text


# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------
@dataclass
class Feature:
    """A single feature from the Flutter app that may be implemented in Rust."""
    category: str
    subcategory: str
    name: str
    status: str = "missing"  # implemented | stubbed | missing
    rust_file: str = ""
    rust_symbol: str = ""
    notes: str = ""


@dataclass
class CrateInfo:
    name: str
    files: list = field(default_factory=list)
    structs: list = field(default_factory=list)
    functions: list = field(default_factory=list)
    enums: list = field(default_factory=list)
    traits: list = field(default_factory=list)
    impls: list = field(default_factory=list)
    test_count: int = 0
    line_count: int = 0


# ---------------------------------------------------------------------------
# Feature matrix - derived from Flutter documentation
# ---------------------------------------------------------------------------

def build_feature_matrix() -> list[Feature]:
    """Build the complete feature matrix from Flutter app documentation."""
    features: list[Feature] = []

    # -----------------------------------------------------------------------
    # REST API Endpoints (from docs/04-api-and-networking.md)
    # -----------------------------------------------------------------------
    api_endpoints = {
        "Server": [
            "GET /api/v1/ping",
            "GET /api/v1/server/info",
            "POST /api/v1/server/restart/soft",
            "POST /api/v1/server/restart/hard",
            "GET /api/v1/server/update/check",
            "POST /api/v1/server/update/install",
            "GET /api/v1/server/statistics/totals",
            "GET /api/v1/server/statistics/media",
            "GET /api/v1/server/logs",
        ],
        "Chat": [
            "GET /api/v1/chat/count",
            "POST /api/v1/chat/query",
            "GET /api/v1/chat/:guid",
            "PUT /api/v1/chat/:guid",
            "DELETE /api/v1/chat/:guid",
            "POST /api/v1/chat/new",
            "POST /api/v1/chat/:guid/read",
            "POST /api/v1/chat/:guid/unread",
            "POST /api/v1/chat/:guid/leave",
            "POST /api/v1/chat/:guid/participant/add",
            "POST /api/v1/chat/:guid/participant/remove",
            "GET /api/v1/chat/:guid/message",
            "GET /api/v1/chat/:guid/icon",
            "DELETE /api/v1/chat/:guid/:messageGuid",
        ],
        "Message": [
            "GET /api/v1/message/count",
            "POST /api/v1/message/query",
            "GET /api/v1/message/:guid",
            "GET /api/v1/message/:guid/embedded-media",
            "POST /api/v1/message/text",
            "POST /api/v1/message/multipart",
            "POST /api/v1/message/react",
            "POST /api/v1/message/:guid/unsend",
            "POST /api/v1/message/:guid/edit",
            "POST /api/v1/message/:guid/notify",
            "GET /api/v1/message/schedule",
            "POST /api/v1/message/schedule",
            "PUT /api/v1/message/schedule/:id",
            "DELETE /api/v1/message/schedule/:id",
        ],
        "Attachment": [
            "GET /api/v1/attachment/count",
            "GET /api/v1/attachment/:guid",
            "GET /api/v1/attachment/:guid/download",
            "GET /api/v1/attachment/:guid/live",
            "GET /api/v1/attachment/:guid/blurhash",
            "POST /api/v1/attachment/upload (multipart)",
        ],
        "Handle": [
            "GET /api/v1/handle/count",
            "POST /api/v1/handle/query",
            "GET /api/v1/handle/:guid",
            "GET /api/v1/handle/:address/focus",
            "POST /api/v1/handle/availability/imessage",
            "POST /api/v1/handle/availability/facetime",
        ],
        "Contact": [
            "GET /api/v1/contact",
            "POST /api/v1/contact/query",
            "POST /api/v1/contact/upload",
        ],
        "FCM": [
            "POST /api/v1/fcm/device",
            "GET /api/v1/fcm/client",
        ],
        "Mac": [
            "POST /api/v1/mac/lock",
            "POST /api/v1/mac/imessage/restart",
        ],
        "Backup": [
            "GET /api/v1/backup/theme",
            "POST /api/v1/backup/theme",
            "DELETE /api/v1/backup/theme",
            "GET /api/v1/backup/settings",
            "POST /api/v1/backup/settings",
            "DELETE /api/v1/backup/settings",
        ],
        "FaceTime": [
            "POST /api/v1/facetime/answer",
            "POST /api/v1/facetime/leave",
        ],
        "iCloud": [
            "GET /api/v1/icloud/findmy/devices",
            "POST /api/v1/icloud/findmy/devices/refresh",
            "GET /api/v1/icloud/findmy/friends",
            "POST /api/v1/icloud/findmy/friends/refresh",
            "GET /api/v1/icloud/account",
            "GET /api/v1/icloud/contact",
            "POST /api/v1/icloud/account/alias",
        ],
    }

    for group, endpoints in api_endpoints.items():
        for ep in endpoints:
            features.append(Feature(
                category="API Endpoints",
                subcategory=group,
                name=ep,
            ))

    # -----------------------------------------------------------------------
    # Socket.IO Events (from docs/04-api-and-networking.md)
    # -----------------------------------------------------------------------
    socket_events = [
        "new-message",
        "updated-message",
        "typing-indicator",
        "chat-read-status-changed",
        "group-name-change",
        "participant-removed",
        "participant-added",
        "participant-left",
        "group-icon-changed",
        "incoming-facetime",
        "ft-call-status-changed",
        "imessage-aliases-removed",
        "server-update",
    ]
    for evt in socket_events:
        features.append(Feature(
            category="Socket Events",
            subcategory="Events",
            name=evt,
        ))

    # -----------------------------------------------------------------------
    # Database Models (from docs/03-database-and-models.md)
    # -----------------------------------------------------------------------
    db_models = {
        "Chat": ["id", "guid", "chat_identifier", "display_name", "is_archived",
                  "mute_type", "is_pinned", "has_unread_message", "style",
                  "latest_message_date", "participants", "from_server_map", "save"],
        "Message": ["id", "guid", "chat_id", "handle_id", "text", "subject",
                     "error", "date_created", "is_from_me", "item_type",
                     "associated_message_guid", "has_attachments", "attributed_body",
                     "payload_data", "from_server_map", "save"],
        "Handle": ["id", "address", "service", "formatted_address", "contact",
                    "from_server_map", "save", "display_name"],
        "Attachment": ["id", "guid", "mime_type", "transfer_name", "total_bytes",
                       "height", "width", "has_live_photo", "from_server_map", "save",
                       "is_image", "is_video", "is_audio"],
        "Contact": ["id", "external_id", "display_name", "phones", "emails",
                     "avatar", "structured_name", "from_server_map", "save",
                     "matches_address"],
        "FcmData": ["project_id", "storage_bucket", "api_key", "firebase_url",
                     "client_id", "application_id", "from_server_map", "save", "load"],
        "ThemeStruct": ["id", "name", "gradient_bg", "google_font", "theme_data",
                        "parsed_data", "is_dark", "save", "load_all", "delete_by_name"],
        "ScheduledMessage": ["id", "type", "chat_guid", "message", "scheduled_for",
                              "status", "from_server_map", "save", "load_all", "delete"],
        "AttributedBody": ["runs", "plain_text", "mentions", "links", "from_server_json"],
        "PayloadData": ["url_preview", "app_data", "from_server_json"],
    }

    for model, fields_methods in db_models.items():
        for fm in fields_methods:
            features.append(Feature(
                category="DB Models",
                subcategory=model,
                name=f"{model}.{fm}",
            ))

    # -----------------------------------------------------------------------
    # Database Infrastructure
    # -----------------------------------------------------------------------
    db_infra = [
        "Connection pooling (r2d2)",
        "WAL mode",
        "Integrity check",
        "Schema creation",
        "Versioned migrations",
        "Transaction support",
        "Database reset",
        "Database stats",
        "Settings key-value store",
        "Chat-Handle join table",
    ]
    for item in db_infra:
        features.append(Feature(
            category="DB Infrastructure",
            subcategory="Core",
            name=item,
        ))

    # -----------------------------------------------------------------------
    # Services (from docs/02-services-and-business-logic.md)
    # -----------------------------------------------------------------------
    services = {
        "ChatService": ["list_chats", "find_chat", "search_chats", "mark_read",
                         "mark_unread", "create_chat", "rename_chat", "count",
                         "toggle_pin", "toggle_archive"],
        "MessageService": ["list_messages", "find_message", "search_messages",
                           "send_text", "send_reaction", "edit_message",
                           "unsend_message", "handle_incoming_message", "count_for_chat"],
        "ContactService": ["list_contacts", "search_contacts", "find_contact",
                           "sync_contacts", "resolve_display_name"],
        "AttachmentService": ["download", "upload_and_send", "attachments_for_message",
                               "cleanup_cache", "cache_path", "is_cached"],
        "SyncService": ["full_sync", "incremental_sync"],
        "SettingsService": ["server_address", "set_server_address", "guid_auth_key",
                            "set_guid_auth_key", "is_setup_complete", "mark_setup_complete",
                            "save"],
        "NotificationService": ["notify_message", "notify_reaction", "notify",
                                 "set_enabled"],
        "QueueService": ["enqueue", "dequeue", "len", "is_empty", "remove", "clear"],
        "ServiceRegistry": ["register", "init_all", "shutdown_all", "set_api_client",
                            "api_client", "health_check"],
    }

    # Flutter services not yet in Rust
    flutter_only_services = [
        "LifecycleService",
        "ActionHandler",
        "BackgroundIsolateService",
        "NotificationListenerService",
        "FCMService",
        "UnifiedPushService",
        "NetworkService",
        "ThemeService",
        "FindMyService",
        "FaceTimeService",
        "SocketService (high-level wrapper)",
        "MethodChannelService",
        "IntentService",
        "PrivateApiService",
    ]

    for svc_name, methods in services.items():
        for method in methods:
            features.append(Feature(
                category="Services",
                subcategory=svc_name,
                name=f"{svc_name}.{method}",
            ))

    for svc in flutter_only_services:
        features.append(Feature(
            category="Services",
            subcategory="Flutter-only",
            name=svc,
        ))

    # -----------------------------------------------------------------------
    # CLI Commands (from bb-cli)
    # -----------------------------------------------------------------------
    cli_commands = {
        "connect": ["connect"],
        "status": ["status"],
        "chats": ["list", "get", "search", "read", "unread"],
        "messages": ["list", "get", "search", "send", "react", "edit", "unsend"],
        "contacts": ["list", "search", "sync"],
        "attachments": ["info", "download"],
        "sync": ["full", "incremental"],
        "settings": ["show", "set-address", "set-password", "export"],
        "server": ["ping", "info", "stats", "restart", "restart-hard", "check-update", "logs"],
        "db": ["stats", "check", "reset", "path"],
        "logs": ["view"],
    }
    for group, cmds in cli_commands.items():
        for cmd in cmds:
            features.append(Feature(
                category="CLI Commands",
                subcategory=group,
                name=f"bb {group} {cmd}",
            ))

    # -----------------------------------------------------------------------
    # Core Infrastructure
    # -----------------------------------------------------------------------
    core_features = [
        "AppConfig (TOML)",
        "ServerConfig",
        "DatabaseConfig",
        "LoggingConfig",
        "SyncConfig",
        "NotificationConfig",
        "DisplayConfig",
        "ConfigHandle (thread-safe)",
        "BbError unified error type",
        "MessageError codes",
        "init_logging (tracing + file rotation)",
        "init_console_logging",
        "Platform detection (Win/Mac/Linux)",
        "Platform data_dir / config_dir / cache_dir",
        "Constants (reactions, effects, balloon bundles)",
        "AES-256-CBC crypto (CryptoJS compatible)",
        "SocketEventType enum",
        "EventDispatcher (broadcast channels)",
        "ConnectionState",
        "SocketManager",
        "Reconnect with exponential backoff + jitter",
        "Event deduplication",
        "Service trait",
        "ServiceState lifecycle",
    ]
    for feat in core_features:
        features.append(Feature(
            category="Core Infrastructure",
            subcategory="Core",
            name=feat,
        ))

    # -----------------------------------------------------------------------
    # Settings (subset - from docs/03-database-and-models.md, 120+ keys)
    # -----------------------------------------------------------------------
    settings_categories = {
        "Connection": [
            "serverAddress", "guidAuthKey", "customHeaders", "apiTimeout",
            "acceptSelfSignedCerts",
        ],
        "Sync": [
            "finishedSetup", "lastIncrementalSync", "lastIncrementalSyncRowId",
            "messagesPerPage", "skipEmptyChats", "syncContactsAutomatically",
        ],
        "Notifications": [
            "notifyReactions", "notifyOnChatList", "filterUnknownSenders",
            "globalTextDetection",
        ],
        "Display": [
            "userName", "use24HrFormat", "redactedMode",
        ],
        "Theme": [
            "selectedTheme", "skin", "colorfulAvatars", "colorfulBubbles",
            "monetTheming",
        ],
        "Privacy": [
            "incognitoMode", "hideMessagePreview", "generateFakeContactNames",
            "generateFakeMessageContent",
        ],
        "Conversation": [
            "autoOpenKeyboard", "swipeToReply", "swipeToArchive",
            "moveChatCreatorToHeader", "doubleTapForDetails",
            "autoPlayGifs", "showDeliveryTimestamps",
        ],
    }
    for cat, keys in settings_categories.items():
        for key in keys:
            features.append(Feature(
                category="Settings",
                subcategory=cat,
                name=key,
            ))

    # -----------------------------------------------------------------------
    # UI Screens (from docs/05-ui-layouts-and-components.md) - NOT in Rust yet
    # -----------------------------------------------------------------------
    ui_screens = [
        "SetupView (initial setup wizard)",
        "ConversationList",
        "ConversationView (message thread)",
        "MessageWidget",
        "SearchView (global search)",
        "CreateChat",
        "ConversationDetails",
        "AttachmentFullscreenViewer",
        "CameraWidget",
        "SettingsPanel (main)",
        "SettingsPanel > Connection & Server",
        "SettingsPanel > Message & Notification",
        "SettingsPanel > Theme & Style",
        "SettingsPanel > Desktop & Tray",
        "SettingsPanel > Chat List",
        "SettingsPanel > Conversation",
        "SettingsPanel > Misc & Advanced",
        "SettingsPanel > About & Links",
        "SettingsPanel > Backup & Restore",
        "SettingsPanel > FindMy",
        "SettingsPanel > Troubleshooting",
        "ThemePicker",
        "FontPicker",
        "FindMyDevicesList",
        "FindMyFriendsList",
        "ScheduledMessagesView",
    ]
    for screen in ui_screens:
        features.append(Feature(
            category="UI Screens",
            subcategory="Tauri UI",
            name=screen,
        ))

    return features


# ---------------------------------------------------------------------------
# Source scanner
# ---------------------------------------------------------------------------

def scan_rust_source(root: Path) -> dict[str, CrateInfo]:
    """Scan all .rs files under root and extract symbols per crate."""
    crates: dict[str, CrateInfo] = {}

    for rs_file in root.rglob("*.rs"):
        # Skip target directory
        parts = rs_file.parts
        if "target" in parts:
            continue

        # Determine crate name from path
        rel = rs_file.relative_to(root)
        crate_name = str(rel.parts[0]) if rel.parts else "unknown"

        if crate_name not in crates:
            crates[crate_name] = CrateInfo(name=crate_name)

        info = crates[crate_name]
        info.files.append(str(rel))

        try:
            content = rs_file.read_text(encoding="utf-8", errors="replace")
        except Exception:
            continue

        info.line_count += content.count("\n") + 1

        # Count tests
        info.test_count += len(re.findall(r"#\[(?:tokio::)?test\]", content))

        # Extract struct definitions
        for m in re.finditer(r"pub\s+struct\s+(\w+)", content):
            info.structs.append(m.group(1))

        # Extract enum definitions
        for m in re.finditer(r"pub\s+enum\s+(\w+)", content):
            info.enums.append(m.group(1))

        # Extract trait definitions
        for m in re.finditer(r"pub\s+trait\s+(\w+)", content):
            info.traits.append(m.group(1))

        # Extract function/method definitions
        for m in re.finditer(r"pub\s+(?:async\s+)?fn\s+(\w+)", content):
            info.functions.append(m.group(1))

        # Extract impl blocks
        for m in re.finditer(r"impl(?:\s+\w+\s+for)?\s+(\w+)", content):
            info.impls.append(m.group(1))

    return crates


# ---------------------------------------------------------------------------
# Matching engine
# ---------------------------------------------------------------------------

def match_features(features: list[Feature], crates: dict[str, CrateInfo]) -> None:
    """Match features against scanned Rust source symbols."""

    # Build lookup indices
    all_functions: set[str] = set()
    all_structs: set[str] = set()
    all_enums: set[str] = set()
    all_content: dict[str, str] = {}  # crate -> joined file content

    for crate_name, info in crates.items():
        all_functions.update(info.functions)
        all_structs.update(info.structs)
        all_enums.update(info.enums)

    # Read all file content for deeper matching
    root = Path(__file__).resolve().parent.parent
    for crate_name, info in crates.items():
        parts = []
        for fpath in info.files:
            try:
                parts.append((root / fpath).read_text(encoding="utf-8", errors="replace"))
            except Exception:
                pass
        all_content[crate_name] = "\n".join(parts)

    joined_content = "\n".join(all_content.values())

    for feat in features:
        _match_single_feature(feat, crates, all_functions, all_structs,
                               all_enums, all_content, joined_content)


def _match_single_feature(
    feat: Feature,
    crates: dict[str, CrateInfo],
    all_functions: set[str],
    all_structs: set[str],
    all_enums: set[str],
    all_content: dict[str, str],
    joined_content: str,
) -> None:
    """Determine implementation status for a single feature."""

    # -- API Endpoints --
    if feat.category == "API Endpoints":
        _match_api_endpoint(feat, crates, all_content)
        return

    # -- Socket Events --
    if feat.category == "Socket Events":
        event_name = feat.name
        # Check if event string literal exists in socket crate
        if f'"{event_name}"' in all_content.get("bb-socket", ""):
            feat.status = "implemented"
            feat.rust_file = "bb-socket/src/events.rs"
            feat.rust_symbol = f'SocketEventType::{_event_to_variant(event_name)}'
        elif event_name in ("chat-read-status-changed", "ft-call-status-changed"):
            feat.status = "missing"
            feat.notes = "Not in event enum yet"
        return

    # -- DB Models --
    if feat.category == "DB Models":
        _match_db_model(feat, crates, all_functions, all_structs, all_content)
        return

    # -- DB Infrastructure --
    if feat.category == "DB Infrastructure":
        _match_db_infra(feat, crates, all_content)
        return

    # -- Services --
    if feat.category == "Services":
        _match_service(feat, crates, all_functions, all_structs, all_content)
        return

    # -- CLI Commands --
    if feat.category == "CLI Commands":
        _match_cli_command(feat, crates, all_content)
        return

    # -- Core Infrastructure --
    if feat.category == "Core Infrastructure":
        _match_core(feat, crates, all_functions, all_structs, all_enums, all_content)
        return

    # -- Settings --
    if feat.category == "Settings":
        _match_setting(feat, all_content)
        return

    # -- UI Screens --
    if feat.category == "UI Screens":
        # No Tauri UI exists yet
        feat.status = "missing"
        feat.notes = "Tauri UI not yet started"
        return


def _match_api_endpoint(feat: Feature, crates: dict[str, CrateInfo],
                        all_content: dict[str, str]) -> None:
    """Match an API endpoint feature."""
    ep = feat.name
    api_content = all_content.get("bb-api", "")

    # Extract path from endpoint string, e.g. "GET /api/v1/ping" -> "/ping"
    parts = ep.split(" ", 1)
    if len(parts) < 2:
        return
    method = parts[0]
    path = parts[1]

    # Strip /api/v1 prefix to get the route
    route = path.replace("/api/v1", "")

    # Normalize route for matching (replace params like :guid)
    route_pattern = re.sub(r":(\w+)", r"\\{[^}]+\\}", route)

    # Search for the route string in the API content
    # The Rust code uses format strings like "/chat/{guid}/read"
    route_rust = re.sub(r":(\w+)", lambda m: "{" + _param_to_rust(m.group(1)) + "}", route)
    # Also check without braces (quoted string segments)
    route_segments = route.split("/")
    significant = [s for s in route_segments if s and not s.startswith(":")]

    if route_rust in api_content or f'"{route}"' in api_content:
        feat.status = "implemented"
        feat.rust_file = f"bb-api/src/endpoints/{feat.subcategory.lower()}.rs"
    elif len(significant) >= 2 and all(s in api_content for s in significant[-2:]):
        # Check for the last 2 significant route segments
        feat.status = "implemented"
        feat.rust_file = f"bb-api/src/endpoints/{feat.subcategory.lower()}.rs"
    else:
        # More lenient: check for any matching function
        route_keywords = {
            "ping": "ping",
            "server/info": "server_info",
            "restart/soft": "restart_soft",
            "restart/hard": "restart_hard",
            "update/check": "check_update",
            "update/install": "install_update",
            "statistics/totals": "totals",
            "statistics/media": "media_totals",
            "server/logs": "logs",
            "chat/count": "chat_count",
            "chat/query": "query_chats",
            "chat/new": "create_chat",
            "message/count": "message_count",
            "message/query": "query_messages",
            "message/text": "send_text",
            "message/multipart": "send_multipart",
            "message/react": "send_reaction",
            "message/schedule": "scheduled_message",
            "attachment/count": "attachment_count",
            "attachment/upload": "send_attachment",
            "handle/count": "handle_count",
            "handle/query": "query_handles",
            "handle/availability/imessage": "check_imessage",
            "handle/availability/facetime": "check_facetime",
            "contact": "contacts",
            "contact/query": "query_contacts",
            "contact/upload": "upload_contacts",
            "fcm/device": "register_fcm",
            "fcm/client": "get_fcm",
            "mac/lock": "lock_mac",
            "mac/imessage/restart": "restart_imessage",
            "backup/theme": "theme_backup",
            "backup/settings": "settings_backup",
            "facetime/answer": "answer_facetime",
            "facetime/leave": "leave_facetime",
            "icloud/findmy/devices": "findmy_devices",
            "icloud/findmy/friends": "findmy_friends",
            "icloud/account": "icloud_account",
            "icloud/contact": "icloud_contact",
            "icloud/account/alias": "icloud_alias",
        }
        matched = False
        for route_key, fn_key in route_keywords.items():
            if route_key in route and fn_key in api_content:
                feat.status = "implemented"
                feat.rust_file = f"bb-api/src/endpoints/{feat.subcategory.lower()}.rs"
                matched = True
                break
        if not matched:
            # Check specific patterns
            if "/read" in route and "mark_chat_read" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif "/unread" in route and "mark_chat_unread" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif "/leave" in route and "leave_chat" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif "/participant/add" in route and "add_participant" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif "/participant/remove" in route and "remove_participant" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif "/icon" in route and "chat_icon" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif "embedded-media" in route and "embedded_media" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/messages.rs"
            elif "/unsend" in route and "unsend_message" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/messages.rs"
            elif "/edit" in route and "edit_message" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/messages.rs"
            elif "/notify" in route and "notify_message" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/messages.rs"
            elif "/live" in route and "live_photo" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/attachments.rs"
            elif "/blurhash" in route and "blurhash" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/attachments.rs"
            elif "/download" in route and "download_attachment" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/attachments.rs"
            elif "/focus" in route and "handle_focus" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/handles.rs"
            elif "findmy/devices/refresh" in route and "refresh_findmy_devices" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/icloud.rs"
            elif "findmy/friends/refresh" in route and "refresh_findmy_friends" in api_content:
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/icloud.rs"
            elif ":guid" in route and method == "GET" and "get_chat" in api_content and feat.subcategory == "Chat":
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif ":guid" in route and method == "PUT" and "update_chat" in api_content and feat.subcategory == "Chat":
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif ":guid" in route and method == "DELETE" and "delete_chat" in api_content and feat.subcategory == "Chat":
                feat.status = "implemented"
                feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif "/message" in route and ":guid" in route and method == "GET" and feat.subcategory == "Chat":
                if "get_chat_messages" in api_content:
                    feat.status = "implemented"
                    feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif ":messageGuid" in route and method == "DELETE" and feat.subcategory == "Chat":
                if "delete_chat_message" in api_content:
                    feat.status = "implemented"
                    feat.rust_file = "bb-api/src/endpoints/chats.rs"
            elif ":guid" in route and method == "GET" and feat.subcategory == "Message":
                if "get_message" in api_content:
                    feat.status = "implemented"
                    feat.rust_file = "bb-api/src/endpoints/messages.rs"
            elif ":guid" in route and method == "GET" and feat.subcategory == "Attachment":
                if "get_attachment" in api_content:
                    feat.status = "implemented"
                    feat.rust_file = "bb-api/src/endpoints/attachments.rs"
            elif ":guid" in route and method == "GET" and feat.subcategory == "Handle":
                if "get_handle" in api_content:
                    feat.status = "implemented"
                    feat.rust_file = "bb-api/src/endpoints/handles.rs"


def _match_db_model(feat: Feature, crates: dict[str, CrateInfo],
                    all_functions: set[str], all_structs: set[str],
                    all_content: dict[str, str]) -> None:
    """Match a DB model feature."""
    model_name = feat.subcategory
    field_or_method = feat.name.split(".", 1)[1] if "." in feat.name else feat.name
    models_content = all_content.get("bb-models", "")

    # Check if the struct exists
    if model_name in all_structs or model_name in models_content:
        # Check if the specific field or method exists
        if field_or_method in models_content:
            feat.status = "implemented"
            model_file = model_name.lower()
            # Handle special names
            file_map = {
                "ThemeStruct": "theme",
                "ScheduledMessage": "scheduled_message",
                "AttributedBody": "attributed_body",
                "PayloadData": "payload_data",
                "FcmData": "fcm_data",
            }
            fname = file_map.get(model_name, model_file)
            feat.rust_file = f"bb-models/src/models/{fname}.rs"
            feat.rust_symbol = f"{model_name}::{field_or_method}"
        else:
            feat.status = "stubbed"
            feat.notes = f"Struct exists but {field_or_method} not found"
    else:
        feat.status = "missing"


def _match_db_infra(feat: Feature, crates: dict[str, CrateInfo],
                    all_content: dict[str, str]) -> None:
    """Match database infrastructure features."""
    models_content = all_content.get("bb-models", "")
    name = feat.name.lower()

    checks = {
        "connection pooling": "r2d2" in models_content or "Pool" in models_content,
        "wal mode": "WAL" in models_content or "wal_mode" in models_content,
        "integrity check": "integrity_check" in models_content,
        "schema creation": "create_tables" in models_content,
        "versioned migrations": "run_migrations" in models_content,
        "transaction support": "transaction" in models_content,
        "database reset": "reset" in models_content and "drop_tables" in models_content,
        "database stats": "DatabaseStats" in models_content or "stats" in models_content,
        "settings key-value store": "settings" in models_content and "key" in models_content,
        "chat-handle join table": "chat_handle_join" in models_content,
    }

    for check_name, result in checks.items():
        if check_name in name:
            feat.status = "implemented" if result else "missing"
            feat.rust_file = "bb-models/src/db.rs"
            return

    # Fallback
    if name in models_content.lower():
        feat.status = "implemented"
    else:
        feat.status = "missing"


def _match_service(feat: Feature, crates: dict[str, CrateInfo],
                   all_functions: set[str], all_structs: set[str],
                   all_content: dict[str, str]) -> None:
    """Match service features."""
    svc_content = all_content.get("bb-services", "")

    if feat.subcategory == "Flutter-only":
        feat.status = "missing"
        feat.notes = "Flutter-specific service, not ported"
        return

    svc_name = feat.subcategory
    method_name = feat.name.split(".", 1)[1] if "." in feat.name else feat.name

    if svc_name in svc_content and method_name in svc_content:
        feat.status = "implemented"
        file_map = {
            "ChatService": "chat",
            "MessageService": "message",
            "ContactService": "contact",
            "AttachmentService": "attachment",
            "SyncService": "sync",
            "SettingsService": "settings",
            "NotificationService": "notification",
            "QueueService": "queue",
            "ServiceRegistry": "registry",
        }
        fname = file_map.get(svc_name, svc_name.lower())
        feat.rust_file = f"bb-services/src/{fname}.rs"
        feat.rust_symbol = f"{svc_name}::{method_name}"
    elif svc_name in svc_content:
        feat.status = "stubbed"
        feat.notes = f"Service exists but {method_name} not found"
    else:
        feat.status = "missing"


def _match_cli_command(feat: Feature, crates: dict[str, CrateInfo],
                       all_content: dict[str, str]) -> None:
    """Match CLI command features."""
    cli_content = all_content.get("bb-cli", "")
    parts = feat.name.split(" ")
    if len(parts) >= 3:
        group = parts[1]
        cmd = parts[2]
        # Check for the command group module and the specific subcommand
        if group in cli_content and (cmd.replace("-", "_") in cli_content or
                                      cmd.replace("-", "").capitalize() in cli_content or
                                      cmd.title() in cli_content):
            feat.status = "implemented"
            feat.rust_file = f"bb-cli/src/commands/{group}.rs"
        elif group in cli_content:
            feat.status = "stubbed"
            feat.notes = f"Module exists, checking for {cmd}"
        else:
            feat.status = "missing"
    elif len(parts) >= 2:
        group = parts[1]
        if group in cli_content:
            feat.status = "implemented"
            feat.rust_file = f"bb-cli/src/commands/{group}.rs"


def _match_core(feat: Feature, crates: dict[str, CrateInfo],
                all_functions: set[str], all_structs: set[str],
                all_enums: set[str], all_content: dict[str, str]) -> None:
    """Match core infrastructure features."""
    core_content = all_content.get("bb-core", "")
    socket_content = all_content.get("bb-socket", "")
    services_content = all_content.get("bb-services", "")
    combined = core_content + socket_content + services_content

    name = feat.name

    # Direct struct/enum/function matching
    keyword_map = {
        "AppConfig": ("AppConfig", core_content, "bb-core/src/config.rs"),
        "ServerConfig": ("ServerConfig", core_content, "bb-core/src/config.rs"),
        "DatabaseConfig": ("DatabaseConfig", core_content, "bb-core/src/config.rs"),
        "LoggingConfig": ("LoggingConfig", core_content, "bb-core/src/config.rs"),
        "SyncConfig": ("SyncConfig", core_content, "bb-core/src/config.rs"),
        "NotificationConfig": ("NotificationConfig", core_content, "bb-core/src/config.rs"),
        "DisplayConfig": ("DisplayConfig", core_content, "bb-core/src/config.rs"),
        "ConfigHandle": ("ConfigHandle", core_content, "bb-core/src/config.rs"),
        "BbError": ("BbError", core_content, "bb-core/src/error.rs"),
        "MessageError": ("MessageError", core_content, "bb-core/src/error.rs"),
        "init_logging": ("init_logging", core_content, "bb-core/src/logging.rs"),
        "init_console_logging": ("init_console_logging", core_content, "bb-core/src/logging.rs"),
        "Platform detection": ("Platform", core_content, "bb-core/src/platform.rs"),
        "Platform data_dir": ("data_dir", core_content, "bb-core/src/platform.rs"),
        "Constants": ("reactions", core_content, "bb-core/src/constants.rs"),
        "AES-256-CBC": ("AesCrypto", socket_content, "bb-socket/src/crypto.rs"),
        "SocketEventType": ("SocketEventType", socket_content, "bb-socket/src/events.rs"),
        "EventDispatcher": ("EventDispatcher", socket_content, "bb-socket/src/events.rs"),
        "ConnectionState": ("ConnectionState", socket_content, "bb-socket/src/events.rs"),
        "SocketManager": ("SocketManager", socket_content, "bb-socket/src/manager.rs"),
        "Reconnect with exponential backoff": ("reconnect_delay", socket_content, "bb-socket/src/manager.rs"),
        "Event deduplication": ("handled_guids", socket_content, "bb-socket/src/manager.rs"),
        "Service trait": ("trait Service", services_content, "bb-services/src/service.rs"),
        "ServiceState": ("ServiceState", services_content, "bb-services/src/service.rs"),
    }

    for key_name, (search_term, content, file) in keyword_map.items():
        if key_name in name:
            if search_term in content:
                feat.status = "implemented"
                feat.rust_file = file
                feat.rust_symbol = search_term
            else:
                feat.status = "missing"
            return

    # Fallback
    feat.status = "missing"


def _match_setting(feat: Feature, all_content: dict[str, str]) -> None:
    """Match settings features."""
    core_content = all_content.get("bb-core", "")
    name = feat.name

    # Convert camelCase to snake_case for Rust matching
    snake = re.sub(r"(?<=[a-z])([A-Z])", r"_\1", name).lower()

    if snake in core_content or name in core_content:
        feat.status = "implemented"
        feat.rust_file = "bb-core/src/config.rs"
    else:
        feat.status = "missing"
        feat.notes = "Not in TOML config yet"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _event_to_variant(event_name: str) -> str:
    """Convert socket event name to Rust enum variant name."""
    mapping = {
        "new-message": "NewMessage",
        "updated-message": "UpdatedMessage",
        "typing-indicator": "TypingIndicator",
        "group-name-change": "ChatUpdate",
        "chat-read-status-changed": "ChatReadStatusChanged",
        "participant-removed": "ParticipantRemoved",
        "participant-added": "ParticipantAdded",
        "participant-left": "ParticipantLeft",
        "group-icon-changed": "GroupIconChanged",
        "incoming-facetime": "IncomingFaceTime",
        "ft-call-status-changed": "FtCallStatusChanged",
        "imessage-aliases-removed": "IMessageAliasRemoved",
        "server-update": "ServerUpdate",
    }
    return mapping.get(event_name, event_name)


def _param_to_rust(param: str) -> str:
    """Convert a route param like :guid to a Rust variable name."""
    return param


# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def generate_terminal_report(features: list[Feature], crates: dict[str, CrateInfo]) -> None:
    """Print colored terminal report."""
    total = len(features)
    implemented = sum(1 for f in features if f.status == "implemented")
    stubbed = sum(1 for f in features if f.status == "stubbed")
    missing = sum(1 for f in features if f.status == "missing")
    pct = (implemented / total * 100) if total else 0

    print()
    print(c(C.BOLD, "=" * 70))
    print(c(C.BOLD + C.CYAN, "  BlueBubbles Rust Rewrite - Implementation Coverage Report"))
    print(c(C.BOLD, "=" * 70))
    print()

    # Overall progress bar
    bar_width = 50
    filled = int(bar_width * pct / 100)
    bar = "[" + "#" * filled + "-" * (bar_width - filled) + "]"
    color = C.GREEN if pct >= 70 else C.YELLOW if pct >= 40 else C.RED
    print(f"  Overall: {c(color, bar)} {c(C.BOLD, f'{pct:.1f}%')}")
    print(f"  {c(C.GREEN, f'{implemented} implemented')}  |  "
          f"{c(C.YELLOW, f'{stubbed} stubbed')}  |  "
          f"{c(C.RED, f'{missing} missing')}  |  "
          f"{c(C.DIM, f'{total} total')}")
    print()

    # Per-category breakdown
    categories = {}
    for f in features:
        categories.setdefault(f.category, []).append(f)

    print(c(C.BOLD, "  Per-Category Breakdown:"))
    print(c(C.DIM, "  " + "-" * 66))
    for cat_name, cat_features in categories.items():
        cat_total = len(cat_features)
        cat_impl = sum(1 for f in cat_features if f.status == "implemented")
        cat_pct = (cat_impl / cat_total * 100) if cat_total else 0
        color = C.GREEN if cat_pct >= 70 else C.YELLOW if cat_pct >= 40 else C.RED
        bar_w = 30
        filled = int(bar_w * cat_pct / 100)
        bar = "[" + "#" * filled + "-" * (bar_w - filled) + "]"
        print(f"  {cat_name:<24} {c(color, bar)} {cat_impl:>3}/{cat_total:<3} ({cat_pct:.0f}%)")
    print()

    # Per-crate breakdown
    print(c(C.BOLD, "  Per-Crate Source Stats:"))
    print(c(C.DIM, "  " + "-" * 66))
    for crate_name in sorted(crates.keys()):
        info = crates[crate_name]
        print(f"  {c(C.CYAN, crate_name):<30} "
              f"{len(info.files):>3} files  "
              f"{info.line_count:>6} lines  "
              f"{len(info.structs):>3} structs  "
              f"{len(info.functions):>3} fns  "
              f"{info.test_count:>3} tests")
    print()

    # Missing features summary (top items)
    missing_feats = [f for f in features if f.status == "missing"]
    if missing_feats:
        print(c(C.BOLD, "  Top Missing Features:"))
        print(c(C.DIM, "  " + "-" * 66))
        # Group by category and show counts
        missing_by_cat = {}
        for f in missing_feats:
            missing_by_cat.setdefault(f.category, []).append(f)
        for cat, items in sorted(missing_by_cat.items(), key=lambda x: -len(x[1])):
            print(f"  {c(C.RED, cat)}: {len(items)} missing")
            for item in items[:5]:
                print(f"    - {item.name}")
            if len(items) > 5:
                print(f"    ... and {len(items) - 5} more")
    print()
    print(c(C.DIM, f"  Generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}"))
    print()


def generate_markdown_report(features: list[Feature], crates: dict[str, CrateInfo],
                              output_path: Path) -> None:
    """Generate COVERAGE_REPORT.md."""
    total = len(features)
    implemented = sum(1 for f in features if f.status == "implemented")
    stubbed = sum(1 for f in features if f.status == "stubbed")
    missing = sum(1 for f in features if f.status == "missing")
    pct = (implemented / total * 100) if total else 0

    lines = [
        "# BlueBubbles Rust Rewrite - Coverage Report",
        "",
        f"*Generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}*",
        "",
        "## Overall Progress",
        "",
        f"| Status | Count | Percentage |",
        f"|--------|-------|------------|",
        f"| Implemented | {implemented} | {pct:.1f}% |",
        f"| Stubbed | {stubbed} | {(stubbed/total*100) if total else 0:.1f}% |",
        f"| Missing | {missing} | {(missing/total*100) if total else 0:.1f}% |",
        f"| **Total** | **{total}** | **100%** |",
        "",
        "---",
        "",
        "## Per-Category Breakdown",
        "",
        "| Category | Implemented | Stubbed | Missing | Total | Coverage |",
        "|----------|-------------|---------|---------|-------|----------|",
    ]

    categories = {}
    for f in features:
        categories.setdefault(f.category, []).append(f)

    for cat_name, cat_features in categories.items():
        cat_total = len(cat_features)
        cat_impl = sum(1 for f in cat_features if f.status == "implemented")
        cat_stub = sum(1 for f in cat_features if f.status == "stubbed")
        cat_miss = sum(1 for f in cat_features if f.status == "missing")
        cat_pct = (cat_impl / cat_total * 100) if cat_total else 0
        lines.append(
            f"| {cat_name} | {cat_impl} | {cat_stub} | {cat_miss} | {cat_total} | {cat_pct:.0f}% |"
        )

    lines.extend(["", "---", ""])

    # Per-crate stats
    lines.extend([
        "## Per-Crate Source Statistics",
        "",
        "| Crate | Files | Lines | Structs | Functions | Tests |",
        "|-------|-------|-------|---------|-----------|-------|",
    ])
    total_lines = 0
    total_tests = 0
    for crate_name in sorted(crates.keys()):
        info = crates[crate_name]
        total_lines += info.line_count
        total_tests += info.test_count
        lines.append(
            f"| {crate_name} | {len(info.files)} | {info.line_count} | "
            f"{len(info.structs)} | {len(info.functions)} | {info.test_count} |"
        )
    lines.append(
        f"| **Total** | | **{total_lines}** | | | **{total_tests}** |"
    )

    lines.extend(["", "---", ""])

    # Detailed feature tables per category
    for cat_name, cat_features in categories.items():
        lines.append(f"## {cat_name}")
        lines.append("")

        # Group by subcategory
        subcats = {}
        for f in cat_features:
            subcats.setdefault(f.subcategory, []).append(f)

        for subcat, feats in subcats.items():
            lines.append(f"### {subcat}")
            lines.append("")
            lines.append("| Feature | Status | Rust File | Notes |")
            lines.append("|---------|--------|-----------|-------|")
            for f in feats:
                status_icon = {"implemented": "OK", "stubbed": "PARTIAL", "missing": "MISSING"}[f.status]
                lines.append(
                    f"| {f.name} | {status_icon} | {f.rust_file} | {f.notes} |"
                )
            lines.append("")

    output_path.write_text("\n".join(lines), encoding="utf-8")


def generate_json_report(features: list[Feature], crates: dict[str, CrateInfo],
                          output_path: Path) -> None:
    """Generate coverage_data.json."""
    total = len(features)
    implemented = sum(1 for f in features if f.status == "implemented")
    stubbed = sum(1 for f in features if f.status == "stubbed")
    missing = sum(1 for f in features if f.status == "missing")

    data = {
        "generated": datetime.now(timezone.utc).isoformat(),
        "summary": {
            "total": total,
            "implemented": implemented,
            "stubbed": stubbed,
            "missing": missing,
            "coverage_pct": round(implemented / total * 100, 1) if total else 0,
        },
        "crates": {},
        "features": [],
    }

    for crate_name, info in sorted(crates.items()):
        data["crates"][crate_name] = {
            "files": len(info.files),
            "lines": info.line_count,
            "structs": len(info.structs),
            "functions": len(info.functions),
            "enums": len(info.enums),
            "traits": len(info.traits),
            "tests": info.test_count,
        }

    for f in features:
        data["features"].append({
            "category": f.category,
            "subcategory": f.subcategory,
            "name": f.name,
            "status": f.status,
            "rust_file": f.rust_file,
            "rust_symbol": f.rust_symbol,
            "notes": f.notes,
        })

    output_path.write_text(json.dumps(data, indent=2), encoding="utf-8")


def generate_feature_matrix(features: list[Feature], output_path: Path) -> None:
    """Generate FEATURE_MATRIX.md with checkboxes."""
    lines = [
        "# BlueBubbles Rust Rewrite - Feature Matrix",
        "",
        f"*Generated: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}*",
        "",
        "Legend: [x] = implemented, [~] = stubbed/partial, [ ] = missing",
        "",
    ]

    categories = {}
    for f in features:
        categories.setdefault(f.category, []).append(f)

    for cat_name, cat_features in categories.items():
        cat_total = len(cat_features)
        cat_impl = sum(1 for f in cat_features if f.status == "implemented")
        cat_pct = (cat_impl / cat_total * 100) if cat_total else 0
        lines.append(f"## {cat_name} ({cat_impl}/{cat_total} = {cat_pct:.0f}%)")
        lines.append("")

        subcats = {}
        for f in cat_features:
            subcats.setdefault(f.subcategory, []).append(f)

        for subcat, feats in subcats.items():
            lines.append(f"### {subcat}")
            lines.append("")
            for f in feats:
                if f.status == "implemented":
                    checkbox = "[x]"
                elif f.status == "stubbed":
                    checkbox = "[~]"
                else:
                    checkbox = "[ ]"
                note = f" *({f.notes})*" if f.notes else ""
                lines.append(f"- {checkbox} {f.name}{note}")
            lines.append("")

    output_path.write_text("\n".join(lines), encoding="utf-8")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    # Determine project root
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent  # bluebubbles-app-rust/

    print(f"Scanning Rust sources in: {project_root}")

    # Step 1: Build feature matrix
    features = build_feature_matrix()
    print(f"Feature matrix: {len(features)} features across "
          f"{len(set(f.category for f in features))} categories")

    # Step 2: Scan Rust source
    crates = scan_rust_source(project_root)
    total_files = sum(len(info.files) for info in crates.values())
    total_lines = sum(info.line_count for info in crates.values())
    print(f"Scanned: {len(crates)} crates, {total_files} files, {total_lines} lines")

    # Step 3: Match features
    match_features(features, crates)

    # Step 4: Generate outputs
    tools_dir = script_dir
    generate_terminal_report(features, crates)
    generate_markdown_report(features, crates, tools_dir / "COVERAGE_REPORT.md")
    generate_json_report(features, crates, tools_dir / "coverage_data.json")
    generate_feature_matrix(features, tools_dir / "FEATURE_MATRIX.md")

    print(f"Reports generated:")
    print(f"  - {tools_dir / 'COVERAGE_REPORT.md'}")
    print(f"  - {tools_dir / 'coverage_data.json'}")
    print(f"  - {tools_dir / 'FEATURE_MATRIX.md'}")


if __name__ == "__main__":
    main()
