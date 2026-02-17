# OTP Toast Notification Implementation

## Overview

iOS 26-inspired liquid glass OTP toast notification system that automatically detects verification codes in incoming messages, displays them in a beautiful frosted glass overlay, and copies them to the clipboard.

## Architecture

### Component Structure

```
src/
├── components/
│   └── OtpToast.tsx           # Main toast component with liquid glass styling
├── contexts/
│   └── OtpToastContext.tsx    # Global state management for OTP toast
├── hooks/
│   └── useOtpDetection.ts     # Tauri event listener hook
└── pages/
    └── Settings.tsx           # OTP settings in Notifications panel
```

### Data Flow

```
1. Message arrives → Rust backend detects OTP pattern
2. Backend emits "otp-detected" Tauri event
3. useOtpDetection hook receives event
4. Hook calls showOtp() from OtpToastContext
5. OtpToast component displays with auto-copy
6. Auto-dismisses after 5 seconds
```

## Features

### Visual Design (iOS 26 Liquid Glass)
- **Frosted glass background** with backdrop blur and saturation boost
- **Smooth animations** using Framer Motion spring physics
- **Top-left positioning** (20px from edges)
- **Elevated glass effect** with subtle borders and shadows
- **Auto-adapts to theme** (light/dark liquid glass variants)

### Functionality
- **Auto-detection** of OTP codes from incoming messages
- **Auto-copy to clipboard** when code appears
- **Visual feedback** with "Copied" badge animation
- **Message snippet preview** showing context
- **Manual dismiss** with close button
- **Auto-dismiss** after 5 seconds
- **Settings toggle** to enable/disable OTP detection

## Usage

### Frontend Integration

The OTP toast is automatically integrated into the app via context provider:

```tsx
// App.tsx
export function App() {
  return (
    <OtpToastProvider>
      <AppContent />
    </OtpToastProvider>
  );
}

function AppContent() {
  useOtpDetection(); // Activates event listener
  const { otpData, dismissOtp } = useOtpToast();

  return (
    <>
      <OtpToast data={otpData} onDismiss={dismissOtp} />
      {/* Rest of app */}
    </>
  );
}
```

### Backend Integration (Rust)

To emit OTP detection events from the Tauri backend:

```rust
use tauri::Emitter;

// When OTP is detected in message processing
#[derive(serde::Serialize, Clone)]
struct OtpDetectionPayload {
    code: String,
    snippet: String,
}

// In message handler
if let Some(otp_code) = detect_otp(&message_text) {
    let payload = OtpDetectionPayload {
        code: otp_code,
        snippet: message_text[..100].to_string(), // First 100 chars
    };

    app_handle.emit("otp-detected", payload)?;
}
```

### OTP Detection Pattern (Example)

Common patterns to detect in message text:
- 6-digit codes: `\b\d{6}\b`
- 4-digit codes: `\b\d{4}\b`
- Alphanumeric codes: `\b[A-Z0-9]{6,8}\b`
- Keywords: "verification code", "OTP", "passcode", etc.

## Settings

Users can control OTP toast behavior in **Settings → Notifications → One-Time Passwords**:

- **OTP Detection** (default: enabled)
  - Automatically detect verification codes in messages

- **Auto-Copy OTP** (default: enabled)
  - Automatically copy codes to clipboard when detected
  - Disabled if OTP Detection is off

## Styling

### CSS Custom Properties

The toast uses liquid glass CSS variables defined in `globals.css`:

```css
/* Light theme */
--glass-bg: rgba(255, 255, 255, 0.70);
--glass-bg-elevated: rgba(255, 255, 255, 0.82);
--glass-border: rgba(255, 255, 255, 0.18);
--glass-border-subtle: rgba(0, 0, 0, 0.06);
--glass-blur: 16px;
--glass-shadow-elevated: 0 8px 32px rgba(0, 0, 0, 0.10);

/* Dark theme */
--glass-bg: rgba(30, 30, 32, 0.65);
--glass-bg-elevated: rgba(44, 44, 46, 0.78);
--glass-border: rgba(255, 255, 255, 0.08);
--glass-border-subtle: rgba(255, 255, 255, 0.04);
--glass-blur: 20px;
--glass-shadow-elevated: 0 8px 32px rgba(0, 0, 0, 0.36);
```

### Fallback for Non-Glass Themes

The toast gracefully falls back to standard surface colors when liquid glass theme is not active:

```tsx
background: "var(--glass-bg-elevated, rgba(255, 255, 255, 0.82))",
backgroundColor: data ? "var(--color-surface-variant)" : "transparent",
```

## Animation Details

### Entry Animation
```tsx
initial={{ opacity: 0, x: -30, scale: 0.95 }}
animate={{ opacity: 1, x: 0, scale: 1 }}
transition={{
  type: "spring",
  stiffness: 400,
  damping: 30,
  mass: 0.8,
}}
```

- Slides in from left (-30px)
- Fades in from transparent
- Scales from 95% to 100%
- Uses spring physics for natural motion

### "Copied" Badge Animation
```tsx
initial={{ opacity: 0, scale: 0.8 }}
animate={{ opacity: 1, scale: 1 }}
exit={{ opacity: 0, scale: 0.8 }}
transition={{ duration: 0.2 }}
```

## Accessibility

- **Keyboard navigation**: Close button is focusable
- **ARIA labels**: Close button has `aria-label="Dismiss"`
- **Semantic HTML**: Uses proper button and div structure
- **High contrast**: Code display uses primary color for visibility
- **Screen reader friendly**: Text content is accessible

## Performance

- **Zero re-renders** when toast is hidden (uses AnimatePresence)
- **Optimized backdrop-filter** with hardware acceleration
- **Automatic cleanup** of event listeners on unmount
- **Debounced clipboard access** to avoid permission spam

## Testing

To test the OTP toast manually, emit a test event from the browser console:

```javascript
// In browser DevTools console (when running in Tauri)
window.__TAURI__.event.emit('otp-detected', {
  code: '123456',
  snippet: 'Your verification code is 123456. Do not share this code.'
});
```

## Future Enhancements

- [ ] Sound notification option
- [ ] Haptic feedback (if supported)
- [ ] History of recent OTP codes
- [ ] Customizable position (top-left, top-right, etc.)
- [ ] Customizable auto-dismiss duration
- [ ] OTP code verification (check if code is used)
- [ ] Multiple OTP codes in single message
- [ ] Regex pattern customization in settings
