# 05 - Setup Wizard

> Complete specification for the 7-page setup flow, including layout, inputs, validation, server connection, and Firebase/FCM setup.

---

## 1. Overview

The Setup Wizard is a multi-page onboarding flow for first-time configuration. It uses a `PageView` with `NeverScrollableScrollPhysics` (only programmatic navigation -- no swipe). Pages auto-skip if their requirements are already met.

### Page Counts by Platform
| Platform | Pages | Skipped Pages |
|----------|-------|---------------|
| Web | 4 | Contacts, Battery, Sync Settings |
| Desktop | 5 | Contacts, Battery |
| Mobile | 7 | None (all shown) |

### Controller State

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `pageController` | PageController | -- | Controls PageView navigation |
| `currentPage` | int | 1 | Current page (1-based for display) |
| `numberToDownload` | int | 25 | Messages per chat to sync |
| `skipEmptyChats` | bool | false | Skip chats with no messages |
| `saveToDownloads` | bool | false | Save attachments to Downloads |
| `error` | String | "" | Connection error message |
| `obscurePass` | bool | true | Password field visibility toggle |

### Point of No Return
After a certain page, disconnection triggers an error dialog:
- Web/Desktop: Page 3 (after server connection)
- Mobile: Page 5 (after server connection)

---

## 2. Setup Header

Consistent header across all pages:

```
Column (center-aligned)
  BlueBubbles icon (animated on splash)
  "BlueBubbles" text (app name)
  PageNumber pill (gradient indicator)
```

### PageNumber Widget
- Gradient-filled pill showing "Page X of Y"
- Updates reactively via `updateWidgets<PageNumber>` mechanism
- Y varies by platform (4 for web, 5 for desktop, 7 for mobile)

---

## 3. Page Specifications

### Page 1: Welcome Page

**Platform:** All

**Layout:**
```
Column (centered)
  SetupHeader
  Welcome text / branding message
  "Get Started" or "Next" button
```

**Content:**
- App logo and name
- Brief description of BlueBubbles
- Welcome message explaining setup process
- Single "Next" button to proceed

**Interaction:**
- Tap "Next" -> Navigate to Page 2 (or skip to appropriate page for platform)

### Page 2: Request Contacts

**Platform:** Mobile only (auto-skipped on web/desktop)

**Layout:**
```
Column (centered)
  SetupHeader
  Permission explanation text
  "Grant Permission" button
  "Skip" button
```

**Content:**
- Explanation of why contacts permission is needed
- Describes that contacts are used for avatar display and name resolution
- Clear indication this is optional

**Interaction:**
- "Grant Permission" -> System permission dialog
  - If granted: Auto-advance to Page 3
  - If denied: Show "Skip" more prominently
- "Skip" -> Navigate to Page 3 without permission

**Auto-skip:** If contacts permission already granted, skip directly to next page

### Page 3: Battery Optimization

**Platform:** Mobile only (Android specifically; auto-skipped on iOS/web/desktop)

**Layout:**
```
Column (centered)
  SetupHeader
  Explanation text about battery optimization
  "Disable Optimization" button
  "Skip" button
```

**Content:**
- Explains that battery optimization can kill background processes
- Recommends disabling for reliable notifications
- Shows which optimization to disable

**Interaction:**
- "Disable Optimization" -> Opens Android battery optimization settings
  - On return: Check if disabled, auto-advance if so
- "Skip" -> Navigate to Page 4

**Auto-skip:** If battery optimization already disabled

### Page 4: Mac Setup Check

**Platform:** All

**Layout:**
```
Column (centered)
  SetupHeader
  Checklist of server requirements
    macOS running
    BlueBubbles Server installed
    Server running and accessible
    iMessage signed in on Mac
  "Continue" button (enabled when checks pass)
```

**Content:**
- Step-by-step verification checklist
- Each step has a check/cross indicator
- Link to download BlueBubbles Server if needed
- Help text for troubleshooting

**Interaction:**
- User confirms they have completed Mac setup
- "Continue" -> Navigate to Page 5

### Page 5: Server Credentials

**Platform:** All

