# 06 - Other Screens

> Specifications for Chat Creator, Contact Selector, Find My, Conversation Details, Fullscreen Media Viewer, and Handle Selector.

---

## 1. Chat Creator

**File:** `layouts/chat_creator/chat_creator.dart`

### Purpose
New conversation screen with contact selection and iMessage/SMS toggle.

### Layout
```
Scaffold
  AppBar
    Back button
    Title: "New Conversation"
    iMessage/SMS toggle (if Private API enabled)
  Body: Column
    Search/contact input field
      Text input for typing contact name/number
      Autocomplete dropdown with matching contacts
    Selected contacts row (horizontal chips)
      Each chip: Contact name + remove button
    Divider
    Expanded: Contact list
      Scrollable list of all contacts
      Each item: ContactCreatorTile (avatar + name + number)
```

### Behavior
| Action | Result |
|--------|--------|
| Type in search | Filters contact list, shows autocomplete |
| Tap contact | Adds to selected contacts row |
| Tap chip X | Removes contact from selection |
| Tap existing conversation match | Opens that conversation directly |
| Send first message | Creates chat, sends message via `onInit` callback |
| Toggle iMessage/SMS | Switches between iMessage and SMS for new chat |

### Sub-Components
| Component | Purpose |
|-----------|---------|
| `ChatCreatorTile` | Contact/chat display tile with avatar, name, address |
| `SelectedContact` | Chip showing selected contact with remove action |

### Navigation
- Opens via FAB in Conversation List
- On chat creation: Replaces route stack with `ConversationView` (zero-duration transition)
- If existing chat found: Opens existing `ConversationView`
- Back: Returns to Conversation List

### States
| State | Visual |
|-------|--------|
| Empty | Search prompt, full contact list |
| Searching | Filtered contact list |
| Contact selected | Chip row visible, can add more for group |
| Multiple selected | Group chat creation mode |
| Existing chat found | Highlighted existing conversation tile |

---

## 2. Contact Selector View

**File:** `layouts/contact_selector_view/contact_selector_view.dart`

### Purpose
Picker to select a contact from the user's contact list. Used by search filters and other UI that needs contact selection.

### Layout
```
Scaffold
  AppBar
    Back button
    Title: "Select Contact"
    Search icon
  Body: Column
    Search field (when activated)
    ListView of contacts
      Each: Avatar + display name + phone/email
```

### Behavior
- Search filters by name or address
- Tap contact -> Returns selected contact to caller
- Used as a modal route that returns a value

---

## 3. Chat Selector View

**File:** `layouts/chat_selector_view/chat_selector_view.dart`

### Purpose
Picker to select an existing chat. Used by search filters to narrow search to a specific conversation.

### Layout
```
Scaffold
  AppBar
    Back button
    Title: "Select Chat"
    Search icon
  Body: Column
    Search field
    ListView of conversations
      Each: Avatar + chat title + last message preview
```

### Behavior
- Search filters conversations by name
- Tap chat -> Returns selected chat to caller

---

## 4. Handle Selector View

**File:** `layouts/handle_selector_view/handle_selector_view.dart`

### Purpose
Picker to select a handle/address (phone number or email). Used by search filters.

### Layout
```
Scaffold
  AppBar
    Back button
    Title: "Select Handle"
    Search icon
  Body: Column
    Search field
    ListView of handles
      Each: Avatar + display name + address
```

### Behavior
- Search filters by name or address
- Tap handle -> Returns selected handle to caller

---

## 5. Find My Page

**File:** `layouts/findmy/findmy_page.dart`

### Purpose
Find My Friends/Devices map with location tracking.

### Layout
```
Scaffold
  AppBar
    Back button
    Title: "Find My"
    Refresh button
  Body: Stack
    Map widget (full screen)
      Map tiles
      Location markers for friends/devices
    Bottom sheet (draggable)
      List of friends/devices
        Each: Avatar + name + last location + time
```

### Map Features
| Feature | Description |
|---------|-------------|
| Location markers | Custom clipped pins using `FindMyPinClipper` and `FindMyLocationClipper` |
| Marker tap | Centers map on that person/device, shows details |
| Current location | User's own position indicator |
| Zoom controls | Pinch-to-zoom and zoom buttons |
| Map type | Standard, satellite, terrain |

### Data Source
- Fetches location data from BlueBubbles server
- Server must have Find My access configured
- Refresh button re-fetches locations

### States
| State | Visual |
|-------|--------|
| Loading | Map with centered spinner |
| Data loaded | Markers on map, list populated |
| No data | "No locations available" message |
| Error | Error message with retry |
| Location selected | Map centered on selected, details shown |

---

## 6. Conversation Details

**File:** `layouts/conversation_details/conversation_details.dart`

### Purpose
Chat details page showing participants, media gallery, links, and chat options.

