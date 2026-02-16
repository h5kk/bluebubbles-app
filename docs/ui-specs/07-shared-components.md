# 07 - Shared Components

> Specifications for all reusable components: avatars, title bar, navigation, dialogs, toasts, loaders, error states, context menus, dropdowns, toggles, scrollbars, and keyboard shortcuts.

---

## 1. Avatar System

### ContactAvatarWidget

**File:** `components/avatars/contact_avatar_widget.dart`

#### Parameters
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `handle` | Handle? | null | Handle to display avatar for |
| `contact` | Contact? | null | Direct contact reference |
| `size` | double? | 40 | Avatar diameter (scaled by `avatarScale`) |
| `fontSize` | double? | 18 | Initials font size |
| `borderThickness` | double | 2.0 | Border width |
| `editable` | bool | true | Tap opens contact form |
| `scaleSize` | bool | true | Apply `avatarScale` setting |
| `preferHighResAvatar` | bool | false | Request higher-res image |
| `padding` | EdgeInsets | zero | Internal padding |

#### Rendering Priority
1. User avatar path (self-avatar when handle is null)
2. Contact photo from device contacts
3. Initials text
   - iOS skin: Full initials (first + last)
   - Material/Samsung: First letter only
4. Person icon fallback (generic silhouette)

#### Color Logic
- `colorfulAvatars` enabled: Gradient from address hash (7 palettes) or custom `Handle.color`
- `colorfulAvatars` disabled: Gray gradient `#928E8E` to `#686868`
- iOS skin: Reversed gradient order (colors[1], colors[0])
- Material/Samsung: Standard order (colors[0], colors[0])
- Long-press: Opens color picker dialog

#### Color Picker
- Wheel diameter: 165px
- Min dialog height: 480px
- Dialog side padding: 70px from each side
- Uses `flex_color_picker` package

### ContactAvatarGroupWidget

**File:** `components/avatars/contact_avatar_group_widget.dart`

#### Parameters
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `chat` | Chat (required) | -- | The group chat |
| `size` | double | 40 | Overall widget size |
| `editable` | bool | true | Individual avatars editable |

#### Layout Logic
| Participant Count | iOS Skin | Material/Samsung Skin |
|-------------------|----------|----------------------|
| 1 | Single ContactAvatarWidget | Single ContactAvatarWidget |
| 2-4 | Circular arrangement (sin/cos positioning) | Grid arrangement (predefined alignment maps) |
| 5+ | Circle + last position = group icon with blur | Grid, last position = group icon |

#### Custom Group Avatar
If `chat.customAvatarPath` is set, displays that image instead of the generated layout.

---

## 2. Title Bar / App Bar

### TitleBarWrapper

**File:** `wrappers/titlebar_wrapper.dart`

Wraps all top-level pages. Desktop-only rendering of custom window title bar.

#### Layout (Desktop)
```
Column
  TitleBar
    Row
      MoveWindow (drag area, fills available space)
      WindowButtons
        MinimizeButton (respects minimizeToTray setting)
        MaximizeButton (toggles maximize/restore)
        CloseButton (themed with errorContainer hover color)
  ConnectionIndicator (optional overlay)
  Content (child widget)
```

#### Layout (Mobile/Web)
```
ConnectionIndicator (optional overlay)
Content (child widget)
```

### Window Buttons Styling
| Button | Normal | Hover |
|--------|--------|-------|
| Minimize | Standard icon | Subtle highlight |
| Maximize | Standard icon | Subtle highlight |
| Close | Standard icon | `errorContainer` background, `onErrorContainer` icon |

### App Bar Styling
| Property | Value |
|----------|-------|
| Background | `headerColor` (skin-dependent) |
| Title style | `titleLarge` |
| Elevation | 0 (flat) |
| Scrolled-under elevation | 3.0 |
| Surface tint | `primary` |
| Center title | Only on iOS skin |
| System overlay | Inverted brightness (dark icons on light bg) |

### App Bar Height
- Standard: 50px
- Samsung expanded: Variable (expands to `screenHeight / 3 - 57` snap point)

---

## 3. Navigation Systems

### Sidebar (Desktop Tablet Mode)
- Left panel in `TabletModeWrapper`
- Contains conversation list
- Width: Split ratio * screen width (default 50%)
- Min width before avatar-only mode: 300px
- Divider: 7.0px wide, draggable, shows three dots

### Bottom Tab Navigation
- Not used as primary navigation (settings use push navigation)
- Samsung skin uses `SamsungFooter` as bottom bar during multi-select

