# 03 - Conversation View

> Detailed specification for the message thread screen, including message bubbles, reactions, replies, typing indicator, input bar, attachments, media viewer, screen effects, context menus, and scroll behavior.

---

## 1. Overview

The Conversation View displays the message thread for a single chat. It consists of a header (app bar), a scrollable message list, and an input bar at the bottom.

### Constructor Parameters
| Parameter | Type | Purpose |
|-----------|------|---------|
| `chat` | Chat (required) | The chat to display |
| `customService` | MessagesService? | Custom message service (search context) |
| `fromChatCreator` | bool | Auto-focus text field when true |
| `onInit` | void Function()? | Callback after init (ChatCreator sends initial message) |

---

## 2. Layout Structure

```
AnnotatedRegion<SystemUiOverlayStyle>
  Theme (bubble color overrides)
    PopScope (back handling: exit select mode, hide picker, pop route)
      SafeArea
        Scaffold
          appBar: Skin-dependent header (Cupertino or Material)
          body: Actions (keyboard shortcuts for reactions)
            GradientBackground (optional animated gradient)
              Stack
                ScreenEffectsWidget (fullscreen animations)
                Column
                  Expanded: Stack
                    MessagesView (scrollable message list)
                    Scroll-to-bottom FAB (animated opacity)
                  ConversationTextField (input bar)
```

---

## 3. Header (App Bar)

### iOS Header (CupertinoHeader)
- Center-aligned layout
- Avatar (ContactAvatarWidget or group avatar)
- Chat title (contact name or group name)
- Subtitle: Connection status or member count
- Right actions: Video call, Audio call buttons
- Back button: Chevron left

### Material Header
- Left-aligned layout
- Back arrow
- Avatar + title in a row
- Right actions: Video call, Audio call, More options
- Connection indicator overlay when disconnected

### Shared Header Widgets
- `ConnectionIndicator`: Shows connection status (connecting, disconnected, error)
- Call buttons: Only visible when Private API is enabled and chat supports calls

---

## 4. Messages View

### Scroll Behavior
- Reversed CustomScrollView (newest messages at bottom/visual top of list)
- Scroll physics: iOS skin = bouncing, Material/Samsung = clamping
- ScrollbarWrapper: Desktop/web gets scrollbar + middle-mouse-button scrolling

### Layout
```
DropRegion (drag-and-drop file attachment)
  GestureDetector (horizontal drag for iOS timestamp reveal)
    Stack
      AnimatedOpacity (dims during file drag)
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

### Pagination
- Initial load: First chunk of messages from MessagesService
- Scroll trigger: When user scrolls to the top of loaded messages (bottom of reversed list)
- Chunk size: 25 messages per fetch
- New message insertion: 500ms animation (SizeTransition + SlideTransition)
- End state: `noMoreMessages` flag when all messages loaded
- Loading state: `fetching` flag, shows loader indicator

### Smart Replies
- Android only: Google ML Kit `smartReply()` generates suggestions
- App-generated: "Attach recent photo", "Jump to unread"
- Displayed as horizontal scrollable chips above the message list

### Drag-and-Drop (Desktop)
- `DropRegion` wraps the entire messages view
- During drag: `dragging` RxBool = true, message list dims via AnimatedOpacity
- Overlay shows "Attach N Files" count
- On drop: Files added to attachment picker

---

## 5. Message Holder (Per Message)

### Layout Tree
```
Timestamp separator (30+ min gap between messages)
Row
  SelectCheckbox (left side, if received message in select mode)
  Expanded Column
    For each MessagePart:
      Previous edits (collapsed, expandable)
      Reply bubble (if replying to another message)
      Message sender name (group chats only)
      Reaction spacing
      Stack
        Avatar (bottom-left for received messages)
        Padded content area
          DecoratedBox (reply line painter)
            GestureDetector (tap, swipe-to-reply)
              ClipPath (TailClipper for bubble tail)
                Stack
                  Content: InteractiveHolder | TextBubble | AttachmentHolder
                  Edit overlay (inline editing indicator)
                  StickerHolder (overlaid stickers)
                  ReactionHolder (positioned top-left or top-right)
              SlideToReply indicator
      MessageProperties (reply count, edit indicator, effect name)
    DeliveredIndicator
  SelectCheckbox (right side, if sent message in select mode)
  Error icon (if send failed)
  MessageTimestamp (iOS: slide-to-reveal)