**Layout:**
```
Column (centered)
  SetupHeader
  Tab bar or toggle: "QR Code" | "Manual Entry"

  QR Code tab:
    Camera preview (mobile) or instruction text (desktop)
    "Scan QR Code" button
    "Enter Manually" link

  Manual Entry tab:
    Server URL text field
      Placeholder: "https://your-server.ngrok.io"
      Validation: Must be valid URL
    Password text field
      Obscured by default
      Toggle visibility button
      Validation: Required
    "Connect" button

  Error display area (shows connection errors)
```

**Validation:**
- Server URL: Must be non-empty, valid URL format
- Password: Must be non-empty
- Connection test runs on "Connect" -- shows `ConnectingDialog`

**Connection Flow:**
1. User enters URL + password (or scans QR)
2. Tap "Connect"
3. `ConnectingDialog` appears: "Connecting to server..." with spinner
4. On success: Auto-advance to Page 6
5. On failure: `FailedToConnectDialog` with error details
   - Options: "Retry" or "Go Back"

**QR Code Flow:**
1. Scan QR from BlueBubbles Server settings
2. Parses URL + password from QR data
3. Auto-fills fields and attempts connection
4. On scan failure: `FailedToScanDialog`

**Dialogs:**
| Dialog | Purpose |
|--------|---------|
| `ConnectingDialog` | Spinner + "Connecting to server..." |
| `FailedToConnectDialog` | Error message + retry/back buttons |
| `FailedToScanDialog` | QR scan failure notification |
| `ManualEntryDialog` | Alternative manual URL/password form |

### Page 6: Sync Settings

**Platform:** Non-web only (auto-skipped on web)

**Layout:**
```
Column (centered)
  SetupHeader
  "Messages per chat" slider or input
    Default: 25
    Range: 0 - 10000 (or "All")
  "Skip empty chats" checkbox
  "Save attachments to Downloads" checkbox
  "Start Sync" button
```

**Controls:**
| Control | Type | Default | Description |
|---------|------|---------|-------------|
| Messages per chat | Slider/Number input | 25 | How many messages to download per conversation |
| Skip empty chats | Checkbox | false | Don't sync chats with 0 messages |
| Save to Downloads | Checkbox | false | Save attachments to device Downloads folder |

**Interaction:**
- Adjust settings as desired
- "Start Sync" -> Navigate to Page 7, begins sync process

### Page 7: Sync Progress

**Platform:** All

**Layout:**
```
Column (centered)
  SetupHeader
  "Syncing your messages..." text
  CircleProgressBar (circular progress indicator)
  Progress percentage text
  Current operation text ("Syncing chats...", "Downloading messages...")
  "Please wait" subtext
```

**Progress Display:**
- `CircleProgressBar` widget with animated fill
- Percentage updates reactively
- Operation description updates as sync progresses through phases
- Phases: Fetching chats -> Downloading messages -> Downloading attachments

**Completion:**
- On sync complete: Auto-navigate to Conversation List
- On error: Show error with retry option
- Cannot go back once sync has started (past point of no return)

---

## 4. Navigation Behavior

### Forward Navigation
- Programmatic only (PageView with NeverScrollableScrollPhysics)
- Page transition: 500ms duration, `Curves.easeInOut`
- Each page validates before allowing forward navigation

### Back Navigation
- Back button/gesture available on pages before point of no return
- Back transition: 300ms duration, `Curves.easeIn`
- After point of no return: Back shows error dialog warning about disconnection

### Page Skipping
Pages auto-skip when their requirements are already met:
- Contacts permission already granted -> skip Page 2
- Battery optimization already disabled -> skip Page 3
- Web platform -> skip Pages 2, 3, 6

---

## 5. States

### Per-Page States

| Page | Loading | Error | Success |
|------|---------|-------|---------|
| Welcome | None | None | Next enabled |
| Contacts | System dialog | Permission denied text | Auto-advance |
| Battery | System settings | Still optimized text | Auto-advance |
| Mac Check | None | Checklist item failed | All checks pass |
| Credentials | ConnectingDialog spinner | FailedToConnectDialog | Auto-advance |
| Sync Settings | None | None | Start sync enabled |
| Sync Progress | CircleProgressBar | Error + retry | Auto-navigate to app |

---

## 6. Accessibility

- All form fields have proper labels and descriptions
- Progress bar announces percentage changes
- Error dialogs are announced to screen readers
- Page transitions are announced
- Skip buttons clearly explain what is being skipped
- QR scanner has manual entry alternative
- All buttons have semantic labels
- Tab through form fields with keyboard
- Enter to submit forms