### Drawer
- Not used in current design (TabletModeWrapper replaces drawer pattern)

### Navigator Stacks
| Key | Purpose | Context |
|-----|---------|---------|
| 1 | Conversation list (left panel) | Tablet mode |
| 2 | Conversation view (right panel) | Tablet mode |
| 3 | Settings detail panel | Settings tablet mode |

### Page Transitions
| Skin | Forward | Reverse |
|------|---------|---------|
| iOS | CustomCupertinoPageTransition (slide right + parallax) | Slide back left |
| Material | MaterialPageRoute (slide up) | Slide down |
| Samsung | MaterialPageRoute (slide up) | Slide down |

---

## 4. Dialogs and Modals

### Standard Dialog Pattern
```
Dialog
  Title text
  Content area
    Description or form fields
  Action row
    Cancel button (text)
    Confirm button (filled)
```

### Platform-Specific Dialogs
| Skin | Dialog Style |
|------|-------------|
| iOS | `CustomCupertinoAlertDialog` (rounded, translucent background) |
| Material | Standard Material AlertDialog |
| Samsung | Standard Material AlertDialog |

### Cupertino Dialog Specifics
- Inset animation: 100ms, `Curves.decelerate`
- Modified from standard Cupertino dialog

### Common Dialogs
| Dialog | Context | Content |
|--------|---------|---------|
| Confirmation | Destructive actions | "Are you sure?" + Cancel/Confirm |
| Error | Connection/sync failures | Error message + OK/Retry |
| Input | Theme name, server URL | Text field + Cancel/Save |
| Progress | Connecting, syncing | Spinner + status text |
| Color picker | Avatar/theme colors | Color wheel + hex input |

---

## 5. Toast / Snackbar Notifications

### Styling
- Background: `inverseSurface`
- Text color: `onInverseSurface`
- Position: Bottom of screen
- Duration: Auto-dismiss after 3-4 seconds
- Action button: Optional (e.g., "Undo")

### Usage Contexts
- Message sent confirmation
- Chat archived/unarchived
- Chat pinned/unpinned
- Settings saved
- Error messages
- Copy confirmation

---

## 6. Loading Spinners

### CircleProgressBar
**File:** `components/circle_progress_bar.dart`

- Circular progress indicator with animated fill
- Used in: Sync progress, file downloads
- Animation curve: `Curves.easeInOut`
- Supports determinate (percentage) and indeterminate modes

### Standard Spinner
- Platform `CircularProgressIndicator`
- Used in: Dialog loading states, list loading
- Color: `primary`

---

## 7. Error States

### Global Error Box
**File:** `components/custom/custom_error_box.dart`
- Replaces Flutter's default red error box
- Styled to match app theme

### Error Screen (Failure to Start)
- Full-screen error display
- Shows error message and stack trace
- "Report" or "Retry" buttons

### Inline Error States
| Context | Error Display |
|---------|--------------|
| Message send failed | Red error icon next to message |
| Connection lost | ConnectionIndicator overlay |
| Image load failed | Placeholder with broken image icon |
| Server error | Error text with retry button |

---

## 8. Context Menus

### Message Context Menu
- Trigger: Long press (mobile) / Right-click (desktop)
- See `03-conversation-view.md` section 15 for full details

### Conversation Tile Context Menu
- Trigger: Right-click (desktop) / Long press (mobile)
- Actions: Pin, Mute, Mark Read, Archive, Delete, Open in New Window

### General Desktop Context Menu
- Right-click prevention on web (`html.document.onContextMenu`)
- Custom context menus replace browser defaults

---

## 9. Dropdown Menus

### Settings Dropdown (`SettingsDropdown`)
- Standard Material dropdown selector
- Full-width within settings tile
- Shows current value as subtitle

### Overflow Menu (AppBar)
- Three-dot icon button
- Opens popup menu below/above the icon
- Used in conversation list headers and conversation view headers

### Theme Selector Dropdown
- Shows all themes divided into three groups by dividers:
  1. Custom/base themes (no sun/moon suffix)
  2. Themes matching current tab brightness
  3. Opposite brightness themes

---

## 10. Toggle Switches

### SettingsSwitch
- Reactive binding to `Rx<bool>` state
- Standard Material Switch widget
- Active color: `primary`
- Track color: follows Material 3 switch styling
- Leading icon: `SettingsLeadingIcon` with colored background

### Checkbox (Multi-select)
- `SelectCheckbox` in message select mode
- Circular checkbox style
- Active: `primary` fill
- Inactive: `outline` border

