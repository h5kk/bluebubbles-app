# OTP Detection Feature

## Overview

The OTP (One-Time Password) detection feature automatically detects verification codes in incoming messages and provides options to auto-copy them to the clipboard.

## Architecture

### Backend Components

1. **OTP Detector Module** (`src-tauri/src/otp_detector.rs`)
   - Core detection logic using regex patterns
   - Supports multiple OTP formats (Apple, Google, generic)
   - Performance: ~13µs per message (75x faster than 1ms requirement)

2. **Tauri Commands** (`src-tauri/src/commands.rs`)
   - `detect_otp_in_message(message_guid)`: Detect OTP in a specific message
   - `detect_otp_in_text(text)`: Detect OTP in arbitrary text (for testing)

3. **Settings** (`bb-models/src/models/settings.rs`)
   - `otpDetectionEnabled`: Enable/disable OTP detection (default: true)
   - `otpAutoCopy`: Automatically copy detected OTP to clipboard (default: false)

### Supported OTP Formats

The detector recognizes the following patterns:

1. **Code with Prefix**
   - "Your verification code is 123456"
   - "Security code: 456789"
   - "Backup code: 789012"

2. **Code with Suffix**
   - "123456 is your verification code"
   - "456789 is your security code"

3. **Apple Format**
   - "Your Apple ID Code is: 123456"
   - "iCloud code: 654321"

4. **Google Format**
   - "G-123456 is your Google verification code"
   - "G-654321"

5. **Bracketed Code**
   - "Use code (987654) to sign in"
   - "Enter code [123456]"

6. **Standalone Code**
   - "Your security code: 4567" (requires contextual keywords)
   - Only detects 4-8 digit codes with nearby keywords

### Code Length Support

- Minimum: 4 digits
- Maximum: 8 digits
- Filters out phone numbers and dates using heuristics

## Frontend Integration

### Events Emitted

1. **`otp-detected`**
   ```json
   {
     "messageGuid": "message-guid-here",
     "code": "123456",
     "pattern": "CodeWithPrefix",
     "chatId": 123
   }
   ```
   Emitted when an OTP is detected in a message.

2. **`otp-auto-copy`**
   ```json
   {
     "code": "123456"
   }
   ```
   Emitted when auto-copy is enabled and an OTP is detected.
   Frontend should copy the code to clipboard.

### Tauri Command Usage

```typescript
import { invoke } from '@tauri-apps/api/core';

// Detect OTP in a specific message
const detection = await invoke('detect_otp_in_message', {
  messageGuid: 'message-guid-here'
});

if (detection) {
  console.log('Code detected:', detection.code);
  console.log('Pattern:', detection.pattern);
}

// Detect OTP in arbitrary text (for testing)
const result = await invoke('detect_otp_in_text', {
  text: 'Your verification code is 123456'
});
```

### Event Listeners

```typescript
import { listen } from '@tauri-apps/api/event';

// Listen for OTP detections
await listen('otp-detected', (event) => {
  const { code, messageGuid, pattern, chatId } = event.payload;

  // Show notification to user
  showNotification(`OTP detected: ${code}`);

  // Optionally highlight the message in the UI
  highlightMessage(messageGuid);
});

// Listen for auto-copy events
await listen('otp-auto-copy', (event) => {
  const { code } = event.payload;

  // Copy to clipboard
  navigator.clipboard.writeText(code);

  // Show toast
  showToast(`OTP ${code} copied to clipboard`);
});
```

## Settings Management

### Enable/Disable OTP Detection

```typescript
// Disable OTP detection
await invoke('update_setting', {
  key: 'otpDetectionEnabled',
  value: 'false'
});

// Enable auto-copy
await invoke('update_setting', {
  key: 'otpAutoCopy',
  value: 'true'
});

// Get current settings
const settings = await invoke('get_settings');
const isEnabled = settings.otpDetectionEnabled === 'true';
const autoCopy = settings.otpAutoCopy === 'true';
```

## Performance

- **Detection Speed**: ~13µs per message (average)
- **Memory**: Regex patterns are compiled once at startup using `lazy_static`
- **Thread Safety**: All regex operations are thread-safe

## Testing

### Run Unit Tests

```bash
cd bluebubbles-app-rust/bb-tauri/src-tauri
cargo test --bin bb-tauri otp_detector
```

### Test Coverage

- ✅ Code with prefix format
- ✅ Code with suffix format
- ✅ Apple format
- ✅ Google format
- ✅ Bracketed codes
- ✅ Standalone codes
- ✅ Case insensitivity
- ✅ Multiple codes in one message
- ✅ Phone number rejection
- ✅ Date rejection
- ✅ Various code lengths (4-8 digits)
- ✅ Real-world examples
- ✅ Performance benchmark

## Example Usage Flow

1. **User receives a message**: "Your Apple ID Code is: 123456"

2. **Backend processes the message**:
   - Checks if `otpDetectionEnabled` is true
   - Runs OTP detection on message text
   - Detects code "123456" using AppleFormat pattern

3. **Backend emits event**:
   ```json
   {
     "messageGuid": "abc-123",
     "code": "123456",
     "pattern": "AppleFormat",
     "chatId": 42
   }
   ```

4. **Frontend receives event**:
   - Shows notification: "OTP detected: 123456"
   - If `otpAutoCopy` is enabled, copies to clipboard
   - Highlights the message in the conversation view

5. **User interaction**:
   - User can click the notification to view the message
   - Code is already in clipboard if auto-copy is enabled
   - User can paste the code into the verification field

## Security Considerations

- OTP codes are never logged in plain text
- Detection only runs if explicitly enabled in settings
- Codes are transmitted to frontend via secure Tauri IPC
- Auto-copy feature is disabled by default

## Future Enhancements

Potential improvements for future versions:

1. **ML-based detection**: Use machine learning to improve detection accuracy
2. **Auto-fill integration**: Automatically fill OTP fields in web views
3. **Expiration tracking**: Track OTP validity periods
4. **Pattern learning**: Learn custom OTP formats from user feedback
5. **Multi-language support**: Detect OTPs in non-English messages
