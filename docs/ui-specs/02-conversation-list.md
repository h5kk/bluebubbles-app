# 02 - Conversation List

> Detailed specification for the main chat list screen, including all three skin variants, conversation tiles, search, FAB, swipe actions, pinned chats, and all interaction states.

---

## 1. Overview

The Conversation List is the primary screen of the application. It displays all active chats sorted by most recent message. It supports three visual modes based on the active skin (iOS, Material, Samsung), each with distinct layout and interaction patterns.

### Controller State

| State Field | Type | Purpose |
|-------------|------|---------|
| `showArchivedChats` | bool | When true, shows archived chats instead of main list |
| `showUnknownSenders` | bool | When true, shows chats from unknown senders |
| `selectedChats` | List<Chat> | Tracks multi-select for batch actions |
| `showMaterialFABText` | bool | Controls FAB extended text visibility |
| `materialScrollStartPosition` | double | Tracks scroll for FAB hide/show |

### Modes
The list operates in three modes, each registered with a different tag:
- `"Messages"` -- Default main chat list
- `"Archived"` -- Archived chats view
- `"Unknown"` -- Unknown senders view

---

## 2. Layout Structure

### Phone Mode
```
AnnotatedRegion<SystemUiOverlayStyle>
  TabletModeWrapper
    left: ThemeSwitcher(iOS | Material | Samsung skin)
    right: Navigator(key: 2) with InitialWidgetRight
```

### Tablet Mode
```
AnnotatedRegion
  TabletModeWrapper
    left: Navigator(key: 1)
      CupertinoPage(child: ThemeSwitcher(...))
    right: Navigator(key: 2)
      CupertinoPage(child: InitialWidgetRight)
```

### Avatar-Only Mode
When the left panel width drops below 300px in tablet mode, a compact view shows only avatars without names or message previews.

---

## 3. Skin Variants

### 3.1 iOS (Cupertino) Conversation List

**Layout:**
```
CustomScrollView
  SliverToBoxAdapter: CupertinoHeader (large title, search bar)
  SliverGrid: PinnedConversationTile grid (paginated with page dots)
  SliverList: ConversationTile widgets (unpinned chats)
  ConversationListFAB (floating)
```

**Header (CupertinoHeader):**
- Large title "Messages" that scrolls with content
- Search bar below title
- Overflow menu (three dots) with options:
  - Select messages (multi-select)
  - Open archived chats
  - Open unknown senders
  - Mark all as read

**Pinned Chats:**
- Rendered as avatar-centric tiles in a paginated grid
- Each tile shows avatar + small text bubble with latest message preview (`PinnedTileTextBubble`)
- Supports drag-and-drop reordering
- Page dots indicator for multiple pages of pins

**Swipe Actions (per tile, via Dismissible):**
- Swipe right: Archive / Unarchive
- Swipe left: Delete
- Additional actions: Pin/Unpin, Mark read/unread, Mute/Unmute

**Interaction:**
- Tap: Open conversation
- Long press: Enter multi-select OR show conversation peek view (3D Touch-style)
- Right-click (desktop): Context menu

### 3.2 Material Conversation List

**Layout:**
```
Scaffold
  AppBar: MaterialHeader (search, camera, overflow menu)
  Body: ListView of ConversationTile widgets
  FAB: ConversationListFAB (extended, collapses on scroll)
```

**Header (MaterialHeader):**
- Standard Material AppBar
- Left-aligned title "Messages"
- Action icons: Search, Camera (mobile), Overflow menu
- During multi-select: Contextual action bar replaces title
  - Shows selection count
  - Actions: Archive, Delete, Mark read, Pin, Mute

**FAB Behavior:**
- Extended FAB with compose icon + "Start chat" text
- On scroll down: Text fades out, FAB collapses to icon-only
- On scroll up: Text fades back in

**Interaction:**
- Tap: Open conversation
- Long press: Toggle multi-select (checkbox appears on each tile)
- Right-click (desktop): Context menu

### 3.3 Samsung Conversation List

