# OTP Toast Quick Start Guide

## For Frontend Developers

### Testing the OTP Toast

1. **Navigate to demo page**: Open the app and go to `/otp-demo`

2. **Click a preset example** or enter a custom code

3. **Watch the magic**:
   - Toast appears in top-left corner
   - Code is auto-copied to clipboard
   - "Copied" badge shows confirmation
   - Toast auto-dismisses after 5 seconds

### Using in Your Code

```tsx
import { useOtpToast } from "@/contexts/OtpToastContext";

function MyComponent() {
  const { showOtp } = useOtpToast();

  const handleShowOtp = () => {
    showOtp(
      "123456",
      "Your verification code is 123456. Do not share this code."
    );
  };

  return <button onClick={handleShowOtp}>Show OTP</button>;
}
```

### Settings Access

Users can control OTP toast in **Settings → Notifications → One-Time Passwords**:
- Toggle OTP detection on/off
- Toggle auto-copy on/off

## For Backend Developers (Rust)

### Emitting OTP Events

Add this to your message processing logic:

```rust
use tauri::Emitter;

#[derive(serde::Serialize, Clone)]
struct OtpDetectionPayload {
    code: String,
    snippet: String,
}

// In your message handler
fn process_message(
    app_handle: &tauri::AppHandle,
    message_text: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Detect OTP code
    if let Some(code) = detect_otp(message_text) {
        let payload = OtpDetectionPayload {
            code,
            snippet: message_text[..100.min(message_text.len())].to_string(),
        };

        // Emit event to frontend
        app_handle.emit("otp-detected", payload)?;
    }

    Ok(())
}

// Simple OTP detection (improve as needed)
fn detect_otp(text: &str) -> Option<String> {
    use regex::Regex;

    // Match 4-8 digit codes
    let re = Regex::new(r"\b\d{4,8}\b").ok()?;

    // Look for keywords first
    let keywords = ["code", "otp", "verification", "passcode"];
    let has_keyword = keywords.iter().any(|k| {
        text.to_lowercase().contains(k)
    });

    if has_keyword {
        re.find(text).map(|m| m.as_str().to_string())
    } else {
        None
    }
}
```

### Advanced OTP Detection

For better accuracy, use multiple patterns:

```rust
fn detect_otp_advanced(text: &str) -> Option<String> {
    use regex::Regex;

    let patterns = vec![
        // 6-digit codes (most common)
        r"(?i)(?:code|otp|verification).*?(\d{6})",
        // 4-digit codes (Apple, banks)
        r"(?i)(?:code|otp|verification).*?(\d{4})",
        // Alphanumeric codes
        r"(?i)(?:code|otp|verification).*?([A-Z0-9]{6,8})",
        // Generic 6-digit anywhere in message
        r"\b(\d{6})\b",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(text) {
                if let Some(code) = captures.get(1) {
                    return Some(code.as_str().to_string());
                }
            }
        }
    }

    None
}
```

### Where to Add OTP Detection

Recommended locations in the codebase:

1. **WebSocket message handler** (`bb-socket` crate)
   - Detect OTP in real-time as messages arrive

2. **Sync process** (`bb-services` crate)
   - Detect OTP in historical messages during sync

3. **Message processing** (`bb-tauri/src/commands.rs`)
   - Add to existing message handling logic

Example integration point:

```rust
// In bb-tauri/src/commands.rs or message handler

#[tauri::command]
pub async fn process_incoming_message(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    message: Message,
) -> Result<(), String> {
    // Existing message processing...

    // OTP detection
    if let Some(text) = &message.text {
        if let Some(code) = detect_otp(text) {
            let payload = OtpDetectionPayload {
                code,
                snippet: text[..100.min(text.len())].to_string(),
            };
            app.emit("otp-detected", payload)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
```

## Testing Checklist

### Manual Testing

- [ ] Navigate to `/otp-demo`
- [ ] Click each preset example
- [ ] Verify toast appears in top-left
- [ ] Verify code is copied to clipboard
- [ ] Verify "Copied" badge appears
- [ ] Verify toast auto-dismisses after 5s
- [ ] Click close button (×) to manually dismiss
- [ ] Test custom OTP input form

### Settings Testing