```

---

## 6. Message Bubble

### Sent Messages (isFromMe)

| Property | Value |
|----------|-------|
| Background | `primary` (or `primary.darkenAmount(0.2)` for pending) |
| Selected state background | `tertiaryContainer` |
| Text color | From BubbleColors extension or on-primary |
| Alignment | Right-aligned |
| Extra right padding | +10px |
| Max width | `screenWidth * 0.75 - 40` |

### Received Messages

| Property | Value |
|----------|-------|
| Background | `properSurface` |
| Colorful bubbles (enabled) | Contact's gradient color |
| Gradient direction | bottomCenter to topCenter |
| Custom handle color | Solid to `color.lightenAmount(0.075)` |
| Text color (colorful) | `getBubbleColors().first.oppositeLightenOrDarken(75)` |
| Alignment | Left-aligned |
| Extra left padding | +10px |

### iMessage vs SMS Distinction
| Type | Default Color | Derived Color |
|------|--------------|---------------|
| iMessage | `#1982FC` (blue) | More "colorful" of primary/primaryContainer |
| SMS | `#43CC47` (green) | Less "colorful" of primary/primaryContainer |

### Bubble Shape

| Skin | Shape | Tail |
|------|-------|------|
| iOS | Rounded rectangle + curved tail arc | Curved iMessage-style at bottom corner |
| Material | Rounded rectangle, no tail | None |
| Samsung | Rounded rectangle, no tail | None |

| Measurement | Value |
|-------------|-------|
| Standard corner radius | 20.0px |
| Connected bubble corner (shared edge) | 5.0px |
| iOS tail inner arc radius | 10.0px |
| iOS tail outer arc radius | 20.0px |
| Tail offset from bottom edge | Offset(6.547, 5.201) |
| Min bubble height | 40.0px |
| Internal padding | 10px vertical, 15px horizontal |

### Big Emoji Mode
When a message contains only emoji characters (no text), the message renders without a bubble background at full screen width with larger emoji sizing.

---

## 7. Message Grouping

Messages are grouped by sender and time proximity:

### Time Separator
- Appears when gap between messages exceeds 30 minutes
- Displays date/time in centered text
- Format: "Today 2:30 PM", "Yesterday 5:00 PM", "Mon, Jan 15 at 3:45 PM"

### Sender Grouping
- Consecutive messages from the same sender within the time threshold are visually grouped
- Connected bubbles: Reduced corner radius (5.0px) on the shared edge
- Only the last message in a group shows the bubble tail (iOS skin)
- Only the last message in a group shows the avatar (received messages)
- Sender name shown above first message in group (group chats only)

---

## 8. Reactions / Tapbacks

### Reaction Display (`ReactionHolder`)
- Positioned above the message bubble
  - Received messages: Top-right of bubble
  - Sent messages: Top-left of bubble
- Each reaction is a small circular badge with the reaction emoji
- Multiple reactions stack horizontally
- `ReactionClipper` provides the badge shape

### Available Reactions
| Reaction | Emoji |
|----------|-------|
| Love | Heart |
| Like | Thumbs up |
| Dislike | Thumbs down |
| Laugh | Ha ha |
| Emphasize | Exclamation marks |
| Question | Question mark |

### Reaction Picker
- Triggered from message popup menu (long-press/right-click)
- Horizontal bar of reaction options
- `ReactionPickerClipper` provides the bar shape
- Tap to add/remove reaction (toggles)

---

## 9. Reply Thread

### Inline Reply (`ReplyBubble`)
- Compact quoted message bubble shown above the replying message
- Shows sender name + truncated original text
- Max width: `screenWidth * 0.75 - 30`
- Min height: 30.0px
- Tap to scroll to original message

### Reply Line (`ReplyLinePainter`)
- iOS-style threading line connecting reply bubble to original
- Custom painter draws the connecting path

### Reply Thread Popup (`ReplyThreadPopup`)
- Full modal view showing the original message and all replies
- Scrollable thread view
- Can reply within the thread

### Swipe to Reply
- Horizontal swipe gesture on a message
- Shows `SlideToReply` arrow indicator during swipe
- Completing swipe sets the reply target in the text field

---

## 10. Typing Indicator

### Layout
- Three animated bouncing dots in a bubble shape
- Uses `TypingClipper` for the bubble shape (matches message bubble style)
- Positioned at the bottom of the message list (above smart replies)
- Animated with staggered bounce timing per dot

### Behavior
- Appears when the other participant is typing
- Driven by socket events (`started-typing` / `stopped-typing`)
- Auto-hides after timeout if no `stopped-typing` event received

