# OTP Toast Component Tree

## Visual Component Structure

```
App (wrapped with OtpToastProvider)
├── OtpToastProvider (Context)
│   └── AppContent
│       ├── useOtpDetection() ──────────┐ (Listens for Tauri events)
│       │                               │
│       ├── OtpToast                    │
│       │   ├── AnimatePresence        │
│       │   └── motion.div (toast)     │
│       │       ├── Header              │
│       │       │   ├── Title           │
│       │       │   └── Close Button (×)
│       │       │                       │
│       │       ├── Code Container      │
│       │       │   ├── Code Display    │
│       │       │   └── Copied Badge ───┤ (Conditional AnimatePresence)
│       │       │                       │
│       │       └── Message Snippet     │
│       │                               │
│       └── BrowserRouter               │
│           └── Routes                  │
│               ├── /setup              │
│               ├── / (AppLayout)       │
│               │   ├── /               │
│               │   ├── /chat/:guid     │
│               │   ├── /new            │
│               │   ├── /settings ──────┤ (OTP Settings Panel)
│               │   ├── /findmy         │
│               │   └── /otp-demo ──────┤ (Testing Page)
│               │                       │
│               └── ...                 │
│                                       │
└─────────────────────────────────────────┘
                    │
                    │ Tauri Event: "otp-detected"
                    │ Payload: { code, snippet }
                    │
                    ▼
            Rust Backend
            (Message Handler)
```

## Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                        Message Received                           │
│                              ↓                                    │
│                    ┌─────────────────┐                           │
│                    │ Rust Backend    │                           │
│                    │ Message Handler │                           │
│                    └────────┬────────┘                           │
│                             │                                     │
│                    Detect OTP Pattern                             │
│                    (regex matching)                               │
│                             │                                     │
│                    ┌────────▼────────┐                           │
│                    │ Emit Tauri Event│                           │
│                    │ "otp-detected"  │                           │
│                    │ { code, snippet }│                          │
│                    └────────┬────────┘                           │
│                             │                                     │
│                    Frontend Event Bus                             │
│                             │                                     │
│                    ┌────────▼────────┐                           │
│                    │ useOtpDetection │                           │
│                    │ Hook (listener) │                           │
│                    └────────┬────────┘                           │
│                             │                                     │
│                    Check Settings                                 │
│                    (otpDetection enabled?)                        │
│                             │                                     │
│                    ┌────────▼────────┐                           │
│                    │  useOtpToast()  │                           │
│                    │  showOtp(code)  │                           │
│                    └────────┬────────┘                           │
│                             │                                     │
│                    Update Context State                           │
│                    (otpData = { code, snippet })                  │
│                             │                                     │
│                    ┌────────▼────────┐                           │
│                    │   OtpToast      │                           │
│                    │   Component     │                           │
│                    └────────┬────────┘                           │
│                             │                                     │
│                    ┌────────▼────────┐                           │
│                    │ AnimatePresence │                           │
│                    │ (Framer Motion) │                           │
│                    └────────┬────────┘                           │
│                             │                                     │
│                    ┌────────▼────────┐                           │
│                    │ Render Toast    │                           │
│                    │ (liquid glass)  │                           │
│                    └────────┬────────┘                           │
│                             │                                     │
│                    Auto-Copy to Clipboard                         │
│                    (navigator.clipboard.writeText)                │
│                             │                                     │
│                    Show "Copied" Badge                            │
│                    (AnimatePresence)                              │
│                             │                                     │
│                    Auto-Dismiss Timer (5s)                        │
│                    or Manual Dismiss (× button)                   │
│                             │                                     │
│                    ┌────────▼────────┐                           │
│                    │  dismissOtp()   │                           │
│                    └────────┬────────┘                           │
│                             │                                     │
│                    Clear Context State                            │
│                    (otpData = null)                               │
│                             │                                     │
│                    ┌────────▼────────┐                           │
│                    │ Toast Exits     │                           │
│                    │ (fade + slide)  │                           │
│                    └─────────────────┘                           │
└──────────────────────────────────────────────────────────────────┘
```

## State Management

```
┌─────────────────────────────────────┐
│     OtpToastContext (Zustand-like)  │
├─────────────────────────────────────┤
│ State:                              │
│   otpData: OtpToastData | null      │
│                                     │
│ Actions:                            │
│   showOtp(code, snippet)            │
│   dismissOtp()                      │
│                                     │
│ Consumed by:                        │
│   - OtpToast (rendering)            │
│   - useOtpDetection (updates)       │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│    SettingsStore (Zustand)          │
├─────────────────────────────────────┤
│ OTP-Related State:                  │
│   otpDetection: boolean             │
│   otpAutoCopy: boolean              │
│                                     │
│ Consumed by:                        │
│   - useOtpDetection (feature flag)  │
│   - Settings (UI toggle)            │
└─────────────────────────────────────┘
```

## Animation Timeline

```
Toast Appearance (entry):
0ms   ──→  Start: { opacity: 0, x: -30, scale: 0.95 }
         │
         │ Spring animation
         │ (stiffness: 400, damping: 30, mass: 0.8)
         │
