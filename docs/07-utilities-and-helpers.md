# Utilities and Helpers

This document provides a comprehensive reference for all utility functions and helper modules in the BlueBubbles Flutter application. These modules live under `lib/utils/` and `lib/helpers/` and supply shared logic consumed throughout the app.

---

## Table of Contents

1. [Crypto Utilities](#1-crypto-utilities)
2. [File Utilities](#2-file-utilities)
3. [String Utilities](#3-string-utilities)
4. [Emoji System](#4-emoji-system)
5. [Share Utilities](#5-share-utilities)
6. [Logger](#6-logger)
7. [Parsers](#7-parsers)
8. [Color Engine](#8-color-engine)
9. [Window Effects](#9-window-effects)
10. [Backend Helpers](#10-backend-helpers)
11. [Network Helpers](#11-network-helpers)
12. [Type Extensions and Helpers](#12-type-extensions-and-helpers)
13. [UI Helpers](#13-ui-helpers)

---

## 1. Crypto Utilities

**File:** `lib/utils/crypto_utils.dart`

Provides AES-256-CBC encryption and decryption compatible with CryptoJS. Used for encrypting data exchanged between the client and the BlueBubbles server when encryption is enabled.

### Algorithm Details

- **Cipher:** AES-256-CBC with PKCS7 padding
- **Key Derivation:** MD5-based EVP_BytesToKey (OpenSSL-compatible), producing a 32-byte key and 16-byte IV from a passphrase and random salt
- **Salt:** 8 bytes of cryptographically secure random non-zero values
- **Output Format:** Base64-encoded `"Salted__" + salt + ciphertext`

### Functions

#### `encryptAESCryptoJS`

```dart
String encryptAESCryptoJS(String plainText, String passphrase)
```

| Parameter | Type | Description |
|---|---|---|
| `plainText` | `String` | The text to encrypt |
| `passphrase` | `String` | The shared secret passphrase |

**Returns:** `String` -- Base64-encoded ciphertext with the `Salted__` prefix and 8-byte salt prepended.

**Usage:** Called when sending encrypted payloads to the server.

#### `decryptAESCryptoJS`

```dart
String decryptAESCryptoJS(String encrypted, String passphrase)
```

| Parameter | Type | Description |
|---|---|---|
| `encrypted` | `String` | Base64-encoded ciphertext (with salt header) |
| `passphrase` | `String` | The shared secret passphrase |

**Returns:** `String` -- The decrypted plaintext.

**Usage:** Called when receiving encrypted payloads from the server.

#### `deriveKeyAndIV`

```dart
Tuple2<Uint8List, Uint8List> deriveKeyAndIV(String passphrase, Uint8List salt)
```

| Parameter | Type | Description |
|---|---|---|
| `passphrase` | `String` | The passphrase to derive from |
| `salt` | `Uint8List` | 8-byte salt |

**Returns:** `Tuple2<Uint8List, Uint8List>` -- Item1 is the 32-byte key, Item2 is the 16-byte IV.

Implements the OpenSSL EVP_BytesToKey derivation using iterative MD5 hashing until 48 bytes are produced.

#### `createUint8ListFromString`

```dart
Uint8List createUint8ListFromString(String s)
```

Converts a string to a `Uint8List` by extracting each character's code unit. Used internally by the key derivation function.

#### `genRandomWithNonZero`

```dart
Uint8List genRandomWithNonZero(int seedLength)
```

Generates a `Uint8List` of the given length filled with cryptographically secure random values in the range 1-245 (no zero bytes). Used for salt generation.

---

## 2. File Utilities

**File:** `lib/utils/file_utils.dart`

### Functions

#### `loadPathAsFile`

```dart
Future<PlatformFile?> loadPathAsFile(String path) async
```

| Parameter | Type | Description |
|---|---|---|
| `path` | `String` | Absolute filesystem path to load |

**Returns:** `Future<PlatformFile?>` -- A `PlatformFile` instance containing the file name, bytes, size, and path. Returns `null` if the file does not exist.

**Usage:** Loads a file from disk into a `PlatformFile` object for use with the file picker / attachment system.

#### `fixSpeedyGifs`

```dart
Future<Uint8List> fixSpeedyGifs(Uint8List image) async
```

| Parameter | Type | Description |
|---|---|---|
| `image` | `Uint8List` | Raw GIF byte data |

**Returns:** `Future<Uint8List>` -- The corrected GIF byte data.

Scans a GIF's binary data for Graphic Control Extension blocks (`0x21 0xF9 0x04`) that have a zero delay time and sets the delay to `0x0A` (100ms). This prevents GIFs with missing delay values from playing at maximum speed. Runs in a background isolate via `compute`.

---

## 3. String Utilities

### `lib/utils/string_utils.dart`

#### `cleansePhoneNumber`

```dart
String cleansePhoneNumber(String phoneNumber)
```

| Parameter | Type | Description |
|---|---|---|
| `phoneNumber` | `String` | Raw phone number string |

**Returns:** `String` -- Phone number containing only digits and the `+` character.

Strips all non-numeric, non-plus characters from a phone number for normalization.

### `lib/helpers/types/helpers/string_helpers.dart`

#### `randomString`

```dart
String randomString(int length)
```

| Parameter | Type | Description |
|---|---|---|
| `length` | `int` | Desired string length |

**Returns:** `String` -- A random alphanumeric string.

Generates a random string from uppercase, lowercase, and digit characters. Used for creating temporary GUIDs (e.g., `temp-<random8>`).

#### `sanitizeString`

```dart
String sanitizeString(String? input)
```

| Parameter | Type | Description |
|---|---|---|
| `input` | `String?` | The string to sanitize |

**Returns:** `String` -- The input with the Unicode Object Replacement Character (`U+FFFC`) removed. Returns an empty string if input is null.

#### `isNullOrEmptyString`

```dart
bool isNullOrEmptyString(String? input)
```

**Returns:** `true` if the sanitized string is empty.

#### `parseLinks`

```dart
List<RegExpMatch> parseLinks(String text)
```

| Parameter | Type | Description |
|---|---|---|
| `text` | `String` | Text to scan for URLs |

**Returns:** `List<RegExpMatch>` -- All URL matches found using the global `urlRegex` pattern.

---

## 4. Emoji System

### `lib/utils/emoji.dart`

Provides lookup maps for emoji data using the `emojis` package.

#### Global Variables

| Variable | Type | Description |
|---|---|---|
| `emojiNames` | `Map<String, Emoji>` | Maps emoji short names (e.g., `thumbsup`) to `Emoji` objects |
| `emojiFullNames` | `Map<String, Emoji>` | Maps emoji full display names to `Emoji` objects |

Both maps are built at initialization from `Emoji.all()`.

### `lib/utils/emoticons.dart`

Provides text emoticon to emoji conversion, following the Apple Messages emoticon mapping.

#### `emoticonMap`

`final Map<String, String>` -- Maps 30+ text emoticons to their Unicode emoji equivalents. Examples:

| Emoticon | Emoji |
|---|---|
| `:)` / `:-)` | `U+1F60A` |
| `;)` | `U+1F609` |
| `<3` | `U+2764` |
| `(y)` | `U+1F44D` |

#### `emoticonRegex`

`final RegExp` -- Dynamically constructed regex that matches any emoticon from the map when surrounded by whitespace boundaries.

#### `replaceEmoticons`

```dart
(String newText, List<(int, int)> offsetsAndDifferences) replaceEmoticons(String text)
```

| Parameter | Type | Description |
|---|---|---|
| `text` | `String` | Input text potentially containing emoticons |

**Returns:** A record containing:
- `newText` -- The text with emoticons replaced by emoji characters
- `offsetsAndDifferences` -- List of `(offset, lengthDifference)` tuples for each replacement, useful for adjusting cursor positions

### Constants (`lib/helpers/types/constants.dart`)

| Constant | Type | Description |
|---|---|---|
| `emojiRegex` | `RegExp` | Comprehensive regex matching Unicode emoji sequences, including multi-codepoint sequences, skin tones, ZWJ sequences, and flag pairs |
| `bigEmojiScaleFactor` | `double` | `3.0` -- Scale multiplier for messages containing only 1-3 emoji |

---

## 5. Share Utilities

**File:** `lib/utils/share.dart`

The `Share` class provides static methods for sharing content with other apps via the `share_plus` package.

### Methods

#### `Share.file`

```dart
static void file(String subject, String filepath) async
```

| Parameter | Type | Description |
|---|---|---|
| `subject` | `String` | Share sheet subject line |
| `filepath` | `String` | Absolute path to the file to share |

Shows a platform share sheet with the specified file. On desktop platforms, displays a snackbar indicating sharing is unsupported.

#### `Share.text`

```dart
static void text(String subject, String text)
```

| Parameter | Type | Description |
|---|---|---|
| `subject` | `String` | Share sheet subject line |
| `text` | `String` | Text content to share |

Shares plain text via the platform share sheet.

#### `Share.location`

```dart
static Future<void> location(Chat chat) async
```

| Parameter | Type | Description |
|---|---|---|
| `chat` | `Chat` | The chat to send the location to |

Performs a complete location-sharing workflow:

1. Checks that location services are enabled (prompts user to enable if not)
2. Requests location permission (prompts user if denied)
3. Gets the current GPS position via `Geolocator`
4. Creates an Apple Maps VCF location file
5. Fetches a map preview image from Apple Maps metadata
6. Shows a confirmation dialog with the map preview
7. On confirmation, queues the location as an outgoing attachment with MIME type `text/x-vlocation`

---

## 6. Logger

**Directory:** `lib/utils/logger/`

### BaseLogger (`logger.dart`)

The `BaseLogger` class extends `GetxService` and wraps the `logger` package with custom configuration. A global singleton `Logger` is registered with GetX.

#### Log Levels

```dart
enum LogLevel { INFO, WARN, ERROR, DEBUG, TRACE, FATAL }
```

#### Key Properties

| Property | Type | Description |
|---|---|---|
| `logStream` | `StreamController<String>` | Broadcast stream for live log viewing |
| `latestLogName` | `String` | `'bluebubbles-latest.log'` |
| `logDir` | `String` | `<appDocDir>/logs` |
| `currentLevel` | `Level?` | Active log level filter (default: `Level.info`) |
| `showColors` | `bool` | ANSI color output (default: `true` in debug mode) |
| `excludeBoxes` | `Map<Level, bool>` | Controls box drawing per level |

#### Log Methods

All log methods follow the same signature pattern:

```dart
void info(dynamic log, {String? tag, Object? error, StackTrace? trace})
void warn(dynamic log, {String? tag, Object? error, StackTrace? trace})
void debug(dynamic log, {String? tag, Object? error, StackTrace? trace})
void error(dynamic log, {String? tag, Object? error, StackTrace? trace})
void trace(dynamic log, {String? tag, Object? error, StackTrace? trace})
void fatal(dynamic log, {String? tag, Object? error, StackTrace? trace})
```

Each method prefixes the log message with: `<ISO8601-UTC-timestamp> [LEVEL] [tag] message`

The default tag is `"BlueBubblesApp"`.

#### Management Methods

| Method | Returns | Description |
|---|---|---|
| `init()` | `Future<void>` | Initializes the logger; sets log level from settings once available |
| `createLogger()` | `Logger` | Builds a new `Logger` instance with current configuration |
| `reset()` | `void` | Resets all overrides back to defaults |
| `enableLiveLogging()` | `void` | Adds `LogStreamOutput` for real-time log viewing; disables ANSI colors |
| `disableLiveLogging()` | `void` | Removes `LogStreamOutput` and restores defaults |
| `compressLogs()` | `String` | Zips all `.log` files into a date-stamped archive; returns the ZIP path |
| `getLogs({maxLines})` | `Future<List<String>>` | Reads the latest log file, combines multi-line entries, and returns up to `maxLines` (default 1000) most recent entries with ANSI codes stripped |
| `clearLogs()` | `void` | Deletes all files in the log directory |

#### File Output Configuration

- **Max file size:** 1024 KB (1 MB)
- **Max rotated files:** 5
- **Flush delay:** 5 seconds
- **File naming:** `bluebubbles-<date>-<epoch>.log`

### DebugConsoleOutput (`outputs/debug_console_output.dart`)

```dart
class DebugConsoleOutput extends LogOutput
```

Routes log output to Flutter's `debugPrint` for console visibility during development.

### LogStreamOutput (`outputs/log_stream_output.dart`)

```dart
class LogStreamOutput extends LogOutput
```

Routes log output into `BaseLogger.logStream` for real-time UI display (e.g., the in-app log viewer).

### Task Logger (`task_logger.dart`)

```dart
AsyncTaskLogger asyncTaskLogger
```

An `AsyncTaskLogger` callback that bridges `async_task` package logging into the BlueBubbles logger. Maps:
- `'ERROR'` type to `Logger.error`
- `'WARN'` type to `Logger.warn`
- `'EXEC'` / `'INFO'` type to `Logger.debug`

---

## 7. Parsers

### API Payload Parser (Commented Out)

**File:** `lib/utils/parsers/event_payload/api_payload_parser.dart`

This file contains a commented-out `ApiPayloadParser` class. It was designed to parse server API event payloads and enrich entity references (message GUIDs, chat GUIDs, etc.) by fetching full objects from the API. The class supported:

- Legacy vs. modern payload formats
- Message, Chat, Attachment, and Handle entity types
- Enrichment by fetching full entity data when payloads contained only GUIDs

**Current status:** Entirely commented out and not in active use. The enrichment endpoints for chats, attachments, and handles were marked as TODO.

---

## 8. Color Engine

**Directory:** `lib/utils/color_engine/`

A Dart port of the kdrag0n Monet color engine, used as a fallback when the host platform does not support wallpaper-based Material You (Monet) theming. The engine generates a complete Material You color scheme from a single seed color.

### Color Spaces (`colors.dart`)

The engine defines a full color space conversion pipeline:

#### Abstract Base

```dart
abstract class Color {
  LinearSrgb toLinearSrgb();
}
```

All color types must convert to `LinearSrgb` as the interchange format.

#### Color Types

| Class | Components | Description |
|---|---|---|
| `LinearSrgb` | `r`, `g`, `b` (double, 0-1) | Linear sRGB color space; central interchange format |
| `Srgb` | `r`, `g`, `b` (double, 0-1) | Standard sRGB with gamma encoding/decoding |
| `Oklab` | `l`, `a`, `b` (double) | Oklab perceptual color space |
| `Oklch` | `l`, `c`, `h` (double) | Oklch cylindrical form of Oklab |
| `CieXyz` | `x`, `y`, `z` (double) | CIE 1931 XYZ color space |
| `CieLab` | `l`, `a`, `b` (double) | CIELAB perceptual color space |
| `CieLch` | `l`, `c`, `h` (double) | Cylindrical form of CIELAB |

#### Conversion Methods

Each class provides conversion methods to related color spaces:

| From | To | Method |
|---|---|---|
| `LinearSrgb` | `Srgb` | `toSrgb()` |
| `LinearSrgb` | `Oklab` | `toOklab()` |
| `LinearSrgb` | `CieXyz` | `toCieXyz()` |
| `Srgb` | `LinearSrgb` | `toLinearSrgb()` |
| `Srgb` | 8-bit int | `quantize8()` |
| `Oklab` | `Oklch` | `toOklch()` |
| `Oklab` | `LinearSrgb` | `toLinearSrgb()` |
| `Oklch` | `Oklab` | `toOklab()` |
| `CieXyz` | `CieLab` | `toCieLab()` |
| `CieLab` | `CieXyz` | `toCieXyz()` |
| `CieLab` | `CieLch` | `toCieLch()` |

`Srgb.fromColor(ui.Color)` constructs from a Flutter `Color` object.

#### Utility Functions

| Function | Signature | Description |
|---|---|---|
| `toRadians` | `double toRadians(double degrees)` | Degrees to radians conversion |
| `toDegrees` | `double toDegrees(double radians)` | Radians to degrees conversion |
| `root` | `double root(num base, num factor)` | Nth root with precision rounding to 9 decimal places |

**Constant:** `illuminantsD65` -- CIE D65 standard illuminant (`CieXyz(0.95047, 1.0, 0.108883)`).

### Theme Generation (`theme.dart`)

#### Type Aliases

```dart
typedef ColorSwatch = Map<int, Color>;
typedef ColorFilter = Oklch Function(Oklch);
```

#### `ColorScheme` (Abstract)

Defines five color palettes: `neutral1`, `neutral2`, `accent1`, `accent2`, `accent3`.

#### `TargetColors`

Provides AOSP-default lightness maps and chroma values for the five palettes:

| Palette | Chroma Constant |
|---|---|
| `neutral1` | `0.0132` |
| `neutral2` | `0.0066` |
| `accent1` | `0.1212` |
| `accent2` | `0.04` |
| `accent3` | `0.06` |

The `lightnessMap` maps shade values (0, 10, 50, 100...1000) to Oklab lightness values. The `lstarLightnessMap` maps shades to CIELAB L* values.

#### `DynamicColorScheme`

Generates a complete dynamic color scheme from a primary seed color.

```dart
const DynamicColorScheme({
  required this.targetColors,
  required this.primaryColor,
  this.chromaMultiplier = 1.0,
  this.accurateShades = true,
})
```

| Parameter | Type | Description |
|---|---|---|
| `targetColors` | `ColorScheme` | Target lightness/chroma palette |
| `primaryColor` | `Color` | The seed color |
| `chromaMultiplier` | `double` | Scales chroma of the primary color |
| `accurateShades` | `bool` | Enables binary search for CIELAB L* accuracy |

Key behavior:
- `accent3` shifts the primary hue by 60 degrees for a complementary tertiary color
- When `accurateShades` is true, a binary search finds the Oklab lightness that matches each target CIELAB L* value after 8-bit sRGB quantization

#### `MonetColors`

A collection of five `MonetPalette` instances:

| Property | Description |
|---|---|
| `accent1` | Main accent, close to primary |
| `accent2` | Darker shades of accent1 |
| `accent3` | Tertiary, hue-shifted by 60 degrees |
| `neutral1` | Main background, tinted with primary |
| `neutral2` | Secondary background, slightly tinted |

#### `MonetPalette`

Extends `MaterialColor` with 13 shades (0, 10, 50, 100, 200...1000). Each shade is accessible via `shade0` through `shade1000` getters.

---

## 9. Window Effects

**File:** `lib/utils/window_effects.dart`

Manages desktop window transparency effects on Windows using the `flutter_acrylic` package.

### `EffectDependencies` Enum

```dart
enum EffectDependencies { brightness, color }
```

### `WindowEffects` Class

#### Supported Effects and Version Requirements

| Effect | Min Build | Max Build | Dependencies |
|---|---|---|---|
| `tabbed` | 22523 | -- | brightness |
| `mica` | 22000 | -- | brightness |
| `aero` | 0 | 22523 | color |
| `acrylic` | 17134 | -- | color |
| `transparent` | 0 | -- | color |
| `disabled` | 0 | -- | none |

#### Static Methods

| Method | Signature | Description |
|---|---|---|
| `effects` | `List<WindowEffect> get effects` | Returns effects available on the current Windows build |
| `descriptions` | `Map<WindowEffect, String> get descriptions` | Human-readable descriptions for available effects |
| `getOpacity` | `double getOpacity({required Color color})` | Returns the user-configured opacity for dark or light based on the color's luminance |
| `defaultOpacity` | `double defaultOpacity({required bool dark})` | Returns the built-in default opacity for the current effect and theme brightness |
| `dependsOnColor` | `bool dependsOnColor()` | Whether the current effect requires a background color |
| `withOpacity` | `Color withOpacity({required Color color})` | Applies the configured opacity to the given color |
| `isDark` | `bool isDark({required Color color})` | Returns `true` if the color's luminance is <= 0.5 |
| `setEffect` | `Future<void> setEffect({required Color color})` | Applies the configured window effect; handles Windows-version-specific acrylic transparency workarounds |

### `parsedWindowsVersion`

```dart
int parsedWindowsVersion()
```

**Returns:** `int` -- The Windows build number parsed from `Platform.operatingSystemVersion`. Returns `0` on parse failure.

---

## 10. Backend Helpers

### Startup Tasks (`lib/helpers/backend/startup_tasks.dart`)

The `StartupTasks` class orchestrates application initialization.

#### Static Properties

| Property | Type | Description |
|---|---|---|
| `uiReady` | `Completer<void>` | Completer that resolves when the UI is ready |

#### Static Methods

| Method | Description |
|---|---|
| `waitForUI()` | Awaits the `uiReady` completer |
| `initStartupServices({bool isBubble = false})` | Main initialization sequence: filesystem, logger, instance lock check, settings, database, FCM data, method channel, lifecycle, theme, contacts, notifications, intents |
| `initIsolateServices()` | Lighter initialization for background isolates: filesystem (headless), logger, settings (headless), database, method channel (headless), lifecycle (headless) |
| `initIncrementalSyncServices()` | Minimal initialization for incremental sync: filesystem, logger, settings, database |
| `onStartup()` | Post-UI startup tasks: chat initialization, server details fetch, FCM registration, delayed update checks, review flow |
| `checkInstanceLock()` | Linux-only single-instance enforcement using a PID lockfile and filesystem watch for foreground signals |

#### `reviewFlow`

```dart
Future<void> reviewFlow() async
```

Checks whether to prompt for an in-app review based on install date and last review request timestamp. Triggers after 7 days for first-time, or 30 days for subsequent requests.

#### `requestReview`

```dart
Future<void> requestReview() async
```

Invokes the `InAppReview` plugin to show the platform review dialog.

### Foreground Service Helpers (`lib/helpers/backend/foreground_service_helpers.dart`)

#### `runForegroundService`

```dart
Future<void> runForegroundService() async
```

Starts or stops the Android foreground service based on the `keepAppAlive` setting. Only operates on Android.

#### `restartForegroundService`

```dart
Future<void> restartForegroundService() async
```

Stops then restarts the Android foreground service. Only operates when `keepAppAlive` is enabled on Android.

### Settings Helpers (`lib/helpers/backend/settings_helpers.dart`)

#### `saveNewServerUrl`

```dart
Future<bool> saveNewServerUrl(
  String newServerUrl,
  {
    bool tryRestartForegroundService = true,
    bool restartSocket = true,
    bool force = false,
    List<String> saveAdditionalSettings = const []
  }
) async
```

| Parameter | Type | Description |
|---|---|---|
| `newServerUrl` | `String` | The new server URL |
| `tryRestartForegroundService` | `bool` | Whether to restart the foreground service |
| `restartSocket` | `bool` | Whether to restart the socket connection |
| `force` | `bool` | Save even if the URL hasn't changed |
| `saveAdditionalSettings` | `List<String>` | Additional setting keys to persist alongside the URL |

**Returns:** `true` if the URL was saved (changed or forced).

Sanitizes the URL, persists it, optionally restarts the foreground service and socket connection.

#### `clearServerUrl`

```dart
Future<void> clearServerUrl({
  bool tryRestartForegroundService = true,
  List<String> saveAdditionalSettings = const []
}) async
```

Clears the server URL to an empty string and persists the change.

#### `disableBatteryOptimizations`

```dart
Future<bool> disableBatteryOptimizations() async
```

**Returns:** `true` if battery optimizations are disabled after the prompt.

Checks the current battery optimization status and prompts the user to disable optimizations if they are currently enabled.

### Sync Helpers (`lib/helpers/backend/sync/sync_helpers.dart`)

#### `syncAttachments`

```dart
List<Attachment> syncAttachments(List<Attachment> attachments)
```

| Parameter | Type | Description |
|---|---|---|
| `attachments` | `List<Attachment>` | Attachments to sync with the local database |

**Returns:** `List<Attachment>` -- The synchronized attachments with database IDs populated.

Performs an upsert operation: queries the database for existing attachments by GUID, inserts new ones, merges updates into existing ones, and returns the final list with database-assigned IDs.

#### `syncMessages`

```dart
List<Message> syncMessages(Chat c, List<Message> messages)
```

| Parameter | Type | Description |
|---|---|---|
| `c` | `Chat` | The chat these messages belong to |
| `messages` | `List<Message>` | Messages to sync with the local database |

**Returns:** `List<Message>` -- The synchronized messages with database IDs and chat relationships populated.

Similar upsert logic to `syncAttachments`, plus a chat-matching step that associates messages with the given chat. The chat-matching step retries up to 3 times on failure.

---

## 11. Network Helpers

### Network Helpers (`lib/helpers/network/network_helpers.dart`)

#### `sanitizeServerAddress`

```dart
String? sanitizeServerAddress({String? address})
```

| Parameter | Type | Description |
|---|---|---|
| `address` | `String?` | Raw server address (falls back to `http.origin`) |

**Returns:** `String?` -- The sanitized URI string, or `null` if empty.

Strips quotes and whitespace, then ensures the URL has a scheme. Automatically applies `https://` for known tunnel providers (ngrok, Cloudflare, zrok), and `http://` for everything else.

#### `getDeviceName`

```dart
Future<String> getDeviceName() async
```

**Returns:** `Future<String>` -- A device identifier string (default: `"bluebubbles-client"`).

Constructs a device name from platform-specific info:
- **Android:** `brand_model_uniqueId`
- **Web:** `browserName_platform`
- **Windows:** `computerName`
- **Linux:** `prettyName`

The unique ID is generated once and stored in settings for idempotency.

### Metadata Helper (`lib/helpers/network/metadata_helper.dart`)

The `MetadataHelper` class provides URL preview metadata extraction with caching.

#### Static Methods

| Method | Signature | Description |
|---|---|---|
| `mapIsNotEmpty` | `bool mapIsNotEmpty(Map<String, dynamic>? data)` | Checks if metadata map has a non-null `title` key |
| `isNotEmpty` | `bool isNotEmpty(Metadata? data)` | Checks if any of title, description, or image is present |
| `fetchMetadata` | `Future<Metadata?> fetchMetadata(Message message)` | Fetches and caches URL metadata for a message |

`fetchMetadata` behavior:
1. Returns cached result if available for the message GUID
2. Extracts metadata using the `metadata_fetch` package
3. Falls back to manual HTML parsing with a search engine crawler user-agent if the initial fetch yields empty data
4. Handles image URLs that are relative paths, protocol-relative, or tracking pixels
5. Cache entries expire after 15 seconds

### Network Error Handler (`lib/helpers/network/network_error_handler.dart`)

#### `handleSendError`

```dart
Message handleSendError(dynamic error, Message m)
```

| Parameter | Type | Description |
|---|---|---|
| `error` | `dynamic` | The caught error (`Response`, `DioException`, or other) |
| `m` | `Message` | The message that failed to send |

**Returns:** `Message` -- The message with its GUID updated to `error-<description>` and error code set.

Maps Dio exception types to human-readable error messages:
- `connectionTimeout` -- "Connect timeout occured!"
- `sendTimeout` -- "Send timeout occured!"
- `receiveTimeout` -- "Receive data timeout occured!"

### Network Tasks (`lib/helpers/network/network_tasks.dart`)

The `NetworkTasks` class manages reconnection and localhost detection.

#### `NetworkTasks.onConnect`

```dart
static Future<void> onConnect() async
```

Called when network connectivity is established. Triggers incremental sync if the app is resumed from a paused/hidden state. On web, reloads chats and contacts if empty. Also triggers localhost detection if configured.

#### `NetworkTasks.detectLocalhost`

```dart
static Future<void> detectLocalhost({bool createSnackbar = false}) async
```

Detects whether the BlueBubbles server is running on the local network. The detection strategy:

1. Fetches the server's local IPv4/IPv6 addresses from the server info endpoint
2. Attempts to ping each local address on the configured localhost port (IPv6 first if enabled, then IPv4)
3. If direct address ping fails, falls back to a network port scan using `network_tools`
4. Sets `http.originOverride` to the discovered local address for direct LAN communication

---

## 12. Type Extensions and Helpers

### Extensions (`lib/helpers/types/extensions/extensions.dart`)

#### `DateHelpers` on `DateTime`

| Method | Returns | Description |
|---|---|---|
| `isToday()` | `bool` | Whether the date is today |
| `isYesterday()` | `bool` | Whether the date is yesterday |
| `isWithin(DateTime other, {ms, seconds, minutes, hours, days})` | `bool` | Whether the date is within the specified duration of another date |

#### `MessageErrorExtension` on `MessageError`

Maps `MessageError` enum values to integer codes:

| Error | Code |
|---|---|
| `NO_ERROR` | 0 |
| `TIMEOUT` | 4 |
| `NO_CONNECTION` | 1000 |
| `BAD_REQUEST` | 1001 |
| `SERVER_ERROR` | 1002 |

#### `EffectHelper` on `MessageEffect`

| Property | Returns | Description |
|---|---|---|
| `isBubble` | `bool` | Whether the effect is a bubble effect (slam, loud, gentle, invisible ink) |

#### `WidgetLocation` on `GlobalKey`

```dart
Rect? globalPaintBounds(BuildContext context)
```

Returns the global paint bounds of the widget as a `Rect`, adjusted for navigation panel width. Used for positioning iMessage effects.

#### `TextBubbleColumn` on `List<Widget>`

```dart
List<Widget> conditionalReverse(bool isFromMe)
```

Reverses the widget list if the message is not from the current user, used for message layout direction.

#### `NonZero` on `int?`

```dart
int? get nonZero
```

Returns `null` if the value is `null` or `0`, otherwise returns the value.

#### `FriendlySize` on `double`

```dart
String getFriendlySize({int decimals = 2, bool withSuffix = true})
```

Formats a byte count as a human-readable size string (KB, MB, GB).

#### `ChatListHelpers` on `RxList<Chat>`

| Method | Description |
|---|---|
| `archivedHelper(bool archived)` | Filters to archived or non-archived chats |
| `bigPinHelper(bool pinned)` | Filters to pinned or non-pinned chats |
| `unknownSendersHelper(bool unknown)` | Filters to chats with known or unknown senders (based on contact data) |

#### `PlatformSpecificCapitalize` on `String`

```dart
String get psCapitalize
```

Returns `toUpperCase()` when the skin is iOS, otherwise returns the original string.

#### `LastChars` on `String`

```dart
String lastChars(int n)
```

Returns the last `n` characters of the string.

#### `UrlParsing` on `String`

```dart
bool get hasUrl
```

Returns `true` if the string contains a URL (not on web platform).

#### `ShortenString` on `String`

```dart
String shorten(int length)
```

Truncates the string to `length` characters with an ellipsis if longer.

### Constants (`lib/helpers/types/constants.dart`)

#### Message Effect Maps

| Constant | Type | Description |
|---|---|---|
| `effectMap` | `Map<String, String>` | Maps human-readable effect names to Apple's internal effect identifiers |
| `stringToMessageEffect` | `Map<String?, MessageEffect>` | Maps string names to the `MessageEffect` enum |

#### Balloon Bundle ID Maps

| Constant | Description |
|---|---|
| `balloonBundleIdMap` | Maps Apple balloon bundle identifiers to human-readable names (GamePigeon, YouTube, Apple Pay, etc.) |
| `balloonBundleIdIconMap` | Maps balloon bundle identifiers to platform-appropriate icons (Cupertino for iOS skin, Material otherwise) |

#### Enums

| Enum | Values |
|---|---|
| `MessageEffect` | `none`, `slam`, `loud`, `gentle`, `invisibleInk`, `echo`, `spotlight`, `balloons`, `confetti`, `love`, `lasers`, `fireworks`, `celebration` |
| `MessageError` | `NO_ERROR`, `TIMEOUT`, `NO_CONNECTION`, `BAD_REQUEST`, `SERVER_ERROR` |
| `Skins` | `iOS`, `Material`, `Samsung` |
| `SwipeDirection` | `LEFT`, `RIGHT` |
| `MaterialSwipeAction` | `pin`, `alerts`, `delete`, `mark_read`, `archive` |
| `SecurityLevel` | `locked`, `locked_and_secured` |
| `Monet` | `none`, `harmonize`, `full` |
| `Indicator` | `READ`, `DELIVERED`, `SENT`, `NONE` |
| `LoadMessageResult` | `RETRIEVED_MESSAGES`, `FAILED_TO_RETRIEVE`, `RETRIEVED_LAST_PAGE` |
| `SearchMethod` | `local`, `network` |
| `LineType` | `meToMe`, `otherToMe`, `meToOther`, `otherToOther` |
| `PlayerStatus` | `NONE`, `STOPPED`, `PAUSED`, `PLAYING`, `ENDED` |
| `PayloadType` | `url`, `app` |

#### Global Regex and Constants

| Constant | Type | Description |
|---|---|---|
| `urlRegex` | `RegExp` | Matches HTTP/HTTPS/FTP URLs and `www.` prefixed URLs |
| `emojiRegex` | `RegExp` | Comprehensive Unicode emoji pattern |
| `bigEmojiScaleFactor` | `double` | `3.0` |
| `appName` | `String` | `"BlueBubbles"` |
| `msStorePackageName` | `String` | `"23344BlueBubbles.BlueBubbles"` |
| `windowsAppPackageName` | `String` | Full Windows MSIX package name |

### Language Codes (`lib/helpers/types/classes/language_codes.dart`)

Provides a list of language names and BCP 47 language codes for use with the spell-check / language tool integration.

| Variable | Type | Description |
|---|---|---|
| `languageData` | `List<Map<String, String>>` | Raw data with `name` and `longCode` entries for 46 languages |
| `languageNameAndCodes` | `List<(String, String)>` | Sorted list of `(name, code)` tuples |

### Miscellaneous Helpers (`lib/helpers/types/helpers/misc_helpers.dart`)

#### `isNullOrEmpty`

```dart
bool isNullOrEmpty(dynamic input)
```

Returns `true` if input is null, empty, or blank (strings are trimmed first).

#### `isNullOrZero`

```dart
bool isNullOrZero(dynamic input)
```

Returns `true` if input is null, `0`, or `0.0`.

#### `mergeTopLevelDicts`

```dart
Map<String, dynamic> mergeTopLevelDicts(Map<String, dynamic>? d1, Map<String, dynamic>? d2)
```

Merges two dictionaries, adding keys from `d2` that are not present in `d1`. Does not overwrite existing keys.

#### `createAsyncTask`

```dart
Future<T?> createAsyncTask<T>(AsyncTask<List<dynamic>, T> task) async
```

Wraps a synchronous task in an `AsyncExecutor` to prevent UI jank during heavy ObjectBox operations.

#### Platform Detection Getters

| Getter | Type | Description |
|---|---|---|
| `kIsDesktop` | `bool` | `true` on Windows, Linux, or macOS (and not web) |
| `isSnap` | `bool` | `true` when running as a Linux Snap package |
| `isFlatpak` | `bool` | `true` when running as a Flatpak |
| `isMsix` | `bool` | `true` when running as an MSIX (Microsoft Store) package |

#### `intersperse`

```dart
Iterable<T> intersperse<T>(T element, Iterable<T> iterable) sync*
```

Inserts `element` between each item of the iterable (like `join` for iterables).

#### `prettyDuration`

```dart
String prettyDuration(Duration duration)
```

Formats a `Duration` as a human-readable string (e.g., `1:30`, `02:15:30`). Leading zero on the first component is stripped.

### Contact Helpers (`lib/helpers/types/helpers/contact_helpers.dart`)

#### `formatPhoneNumber`

```dart
Future<String> formatPhoneNumber(dynamic item) async
```

| Parameter | Type | Description |
|---|---|---|
| `item` | `dynamic` | A `String`, `Handle`, or other object |

**Returns:** Internationally formatted phone number string, or the original value for emails.

Uses the device locale's country code and `dlibphonenumber` for formatting.

#### `getUniqueNumbers`

```dart
List<String> getUniqueNumbers(Iterable<String> numbers)
```

Deduplicates phone numbers by comparing their numeric-only representations.

#### `getUniqueEmails`

```dart
List<String> getUniqueEmails(Iterable<String> list)
```

Deduplicates email addresses by trimmed string comparison.

#### `getDisplayName`

```dart
String getDisplayName(String? displayName, String? firstName, String? lastName)
```

Returns the display name if non-empty, otherwise joins first and last names with a space.

### Date Helpers (`lib/helpers/types/helpers/date_helpers.dart`)

#### `parseDate`

```dart
DateTime? parseDate(dynamic value)
```

Parses a date from `int` (epoch ms), `String` (ISO 8601), or `DateTime`. Returns `null` for unsupported types.

#### `buildDate`

```dart
String buildDate(DateTime? dateTime)
```

Formats a date for display in chat lists, respecting the active skin (iOS, Material, Samsung), 24-hour format setting, and relative time thresholds (Just Now, X min, Today, Yesterday, weekday, full date).

#### `buildChatListDateMaterial`

```dart
String buildChatListDateMaterial(DateTime? dateTime)
```

Similar to `buildDate` but tailored for Material/Samsung chat list timestamps without time components for older dates.

#### `buildSeparatorDateMaterial`

```dart
String buildSeparatorDateMaterial(DateTime dateTime)
```

Formats as abbreviated month-day-weekday (e.g., "Mon, Jan 15").

#### `buildSeparatorDateSamsung`

```dart
String buildSeparatorDateSamsung(DateTime dateTime)
```

Formats as full date with day name (e.g., "Monday, January 15, 2024").

#### `buildTime`

```dart
String buildTime(DateTime? dateTime)
```

Formats just the time portion, respecting the 24-hour format setting.

#### `buildFullDate`

```dart
String buildFullDate(DateTime time, {bool includeTime = true, bool useTodayYesterday = true})
```

Formats a complete date with optional time and Today/Yesterday labels.

### File Helpers (`lib/helpers/types/helpers/file_helpers.dart`)

#### `getGifDimensions`

```dart
Size getGifDimensions(Uint8List bytes)
```

| Parameter | Type | Description |
|---|---|---|
| `bytes` | `Uint8List` | Raw GIF byte data |

**Returns:** `Size` -- The width and height extracted from the GIF header (bytes 6-9).

Reads the GIF Logical Screen Descriptor bytes to determine the image dimensions.

### Message Helper (`lib/helpers/types/helpers/message_helper.dart`)

The `MessageHelper` class provides static methods for message processing.

#### `MessageHelper.bulkAddMessages`

```dart
static Future<List<Message>> bulkAddMessages(
  Chat? chat,
  List<dynamic> messages,
  {bool checkForLatestMessageText = true,
   Function(int progress, int length)? onProgress}
) async
```

Bulk ingests messages into the database with progress reporting. Handles chat resolution, attachment saving, and logs progress every 50 messages.

#### `MessageHelper.handleNotification`

```dart
static Future<void> handleNotification(Message message, Chat chat, {bool findExisting = true}) async
```

Determines whether to show a notification for a message. Suppresses notifications for:
- Messages from the current user
- "Kept audio" messages
- Already-existing messages
- Muted chats
- Active chat conversations
- Chat list when `notifyOnChatList` is disabled

#### `MessageHelper.handleSummaryNotification`

```dart
static Future<void> handleSummaryNotification(List<Message> messages, {bool findExisting = true}) async
```

Creates a summary notification for multiple messages on desktop platforms.

#### `MessageHelper.getNotificationText`

```dart
static String getNotificationText(Message message, {bool withSender = false})
```

Generates human-readable notification text from a message, handling:
- Group events
- Invisible ink messages
- Reactions (with verb mapping like "loved", "laughed at")
- Attachment summaries (counts by type: "2 images & 1 movie")
- Interactive messages (GamePigeon, Apple Pay, etc.)
- Unsent/empty messages

#### `MessageHelper.normalizedAssociatedMessages`

```dart
static List<Message> normalizedAssociatedMessages(List<Message> associatedMessages)
```

Deduplicates reaction messages, keeping only the latest reaction per user and excluding removed reactions.

#### `MessageHelper.shouldShowBigEmoji`

```dart
static bool shouldShowBigEmoji(String text)
```

Returns `true` if the message text contains 1-3 emoji and no other content, triggering enlarged emoji display.

#### `MessageHelper.buildEmojiText`

```dart
static List<TextSpan> buildEmojiText(String text, TextStyle style, {TapGestureRecognizer? recognizer})
```

Splits text into segments, applying the "Apple Color Emoji" font family to emoji runs for proper rendering when a custom emoji font is installed.

---

## 13. UI Helpers

### Async Task (`lib/helpers/ui/async_task.dart`)

#### `runAsync`

```dart
Future<T> runAsync<T>(T Function() function) async
```

Schedules a function on Flutter's scheduler at animation priority, preventing UI jank for synchronous work.

### Attributed Body Helpers (`lib/helpers/ui/attributed_body_helpers.dart`)

#### `getAudioTranscriptsFromAttributedBody`

```dart
Map<int, String> getAudioTranscriptsFromAttributedBody(List<AttributedBody> attrBodies)
```

Extracts audio transcription text from attributed body runs, keyed by message part number.

### FaceTime Helpers (`lib/helpers/ui/facetime_helpers.dart`)

#### `faceTimeOverlays`

```dart
Map<String, Route> faceTimeOverlays
```

Global map tracking active FaceTime overlay dialogs by call UUID.

#### `hideFaceTimeOverlay`

```dart
void hideFaceTimeOverlay(String callUuid)
```

Dismisses the FaceTime overlay dialog and clears the associated notification.

#### `showFaceTimeOverlay`

```dart
Future<void> showFaceTimeOverlay(String callUuid, String caller, Uint8List? chatIcon, bool isAudio) async
```

Displays a blurred backdrop dialog for incoming FaceTime calls with Accept/Ignore buttons. Handles redacted mode by substituting a fake name and removing the avatar.

### OAuth Helpers (`lib/helpers/ui/oauth_helpers.dart`)

#### `googleOAuth`

```dart
Future<String?> googleOAuth(BuildContext context) async
```

Performs Google OAuth sign-in to obtain an access token. Uses `GoogleSignIn` on Android/Web and `DesktopWebviewAuth` on desktop. Requests scopes for Cloud Platform, Firebase, and Datastore.

**Returns:** The OAuth access token, or `null` on failure.

#### `fetchFirebaseProjects`

```dart
Future<List<Map>> fetchFirebaseProjects(String token) async
```

Queries Firebase for projects accessible with the given token, extracting the server URL from either Realtime Database or Cloud Firestore.

#### `requestPassword`

```dart
Future<void> requestPassword(BuildContext context, String serverUrl, Future<void> Function(String url, String password) connect) async
```

Shows a password input dialog and calls the `connect` callback with the server URL and entered password.

### Reaction Helpers (`lib/helpers/ui/reaction_helpers.dart`)

#### `ReactionTypes` Class

Static constants and maps for iMessage reaction types:

| Constant | Value |
|---|---|
| `LOVE` | `"love"` |
| `LIKE` | `"like"` |
| `DISLIKE` | `"dislike"` |
| `LAUGH` | `"laugh"` |
| `EMPHASIZE` | `"emphasize"` |
| `QUESTION` | `"question"` |

**Maps:**
- `reactionToVerb` -- Maps reaction types (and their negations) to English verbs (e.g., `"love"` to `"loved"`)
- `reactionToEmoji` -- Maps reaction types to emoji characters
- `emojiToReaction` -- Reverse mapping from emoji to reaction type

#### `getUniqueReactionMessages`

```dart
List<Message> getUniqueReactionMessages(List<Message> messages)
```

Filters reaction messages to only the latest non-negative reaction per user handle.

### Theme Helpers (`lib/helpers/ui/theme_helpers.dart`)

#### `HexColor`

```dart
class HexColor extends Color
```

Constructs a `Color` from a hex string (with or without `#` prefix, 6 or 8 characters).

#### `BubbleColors` ThemeExtension

Custom `ThemeExtension` holding six optional colors for message bubbles:

| Property | Description |
|---|---|
| `iMessageBubbleColor` | iMessage sent bubble background |
| `oniMessageBubbleColor` | iMessage sent bubble text |
| `smsBubbleColor` | SMS sent bubble background |
| `onSmsBubbleColor` | SMS sent bubble text |
| `receivedBubbleColor` | Received bubble background |
| `onReceivedBubbleColor` | Received bubble text |

#### `BubbleText` ThemeExtension

```dart
class BubbleText extends ThemeExtension<BubbleText>
```

Holds the `bubbleText` `TextStyle` used within message bubbles.

#### `ThemeHelpers` Mixin

A mixin for `StatefulWidget`s providing common theme accessors:

| Property | Type | Description |
|---|---|---|
| `reverseMapping` | `bool` | Whether to swap header/tile colors (Material dark mode) |
| `iosSubtitle` | `TextStyle` | Subtitle style for iOS skin ListTiles |
| `materialSubtitle` | `TextStyle` | Subtitle style for Material/Samsung skin ListTiles |
| `headerColor` | `Color` | Background color for settings page headers |
| `tileColor` | `Color` | Background color for settings page tiles |
| `showAltLayout` | `bool` | Whether to use tablet/split-view layout |
| `iOS` / `material` / `samsung` | `bool` | Skin type checks |
| `brightness` | `Brightness` | Current theme brightness |

#### `ColorSchemeHelpers` Extension on `ColorScheme`

| Property | Description |
|---|---|
| `properSurface` | Returns `surfaceVariant` if `surface` and `background` are too similar (difference < 8), otherwise `surface` |
| `properOnSurface` | Corresponding on-color for `properSurface` |
| `iMessageBubble` | The more colorful of `primary` and `primaryContainer` |
| `smsBubble` | The less colorful of `primary` and `primaryContainer` |
| `bubble(context, iMessage)` | Resolves bubble color with Monet and custom theme overrides |
| `onBubble(context, iMessage)` | Resolves bubble text color with Monet and custom theme overrides |

#### `ColorHelpers` Extension on `Color`

| Method | Signature | Description |
|---|---|---|
| `darkenPercent` | `Color darkenPercent([double percent = 10])` | Darkens by percentage (1-100) using RGB multiplication |
| `lightenPercent` | `Color lightenPercent([double percent = 10])` | Lightens by percentage (1-100) |
| `lightenOrDarken` | `Color lightenOrDarken([double percent = 10])` | Auto-selects darken or lighten based on proximity to black |
| `oppositeLightenOrDarken` | `Color oppositeLightenOrDarken([double percent = 10])` | Opposite of `lightenOrDarken` |
| `themeLightenOrDarken` | `Color themeLightenOrDarken(BuildContext context, [double percent = 10])` | Lightens in dark mode, darkens in light mode |
| `themeOpacity` | `Color themeOpacity(BuildContext context)` | Applies window effect opacity settings |
| `darkenAmount` | `Color darkenAmount([double amount = .1])` | Darkens using HSL lightness (0-1 range) |
| `lightenAmount` | `Color lightenAmount([double amount = .1])` | Lightens using HSL lightness (0-1 range) |
| `computeDifference` | `double computeDifference(Color? other)` | Euclidean distance between two colors as a percentage (0-100) |

#### `HSLHelpers` Extension on `HSLColor`

```dart
double get colorfulness
```

Computes a "colorfulness" metric based on distance from neutral gray in HSL space. Lower values indicate more saturated, mid-lightness colors.

#### Standalone Functions

| Function | Signature | Description |
|---|---|---|
| `createMaterialColor` | `MaterialColor createMaterialColor(Color color)` | Generates a `MaterialColor` swatch with 10 shades from a base color |
| `toColorGradient` | `List<Color> toColorGradient(String? str)` | Generates a deterministic two-color gradient from a string (used for avatar backgrounds) |

### UI Helpers (`lib/helpers/ui/ui_helpers.dart`)

#### Widgets

##### `BackButton`

```dart
class BackButton extends StatelessWidget {
  final bool Function()? onPressed;
  final Color? color;
}
```

Platform-aware back button that uses `CupertinoIcons.back` for non-Material skins and `Icons.arrow_back` for Material. Supports desktop tap handling.

##### `buildBackButton`

```dart
Widget buildBackButton(BuildContext context, {EdgeInsets padding, double? iconSize, Skins? skin, bool Function()? callback})
```

Legacy function-based back button builder (marked for removal).

##### `buildProgressIndicator`

```dart
Widget buildProgressIndicator(BuildContext context, {double size = 20, double strokeWidth = 2})
```

Returns a `CupertinoActivityIndicator` for iOS skin or a `CircularProgressIndicator` for other skins.

#### Dialog Functions

##### `showConversationTileMenu`

```dart
Future<void> showConversationTileMenu(BuildContext context, ConversationTileController _this, Chat chat, Offset tapPosition, TextTheme textTheme) async
```

Shows a context menu at the tap position with options: Pin/Unpin, Show/Hide Alerts, Mark Read/Unread, Archive/Unarchive, Delete.

##### `areYouSure`

```dart
AlertDialog areYouSure(BuildContext context, {Widget? content, String? title, required Function onNo, required Function onYes})
```

Returns a confirmation dialog with Yes/No buttons.

#### Utility Functions

##### `getAttachmentIcon`

```dart
IconData getAttachmentIcon(String mimeType)
```

Maps MIME types to appropriate icons (PDF, ZIP, audio, image, video, text) with platform-specific icon selection.

##### `showSnackbar`

```dart
void showSnackbar(String title, String message, {int animationMs = 250, int durationMs = 1500, Function(GetSnackBar)? onTap, TextButton? button})
```

Shows a bottom snackbar using GetX with the app's inverse surface colors.

##### `getIndicatorIcon` / `getIndicatorColor`

```dart
Widget getIndicatorIcon(SocketState socketState, {double size = 24, bool showAlpha = true})
Color getIndicatorColor(SocketState socketState)
```

Returns a colored dot indicator for socket connection state: yellow (connecting), green (connected), red (disconnected).

##### `avatarAsBytes`

```dart
Future<Uint8List> avatarAsBytes({required Chat chat, List<Handle>? participantsOverride, double quality = 256}) async
```

Renders a chat avatar to a PNG byte array at the specified quality (size in pixels).

##### `paintGroupAvatar`

```dart
Future<void> paintGroupAvatar({required Chat chat, required List<Handle>? participants, required Canvas canvas, required double size, required bool usingParticipantsOverride}) async
```

Paints a group avatar onto a canvas. Handles custom avatars, single-participant avatars, and multi-participant circular layouts with a configurable maximum avatar count.

##### `paintAvatar`

```dart
Future<void> paintAvatar({required Handle? handle, required Canvas canvas, required Offset offset, required double size, double? fontSize, double? borderWidth, bool inGroup = false}) async
```

Paints a single participant avatar with contact photo, gradient background, or initials.

##### `clip`

```dart
Future<Uint8List?> clip(Uint8List data, {required int size, required bool circle}) async
```

Resizes an image to the specified square size and optionally clips it to a circle.

##### `loadImage`

```dart
Future<ui.Image> loadImage(Uint8List data) async
```

Decodes a `Uint8List` into a `dart:ui` `Image` object.

##### `findChildIndexByKey`

```dart
int? findChildIndexByKey<T>(List<T> input, Key key, Function(T) getField)
```

Finds the index of an item in a list by matching against various Flutter `Key` types (`UniqueKey`, `ObjectKey`, `GlobalKey`, `ValueKey`). Returns `null` if not found.

### Message Widget Helpers (`lib/helpers/ui/message_widget_helpers.dart`)

#### `buildMessageSpans`

```dart
List<InlineSpan> buildMessageSpans(BuildContext context, MessagePart part, Message message, {Color? colorOverride, bool hideBodyText = false})
```

Builds styled `InlineSpan` list for message rendering, handling subjects (bold), mentions (bold with tap-to-view-contact), and emoji text styling.

#### `buildEnrichedMessageSpans`

```dart
Future<List<InlineSpan>> buildEnrichedMessageSpans(BuildContext context, MessagePart part, Message message, {Color? colorOverride, bool hideBodyText = false}) async
```

Extended version of `buildMessageSpans` that integrates Google ML Kit entity extraction for smart content detection:

| Entity Type | Action on Tap |
|---|---|
| Address | Opens maps app |
| Phone number | Opens dialer |
| Email | Opens email client |
| URL | Opens browser |
| Date/Time | Opens calendar |
| Tracking number | Copies number and searches Google |
| Flight number | Searches Google for flight info |
| Mention | Opens contact form |

Falls back to regex-based URL detection when ML Kit is not available (desktop/web).

---

## File Reference

| File Path | Section |
|---|---|
| `lib/utils/crypto_utils.dart` | [Crypto Utilities](#1-crypto-utilities) |
| `lib/utils/file_utils.dart` | [File Utilities](#2-file-utilities) |
| `lib/utils/string_utils.dart` | [String Utilities](#3-string-utilities) |
| `lib/utils/emoji.dart` | [Emoji System](#4-emoji-system) |
| `lib/utils/emoticons.dart` | [Emoji System](#4-emoji-system) |
| `lib/utils/share.dart` | [Share Utilities](#5-share-utilities) |
| `lib/utils/logger/logger.dart` | [Logger](#6-logger) |
| `lib/utils/logger/outputs/debug_console_output.dart` | [Logger](#6-logger) |
| `lib/utils/logger/outputs/log_stream_output.dart` | [Logger](#6-logger) |
| `lib/utils/logger/task_logger.dart` | [Logger](#6-logger) |
| `lib/utils/parsers/event_payload/api_payload_parser.dart` | [Parsers](#7-parsers) |
| `lib/utils/color_engine/colors.dart` | [Color Engine](#8-color-engine) |
| `lib/utils/color_engine/engine.dart` | [Color Engine](#8-color-engine) |
| `lib/utils/color_engine/theme.dart` | [Color Engine](#8-color-engine) |
| `lib/utils/window_effects.dart` | [Window Effects](#9-window-effects) |
| `lib/helpers/backend/startup_tasks.dart` | [Backend Helpers](#10-backend-helpers) |
| `lib/helpers/backend/foreground_service_helpers.dart` | [Backend Helpers](#10-backend-helpers) |
| `lib/helpers/backend/settings_helpers.dart` | [Backend Helpers](#10-backend-helpers) |
| `lib/helpers/backend/sync/sync_helpers.dart` | [Backend Helpers](#10-backend-helpers) |
| `lib/helpers/network/network_helpers.dart` | [Network Helpers](#11-network-helpers) |
| `lib/helpers/network/metadata_helper.dart` | [Network Helpers](#11-network-helpers) |
| `lib/helpers/network/network_error_handler.dart` | [Network Helpers](#11-network-helpers) |
| `lib/helpers/network/network_tasks.dart` | [Network Helpers](#11-network-helpers) |
| `lib/helpers/types/extensions/extensions.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/types/constants.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/types/classes/language_codes.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/types/helpers/misc_helpers.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/types/helpers/contact_helpers.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/types/helpers/date_helpers.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/types/helpers/file_helpers.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/types/helpers/message_helper.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/types/helpers/string_helpers.dart` | [Type Extensions](#12-type-extensions-and-helpers) |
| `lib/helpers/ui/async_task.dart` | [UI Helpers](#13-ui-helpers) |
| `lib/helpers/ui/attributed_body_helpers.dart` | [UI Helpers](#13-ui-helpers) |
| `lib/helpers/ui/facetime_helpers.dart` | [UI Helpers](#13-ui-helpers) |
| `lib/helpers/ui/oauth_helpers.dart` | [UI Helpers](#13-ui-helpers) |
| `lib/helpers/ui/reaction_helpers.dart` | [UI Helpers](#13-ui-helpers) |
| `lib/helpers/ui/theme_helpers.dart` | [UI Helpers](#13-ui-helpers) |
| `lib/helpers/ui/ui_helpers.dart` | [UI Helpers](#13-ui-helpers) |
| `lib/helpers/ui/message_widget_helpers.dart` | [UI Helpers](#13-ui-helpers) |
| `lib/helpers/helpers.dart` | Barrel export file for all helpers |