---

## 11. Input Bar (ConversationTextField)

### Layout
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
          TextFieldComponent (actual text input)
          VoiceMessageRecorder (overlay when recording)
          SendAnimation
        TextFieldSuffix (Samsung only, outside field)
      AttachmentPicker (expanded gallery/file picker panel)
      EmojiPicker (desktop/web emoji grid)
```

### Features
| Feature | Description |
|---------|-------------|
| Draft persistence | Text and attachments saved to chat model on dispose |
| Typing indicators | Sends `started-typing` / `stopped-typing` with 3s debounce |
| Emoji shortcodes | Typing `:name` shows emoji autocomplete |
| Mentions | Typing `@name` in group chats shows participant list |
| Subject line | Optional second text field (requires Private API) |
| Voice recording | Hold-to-record with waveform visualization |
| Clipboard paste | Desktop: paste image from clipboard |
| Content insertion | Keyboard GIFs, stickers |
| Send with Return | Configurable setting |
| Auto-effects | Detects "Congratulations", "Happy Birthday", "Happy New Year", "Pew Pew" |

### Text Field Sub-Components

| Component | Purpose |
|-----------|---------|
| `TextFieldComponent` | Actual TextField with reply holder, attachments, subject |
| `ReplyHolder` | Shows message being replied to above text field |
| `PickedAttachmentsHolder` | Horizontal scroll of attachment previews |
| `PickedAttachment` | Single thumbnail with remove button |
| `TextFieldSuffix` | Send button, voice record, scheduled send |
| `SendButton` | Animated send button; long-press for effect picker |
| `VoiceMessageRecorder` | Audio waveform recorder overlay |

### Keyboard Shortcuts
| Shortcut | Action |
|----------|--------|
| Enter | Send message (when "send with Return" enabled) |
| Shift+Enter | New line |
| Escape | Dismiss emoji/mention popup, clear reply |
| Arrow Up/Down | Navigate emoji/mention suggestions |
| Tab | Switch between subject and message fields |
| Up Arrow (empty field) | Edit last sent message |

---

## 12. Attachment Previews

### Inline Attachment Types

| Type | Widget | Description |
|------|--------|-------------|
| Image | `ImageViewer` | Inline thumbnail, tap for fullscreen |
| Video | `VideoPlayer` | Inline player with controls |
| Audio | `AudioPlayer` | Waveform visualization with play/pause |
| Contact | `ContactCard` | vCard display with name and details |
| Location | (in InteractiveHolder) | Map preview |
| Generic file | `OtherFile` | File icon + name + download/open |
| Sticker | `StickerHolder` | Overlaid on top of message bubble |

### Interactive Message Types

| Type | Widget | Description |
|------|--------|-------------|
| URL Preview | `UrlPreview` | Rich link card with title, description, image |
| URL Preview (legacy) | `UrlPreviewLegacy` | Fallback for older data |
| Apple Pay | `ApplePay` | Transaction display |
| Embedded Media | `EmbeddedMedia` | Apple Music, etc. |
| Game Pigeon | `GamePigeon` | Game interactive message |
| Supported Interactive | `SupportedInteractive` | Known interactive types |
| Unsupported Interactive | `UnsupportedInteractive` | Fallback for unknown types |

---

## 13. Fullscreen Media Viewer

### FullscreenHolder
- PageView for swiping through media gallery
- Horizontal swipe navigation between media items
- Swipe animation: 300ms, easeIn curve

### FullscreenImage
- Pinch-to-zoom and pan
- Double-tap to zoom in/out
- Swipe down to dismiss
- Metadata dialog accessible

### FullscreenVideo
- Full playback controls
- Controls auto-hide: 100ms animation
- Play/pause, seek, volume
- Fullscreen toggle

### Known Issues (from original app)
- Fullscreen dismiss handling had bugs in the Flutter version -- ensure proper dismiss gesture handling in the rewrite
- Need robust handling of orientation changes during fullscreen

---

## 14. Message Effects

### Bubble Effects (`BubbleEffects`)
| Effect | Description | Animation |
|--------|-------------|-----------|
| Slam | Message slams down with impact | Scale + position tween |
| Loud | Message grows and shakes | Scale oscillation |
| Gentle | Message shrinks then grows | 3-phase MovieTween: hold(0-1ms) -> shrink(1-500ms, scale 1.0->0.5) -> grow(1000-1800ms, scale 0.5->1.0), easeInOut |
| Invisible Ink | Message obscured by particle noise | Noise overlay, tap to reveal |

### Screen Effects (`ScreenEffectsWidget`)
Fullscreen Canvas-rendered animations:

| Effect | Controller | Renderer | Auto-launch Delay |
|--------|-----------|----------|-------------------|
| Fireworks | `FireworkController` | `FireworkRendering` | 100ms |
| Celebration | `CelebrationController` | `CelebrationRendering` | 100ms |
| Confetti | `ConfettiController` | confetti package | 100ms |
| Balloons | `BalloonController` | `BalloonRendering` | 100ms |
| Love | `LoveController` | `LoveRendering` | 100ms |
| Spotlight | `SpotlightController` | `SpotlightRendering` | 100ms |
| Lasers | `LaserController` | `LaserRendering` | 500ms |

### Send Effect Picker (`SendEffectPicker`)
- UI for selecting a send effect before sending
- Triggered by long-press on send button
- Grid of effect options with preview animations

### Auto-Effect Detection
The text field automatically detects trigger phrases and applies effects:
| Phrase | Effect |
|--------|--------|
| "Congratulations" | Confetti/Celebration |
| "Happy Birthday" | Balloons |
| "Happy New Year" | Fireworks |
| "Pew Pew" | Lasers |

---

## 15. Context Menu (Message Popup)

### Trigger
- Long press (mobile)
- Right-click (desktop)

### Layout (`MessagePopup`)
```
Overlay
  Blurred/dimmed background
  Positioned bubble (original message, slightly enlarged)
  ReactionPicker bar (top, with reaction options)
  Action menu (below bubble)
    List of DetailsMenuAction items
