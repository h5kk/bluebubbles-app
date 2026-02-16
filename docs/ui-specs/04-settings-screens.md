# 04 - Settings Screens

> Complete specification for every settings page, section, and individual setting control in the application.

---

## 1. Settings Page Structure

The main settings page uses a `TabletModeWrapper` with the settings list on the left and a detail panel on the right (navigator key 3). In phone mode, each section pushes a new route.

### Layout
```
TabletModeWrapper
  left: SettingsScaffold
    AppBar (iOS/Material) or expanding header (Samsung)
    SliverList of SettingsSection groups
      SettingsTile items (navigational, each opens a detail panel)
  right: Navigator(key: 3)
    Selected settings panel
```

### Settings Widget Library

#### Layout Widgets
| Widget | Purpose |
|--------|---------|
| `SettingsScaffold` | Standard settings page scaffold with AppBar/header, scrollbar, sliver body |
| `SettingsSection` | Grouped section container with background color |
| `SettingsHeader` | Section header text with skin-appropriate styling |
| `SettingsDivider` | Horizontal divider between tiles |

#### Content Widgets
| Widget | Purpose |
|--------|---------|
| `SettingsTile` | Standard row: title, subtitle, leading icon, trailing widget, onTap |
| `SettingsSwitch` | Toggle switch with reactive `Rx<bool>` binding |
| `SettingsSlider` | Slider with min/max/divisions, reactive `Rx<double>` binding |
| `SettingsDropdown` | Dropdown selector |
| `SettingsSubtitle` | Subtitle text for secondary descriptions |
| `SettingsLeadingIcon` | Colored icon container (different icons per skin) |
| `NextButton` | Chevron-right trailing indicator |
| `LogLevelSelector` | Log level dropdown |
| `AdvancedThemingTile` | Color-preview tile for theme customization |

#### Settings Tile Styling

| Skin | Subtitle Style | Color Behavior |
|------|---------------|----------------|
| iOS | `labelLarge`, w300, muted color (adapts to window effects) | Standard |
| Material | `labelLarge`, bold, `primary` color | Header/tile colors swap in dark mode |
| Samsung | `labelLarge`, bold, `primary` color | Standard |

---

## 2. Settings Organization Hierarchy

### Profile Section
**Condition:** Non-web, Material/Samsung skin only
- User profile tile with avatar -> `ProfilePanel`

### Server & Message Management
| Setting | Target Panel | Condition |
|---------|-------------|-----------|
| Connection & Server | `ServerManagementPanel` | Always |
| Scheduled Messages | `ScheduledMessagesPanel` | Server version >= 205 |
| Message Reminders | `MessageRemindersPanel` | Android only |

### Appearance
| Setting | Target Panel |
|---------|-------------|
| Appearance Settings | `ThemingPanel` |

### Application Settings
| Setting | Target Panel | Condition |
|---------|-------------|-----------|
| Media Settings | `AttachmentPanel` | Always |
| Notification Settings | `NotificationPanel` | Always |
| Chat List Settings | `ChatListPanel` | Always |
| Conversation Settings | `ConversationPanel` | Always |
| Desktop Settings | `DesktopPanel` | Desktop only |
| More Settings | `MiscPanel` | Always |

### Advanced
| Setting | Target Panel | Condition |
|---------|-------------|-----------|
| Private API Features | `PrivateAPIPanel` | Always |
| Redacted Mode | `RedactedModePanel` | Always |
| Tasker Integration | `TaskerPanel` | Android only |
| Notification Providers | `NotificationProvidersPanel` | Always |
| Developer Tools | `TroubleshootPanel` | Always |

### Backup and Restore
| Setting | Target Panel | Condition |
|---------|-------------|-----------|
| Backup & Restore | `BackupRestorePanel` | Always |
| Export Contacts | Direct action | Non-web, non-desktop |

### About & Links
| Item | Action | Condition |
|------|--------|-----------|
| Leave Us a Review | External link | Android/Windows |
| Make a Donation | External link | Always |
| Join Our Discord | External link | Always |
| About & More | `AboutPanel` | Always |

### Danger Zone
| Action | Description | Condition |
|--------|-------------|-----------|
| Delete All Attachments | Removes all cached attachments | Non-web |
| Reset App / Logout | Resets all settings and data (web: logout only) | Always |

