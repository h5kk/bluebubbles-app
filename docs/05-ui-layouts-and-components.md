# UI Layouts and Components

This document provides a comprehensive reference for every screen, layout, reusable component, wrapper, and animation widget in the BlueBubbles Flutter application. All source paths are relative to `lib/app/` within the `bluebubbles-app-ELECTRON` project directory.

---

## Table of Contents

1. [Screen Map](#1-screen-map)
2. [Navigation Architecture](#2-navigation-architecture)
3. [Layout System](#3-layout-system)
4. [Conversation List](#4-conversation-list)
5. [Conversation View](#5-conversation-view)
6. [Settings Pages](#6-settings-pages)
7. [Setup Wizard](#7-setup-wizard)
8. [Reusable Components](#8-reusable-components)
9. [Wrapper Components](#9-wrapper-components)
10. [Responsive Design](#10-responsive-design)

---

## 1. Screen Map

Every navigable screen in the application, its widget class, source file, and purpose.

| Screen | Widget Class | Source File | Purpose |
|--------|-------------|-------------|---------|
| Splash Screen | `SplashScreen` | `layouts/startup/splash_screen.dart` | Initial loading screen with animated BlueBubbles icon; navigates to `SetupView` if setup not complete |
| Failure to Start | `FailureToStart` | `layouts/startup/failure_to_start.dart` | Fatal error screen shown when the app cannot initialize; displays error and stack trace |
| Setup Wizard | `SetupView` | `layouts/setup/setup_view.dart` | Multi-page onboarding wizard for first-time configuration |
| Welcome Page | `WelcomePage` | `layouts/setup/pages/welcome/welcome_page.dart` | First setup page with branding and getting-started content |
| Request Contacts | `RequestContacts` | `layouts/setup/pages/contacts/request_contacts.dart` | Requests contact permission (Android only) |
| Battery Optimization | `BatteryOptimizationCheck` | `layouts/setup/pages/setup_checks/battery_optimization.dart` | Prompts user to disable battery optimization (Android only) |
| Mac Setup Check | `MacSetupCheck` | `layouts/setup/pages/setup_checks/mac_setup_check.dart` | Verifies the macOS server is properly configured |
| Server Credentials | `ServerCredentials` | `layouts/setup/pages/sync/server_credentials.dart` | Manual server URL and password entry |
| QR Code Scanner | `QRCodeScanner` | `layouts/setup/pages/sync/qr_code_scanner.dart` | Scans QR code from BlueBubbles server for connection |
| Sync Settings | `SyncSettings` | `layouts/setup/pages/sync/sync_settings.dart` | Configure number of messages to download and sync options (non-web only) |
| Sync Progress | `SyncProgress` | `layouts/setup/pages/sync/sync_progress.dart` | Progress indicator during initial message sync |
| Conversation List | `ConversationList` | `layouts/conversation_list/pages/conversation_list.dart` | Main chat list screen; delegates to skin-specific implementations |
| Cupertino Conversation List | `CupertinoConversationList` | `layouts/conversation_list/pages/cupertino_conversation_list.dart` | iOS-styled chat list with pinned chats and large header |
| Material Conversation List | `MaterialConversationList` | `layouts/conversation_list/pages/material_conversation_list.dart` | Material Design-styled chat list |
| Samsung Conversation List | `SamsungConversationList` | `layouts/conversation_list/pages/samsung_conversation_list.dart` | Samsung One UI-styled chat list with expanding header |
| Search View | `SearchView` | `layouts/conversation_list/pages/search/search_view.dart` | Full message search with local and network modes, chat and handle filters |
| Conversation Peek View | `ConversationPeekView` | `layouts/conversation_list/dialogs/conversation_peek_view.dart` | 3D Touch / long-press preview of a conversation |
| Chat Creator | `ChatCreator` | `layouts/chat_creator/chat_creator.dart` | New conversation screen with contact selection, iMessage/SMS toggle |
| Conversation View | `ConversationView` | `layouts/conversation_view/pages/conversation_view.dart` | Message thread view for a single chat |
| Messages View | `MessagesView` | `layouts/conversation_view/pages/messages_view.dart` | Scrollable message list within ConversationView |
| Conversation Details | `ConversationDetails` | `layouts/conversation_details/conversation_details.dart` | Chat details: participants, media gallery, links, options |
| Find My | `FindMyPage` | `layouts/findmy/findmy_page.dart` | Find My Friends/Devices map with location tracking |
| Fullscreen Media Holder | `FullscreenHolder` | `layouts/fullscreen_media/fullscreen_holder.dart` | Fullscreen PageView for swiping through media |
| Fullscreen Image | `FullscreenImage` | `layouts/fullscreen_media/fullscreen_image.dart` | Fullscreen image viewer with zoom and pan |
| Fullscreen Video | `FullscreenVideo` | `layouts/fullscreen_media/fullscreen_video.dart` | Fullscreen video player |
| Chat Selector View | `ChatSelectorView` | `layouts/chat_selector_view/chat_selector_view.dart` | Picker to select an existing chat (used by search filters) |
| Contact Selector View | `ContactSelectorView` | `layouts/contact_selector_view/contact_selector_view.dart` | Picker to select a contact |
| Handle Selector View | `HandleSelectorView` | `layouts/handle_selector_view/handle_selector_view.dart` | Picker to select a handle/address |
| Settings Page | `SettingsPage` | `layouts/settings/settings_page.dart` | Main settings hub with categorized navigation tiles |
| Server Management | `ServerManagementPanel` | `layouts/settings/pages/server/server_management_panel.dart` | Server connection, URL, password, sync controls |
| Scheduled Messages | `ScheduledMessagesPanel` | `layouts/settings/pages/scheduling/scheduled_messages_panel.dart` | View and manage scheduled messages |
| Create Scheduled | `CreateScheduledPanel` | `layouts/settings/pages/scheduling/create_scheduled_panel.dart` | Create a new scheduled message |
| Message Reminders | `MessageRemindersPanel` | `layouts/settings/pages/scheduling/message_reminders_panel.dart` | Manage message reminder notifications (Android only) |
| Theming Panel | `ThemingPanel` | `layouts/settings/pages/theming/theming_panel.dart` | Skin selection, colors, fonts, Monet theming |
| Advanced Theming | `AdvancedThemingPanel` | `layouts/settings/pages/theming/advanced/advanced_theming_panel.dart` | Per-element color overrides |
| Custom Avatar Panel | `CustomAvatarPanel` | `layouts/settings/pages/theming/avatar/custom_avatar_panel.dart` | Manage custom contact avatars |
| Custom Avatar Color | `CustomAvatarColorPanel` | `layouts/settings/pages/theming/avatar/custom_avatar_color_panel.dart` | Per-contact avatar color customization |
| Avatar Crop | `AvatarCrop` | `layouts/settings/pages/theming/avatar/avatar_crop.dart` | Image cropping tool for avatars |
| Attachment Panel | `AttachmentPanel` | `layouts/settings/pages/message_view/attachment_panel.dart` | Media and attachment download/display settings |
| Conversation Panel | `ConversationPanel` | `layouts/settings/pages/message_view/conversation_panel.dart` | Message view appearance and behavior settings |
| Message Options Order | `MessageOptionsOrderPanel` | `layouts/settings/pages/message_view/message_options_order_panel.dart` | Reorder message long-press action menu items |
| Chat List Panel | `ChatListPanel` | `layouts/settings/pages/conversation_list/chat_list_panel.dart` | Chat list sorting, filtering, display options |
| Pinned Order Panel | `PinnedOrderPanel` | `layouts/settings/pages/conversation_list/pinned_order_panel.dart` | Reorder pinned conversations |
| Notification Panel | `NotificationPanel` | `layouts/settings/pages/system/notification_panel.dart` | Notification sounds, behavior, per-chat overrides |
| Desktop Panel | `DesktopPanel` | `layouts/settings/pages/desktop/desktop_panel.dart` | Desktop-specific settings (window effects, titlebar, tray) |
| Private API Panel | `PrivateAPIPanel` | `layouts/settings/pages/advanced/private_api_panel.dart` | Enable/configure Private API features |
| Redacted Mode Panel | `RedactedModePanel` | `layouts/settings/pages/advanced/redacted_mode_panel.dart` | Privacy/screenshot redaction settings |
| Tasker Panel | `TaskerPanel` | `layouts/settings/pages/advanced/tasker_panel.dart` | Tasker automation integration (Android only) |
| Notification Providers | `NotificationProvidersPanel` | `layouts/settings/pages/advanced/notification_providers_panel.dart` | FCM, WebSocket, UnifiedPush configuration |
| Firebase Panel | `FirebasePanel` | `layouts/settings/pages/advanced/firebase_panel.dart` | Firebase/FCM credential management |
| Unified Push | `UnifiedPushPanel` | `layouts/settings/pages/advanced/unified_push.dart` | UnifiedPush distributor configuration |
| Misc Panel | `MiscPanel` | `layouts/settings/pages/misc/misc_panel.dart` | Miscellaneous settings (incognito keyboard, send with return, etc.) |
| Troubleshoot Panel | `TroubleshootPanel` | `layouts/settings/pages/misc/troubleshoot_panel.dart` | Developer tools, log viewer, re-sync, database actions |
| Logging Panel | `LoggingPanel` | `layouts/settings/pages/misc/logging_panel.dart` | Log level configuration |
| Live Logging | `LiveLoggingPanel` | `layouts/settings/pages/misc/live_logging_panel.dart` | Real-time log stream viewer |
| About Panel | `AboutPanel` | `layouts/settings/pages/misc/about_panel.dart` | App version, changelog, links |
| Profile Panel | `ProfilePanel` | `layouts/settings/pages/profile/profile_panel.dart` | User name, avatar, and profile settings |
| Backup & Restore | `BackupRestorePanel` | `layouts/settings/pages/server/backup_restore_panel.dart` | Backup/restore settings and themes |
| OAuth Panel | `OAuthPanel` | `layouts/settings/pages/server/oauth_panel.dart` | OAuth/Google authentication configuration |

---

## 2. Navigation Architecture

### 2.1 Navigation Service

**File:** `services/ui/navigator/navigator_service.dart`
**Class:** `NavigatorService` (extends `GetxService`)
**Global accessor:** `ns` (singleton via `Get.put`)

The application uses a hybrid navigation approach combining Flutter's `Navigator` with GetX nested navigation. The `NavigatorService` manages three independent navigator stacks identified by numeric keys:

| Navigator Key | Purpose | Used By |
|--------------|---------|---------|
| `1` | Conversation list left panel | `ConversationList` in tablet mode |
| `2` | Conversation view right panel | `ConversationView` in tablet mode |
| `3` | Settings detail panel | `SettingsPage` in tablet mode |

### 2.2 Navigation Methods

The `NavigatorService` provides the following routing methods:

- **`push(context, widget)`** -- Pushes to the right panel (key 2) in tablet mode, or uses `Navigator.of(context).push` in phone mode. All routes are wrapped in `TitleBarWrapper` and use `ThemeSwitcher.buildPageRoute` for skin-appropriate transitions.
- **`pushLeft(context, widget)`** -- Pushes to the left panel (key 1) in tablet mode.
- **`pushSettings(context, widget, {binding})`** -- Pushes to the settings panel (key 3) in tablet mode, or the main navigator otherwise.
- **`pushAndRemoveUntil(context, widget, predicate)`** -- Replaces the right panel stack; closes the active chat controller before navigating. Accepts an optional `customRoute` for zero-duration transitions (used by `ChatCreator`).
- **`pushAndRemoveSettingsUntil(context, widget, predicate)`** -- Replaces the settings panel stack in tablet mode; in phone mode it pushes (does not remove) to preserve the back stack.
- **`backConversationView(context)`** -- Pops the conversation view stack; handles multi-level nested navigator popping.
- **`closeSettings(context)`** -- Pops all settings routes back to the first route.
- **`closeAllConversationView(context)`** -- Pops conversation view navigator to its initial page.
- **`backSettings(context)`** -- Pops one level in the settings navigator.

### 2.3 Tablet Mode Detection

```
bool isTabletMode(BuildContext context) =>
    (!context.isPhone || context.width / context.height > 0.8) &&
    ss.settings.tabletMode.value && context.width > 600;
```

Tablet mode is active when:
1. The device is not a phone OR the aspect ratio exceeds 0.8
2. The user has enabled the `tabletMode` setting
3. The screen width exceeds 600 logical pixels

### 2.4 Page Transitions

Page transitions are determined by the active skin via `ThemeSwitcher.buildPageRoute`:

| Skin | Transition |
|------|-----------|
| iOS | `CustomCupertinoPageTransition` (slide from right with parallax) |
| Material | `MaterialPageRoute` (standard Material slide-up) |
| Samsung | `MaterialPageRoute` (same as Material) |

### 2.5 Deep Linking

On desktop and web, the app restores the last opened chat on launch by reading `ss.prefs.getString('lastOpenedChat')` and pushing a `ConversationView` for that chat GUID. The `ConversationList.initState` handles this in a `WidgetsBinding.instance.addPostFrameCallback`.

---

## 3. Layout System

### 3.1 Skin-Based Layout Delegation

The app supports three visual skins: **iOS (Cupertino)**, **Material**, and **Samsung (One UI)**. The `ThemeSwitcher` widget selects the correct child based on the `ss.settings.skin` reactive value:

```
ThemeSwitcher(
  iOSSkin: CupertinoConversationList(...),
  materialSkin: MaterialConversationList(...),
  samsungSkin: SamsungConversationList(...),
)
```

If `samsungSkin` is null, the `materialSkin` is used as a fallback.

### 3.2 Optimized State Management

**File:** `wrappers/stateful_boilerplate.dart`

The app uses a custom state management layer built on top of GetX:

#### `StatefulController` (extends `GetxController`)
- Maintains a map of `updateWidgetFunctions` keyed by widget type
- Provides `updateWidgets<T>(arg)` to trigger rebuilds of specific widget types without full setState
- Exposes `updateObx(VoidCallback)` for frame-safe reactive updates

#### `CustomStateful<T>` (extends `StatefulWidget`)
- Requires a `parentController` of type `T extends StatefulController`
- The controller is accessible in the state via `controller` getter

#### `CustomState<T, R, S>` (extends `State<T>` with `ThemeHelpers`)
- Waits for page transition animation to complete before calling setState (via a `Completer<void>`)
- Optimized `setState` checks `SchedulerBinding.instance.schedulerPhase` and defers updates to `endOfFrame` if a frame is in progress
- Auto-deletes the GetX controller on dispose (configurable via `forceDelete`)
- Registers update listeners in `initState` for targeted widget updates

#### `OptimizedState<T>` (extends `State<T>`)
- Same animation-aware and frame-optimized setState as `CustomState` but without the GetX controller integration
- Used for widgets that do not need a dedicated controller

### 3.3 Widget Hierarchy Patterns

Most layouts follow this structure:

```
AnnotatedRegion<SystemUiOverlayStyle>    -- Status bar / nav bar styling
  Theme(data: ...)                        -- Bubble color overrides
    PopScope(canPop: false, ...)          -- Custom back-button handling
      SafeArea
        Scaffold
          appBar: PreferredSize(...)
          body: ...
```

---

## 4. Conversation List

### 4.1 Controller

**Class:** `ConversationListController` (extends `StatefulController`)
**File:** `layouts/conversation_list/pages/conversation_list.dart`

**Constructor parameters:**
- `showArchivedChats: bool` -- When true, shows archived chats instead of main list
- `showUnknownSenders: bool` -- When true, shows chats from unknown senders

**State:**
- `selectedChats: List<Chat>` -- Tracks multi-select for batch actions
- `showMaterialFABText: bool` -- Controls whether the FAB shows extended text
- `materialScrollStartPosition: double` -- Tracks scroll position for FAB hide/show

**Key methods:**
- `updateSelectedChats()` -- Triggers header and footer rebuilds based on selection count
- `clearSelectedChats()` -- Deselects all chats and updates each tile
- `openCamera(context)` -- Opens device camera, then navigates to `ChatCreator` with captured image
- `openNewChatCreator(context, {existing})` -- Navigates to `ChatCreator`, replacing conversation view stack

### 4.2 Top-Level Widget

**Class:** `ConversationList` (extends `CustomStateful<ConversationListController>`)

The controller is registered with a tag based on the mode: `"Archived"`, `"Unknown"`, or `"Messages"`.

**Build tree (phone mode):**
```
AnnotatedRegion<SystemUiOverlayStyle>
  TabletModeWrapper
    left: ThemeSwitcher(iOS | Material | Samsung skin)
    right: Navigator(key: 2) with InitialWidgetRight
```

**Build tree (tablet mode):**
```
AnnotatedRegion
  TabletModeWrapper
    left: Navigator(key: 1)
      CupertinoPage(name: "initial", child: ThemeSwitcher(...))
    right: Navigator(key: 2)
      CupertinoPage(name: "initial", child: InitialWidgetRight())
```

### 4.3 Skin-Specific Implementations

#### CupertinoConversationList
- Uses `CustomScrollView` with `CupertinoHeader` as the first sliver
- Pinned chats rendered via `PinnedConversationTile` in a grid layout with page dots
- Unpinned chats rendered as `ConversationTile` widgets via `SliverList`
- Supports swipe actions (archive, delete, pin, mark read/unread, mute)
- `ConversationListFAB` as a floating action button

#### MaterialConversationList
- `AppBar` with `MaterialHeader` containing search, camera, and overflow menu
- Standard `ListView` of `ConversationTile` widgets
- Multi-select mode activates a contextual action bar in the header
- FAB text collapses on scroll

#### SamsungConversationList
- Expanding/collapsing header (`SamsungHeader`) with large title
- Bottom navigation footer (`SamsungFooter`) with contextual actions during multi-select
- Same tile structure as Material but with Samsung-specific styling

### 4.4 Conversation Tile

**File:** `layouts/conversation_list/widgets/tile/conversation_tile.dart`
**Controller:** `ConversationTileController` (extends `StatefulController`)

**Controller state:**
- `shouldHighlight: RxBool` -- Active chat highlight (tablet mode)
- `shouldPartialHighlight: RxBool` -- Right-click context menu highlight
- `hoverHighlight: RxBool` -- Mouse hover highlight (desktop)
- `chat: Chat` -- The chat model
- `listController: ConversationListController` -- Parent list controller reference

**Key behavior:**
- `onTap` -- Opens `ConversationView` via `ns.pushAndRemoveUntil` or focuses existing chat
- `onSecondaryTap` -- Shows context menu (right-click on desktop, long-press on mobile)
- `onLongPress` -- Toggles multi-select mode

Each tile delegates to a skin-specific widget:
- `CupertinoConversationTile` -- iOS-style with swipe actions (Dismissible)
- `MaterialConversationTile` -- Material-style with ripple effect
- `SamsungConversationTile` -- Samsung-style with rounded tiles

### 4.5 Pinned Conversation Tile

**File:** `layouts/conversation_list/widgets/tile/pinned_conversation_tile.dart`

Renders pinned chats as avatar-centric tiles with a small text bubble showing the latest message preview (`PinnedTileTextBubble`). Supports drag-and-drop reordering (iOS skin). Displayed in a paginated grid layout.

### 4.6 Search

**File:** `layouts/conversation_list/pages/search/search_view.dart`
**Class:** `SearchView` / `SearchViewState`

**Features:**
- Text search with debounce
- Toggle between local database search and network (server) search
- Filter by specific chat (`ChatSelectorView`) or specific handle (`HandleSelectorView`)
- Filter by sender (from me)
- Date range filter (`TimeframePicker`)
- Results displayed as chat + message tuples; tapping opens `ConversationView` scrolled to that message
- Previous search history maintained in `pastSearches`

### 4.7 Floating Action Button

**File:** `layouts/conversation_list/widgets/conversation_list_fab.dart`
**Class:** `ConversationListFAB`

Adapts per skin:
- iOS: Circular compose button
- Material: Extended FAB that collapses on scroll (text fades out)
- Samsung: Circular FAB

Shows batch action icons during multi-select (archive, delete, mark read, pin, mute).

---

## 5. Conversation View

### 5.1 ConversationView

**File:** `layouts/conversation_view/pages/conversation_view.dart`
**Class:** `ConversationView` (extends `StatefulWidget`)

**Constructor parameters:**
- `chat: Chat` (required) -- The chat to display
- `customService: MessagesService?` -- Optional custom message service (for search context view)
- `fromChatCreator: bool` -- When true, auto-focuses the text field
- `onInit: void Function()?` -- Callback run after initialization (used by ChatCreator to send the initial message)

**Build tree:**
```
AnnotatedRegion<SystemUiOverlayStyle>
  Theme (bubble color overrides)
    PopScope (handles back: exits select mode, hides attachment picker, pops route)
      SafeArea
        Scaffold
          appBar: Cupertino or Material header (skin-dependent)
          body: Actions (keyboard shortcuts for reactions)
            GradientBackground
              Stack
                ScreenEffectsWidget (fullscreen animations)
                Column
                  Expanded: Stack
                    MessagesView (scrollable message list)
                    Scroll-to-bottom FAB (animated opacity)
                  ConversationTextField (input bar)
```

**Keyboard shortcuts (Actions):**
When Private API is enabled, the conversation view registers keyboard `Intent`/`Action` pairs for:
- `ReplyRecentIntent` -- Reply to the most recent message
- `HeartRecentIntent`, `LikeRecentIntent`, `DislikeRecentIntent`, `LaughRecentIntent`, `EmphasizeRecentIntent`, `QuestionRecentIntent` -- Add reactions to the most recent message
- `OpenChatDetailsIntent` -- Open conversation details

### 5.2 Messages View

**File:** `layouts/conversation_view/pages/messages_view.dart`
**Class:** `MessagesView` (extends `StatefulWidget`)

**Constructor parameters:**
- `customService: MessagesService?` -- For search/context message loading
- `controller: ConversationViewController` (required)

**Key state:**
- `_messages: List<Message>` -- Sorted message list
- `smartReplies: RxList<Widget>` -- Google ML Kit smart reply suggestions (Android only)
- `internalSmartReplies: RxMap` -- App-generated smart replies (attach recent photo, jump to unread)
- `noMoreMessages: bool` -- Pagination complete flag
- `fetching: bool` -- Currently loading flag
- `dragging: RxBool` -- File drag-and-drop active state
- `listKey: GlobalKey<SliverAnimatedListState>` -- For animated insertions/removals

**Build tree:**
```
DropRegion (drag-and-drop file attach)
  GestureDetector (horizontal drag for iOS timestamp reveal)
    Stack
      AnimatedOpacity (dims during drag)
        DeferredPointerHandler
          ScrollbarWrapper
            CustomScrollView (reverse: true)
              SliverToBoxAdapter: Smart replies row
              SliverToBoxAdapter: Focus status (notifications silenced)
              SliverToBoxAdapter: Typing indicator
              SliverAnimatedList: Message items (paginated)
              SliverPadding (top spacer)
      AnimatedContainer: Drag overlay ("Attach N Files")
```

**Pagination:** When the user scrolls to the bottom of the loaded messages (topmost in the reversed list), `loadNextChunk()` fetches the next 25 messages from the `MessagesService`. New messages are inserted with a 500ms `SizeTransition` + `SlideTransition` animation.

**Smart Replies:** On Android, the `GoogleMlKit.nlp.smartReply()` API generates reply suggestions displayed as horizontal chips above the message list.

### 5.3 Message Holder

**File:** `layouts/conversation_view/widgets/message/message_holder.dart`
**Class:** `MessageHolder` (extends `CustomStateful<MessageWidgetController>`)

**Constructor parameters:**
- `cvController: ConversationViewController` (required)
- `message: Message` (required)
- `oldMessageGuid: String?` -- GUID of the older adjacent message
- `newMessageGuid: String?` -- GUID of the newer adjacent message
- `isReplyThread: bool` -- When true, renders in reply thread popup context
- `replyPart: int?` -- Specific part index for reply threads

**Layout tree (documented in source):**
```
Timestamp separator
Row
  SelectCheckbox (left, if not from me)
  Expanded Column
    For each MessagePart:
      Previous edits (collapsed by default)
      Reply bubble (if replying to another message)
      Message sender name (group chats)
      Reaction spacing
      Stack
        Avatar (bottom-left for received messages)
        Padded content area
          DecoratedBox (reply line painter)
            GestureDetector (tap, swipe-to-reply)
              ClipPath (TailClipper for bubble tail)
                Stack
                  Content: InteractiveHolder | TextBubble | AttachmentHolder
                  Edit overlay (inline editing)
                  StickerHolder
                  ReactionHolder (positioned top-left or top-right)
              SlideToReply indicator
      MessageProperties (reply count, edit indicator, effect name)
    DeliveredIndicator
  SelectCheckbox (right, if from me)
  Error icon (if send failed)
  MessageTimestamp (iOS slide-to-reveal)
```

### 5.4 Message Content Widgets

| Widget | File | Purpose |
|--------|------|---------|
| `TextBubble` | `widgets/message/text/text_bubble.dart` | Text message bubble with emoji detection and big-emoji mode |
| `AttachmentHolder` | `widgets/message/attachment/attachment_holder.dart` | Routes to appropriate attachment viewer |
| `ImageViewer` | `widgets/message/attachment/image_viewer.dart` | Inline image with tap-to-fullscreen |
| `VideoPlayer` | `widgets/message/attachment/video_player.dart` | Inline video player with controls |
| `AudioPlayer` | `widgets/message/attachment/audio_player.dart` | Audio waveform player |
| `ContactCard` | `widgets/message/attachment/contact_card.dart` | vCard contact attachment display |
| `OtherFile` | `widgets/message/attachment/other_file.dart` | Generic file attachment with download/open |
| `StickerHolder` | `widgets/message/attachment/sticker_holder.dart` | Overlaid sticker display |
| `InteractiveHolder` | `widgets/message/interactive/interactive_holder.dart` | Routes Apple Pay, URL previews, Game Pigeon, etc. |
| `UrlPreview` | `widgets/message/interactive/url_preview.dart` | Rich link preview card |
| `UrlPreviewLegacy` | `widgets/message/interactive/url_preview.legacy.dart` | Fallback URL preview for older data |
| `ApplePay` | `widgets/message/interactive/apple_pay.dart` | Apple Pay transaction display |
| `EmbeddedMedia` | `widgets/message/interactive/embedded_media.dart` | Embedded rich media (Apple Music, etc.) |
| `GamePigeon` | `widgets/message/interactive/game_pigeon.dart` | Game Pigeon interactive message |
| `SupportedInteractive` | `widgets/message/interactive/supported_interactive.dart` | Supported interactive message types |
| `UnsupportedInteractive` | `widgets/message/interactive/unsupported_interactive.dart` | Fallback for unknown interactive types |
| `ChatEvent` | `widgets/message/chat_event/chat_event.dart` | Group events (member added/removed, name changed) |

### 5.5 Message Auxiliary Widgets

| Widget | File | Purpose |
|--------|------|---------|
| `ReactionHolder` | `widgets/message/reaction/reaction_holder.dart` | Container for reaction badges positioned above a message bubble |
| `Reaction` | `widgets/message/reaction/reaction.dart` | Individual reaction icon display |
| `ReactionClipper` | `widgets/message/reaction/reaction_clipper.dart` | Custom clipper for reaction badge shape |
| `ReplyBubble` | `widgets/message/reply/reply_bubble.dart` | Compact quoted message bubble for inline replies |
| `ReplyLinePainter` | `widgets/message/reply/reply_line_painter.dart` | Custom painter for the iOS-style reply threading line |
| `ReplyThreadPopup` | `widgets/message/reply/reply_thread_popup.dart` | Full reply thread modal view |
| `BubbleEffects` | `widgets/message/misc/bubble_effects.dart` | Bubble-level send effects (slam, loud, gentle, invisible ink) |
| `MessageProperties` | `widgets/message/misc/message_properties.dart` | "Edited", "N replies", effect name labels below a bubble |
| `MessageSender` | `widgets/message/misc/message_sender.dart` | Sender name display in group chats |
| `SelectCheckbox` | `widgets/message/misc/select_checkbox.dart` | Checkbox for multi-select mode |
| `SlideToReply` | `widgets/message/misc/slide_to_reply.dart` | Reply arrow that appears during swipe gesture |
| `TailClipper` | `widgets/message/misc/tail_clipper.dart` | CustomClipper for the iMessage-style bubble tail |
| `SendAnimation` | `widgets/message/send_animation.dart` | Send button animation (fly-up effect) |

### 5.6 Timestamps

| Widget | File | Purpose |
|--------|------|---------|
| `TimestampSeparator` | `widgets/message/timestamp/timestamp_separator.dart` | Large date/time separator between message groups (30+ min gap) |
| `MessageTimestamp` | `widgets/message/timestamp/message_timestamp.dart` | Per-message timestamp (iOS: revealed on horizontal swipe; Samsung: always visible) |
| `DeliveredIndicator` | `widgets/message/timestamp/delivered_indicator.dart` | "Delivered", "Read", or time-since-read indicator |

### 5.7 Typing Indicator

| Widget | File | Purpose |
|--------|------|---------|
| `TypingIndicator` | `widgets/message/typing/typing_indicator.dart` | Animated three-dot typing bubble |
| `TypingClipper` | `widgets/message/typing/typing_clipper.dart` | CustomClipper for typing bubble shape |

### 5.8 Message Popup Menu

| Widget | File | Purpose |
|--------|------|---------|
| `MessagePopupHolder` | `widgets/message/popup/message_popup_holder.dart` | Wrapper that triggers the popup on long-press / right-click |
| `MessagePopup` | `widgets/message/popup/message_popup.dart` | The popup menu overlay with reactions and actions |
| `DetailsMenuAction` | `widgets/message/popup/details_menu_action.dart` | Individual action button in the popup (reply, copy, forward, etc.) |
| `ReactionPickerClipper` | `widgets/message/popup/reaction_picker_clipper.dart` | CustomClipper for the reaction picker bar shape |

### 5.9 Screen Effects

**File:** `layouts/conversation_view/widgets/effects/screen_effects_widget.dart`
**Class:** `ScreenEffectsWidget`

Renders fullscreen iMessage screen effects using custom `Canvas` rendering. Each effect has a controller class and a rendering class:

| Effect | Controller | Renderer |
|--------|-----------|----------|
| Fireworks | `FireworkController` | `FireworkRendering` (`fireworks_classes.dart` / `fireworks_rendering.dart`) |
| Celebration | `CelebrationController` | `CelebrationRendering` (`celebration_class.dart` / `celebration_rendering.dart`) |
| Confetti | `ConfettiController` | Uses the `confetti` package |
| Balloons | `BalloonController` | `BalloonRendering` (`balloon_classes.dart` / `balloon_rendering.dart`) |
| Love | `LoveController` | `LoveRendering` (`love_classes.dart` / `love_rendering.dart`) |
| Spotlight | `SpotlightController` | `SpotlightRendering` (`spotlight_classes.dart` / `spotlight_rendering.dart`) |
| Lasers | `LaserController` | `LaserRendering` (`laser_classes.dart` / `laser_rendering.dart`) |

The `SendEffectPicker` widget (`widgets/effects/send_effect_picker.dart`) provides the UI for selecting a send effect before sending a message.

### 5.10 Conversation Text Field

**File:** `layouts/conversation_view/widgets/text_field/conversation_text_field.dart`
**Class:** `ConversationTextField` (extends `CustomStateful<ConversationViewController>`)

**Build tree:**
```
SafeArea
  Padding
    Column
      Row (main input row)
        Camera button (iOS + Android only)
        Attachment picker button (+)
        GIF picker button (non-Android)
        Emoji picker button (desktop/web)
        Location button (desktop, non-Linux)
        Expanded: Stack
          TextFieldComponent (actual text input area)
          VoiceMessageRecorder (overlay when recording)
          SendAnimation
        TextFieldSuffix (Samsung only, outside field)
      AttachmentPicker (expanded gallery/file picker)
      EmojiPicker (desktop/web emoji grid)
```

**Features:**
- Draft persistence: text and attachments are saved to the chat model on dispose
- Typing indicators: sends `started-typing` / `stopped-typing` socket events with 3-second debounce
- Emoji shortcode completion: typing `:name` shows emoji autocomplete
- Mention completion: typing `@name` in group chats shows mentionable participant list
- Subject line: optional second text field (requires Private API)
- Voice recording: hold-to-record with waveform visualization
- Paste image from clipboard (desktop)
- Content insertion (keyboard GIFs, stickers)
- Send with Return key (configurable)
- Keyboard shortcut handling (arrow keys for emoji/mention selection, Enter to send, Escape to dismiss, Tab to switch fields)
- Auto-effect detection: automatically adds effects for "Congratulations", "Happy Birthday", "Happy New Year", "Pew Pew", etc.

### 5.11 Text Field Sub-Components

| Widget | File | Purpose |
|--------|------|---------|
| `TextFieldComponent` | (in `conversation_text_field.dart`) | The actual `TextField` widget with reply holder, picked attachments, subject line |
| `ReplyHolder` | `widgets/text_field/reply_holder.dart` | Shows the message being replied to above the text field |
| `PickedAttachmentsHolder` | `widgets/text_field/picked_attachments_holder.dart` | Horizontal scroll of picked attachment previews |
| `PickedAttachment` | `widgets/text_field/picked_attachment.dart` | Single attachment thumbnail with remove button |
| `TextFieldSuffix` | `widgets/text_field/text_field_suffix.dart` | Send button, voice record button, scheduled send |
| `SendButton` | `widgets/text_field/send_button.dart` | Animated send button with long-press for effects |
| `VoiceMessageRecorder` | `widgets/text_field/voice_message_recorder.dart` | Audio waveform recorder overlay |

### 5.12 Media Picker

| Widget | File | Purpose |
|--------|------|---------|
| `AttachmentPicker` | `widgets/media_picker/text_field_attachment_picker.dart` | Gallery/camera/file picker panel below the text field |
| `AttachmentPickerFile` | `widgets/media_picker/attachment_picker_file.dart` | Individual file tile in the attachment picker grid |

### 5.13 Conversation Headers

| Widget | File | Purpose |
|--------|------|---------|
| `CupertinoHeader` (conversation) | `widgets/header/cupertino_header.dart` | iOS-style header with avatar, name, video/audio call buttons |
| `MaterialHeader` (conversation) | `widgets/header/material_header.dart` | Material-style AppBar with title and action icons |
| `HeaderWidgets` | `widgets/header/header_widgets.dart` | Shared header components (ConnectionIndicator, call buttons) |

---

## 6. Settings Pages

### 6.1 Settings Page Structure

**File:** `layouts/settings/settings_page.dart`
**Class:** `SettingsPage` (extends `StatefulWidget`)

**Constructor parameters:**
- `initialPage: Widget?` -- When provided, auto-navigates to this settings panel on mount

The settings page uses `TabletModeWrapper` with the settings list on the left and a detail panel on the right (navigator key 3). In phone mode, each settings section pushes a new route.

### 6.2 Settings Organization

The main settings page is organized into sections:

**Profile** (non-web, Material/Samsung only):
- User profile tile with avatar, navigates to `ProfilePanel`

**Server & Message Management:**
- Connection & Server -> `ServerManagementPanel`
- Scheduled Messages -> `ScheduledMessagesPanel` (server version >= 205)
- Message Reminders -> `MessageRemindersPanel` (Android only)

**Appearance:**
- Appearance Settings -> `ThemingPanel`

**Application Settings:**
- Media Settings -> `AttachmentPanel`
- Notification Settings -> `NotificationPanel`
- Chat List Settings -> `ChatListPanel`
- Conversation Settings -> `ConversationPanel`
- Desktop Settings -> `DesktopPanel` (desktop only)
- More Settings -> `MiscPanel`

**Advanced:**
- Private API Features -> `PrivateAPIPanel`
- Redacted Mode -> `RedactedModePanel`
- Tasker Integration -> `TaskerPanel` (Android only)
- Notification Providers -> `NotificationProvidersPanel`
- Developer Tools -> `TroubleshootPanel`

**Backup and Restore:**
- Backup & Restore -> `BackupRestorePanel`
- Export Contacts (non-web, non-desktop)

**About & Links:**
- Leave Us a Review (Android/Windows)
- Make a Donation (external link)
- Join Our Discord (external link)
- About & More -> `AboutPanel`

**Danger Zone:**
- Delete All Attachments (non-web)
- Reset App / Logout

### 6.3 Settings Widget Library

**File:** `layouts/settings/widgets/settings_widgets.dart` (barrel export)

#### Layout Widgets

| Widget | File | Purpose |
|--------|------|---------|
| `SettingsScaffold` | `widgets/layout/settings_scaffold.dart` | Standard settings page scaffold with AppBar (iOS/Material) or expanding header (Samsung), scrollbar, and sliver body |
| `SettingsSection` | `widgets/layout/settings_section.dart` | Grouped section container with background color |
| `SettingsHeader` | `widgets/layout/settings_header.dart` | Section header text with skin-appropriate styling |
| `SettingsDivider` | `widgets/layout/settings_divider.dart` | Horizontal divider between tiles |

#### Content Widgets

| Widget | File | Purpose |
|--------|------|---------|
| `SettingsTile` | `widgets/content/settings_tile.dart` | Standard settings row with title, subtitle, leading icon, trailing widget, onTap |
| `SettingsSwitch` | `widgets/content/settings_switch.dart` | Toggle switch tile with reactive `Rx<bool>` binding |
| `SettingsSlider` | `widgets/content/settings_slider.dart` | Slider tile with min/max/divisions and reactive `Rx<double>` binding |
| `SettingsDropdown` | `widgets/content/settings_dropdown.dart` | Dropdown selector tile |
| `SettingsSubtitle` | `widgets/content/settings_subtitle.dart` | Subtitle text widget for secondary descriptions |
| `SettingsLeadingIcon` | `widgets/content/settings_leading_icon.dart` | Colored icon container (different icons for iOS vs Material) |
| `NextButton` | `widgets/content/next_button.dart` | Chevron-right trailing indicator |
| `LogLevelSelector` | `widgets/content/log_level_selector.dart` | Log level dropdown |
| `AdvancedThemingTile` | `widgets/content/advanced_theming_tile.dart` | Color-preview tile for theme customization |

### 6.4 Settings Dialogs

| Dialog | File | Purpose |
|--------|------|---------|
| `CreateNewThemeDialog` | `dialogs/create_new_theme_dialog.dart` | Name input for new custom theme |
| `CustomHeadersDialog` | `dialogs/custom_headers_dialog.dart` | Edit custom HTTP headers for server connection |
| `NotificationSettingsDialog` | `dialogs/notification_settings_dialog.dart` | Per-chat notification override configuration |
| `OldThemesDialog` | `dialogs/old_themes_dialog.dart` | Legacy theme import/migration |
| `SyncDialog` | `dialogs/sync_dialog.dart` | Full message re-sync progress dialog |

---

## 7. Setup Wizard

### 7.1 Controller

**Class:** `SetupViewController` (extends `StatefulController`)
**File:** `layouts/setup/setup_view.dart`

**State:**
- `pageController: PageController` -- Controls the PageView
- `currentPage: int` -- Current page number (1-based for display)
- `numberToDownload: int` -- Messages per chat to sync (default 25)
- `skipEmptyChats: bool` -- Skip chats with no messages during sync
- `saveToDownloads: bool` -- Save attachments to device Downloads folder
- `error: String` -- Connection error message
- `obscurePass: bool` -- Password field visibility toggle

**Computed:**
- `pageOfNoReturn: int` -- Page index after which disconnection triggers error dialog (3 for web/desktop, 5 for mobile)

### 7.2 Page Flow

The setup wizard uses a `PageView` with `NeverScrollableScrollPhysics` (only programmatic navigation). Pages skip automatically if their requirements are already met (contacts already granted, battery optimization already disabled).

| Page | Widget | Platform | Purpose |
|------|--------|----------|---------|
| 1 | `WelcomePage` | All | Welcome message and branding |
| 2 | `RequestContacts` | Mobile only | Requests contacts permission |
| 3 | `BatteryOptimizationCheck` | Mobile only | Requests battery optimization exemption |
| 4 | `MacSetupCheck` | All | Verifies server is configured on macOS |
| 5 | `ServerCredentials` | All | Server URL + password entry OR QR scan |
| 6 | `SyncSettings` | Non-web | Configure message count and sync options |
| 7 | `SyncProgress` | All | Progress bar during initial sync |

**Page count per platform:**
- Web: 4 pages
- Desktop: 5 pages
- Mobile: 7 pages

### 7.3 Setup Header

The header displays the BlueBubbles icon, app name, and a gradient page indicator pill (`PageNumber` widget). The page counter updates reactively via the `updateWidgets<PageNumber>` mechanism.

### 7.4 Setup Dialogs

| Dialog | File | Purpose |
|--------|------|---------|
| `ConnectingDialog` | `dialogs/connecting_dialog.dart` | "Connecting to server..." progress indicator |
| `FailedToConnectDialog` | `dialogs/failed_to_connect_dialog.dart` | Connection failure with retry/back options |
| `FailedToScanDialog` | `dialogs/failed_to_scan_dialog.dart` | QR scan failure notification |
| `ManualEntryDialog` | `dialogs/manual_entry_dialog.dart` | Manual server URL and password entry form |

---

## 8. Reusable Components

### 8.1 Avatar System

#### ContactAvatarWidget

**File:** `components/avatars/contact_avatar_widget.dart`

**Constructor parameters:**
- `handle: Handle?` -- The handle to display avatar for
- `contact: Contact?` -- Direct contact reference (overrides handle.contact)
- `size: double?` -- Avatar diameter (default 40, scaled by `avatarScale` setting)
- `fontSize: double?` -- Initials font size (default 18)
- `borderThickness: double` -- Border width (default 2.0)
- `editable: bool` -- Whether tapping opens contact form (default true)
- `scaleSize: bool` -- Whether to apply the `avatarScale` setting (default true)
- `preferHighResAvatar: bool` -- Request higher-res avatar image (default false)
- `padding: EdgeInsets` -- Internal padding (default zero)

**Rendering logic (priority order):**
1. User avatar path (for self-avatar when handle is null)
2. Contact photo from device contacts
3. Initials text (iOS: full initials, Material: first letter only)
4. Person icon fallback

**Color logic:**
- If `colorfulAvatars` is enabled: gradient from address hash, or custom `Handle.color`
- Otherwise: static gray gradient
- Long-press opens a color picker to set custom avatar color

#### ContactAvatarGroupWidget

**File:** `components/avatars/contact_avatar_group_widget.dart`

**Constructor parameters:**
- `chat: Chat` (required) -- The group chat
- `size: double` -- Overall widget size (default 40)
- `editable: bool` -- Whether individual avatars are editable (default true)

**Layout logic:**
- Single participant: renders one `ContactAvatarWidget`
- Multiple participants (iOS skin): Circular arrangement using trigonometric positioning. Avatars are arranged in a circle with `sin`/`cos` placement. If participants exceed `maxAvatarsInGroupWidget`, the last position shows a group icon with blurred background.
- Multiple participants (Material skin): Grid arrangement using predefined alignment maps for 2, 3, and 4 participants.
- Custom group avatar: If `chat.customAvatarPath` is set, displays that image instead.

### 8.2 Custom Components

| Component | File | Purpose |
|-----------|------|---------|
| `CustomBouncingScrollPhysics` | `components/custom/custom_bouncing_scroll_physics.dart` | iOS-style bouncing scroll with custom overscroll behavior |
| `CustomCupertinoAlertDialog` | `components/custom/custom_cupertino_alert_dialog.dart` | Modified Cupertino alert dialog |
| `CustomCupertinoPageTransition` | `components/custom/custom_cupertino_page_transition.dart` | Custom iOS page transition with parallax effect |
| `CustomErrorBox` | `components/custom/custom_error_box.dart` | Error display widget (replaces Flutter's default red error box) |

### 8.3 Other Components

| Component | File | Purpose |
|-----------|------|---------|
| `MentionTextEditingController` | `components/custom_text_editing_controllers.dart` | Text controller with mention detection, rendering, and management. Uses special escaping characters to encode mention indices. |
| `SpellCheckTextEditingController` | `components/custom_text_editing_controllers.dart` | Text controller with spell-check support and optional focus node tracking. |
| `CircleProgressBar` | `components/circle_progress_bar.dart` | Circular progress indicator widget |
| `SliverDecoration` | `components/sliver_decoration.dart` | Decoration for sliver list items |

---

## 9. Wrapper Components

### 9.1 TitleBarWrapper

**File:** `wrappers/titlebar_wrapper.dart`

Wraps all top-level pages. On desktop, renders the custom window title bar (`TitleBar`) with minimize/maximize/close buttons and the drag-to-move area. On all platforms, optionally shows a `ConnectionIndicator` overlay when `showConnectionIndicator` is enabled.

**Sub-widgets:**
- `TitleBar` -- `WindowTitleBarBox` with `MoveWindow` (drag area) and `WindowButtons`
- `WindowButtons` -- Minimize, Maximize, Close buttons with themed colors. Minimize respects `minimizeToTray` setting.

### 9.2 TabletModeWrapper

**File:** `wrappers/tablet_mode_wrapper.dart`

**Constructor parameters:**
- `left: Widget` (required) -- Left panel content
- `right: Widget` (required) -- Right panel content
- `initialRatio: double` -- Initial split ratio (default 0.5)
- `dividerWidth: double` -- Divider handle width (default 7.0)
- `minRatio: double` -- Minimum left panel ratio
- `maxRatio: double` -- Maximum left panel ratio
- `allowResize: bool` -- Whether the divider is draggable (default true)
- `minWidthLeft: double?` -- Minimum left panel pixel width
- `maxWidthLeft: double?` -- Maximum left panel pixel width

**Behavior:**
- In phone mode (`showAltLayout` is false): renders only the `left` widget wrapped in `TitleBarWrapper`
- In tablet mode: renders a `Row` with left panel, draggable divider, and right panel
- The split ratio is persisted to `SharedPreferences` via `splitRatio` key
- The divider shows three dots and uses `MouseRegion` with `SystemMouseCursors.resizeLeftRight`
- Handles rotation from landscape to portrait by closing the active chat controller

### 9.3 ThemeSwitcher

**File:** `wrappers/theme_switcher.dart`

**Constructor parameters:**
- `iOSSkin: Widget` (required)
- `materialSkin: Widget` (required)
- `samsungSkin: Widget?` -- Falls back to `materialSkin` if null

Reactively switches between skin implementations using `Obx(() => switch(ss.settings.skin.value))`.

**Static methods:**
- `buildPageRoute<T>(builder)` -- Returns a skin-appropriate `PageRoute` (Cupertino transition for iOS, MaterialPageRoute for Material/Samsung)
- `getScrollPhysics()` -- Returns skin-appropriate scroll physics (bouncing for iOS, clamping for Material/Samsung)

### 9.4 ScrollbarWrapper

**File:** `wrappers/scrollbar_wrapper.dart`

On mobile: renders the child directly.
On desktop/web: wraps with `ImprovedScrolling` (middle-mouse-button scrolling support) and optionally a `RawScrollbar`. Includes a `Focus` handler that redirects `Tab` key presses back to the active chat's text field.

**Constructor parameters:**
- `child: Widget` (required)
- `showScrollbar: bool` -- Whether to show the scrollbar (default false)
- `reverse: bool` -- Reverses MMB scroll deceleration direction (default false)
- `controller: ScrollController` (required)

### 9.5 GradientBackground

**File:** `wrappers/gradient_background_wrapper.dart`

Renders an animated gradient background behind the conversation view when the theme uses gradient backgrounds (`ts.isGradientBg`). Uses `MirrorAnimationBuilder` from the `simple_animations` package to create a pulsing gradient between the bubble color and the background color.

### 9.6 FadeOnScroll

**File:** `wrappers/fade_on_scroll.dart`

Fades a child widget in or out based on scroll position. Used for header elements that should fade as the user scrolls.

**Constructor parameters:**
- `scrollController: ScrollController` (required)
- `child: Widget` (required)
- `zeroOpacityOffset: double` -- Scroll offset where opacity reaches 0
- `fullOpacityOffset: double` -- Scroll offset where opacity reaches 1

### 9.7 CupertinoIconWrapper

**File:** `wrappers/cupertino_icon_wrapper.dart`

Adds 1px left padding to icons when the iOS skin is active. This corrects visual alignment differences between Cupertino and Material icons.

---

## 10. Responsive Design

### 10.1 Platform Detection

The app uses several compile-time and runtime checks:

| Check | Meaning |
|-------|---------|
| `kIsDesktop` | Running on Windows, macOS, or Linux |
| `kIsWeb` | Running in a web browser |
| `Platform.isAndroid` | Native Android build |
| `Platform.isLinux` / `Platform.isWindows` | Specific desktop platform |

### 10.2 Layout Modes

**Phone mode (single-column):**
- Full-width conversation list or conversation view
- Navigation uses `Navigator.of(context).push` for all routes
- Back button pops the current route

**Tablet mode (split-view):**
- `TabletModeWrapper` splits the screen into left (chat list) and right (conversation view) panels
- Three nested navigators (keys 1, 2, 3) manage independent route stacks
- The divider between panels is draggable
- Split ratio is persisted and synchronized across wrapper instances via `eventDispatcher`
- `NavigatorService.width(context)` returns the appropriate panel width based on which nested navigator the widget belongs to

**Conditions for tablet mode activation:**
1. Screen width > 600px
2. `tabletMode` setting is enabled
3. Not a phone device OR aspect ratio > 0.8

### 10.3 Desktop-Specific Features

- Custom title bar with minimize/maximize/close buttons (`TitleBarWrapper`)
- Window effects: transparency, acrylic, mica (via `flutter_acrylic`)
- Minimize to system tray option
- Middle-mouse-button scrolling (`ImprovedScrolling`)
- Keyboard shortcuts for reactions, reply, and navigation
- Inline message editing (Up arrow to edit last sent message)
- Paste image from clipboard
- Drag-and-drop file attachment (`DropRegion` in `MessagesView`)
- Emoji picker panel (desktop/web, since no keyboard emoji access)
- Scrollbar visibility on all scroll views

### 10.4 Web-Specific Adaptations

- No file system access (attachments use bytes instead of paths)
- No voice recording, no Google ML Kit smart replies
- No battery optimization check in setup
- Async chat loading (`chats.loadedAllChats.future`)
- Logout instead of "Reset App" in settings
- Context menu prevention on right-click (`html.document.onContextMenu`)

### 10.5 Mobile-Specific Features

- Full camera integration (photo and video capture)
- Contact permission management
- Battery optimization exemption
- Swipe-to-close-keyboard / swipe-to-open-keyboard gestures
- Hardware back button handling via `PopScope`
- System UI overlay style management (status bar, navigation bar colors)
- Immersive mode (transparent navigation bar)

### 10.6 Avatar-Only Mode

When the left panel width drops below 300px on desktop/web in tablet mode, `NavigatorService.isAvatarOnly` returns true. This triggers a compact view showing only avatars without names or message previews in the conversation list.

### 10.7 Samsung One UI Specifics

The Samsung skin implements the One UI design language:
- Expanding/collapsing large title header (`SliverAppBar` with custom `flexibleSpace`)
- Snap-to-position scroll behavior (header snaps to expanded or collapsed state)
- Bottom footer bar instead of top action bar for multi-select actions
- `SquircleBorder` shapes for icons and avatars
- Timestamp always visible next to message bubbles (instead of iOS swipe-to-reveal)

---

## Appendix: File Index

### layouts/chat_creator/
- `chat_creator.dart` -- `ChatCreator`, `SelectedContact`, `ChatCreatorState`
- `widgets/chat_creator_tile.dart` -- `ChatCreatorTile` (contact/chat display tile)

### layouts/chat_selector_view/
- `chat_selector_view.dart` -- `ChatSelectorView` (chat picker for search filters)

### layouts/contact_selector_view/
- `contact_selector_view.dart` -- `ContactSelectorView` (contact picker)

### layouts/conversation_details/
- `conversation_details.dart` -- `ConversationDetails`
- `dialogs/add_participant.dart` -- Add member dialog
- `dialogs/address_picker.dart` -- Address selection for adding participants
- `dialogs/change_name.dart` -- Group chat rename dialog
- `dialogs/chat_sync_dialog.dart` -- Per-chat message re-sync dialog
- `dialogs/timeframe_picker.dart` -- Date range picker for search
- `widgets/chat_info.dart` -- Chat info header (avatar, name, actions)
- `widgets/chat_options.dart` -- Chat toggle options (mute, pin, archive, hide alerts)
- `widgets/contact_tile.dart` -- Participant list tile with remove action
- `widgets/media_gallery_card.dart` -- Thumbnail card for the media gallery grid

### layouts/conversation_list/
- `pages/conversation_list.dart` -- `ConversationList`, `ConversationListController`
- `pages/cupertino_conversation_list.dart` -- `CupertinoConversationList`
- `pages/material_conversation_list.dart` -- `MaterialConversationList`
- `pages/samsung_conversation_list.dart` -- `SamsungConversationList`
- `pages/search/search_view.dart` -- `SearchView`, `SearchResult`
- `dialogs/conversation_peek_view.dart` -- `ConversationPeekView`
- `widgets/conversation_list_fab.dart` -- `ConversationListFAB`
- `widgets/initial_widget_right.dart` -- `InitialWidgetRight` (placeholder for empty right panel)
- `widgets/header/cupertino_header.dart` -- iOS-style chat list header
- `widgets/header/material_header.dart` -- Material-style chat list header
- `widgets/header/samsung_header.dart` -- Samsung-style chat list header
- `widgets/header/header_widgets.dart` -- Shared header widgets (`ExpandedHeaderText`, etc.)
- `widgets/footer/samsung_footer.dart` -- Samsung bottom action bar
- `widgets/tile/conversation_tile.dart` -- `ConversationTile`, `ConversationTileController`
- `widgets/tile/cupertino_conversation_tile.dart` -- `CupertinoConversationTile`
- `widgets/tile/material_conversation_tile.dart` -- `MaterialConversationTile`
- `widgets/tile/samsung_conversation_tile.dart` -- `SamsungConversationTile`
- `widgets/tile/pinned_conversation_tile.dart` -- `PinnedConversationTile`
- `widgets/tile/pinned_tile_text_bubble.dart` -- `PinnedTileTextBubble`
- `widgets/tile/list_item.dart` -- Shared list item base

### layouts/conversation_view/
- `pages/conversation_view.dart` -- `ConversationView`
- `pages/messages_view.dart` -- `MessagesView`, `Loader`
- `dialogs/custom_mention_dialog.dart` -- Custom mention display name dialog
- `widgets/effects/screen_effects_widget.dart` -- `ScreenEffectsWidget`
- `widgets/effects/send_effect_picker.dart` -- `SendEffectPicker`
- `widgets/header/cupertino_header.dart` -- Conversation iOS header
- `widgets/header/material_header.dart` -- Conversation Material header
- `widgets/header/header_widgets.dart` -- `ConnectionIndicator`, call buttons
- `widgets/media_picker/text_field_attachment_picker.dart` -- `AttachmentPicker`
- `widgets/media_picker/attachment_picker_file.dart` -- `AttachmentPickerFile`
- `widgets/message/message_holder.dart` -- `MessageHolder`
- `widgets/message/send_animation.dart` -- `SendAnimation`
- `widgets/message/attachment/` -- Attachment viewers (image, video, audio, contact, other, sticker)
- `widgets/message/chat_event/chat_event.dart` -- `ChatEvent`
- `widgets/message/interactive/` -- Interactive message holders (URL, Apple Pay, etc.)
- `widgets/message/misc/` -- Bubble effects, message properties, sender, select, slide-to-reply, tail clipper
- `widgets/message/popup/` -- Message popup menu and reaction picker
- `widgets/message/reaction/` -- Reaction display widgets
- `widgets/message/reply/` -- Reply bubble, line painter, thread popup
- `widgets/message/text/text_bubble.dart` -- `TextBubble`
- `widgets/message/timestamp/` -- Timestamp separator, message timestamp, delivered indicator
- `widgets/message/typing/` -- Typing indicator and clipper
- `widgets/text_field/` -- Text field, picked attachments, reply holder, send button, suffix, voice recorder

### layouts/findmy/
- `findmy_page.dart` -- `FindMyPage`
- `findmy_location_clipper.dart` -- Custom clipper for location markers
- `findmy_pin_clipper.dart` -- Custom clipper for pin markers

### layouts/fullscreen_media/
- `fullscreen_holder.dart` -- `FullscreenHolder`
- `fullscreen_image.dart` -- `FullscreenImage`
- `fullscreen_video.dart` -- `FullscreenVideo`
- `dialogs/metadata_dialog.dart` -- Image/video metadata display

### layouts/handle_selector_view/
- `handle_selector_view.dart` -- `HandleSelectorView`

### layouts/settings/
- `settings_page.dart` -- `SettingsPage`
- `pages/` -- All settings panels (see Section 6)
- `widgets/content/` -- Settings content widgets (tile, switch, slider, dropdown, etc.)
- `widgets/layout/` -- Settings layout widgets (scaffold, section, header, divider)
- `dialogs/` -- Settings dialogs (theme, headers, notifications, sync)

### layouts/setup/
- `setup_view.dart` -- `SetupView`, `SetupViewController`, `SetupHeader`, `PageNumber`, `SetupPages`
- `pages/welcome/welcome_page.dart` -- `WelcomePage`
- `pages/contacts/request_contacts.dart` -- `RequestContacts`
- `pages/setup_checks/battery_optimization.dart` -- `BatteryOptimizationCheck`
- `pages/setup_checks/mac_setup_check.dart` -- `MacSetupCheck`
- `pages/sync/server_credentials.dart` -- `ServerCredentials`
- `pages/sync/qr_code_scanner.dart` -- `QRCodeScanner`
- `pages/sync/sync_settings.dart` -- `SyncSettings`
- `pages/sync/sync_progress.dart` -- `SyncProgress`
- `pages/unfinished/theme_selector.dart` -- `ThemeSelector` (commented out / unfinished)
- `dialogs/` -- Setup dialogs (connecting, failed-to-connect, failed-to-scan, manual-entry)

### layouts/startup/
- `splash_screen.dart` -- `SplashScreen`
- `failure_to_start.dart` -- `FailureToStart`

### components/
- `avatars/contact_avatar_widget.dart` -- `ContactAvatarWidget`
- `avatars/contact_avatar_group_widget.dart` -- `ContactAvatarGroupWidget`
- `circle_progress_bar.dart` -- `CircleProgressBar`
- `sliver_decoration.dart` -- `SliverDecoration`
- `custom_text_editing_controllers.dart` -- `MentionTextEditingController`, `SpellCheckTextEditingController`
- `custom/custom_bouncing_scroll_physics.dart` -- `CustomBouncingScrollPhysics`
- `custom/custom_cupertino_alert_dialog.dart` -- `CustomCupertinoAlertDialog`
- `custom/custom_cupertino_page_transition.dart` -- `CustomCupertinoPageTransition`
- `custom/custom_error_box.dart` -- `CustomErrorBox`

### wrappers/
- `titlebar_wrapper.dart` -- `TitleBarWrapper`, `TitleBar`, `WindowButtons`
- `tablet_mode_wrapper.dart` -- `TabletModeWrapper`
- `theme_switcher.dart` -- `ThemeSwitcher`
- `scrollbar_wrapper.dart` -- `ScrollbarWrapper`
- `gradient_background_wrapper.dart` -- `GradientBackground`
- `fade_on_scroll.dart` -- `FadeOnScroll`
- `cupertino_icon_wrapper.dart` -- `CupertinoIconWrapper`
- `stateful_boilerplate.dart` -- `StatefulController`, `CustomStateful`, `CustomState`, `OptimizedState`

### animations/
- `balloon_classes.dart` / `balloon_rendering.dart` -- Balloon effect (controller + canvas rendering)
- `celebration_class.dart` / `celebration_rendering.dart` -- Celebration/Lunar New Year effect
- `fireworks_classes.dart` / `fireworks_rendering.dart` -- Fireworks effect
- `laser_classes.dart` / `laser_rendering.dart` -- Laser effect
- `love_classes.dart` / `love_rendering.dart` -- Love/hearts effect
- `spotlight_classes.dart` / `spotlight_rendering.dart` -- Spotlight effect