---

## 11. Scrollbar Styling

### ScrollbarWrapper
**File:** `wrappers/scrollbar_wrapper.dart`

#### Mobile
- No visible scrollbar (renders child directly)

#### Desktop/Web
```
ImprovedScrolling (middle-mouse-button support)
  RawScrollbar (optional visibility)
    Child content
```

| Property | Value |
|----------|-------|
| Scrollbar visibility | Configurable (`showScrollbar` parameter) |
| Middle-mouse scrolling | Enabled on desktop/web |
| Tab key handling | Redirects focus back to active chat text field |
| Reverse MMB | Configurable for reversed scroll direction |

### Scroll Physics Per Skin
| Skin | Physics |
|------|---------|
| iOS | `CustomBouncingScrollPhysics` (always scrollable, bouncing overscroll) |
| Material | `ClampingScrollPhysics` (standard Material overscroll glow) |
| Samsung | `ClampingScrollPhysics` (standard Material overscroll glow) |

---

## 12. Keyboard Shortcuts

### Global Shortcuts (Desktop)
| Shortcut | Action |
|----------|--------|
| Ctrl+N | New conversation (ChatCreator) |
| Ctrl+F | Focus search |
| Escape | Exit current modal/mode |

### Conversation View Shortcuts (Desktop, with Private API)
| Intent | Action |
|--------|--------|
| `ReplyRecentIntent` | Reply to most recent message |
| `HeartRecentIntent` | Heart react to most recent |
| `LikeRecentIntent` | Like react to most recent |
| `DislikeRecentIntent` | Dislike react to most recent |
| `LaughRecentIntent` | Laugh react to most recent |
| `EmphasizeRecentIntent` | Emphasize react to most recent |
| `QuestionRecentIntent` | Question react to most recent |
| `OpenChatDetailsIntent` | Open conversation details |

### Text Field Shortcuts
| Shortcut | Action |
|----------|--------|
| Enter | Send (when configured) |
| Shift+Enter | New line |
| Escape | Dismiss popup / clear reply |
| Arrow Up (empty) | Edit last sent message |
| Arrow Up/Down | Navigate autocomplete suggestions |
| Tab | Switch between subject/message fields |

---

## 13. FadeOnScroll Wrapper

**File:** `wrappers/fade_on_scroll.dart`

### Parameters
| Parameter | Type | Purpose |
|-----------|------|---------|
| `scrollController` | ScrollController (required) | Scroll source |
| `child` | Widget (required) | Widget to fade |
| `zeroOpacityOffset` | double | Scroll offset where opacity = 0 |
| `fullOpacityOffset` | double | Scroll offset where opacity = 1 |

Used for header elements that fade as user scrolls.

---

## 14. GradientBackground Wrapper

**File:** `wrappers/gradient_background_wrapper.dart`

### Behavior
- Active when theme has `gradientBg: true` (checked via `ts.isGradientBg`)
- Gradient direction: topRight to bottomLeft
- Color 1: Bubble color at 50% opacity (iMessage or SMS based on chat type)
- Color 2: Background color at 100% opacity
- Animation: `MirrorAnimationBuilder`, `Curves.fastOutSlowIn`, 3000ms duration
- Stops animate: [0.0, 0.8] <-> [0.2, 1.0]
- Responds to platform brightness changes

---

## 15. ThemeSwitcher Wrapper

**File:** `wrappers/theme_switcher.dart`

### Parameters
| Parameter | Type | Required |
|-----------|------|----------|
| `iOSSkin` | Widget | Yes |
| `materialSkin` | Widget | Yes |
| `samsungSkin` | Widget? | No (falls back to materialSkin) |

### Static Methods
| Method | Returns |
|--------|---------|
| `buildPageRoute<T>(builder)` | Skin-appropriate PageRoute |
| `getScrollPhysics()` | Skin-appropriate ScrollPhysics |

---

## 16. Custom Scroll Physics

### CustomBouncingScrollPhysics
**File:** `components/custom/custom_bouncing_scroll_physics.dart`

- iOS-style bouncing with custom overscroll behavior
- Always scrollable (even when content doesn't fill viewport)
- Used exclusively with iOS skin

---

## 17. CupertinoIconWrapper

**File:** `wrappers/cupertino_icon_wrapper.dart`

- Adds 1px left padding when iOS skin is active
- Corrects visual alignment differences between Cupertino and Material icons
- Transparent wrapper on non-iOS skins