---

## 3. Individual Settings Panels

### 3.1 Server Management Panel (`ServerManagementPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Server URL | Text input | URL of the BlueBubbles server |
| Server password | Password input | Authentication password |
| Connection status | Status indicator | Shows connected/disconnected/connecting |
| Test connection | Button | Tests server connectivity |
| Fetch server info | Button | Retrieves server version and capabilities |
| Sync now | Button | Triggers manual sync |
| Last sync time | Display | Shows when last sync occurred |
| Custom headers | Dialog (`CustomHeadersDialog`) | Edit custom HTTP headers |

### 3.2 Scheduled Messages Panel (`ScheduledMessagesPanel`)

| Feature | Control Type | Description |
|---------|-------------|-------------|
| Scheduled messages list | List view | Shows all pending scheduled messages |
| Each item | Tile | Message content, scheduled time, target chat |
| Delete scheduled | Button per item | Cancel a scheduled message |
| Create new | FAB / Button | Navigate to `CreateScheduledPanel` |

### 3.3 Create Scheduled Panel (`CreateScheduledPanel`)

| Field | Control Type | Description |
|-------|-------------|-------------|
| Chat selector | Picker | Choose target conversation |
| Message text | Text input | Message content |
| Schedule date | Date picker | When to send |
| Schedule time | Time picker | When to send |
| Repeat | Dropdown | None, daily, weekly, monthly |

### 3.4 Message Reminders Panel (`MessageRemindersPanel`)
**Android only**

| Feature | Description |
|---------|-------------|
| Active reminders list | Shows all pending reminders |
| Each item | Message content + reminder time |
| Delete reminder | Cancel a reminder |

### 3.5 Theming Panel (`ThemingPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Skin selector | Dropdown | iOS, Material, Samsung |
| Light theme | Dropdown | Select light mode theme |
| Dark theme | Dropdown | Select dark mode theme |
| Theme mode | Toggle | Light / Dark / System |
| Material You (Monet) | Dropdown | Disabled / Harmonize / Full |
| Advanced theming | Navigation tile | Opens `AdvancedThemingPanel` |
| Custom avatars | Navigation tile | Opens `CustomAvatarPanel` |
| Avatar scale | Slider | Adjust avatar size |
| Colorful avatars | Switch | Enable per-contact avatar colors |
| Colorful bubbles | Switch | Enable per-contact bubble colors |

### 3.6 Advanced Theming Panel (`AdvancedThemingPanel`)

| Feature | Control Type | Description |
|---------|-------------|-------------|
| Theme tab bar | NavigationBar | Light Theme / Dark Theme tabs |
| Theme selector | Dropdown | Select theme to edit (3 groups separated by dividers) |
| Color pairs grid | Grid of `AdvancedThemingTile` | Each shows two 12x12 color swatches |
| Text sizes | Slider per slot | Adjust each text style size |
| Font family | Searchable dropdown | Select Google Font |
| Gradient background | Switch | Toggle animated gradient |
| Create new theme | FAB | Opens `CreateNewThemeDialog` |
| View old themes | AppBar action | Opens `OldThemesDialog` |

**Constraints:**
- Preset themes: Cannot edit colors (must create custom theme)
- Monet active: Color editor disabled (Monet overrides at runtime)
- Custom themes: Can be deleted (presets cannot)

### 3.7 Custom Avatar Panel (`CustomAvatarPanel`)

| Feature | Description |
|---------|-------------|
| Contact list | Grid of all contacts with avatars |
| Each contact | Tap to select custom image |
| Image cropping | Opens `AvatarCrop` |
| Reset | Remove custom avatar |

### 3.8 Custom Avatar Color Panel (`CustomAvatarColorPanel`)

| Feature | Description |
|---------|-------------|
| Contact list | All contacts with current avatar colors |
| Color picker | Per-contact color selection dialog |
| Reset | Revert to auto-generated color |

### 3.9 Attachment Panel (`AttachmentPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Auto-download attachments | Switch | Auto-download media on WiFi |
| Auto-download on cellular | Switch | Also auto-download on cellular |
| Attachment quality | Dropdown | Original / Compressed |
| Save to gallery | Switch | Auto-save media to device gallery |
| Save path | Text input | Custom save directory |
| Preview links | Switch | Show URL previews in conversations |