**Layout:**
```
Scaffold
  Body: CustomScrollView
    SliverAppBar: SamsungHeader (expanding/collapsing large title)
    SliverList: ConversationTile widgets
  BottomBar: SamsungFooter (contextual during multi-select)
  FAB: ConversationListFAB (circular)
```

**Header (SamsungHeader):**
- Large expanding/collapsing Samsung One UI style header
- Scroll snap behavior: Snaps to expanded or collapsed state
- Snap threshold: `screenHeight / 3 - 57`
- Collapsed: Standard-height app bar with title
- Expanded: Large title area with extra whitespace above

**Footer (SamsungFooter):**
- Bottom navigation bar
- During multi-select: Shows batch action buttons (archive, delete, mark read, pin, mute)

**Interaction:**
- Same as Material (tap, long press, right-click)
- Rounded tile shapes (Samsung-specific styling)

---

## 4. Conversation Tile

### Controller State

| Field | Type | Purpose |
|-------|------|---------|
| `shouldHighlight` | RxBool | Active chat highlight (tablet mode) |
| `shouldPartialHighlight` | RxBool | Right-click context menu highlight |
| `hoverHighlight` | RxBool | Mouse hover highlight (desktop) |
| `chat` | Chat | The chat model |
| `listController` | ConversationListController | Parent controller reference |

### Tile Layout

```
Row
  Avatar (ContactAvatarWidget or ContactAvatarGroupWidget)
  Expanded Column
    Row
      Chat title (contact name / group name)
      Spacer
      Timestamp (relative time)
    Row
      Message preview text (truncated)
      Spacer
      Status indicators
        Unread badge (blue dot or count)
        Pin icon (if pinned)
        Mute icon (if muted)
```

### Tile Content Details

| Element | Description | Styling |
|---------|-------------|---------|
| Avatar | 40px default, scaled by `avatarScale` setting | Circular, gradient background |
| Title | Contact display name or group name | `bodyLarge` weight |
| Timestamp | Relative time ("2m", "1h", "Yesterday") | `bodySmall`, muted color |
| Preview | Last message text, truncated to 2 lines | `bodyMedium`, muted color |
| Unread badge | Circular indicator | `primary` color |
| Pin icon | Pushpin icon | Muted color |
| Mute icon | Bell-slash icon | Muted color |

### Skin-Specific Tile Variants

| Aspect | iOS | Material | Samsung |
|--------|-----|----------|---------|
| Widget | `CupertinoConversationTile` | `MaterialConversationTile` | `SamsungConversationTile` |
| Swipe | Dismissible with actions | No swipe (multi-select instead) | No swipe (multi-select instead) |
| Ripple | None | InkSparkle | InkSparkle |
| Shape | Standard rectangle | Standard rectangle | Rounded rectangle |
| Highlight | Background color change | Ripple + background | Ripple + background |

### Tile Interaction

| Action | Behavior |
|--------|----------|
| Tap | Opens ConversationView via `ns.pushAndRemoveUntil` or focuses existing chat |
| Secondary tap (right-click) | Shows context menu |
| Long press | Toggles multi-select mode |
| Hover (desktop) | `hoverHighlight` triggers subtle background change |

### Tile States

| State | Visual |
|-------|--------|
| Default | Standard background |
| Hover | Subtle highlight (`hoverHighlight`) |
| Active (tablet) | `shouldHighlight` -- stronger background tint |
| Context menu open | `shouldPartialHighlight` -- medium highlight |
| Selected (multi-select) | Checkbox visible, `tertiaryContainer` background |
| Unread | Bold title text, unread badge visible |
| Muted | Mute icon visible, badge may be hidden |
| Pinned | Pin icon visible |

---

## 5. Pinned Conversation Tile

**File:** `pinned_conversation_tile.dart`

### Layout
- Avatar-centric: Large circular avatar as the main visual
- Small text bubble below/beside avatar showing latest message (`PinnedTileTextBubble`)
- Paginated grid layout with page dots
- Drag-and-drop reorderable (iOS skin)

### PinnedTileTextBubble
- Compact message preview in a small rounded bubble
- Respects `colorfulBubbles` and `colorfulAvatars` settings
- Truncated to single line
- Uses `tertiaryContainer` for mute/unmute status indicator