- [ ] Go to Settings → Notifications
- [ ] Toggle "OTP Detection" off
- [ ] Try showing OTP (should not appear)
- [ ] Toggle "OTP Detection" on
- [ ] Toggle "Auto-Copy OTP" off
- [ ] Verify code still shows but doesn't auto-copy
- [ ] Toggle "Auto-Copy OTP" on
- [ ] Verify settings persist after restart

### Theme Testing

- [ ] Test in Light theme
- [ ] Test in Dark theme
- [ ] Test in Liquid Glass Light theme (best appearance)
- [ ] Test in Liquid Glass Dark theme (best appearance)
- [ ] Test in other themes (OLED, Nord, etc.)
- [ ] Verify fallback styling works

### Accessibility Testing

- [ ] Tab to close button (keyboard navigation)
- [ ] Press Enter to dismiss (keyboard activation)
- [ ] Test with screen reader (VoiceOver/NVDA)
- [ ] Verify high contrast for code display
- [ ] Check ARIA labels are present

## Common Issues & Solutions

### Issue: Toast not appearing
**Solutions:**
1. Check if OTP Detection is enabled in Settings
2. Verify Tauri event is being emitted from backend
3. Check browser console for errors
4. Confirm you're running in Tauri context (not plain browser)

### Issue: Clipboard copy failing
**Solutions:**
1. Check browser clipboard permissions
2. Verify app is running in secure context (HTTPS or localhost)
3. Check if Auto-Copy is enabled in Settings
4. Try manual clipboard access in console:
   ```javascript
   navigator.clipboard.writeText('test')
   ```

### Issue: Liquid glass effect not visible
**Solutions:**
1. Switch to Liquid Glass Light or Liquid Glass Dark theme
2. Verify browser supports backdrop-filter (modern browsers)
3. Check CSS custom properties are defined in globals.css
4. Fallback styling should still work (surface-variant background)

### Issue: Animation choppy or laggy
**Solutions:**
1. Check if GPU acceleration is enabled in browser
2. Reduce backdrop-filter blur amount if needed
3. Verify Framer Motion is installed and up-to-date
4. Check for other performance issues in app

## Performance Tips

1. **Debounce rapid OTP events**: If backend detects OTP in multiple messages quickly, consider debouncing the events

2. **Limit snippet length**: Keep snippet under 100 characters to avoid large payloads

3. **Clean up listeners**: useOtpDetection hook automatically cleans up on unmount

4. **Optimize regex**: Use efficient patterns for OTP detection

5. **Cache clipboard permissions**: Browser caches clipboard permissions, so repeated access is fast

## Next Steps

1. **Implement backend OTP detection**: Add detection logic to message handler
2. **Test with real messages**: Send verification codes to test device
3. **Gather user feedback**: Monitor how users interact with toast
4. **Iterate on patterns**: Improve OTP detection regex based on real data
5. **Add telemetry** (optional): Track OTP detection success rate

## Resources

- **Full Documentation**: `docs/OTP_TOAST_IMPLEMENTATION.md`
- **Component Tree**: `OTP_TOAST_COMPONENT_TREE.md`
- **Implementation Summary**: `OTP_TOAST_SUMMARY.md`
- **Demo Page**: Navigate to `/otp-demo` in the app

## Support

If you encounter issues:

1. Check the console for errors
2. Review the implementation docs
3. Test with the demo page first
4. Verify settings are enabled
5. Check Tauri event emissions in backend

## Quick Copy-Paste Snippets

### Frontend: Show OTP Manually
```tsx
const { showOtp } = useOtpToast();
showOtp("123456", "Your code is 123456");
```

### Frontend: Check if OTP is Enabled
```tsx
const { settings } = useSettingsStore();
const isEnabled = settings["otpDetection"] !== "false";
```

### Backend: Emit OTP Event
```rust
app.emit("otp-detected", OtpDetectionPayload {
    code: "123456".to_string(),
    snippet: "Your verification code is 123456".to_string(),
})?;
```

### Browser Console: Test Event
```javascript
window.__TAURI__.event.emit('otp-detected', {
  code: '123456',
  snippet: 'Test message'
});
```

---

**Ready to go!** Start with the demo page (`/otp-demo`) and work your way to backend integration.