### 3.10 Conversation Panel (`ConversationPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Show delivery receipts | Switch | Show delivered/read indicators |
| Send read receipts | Switch | Notify sender when you read |
| Show timestamps | Dropdown | Per skin: always, swipe, header only |
| Swipe to dismiss keyboard | Switch | Swipe down to hide keyboard |
| Double tap to tapback | Switch | Quick reaction via double-tap |
| Reduce motion | Switch | Disable message animations |
| Scroll to bottom on send | Switch | Auto-scroll after sending |
| Message options order | Navigation tile | Opens `MessageOptionsOrderPanel` |

### 3.11 Message Options Order Panel (`MessageOptionsOrderPanel`)

| Feature | Description |
|---------|-------------|
| Reorderable list | Drag handles to reorder long-press menu actions |
| Toggle visibility | Show/hide individual actions |
| Reset | Restore default order |

### 3.12 Chat List Panel (`ChatListPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Sort order | Dropdown | By date (default), alphabetical |
| Pinned chats at top | Switch | Keep pinned chats separate |
| Swipe actions | Switch | Enable swipe gestures (iOS skin) |
| Show unread count | Switch | Badge with count vs simple dot |
| Filter unknown senders | Switch | Separate unknown senders |
| Pinned order | Navigation tile | Opens `PinnedOrderPanel` |

### 3.13 Pinned Order Panel (`PinnedOrderPanel`)

| Feature | Description |
|---------|-------------|
| Reorderable list | Drag handles to reorder pinned conversations |
| Unpin | Swipe or button to unpin |

### 3.14 Notification Panel (`NotificationPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Enable notifications | Master switch | Toggle all notifications |
| Notification sound | Dropdown | Select notification sound |
| Vibration pattern | Dropdown | Vibration style |
| Show preview | Switch | Show message content in notification |
| Per-chat overrides | Button per chat | Opens `NotificationSettingsDialog` |
| Notification grouping | Dropdown | By conversation, all together |
| Priority | Dropdown | Default, high, low |

### 3.15 Desktop Panel (`DesktopPanel`)
**Desktop only**

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Window effect | Dropdown | Disabled, Transparent, Acrylic, Mica, Aero, Tabbed |
| Custom opacity (dark) | Slider | 0.0 - 1.0 |
| Custom opacity (light) | Slider | 0.0 - 1.0 |
| Minimize to tray | Switch | Minimize to system tray instead of taskbar |
| Launch at startup | Switch | Auto-start with OS |
| Close to tray | Switch | Close button sends to tray |
| Startup page | Dropdown | Conversation list, last chat |
| Tablet mode | Switch | Enable split-view layout |

### 3.16 Private API Panel (`PrivateAPIPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Enable Private API | Master switch | Toggle Private API features |
| Send tapbacks | Switch | React to messages |
| Send typing indicators | Switch | Show when you're typing |
| Send read receipts | Switch | Mark as read |
| Change chat name | Switch | Rename group chats |
| Leave group | Switch | Leave group conversations |
| Send with effects | Switch | iMessage effects |
| Edit messages | Switch | Edit sent messages |
| Unsend messages | Switch | Retract sent messages |

### 3.17 Redacted Mode Panel (`RedactedModePanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Enable redacted mode | Master switch | Privacy/screenshot redaction |
| Hide message text | Switch | Replace text with redacted blocks |
| Hide contact names | Switch | Replace names with placeholders |
| Hide avatars | Switch | Replace avatars with generic icons |
| Hide attachments | Switch | Hide media previews |

### 3.18 Tasker Panel (`TaskerPanel`)
**Android only**

| Feature | Description |
|---------|-------------|
| Enable Tasker integration | Master switch |
| Event triggers | List of available Tasker events |
| Action documentation | How to use BlueBubbles with Tasker |

### 3.19 Notification Providers Panel (`NotificationProvidersPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Provider | Radio group | FCM, WebSocket, UnifiedPush |
| FCM config | Navigation tile | Opens `FirebasePanel` |
| UnifiedPush config | Navigation tile | Opens `UnifiedPushPanel` |