---

## 6. Search

**File:** `search_view.dart`

### Features
- Text search input with debounce (250ms)
- Toggle between local database search and network (server) search
- Filter by specific chat (via `ChatSelectorView`)
- Filter by specific handle (via `HandleSelectorView`)
- Filter by sender ("from me" toggle)
- Date range filter (via `TimeframePicker`)
- Results displayed as chat + message tuples
- Tapping a result opens ConversationView scrolled to that message
- Previous search history maintained in `pastSearches` list

### Layout
```
Column
  Search input field
  Row: Filter toggles (local/network, from me)
  Row: Active filter chips (chat, handle, date range)
  Expanded: ListView of search results
    Each result: chat tile + message preview snippet
```

### States
| State | Visual |
|-------|--------|
| Empty (no query) | Search history list or empty prompt |
| Loading | Circular progress indicator |
| Results | List of matching messages with context |
| No results | "No messages found" text |
| Error | Error message with retry option |

---

## 7. Floating Action Button (FAB)

**File:** `conversation_list_fab.dart`

### Skin Variants

| Skin | FAB Style | Behavior |
|------|-----------|----------|
| iOS | Circular compose button | Static position |
| Material | Extended FAB (icon + "Start chat" text) | Text collapses on scroll down, expands on scroll up |
| Samsung | Circular FAB | Static position |

### FAB Styling
- Background: `primary`
- Icon/label color: `onPrimary`
- Label style: `labelLarge`

### Multi-Select Mode
During multi-select, the FAB transforms to show batch action icons:
- Archive
- Delete
- Mark read/unread
- Pin/Unpin
- Mute/Unmute

### Actions
| Action | Result |
|--------|--------|
| Tap (normal) | Opens ChatCreator (new conversation) |
| Tap (multi-select) | Executes selected batch action |

---

## 8. Context Menu

Right-click (desktop) or long-press shows a context menu for a conversation tile:

| Action | Description |
|--------|-------------|
| Pin / Unpin | Toggle pinned state |
| Mute / Unmute | Toggle mute state |
| Mark Read / Unread | Toggle read state |
| Archive / Unarchive | Move to/from archive |
| Delete | Delete conversation |
| Open in New Window | Desktop only: opens chat in separate window |

---

## 9. Pull-to-Refresh

- Available on mobile platforms
- Triggers a sync with the server to fetch new messages
- Shows standard platform pull-to-refresh indicator
- On iOS skin: Bouncing overscroll naturally accommodates the gesture
- On Material/Samsung: Standard overscroll indicator

---

## 10. Sort and Filter Options

Accessible via Chat List Settings panel:

| Option | Description |
|--------|-------------|
| Sort by | Date (default), alphabetical |
| Filter unknown senders | Separate unknown senders to their own list |
| Filter archived | Archived chats in separate view |
| Show unread count | Badge showing unread message count vs simple dot |

---

## 11. Empty State

When no conversations exist:
- Center-aligned message: "No conversations"
- Subtitle: "Start a new conversation using the compose button"
- Compose FAB visible and prominent

---

## 12. Loading State

- On initial load: Chat list loads from local database first (immediate display)
- Web platform: Async chat loading via `chats.loadedAllChats.future`
- During sync: Connection indicator may show at top of screen
- Individual tiles: Avatars load progressively (contact photo fetch)

---

## 13. Accessibility

- All interactive tiles have semantic labels (chat name + last message)
- Multi-select checkboxes have proper labels
- FAB has semantic label "Compose new message"
- Search field has proper placeholder text and label
- Swipe actions (iOS) have semantic action descriptions
- Keyboard navigation: Tab through tiles, Enter to open, Space to select
- Screen reader: Announces unread count, pinned status, muted status

---

## 14. Keyboard Shortcuts (Desktop)

| Shortcut | Action |
|----------|--------|
| Ctrl+N | Open new chat creator |
| Ctrl+F | Focus search |
| Escape | Exit multi-select mode / close search |
| Arrow Up/Down | Navigate through chat list |
| Enter | Open selected chat |