### Layout
```
Scaffold
  AppBar
    Back button
    Title: "Details"
  Body: CustomScrollView
    SliverToBoxAdapter: ChatInfo header
      Large avatar (group or single)
      Chat name (editable for groups with Private API)
      Member count (groups)
      Action buttons row: Audio call, Video call, Info
    SliverToBoxAdapter: ChatOptions
      Mute notifications toggle
      Pin conversation toggle
      Archive toggle
      Hide alerts toggle
    SliverToBoxAdapter: Participants section header
    SliverList: Participant tiles
      Each: ContactTile (avatar + name + remove button for groups)
      "Add Member" tile (groups, with Private API)
    SliverToBoxAdapter: Media gallery section header
    SliverGrid: Media gallery thumbnails
      Each: MediaGalleryCard (thumbnail with tap to fullscreen)
    SliverToBoxAdapter: Links section header
    SliverList: Shared links
```

### Sub-Components

| Component | File | Purpose |
|-----------|------|---------|
| `ChatInfo` | `widgets/chat_info.dart` | Header with avatar, name, action buttons |
| `ChatOptions` | `widgets/chat_options.dart` | Toggle options (mute, pin, archive, hide alerts) |
| `ContactTile` | `widgets/contact_tile.dart` | Participant with remove action |
| `MediaGalleryCard` | `widgets/media_gallery_card.dart` | Thumbnail for media grid |

### Dialogs

| Dialog | Purpose |
|--------|---------|
| `AddParticipant` | Add member to group (with address picker) |
| `AddressPicker` | Select address for new participant |
| `ChangeName` | Rename group chat |
| `ChatSyncDialog` | Per-chat message re-sync |
| `TimeframePicker` | Date range for filtering |

### Chat Options Detail

| Option | Control | Description |
|--------|---------|-------------|
| Mute notifications | Switch | Mute this conversation |
| Pin conversation | Switch | Pin to top of chat list |
| Archive | Switch | Archive this conversation |
| Hide alerts | Switch | Hide notification banners |

### Participant Management (Group Chats)
- View all participants with avatars and names
- Remove participant (with Private API) -- shows confirmation
- Add participant (with Private API) -- opens add dialog
- Tap participant -> Opens contact card

### Media Gallery
- Grid of shared media thumbnails
- Tap thumbnail -> Opens fullscreen media viewer at that item
- Section header shows count of media items

---

## 7. Fullscreen Media Viewer

### FullscreenHolder
**File:** `layouts/fullscreen_media/fullscreen_holder.dart`

```
Scaffold (black background)
  AppBar (transparent, overlaid)
    Back button
    Title: filename or index
    Share button
    Info button (opens MetadataDialog)
  Body: PageView
    Pages: FullscreenImage or FullscreenVideo per media item
```

### FullscreenImage
**File:** `layouts/fullscreen_media/fullscreen_image.dart`

| Feature | Description |
|---------|-------------|
| Zoom | Pinch-to-zoom with min/max bounds |
| Pan | Drag to pan when zoomed |
| Double-tap | Toggle between 1x and 2x zoom |
| Swipe down | Dismiss fullscreen (return to conversation) |
| Swipe left/right | Navigate to prev/next media |
| Background | Black |

### FullscreenVideo
**File:** `layouts/fullscreen_media/fullscreen_video.dart`

| Feature | Description |
|---------|-------------|
| Play/Pause | Center button + bottom bar |
| Seek | Drag progress bar |
| Volume | System volume |
| Controls auto-hide | Fade after 100ms of no interaction |
| Swipe to navigate | Left/right for prev/next media |

### Metadata Dialog
**File:** `layouts/fullscreen_media/dialogs/metadata_dialog.dart`

Shows image/video metadata:
- File name
- File size
- Dimensions (images)
- Duration (videos)
- Date taken
- MIME type

### Swipe Animation
- Duration: 300ms
- Curve: easeIn
- Horizontal PageView with standard swipe physics

### Known Issues from Original
- Fullscreen dismiss handling was buggy in Flutter -- implement robust dismiss gesture
- Handle orientation changes properly during fullscreen

---

## 8. Conversation Peek View

**File:** `layouts/conversation_list/dialogs/conversation_peek_view.dart`

### Purpose
3D Touch / long-press preview of a conversation (iOS-style peek).

### Layout
```
Overlay (dimmed background)
  Centered Card (rounded corners, elevated)
    Compact conversation view
      Recent messages (last 5-10)
      No input bar
    Action bar at bottom
      Quick actions: Pin, Mute, Archive, Delete
```

### Trigger
- Long press on conversation tile (mobile)
- Force touch / 3D Touch (iOS devices)

### Behavior
- Shows preview of recent messages
- Swipe up on preview -> Show action menu
- Tap outside -> Dismiss
- Tap "Pop" or swipe further -> Open full conversation view

---

## 9. Accessibility

All screens in this section:
- Back buttons have semantic label "Go back"
- All interactive elements have labels
- Map (Find My) has text alternatives for markers
- Media gallery items announce file type and name
- Participant tiles announce contact name and role
- Fullscreen media has controls accessible by screen reader
- Peek view dismisses properly with accessibility gestures