### 3.20 Firebase Panel (`FirebasePanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Firebase project config | JSON input | Firebase configuration |
| Test push | Button | Send test notification |
| Reset | Button | Clear Firebase config |

### 3.21 Unified Push Panel (`UnifiedPushPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Distributor | Dropdown | Select UnifiedPush distributor app |
| Endpoint | Display | Current push endpoint |
| Test | Button | Send test push |

### 3.22 Misc Panel (`MiscPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Incognito keyboard | Switch | Request keyboard to not learn from typing |
| Send with Return | Switch | Enter key sends message |
| Auto open keyboard | Switch | Auto-open keyboard when entering conversation |
| Swipe to close keyboard | Switch | Swipe gesture to dismiss keyboard |
| Generate link previews | Switch | Create previews for sent URLs |
| Tablet mode | Switch | Enable split-view (duplicated here for convenience) |
| Show sync indicator | Switch | Show connection status indicator |

### 3.23 Troubleshoot Panel (`TroubleshootPanel`)

| Feature | Control Type | Description |
|---------|-------------|-------------|
| Re-sync messages | Button | Opens `SyncDialog` |
| Re-sync handles | Button | Refresh contact handles from server |
| Clear local database | Button | Wipe and re-sync |
| Reset settings | Button | Restore default settings |
| Log viewer | Navigation tile | Opens `LoggingPanel` |
| Live logging | Navigation tile | Opens `LiveLoggingPanel` |
| Export logs | Button | Share log file |
| Database info | Display | Row count, size |

### 3.24 Logging Panel (`LoggingPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Log level | `LogLevelSelector` dropdown | Trace, Debug, Info, Warn, Error |
| Enable file logging | Switch | Write logs to file |

### 3.25 Live Logging Panel (`LiveLoggingPanel`)

| Feature | Description |
|---------|-------------|
| Log stream | Real-time scrolling log output |
| Filter | Text input to filter log entries |
| Auto-scroll | Follows newest entries |
| Clear | Clear current display |

### 3.26 About Panel (`AboutPanel`)

| Item | Description |
|------|-------------|
| App version | Current version string |
| Build number | Build identifier |
| Changelog | Recent changes |
| Source code link | GitHub repository |
| License | Open source license |
| Contributors | Credits |

### 3.27 Profile Panel (`ProfilePanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Display name | Text input | User's display name |
| Avatar | Image picker | User's profile picture |
| Remove avatar | Button | Reset to default |

### 3.28 Backup & Restore Panel (`BackupRestorePanel`)

| Feature | Control Type | Description |
|---------|-------------|-------------|
| Backup settings | Button | Export settings to server/file |
| Restore settings | Button | Import settings from server/file |
| Backup themes | Button | Export custom themes |
| Restore themes | Button | Import custom themes |

### 3.29 OAuth Panel (`OAuthPanel`)

| Setting | Control Type | Description |
|---------|-------------|-------------|
| Google auth status | Status display | Signed in/out |
| Sign in with Google | Button | OAuth flow |
| Sign out | Button | Clear OAuth tokens |

---

## 4. Settings Dialogs

| Dialog | Trigger | Purpose |
|--------|---------|---------|
| `CreateNewThemeDialog` | Advanced Theming FAB | Name input for new custom theme |
| `CustomHeadersDialog` | Server Management | Edit custom HTTP headers |
| `NotificationSettingsDialog` | Per-chat notification | Override notification settings per chat |
| `OldThemesDialog` | Advanced Theming AppBar | Legacy theme import/migration |
| `SyncDialog` | Troubleshoot Panel | Full message re-sync progress |

---

## 5. States

### Loading
- Settings load from reactive state (instant, no loading spinner needed)
- Server-dependent settings show loading while fetching server info

### Error
- Invalid server URL: Red border on input, error text below
- Connection test failed: Error dialog with details
- Sync failed: Error message with retry button

### Disabled
- Settings that depend on a master switch: Grayed out when master is off
- Platform-specific settings: Hidden on unsupported platforms (not grayed)
- Monet-dependent: Color editors disabled when Monet is active

---

## 6. Accessibility

- All switches have semantic labels describing their function
- All sliders announce current value
- All dropdowns have proper labels
- Section headers serve as group labels
- Navigation tiles announce their destination
- Danger zone actions require confirmation dialog
- High-contrast support via theme system