```

### Available Actions

| Action | Description | Condition |
|--------|-------------|-----------|
| Reply | Set reply target | Always |
| Copy | Copy message text | Text messages |
| Copy Selection | Open text selection | Text messages |
| Forward | Forward to another chat | Always |
| Share | System share sheet | Mobile only |
| Save | Save attachment to device | Attachments only |
| Remind Me | Set reminder notification | Android only |
| Edit | Inline edit message | Own messages, Private API |
| Unsend | Remove message | Own messages, Private API |
| Delete | Delete locally | Always |
| Select More | Enter multi-select | Always |
| Message Info | Show delivery details | Always |

### Message Options Order
Actions can be reordered via the `MessageOptionsOrderPanel` in settings.

---

## 16. Scroll-to-Bottom FAB

- Circular button with down-arrow icon
- Positioned above the input bar, right-aligned
- Visibility: AnimatedOpacity, appears when scrolled up from bottom
- Tap: Scrolls to the newest message
- Badge: Shows unread count when messages received while scrolled up

---

## 17. Read Receipts

### Delivered Indicator (`DeliveredIndicator`)
- Shown below the last sent message
- States:
  - "Delivered" -- Message delivered to recipient
  - "Read" -- Message read by recipient
  - "Read [time]" -- Time since read (e.g., "Read 5m ago")
- Only visible on the most recent sent message

---

## 18. Chat Events

`ChatEvent` widget displays group chat system events:
- Member added
- Member removed
- Group name changed
- Group photo changed
- Styled as centered, muted text (not in a bubble)

---

## 19. Gradient Background

When theme has `gradientBg: true`:
- Gradient direction: topRight to bottomLeft
- Color 1: Bubble color (iMessage or SMS based on chat type) at 50% opacity
- Color 2: Background color at full opacity
- Animation: MirrorAnimationBuilder, fastOutSlowIn curve, 3-second duration
- Gradient stops: Animate between [0.0, 0.8] and [0.2, 1.0]

---

## 20. Keyboard Shortcuts (Desktop)

### Reaction Shortcuts (when Private API enabled)
| Shortcut | Action |
|----------|--------|
| (configurable) | Reply to most recent message |
| (configurable) | Heart react to most recent |
| (configurable) | Like react to most recent |
| (configurable) | Dislike react to most recent |
| (configurable) | Laugh react to most recent |
| (configurable) | Emphasize react to most recent |
| (configurable) | Question react to most recent |
| (configurable) | Open chat details |

---

## 21. Accessibility

- All messages have semantic labels (sender + content + timestamp)
- Reactions announced by screen reader
- Reply context announced
- Typing indicator announced
- Send button has semantic label
- Attachment buttons have labels
- Message popup actions have labels
- Scroll-to-bottom button has semantic label
- Multi-select checkboxes have proper labels