~300ms ──→  End: { opacity: 1, x: 0, scale: 1 }


"Copied" Badge:
0ms   ──→  Auto-copy to clipboard
         │
50ms  ──→  Badge appears: { opacity: 0, scale: 0.8 }
         │
         │ Fade + scale animation
         │ (duration: 200ms)
         │
250ms ──→  Badge visible: { opacity: 1, scale: 1 }
         │
         │ Stays visible until toast dismisses
         │

Auto-Dismiss:
0ms   ──→  Toast appears
         │
         │ User interaction allowed
         │
5000ms ──→  Auto-dismiss triggered
         │
         │ Exit animation (spring)
         │
~5300ms ──→ Toast removed from DOM
```

## File Dependencies

```
OtpToast.tsx
├── depends on: framer-motion (AnimatePresence, motion)
├── depends on: React (useState, useEffect, CSSProperties)
└── exports: OtpToast, OtpToastData

OtpToastContext.tsx
├── depends on: React (createContext, useContext, useState, useCallback)
├── depends on: OtpToast.tsx (OtpToastData type)
└── exports: OtpToastProvider, useOtpToast

useOtpDetection.ts
├── depends on: @tauri-apps/api/event (listen, UnlistenFn)
├── depends on: OtpToastContext (useOtpToast)
├── depends on: settingsStore (useSettingsStore)
└── exports: useOtpDetection

OtpDemo.tsx
├── depends on: React (useState, CSSProperties)
├── depends on: react-router-dom (useNavigate)
├── depends on: OtpToastContext (useOtpToast)
└── exports: OtpDemo

Settings.tsx
├── depends on: settingsStore (useSettingsStore)
└── modified: Added NotificationsPanel OTP section

settingsStore.ts
├── depends on: zustand (create)
├── depends on: useTauri (tauriGetSettings, tauriUpdateSetting)
└── modified: Added otpDetection, otpAutoCopy state

App.tsx
├── depends on: OtpToastContext (OtpToastProvider, useOtpToast)
├── depends on: OtpToast (OtpToast component)
├── depends on: useOtpDetection (hook)
├── depends on: OtpDemo (page)
└── modified: Wrapped with provider, added toast rendering
```

## CSS Variables Used

```css
/* From globals.css */

/* Liquid Glass (theme-specific) */
--glass-bg-elevated
--glass-border
--glass-border-subtle
--glass-blur
--glass-shadow-elevated

/* Color System */
--color-on-surface
--color-on-surface-variant
--color-primary
--color-on-primary
--color-primary-container
--color-on-primary-container
--color-surface-variant
--color-outline

/* Typography */
--font-body-large
--font-body-medium
--font-body-small

/* Z-Index */
--z-toast: 500 (hardcoded in component as z-index: 1000)
```

## Event Flow

```
Backend (Rust)                Frontend (React)
─────────────                 ─────────────────

emit("otp-detected")  ─────→  listen<OtpDetectionPayload>
                               │
                               ├─ Check: otpDetection enabled?
                               │    ├─ Yes → Continue
                               │    └─ No  → Ignore event
                               │
                               ├─ Extract: { code, snippet }
                               │
                               ├─ Call: showOtp(code, snippet)
                               │    │
                               │    ├─ Update Context State
                               │    │   (otpData = { code, snippet, timestamp })
                               │    │
                               │    └─ Trigger Re-render
                               │         │
                               │         └─ OtpToast sees data !== null
                               │              │
                               │              ├─ AnimatePresence mounts toast
                               │              │
                               │              ├─ Auto-copy to clipboard
                               │              │
                               │              └─ Start auto-dismiss timer
                               │
                               └─ After 5s OR close button click:
                                    │
                                    ├─ Call: dismissOtp()
                                    │
                                    └─ Clear State (otpData = null)
                                         │
                                         └─ AnimatePresence unmounts toast
```

## Component Props

```typescript
// OtpToast.tsx
interface OtpToastProps {
  data: OtpToastData | null;
  onDismiss: () => void;
  autoDismissMs?: number; // default: 5000
}

interface OtpToastData {
  code: string;
  snippet: string;
  timestamp: number;
}

// OtpToastContext.tsx
interface OtpToastContextValue {
  otpData: OtpToastData | null;
  showOtp: (code: string, snippet: string) => void;
  dismissOtp: () => void;
}

interface OtpToastProviderProps {
  children: ReactNode;
}

// useOtpDetection.ts
interface OtpDetectionPayload {
  code: string;
  snippet: string;
}

// No props - it's a hook that returns void
```
