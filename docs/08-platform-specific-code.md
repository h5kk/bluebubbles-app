# 08 - Platform-Specific and Native Code

This document covers every platform target in the BlueBubbles Flutter application, including build configuration, native code, permissions, protocol handling, and packaging. All values are sourced directly from the repository at `bluebubbles-app-ELECTRON/`.

---

## Table of Contents

1. [Android Configuration](#1-android-configuration)
2. [iOS Configuration](#2-ios-configuration)
3. [Windows Configuration](#3-windows-configuration)
4. [Linux Configuration](#4-linux-configuration)
5. [macOS Configuration](#5-macos-configuration)
6. [Web Configuration](#6-web-configuration)
7. [Native Code (Kotlin)](#7-native-code-kotlin)
8. [Protocol Handling](#8-protocol-handling)
9. [Permissions Matrix](#9-permissions-matrix)
10. [Build Instructions](#10-build-instructions)

---

## 1. Android Configuration

Source files: `android/build.gradle`, `android/settings.gradle`, `android/app/build.gradle`, `android/app/src/main/AndroidManifest.xml`

### 1.1 SDK and Compile Targets

| Property | Value |
|---|---|
| `compileSdk` | 34 |
| `minSdkVersion` | 23 (Android 6.0 Marshmallow) |
| `targetSdkVersion` | 34 (Android 14) |
| `namespace` | `com.bluebubbles.messaging` |
| Java source/target compatibility | Java 17 |
| Core library desugaring | Enabled (`com.android.tools:desugar_jdk_libs:2.0.4`) |

### 1.2 Gradle Plugin Versions

Defined in `android/settings.gradle`:

| Plugin | Version |
|---|---|
| `com.android.application` | 8.3.1 |
| `com.android.library` | 8.3.1 |
| `org.jetbrains.kotlin.android` | 1.9.23 |
| `com.google.gms.google-services` | 4.4.1 |
| `dev.flutter.flutter-plugin-loader` | 1.0.0 |
| `org.gradle.toolchains.foojay-resolver-convention` | 0.8.0 |

### 1.3 Repositories

The root `build.gradle` configures these Maven repositories for all subprojects:

- `google()`
- `https://jitpack.io` (for UnifiedPush and other GitHub-hosted libraries)
- `https://maven.google.com`
- `mavenCentral()`

### 1.4 Product Flavors

The app uses a `dimension "app"` flavor axis with five flavors:

| Flavor | Application ID | Display Name | Launcher Background Color |
|---|---|---|---|
| `joel` | `com.bluebubbles.messaging.joel` | BlueBubbles Dev (Joel) | `#4c49de` |
| `tanay` | `com.bluebubbles.messaging.tanay` | BlueBubbles (Tanay Special Sauce) | `#4c49de` |
| `alpha` | `com.bluebubbles.messaging.alpha` | BlueBubbles (Alpha) | `#49dbde` |
| `beta` | `com.bluebubbles.messaging.beta` | BlueBubbles (Beta) | `#4990de` |
| `prod` (default) | `com.bluebubbles.messaging` | BlueBubbles | `#4990de` |

Each flavor defines its own `file_provider` authority string following the pattern `<applicationId>.fileprovider`.

### 1.5 Signing Configuration

- **Release builds** for `alpha`, `beta`, and `prod` use the `release` signing config, which loads keystore properties from `android/key.properties` (not committed to the repository).
- **Release builds** for `joel` and `tanay` developer flavors use the `debug` signing config.
- `minifyEnabled` and `shrinkResources` are both `false` for release builds.

### 1.6 Version Code Calculation

```
versionCode = 20002000 + flutterVersionCode
```

With `pubspec.yaml` version `1.15.0+70`, the resulting Android version code is `20002070`.

### 1.7 Build Options

| Option | Value | Purpose |
|---|---|---|
| `aaptOptions.noCompress` | `"tflite"` | Prevents compression of TensorFlow Lite model files |
| `lintOptions.checkReleaseBuilds` | `false` | Disables lint checks on release builds |
| `lintOptions.disable` | `'InvalidPackage'` | Suppresses InvalidPackage lint warnings |

### 1.8 Dependencies (Native Android)

```groovy
// AndroidX
implementation "androidx.core:core-ktx:1.13.1"
implementation "androidx.sharetarget:sharetarget:1.2.0"
implementation 'androidx.browser:browser:1.8.0'
implementation 'androidx.activity:activity-ktx:1.9.1'
implementation "androidx.work:work-runtime:2.9.0"
implementation "androidx.concurrent:concurrent-futures-ktx:1.2.0"

// Firebase
implementation 'com.google.firebase:firebase-messaging:24.0.0'
implementation 'com.google.firebase:firebase-database:21.0.0'
implementation 'com.google.firebase:firebase-messaging-directboot:24.0.0'
implementation 'com.google.firebase:firebase-iid:21.1.0'
implementation 'com.google.firebase:firebase-firestore:25.0.0'

// Kotlin Coroutines
implementation "org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1"
implementation "org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1"
implementation "org.jetbrains.kotlinx:kotlinx-coroutines-play-services:1.8.1"
implementation "org.jetbrains.kotlinx:kotlinx-coroutines-guava:1.8.1"

// Socket.IO (with org.json excluded; provided by Android)
implementation 'io.socket:socket.io-client:2.1.1'

// JSON
implementation 'com.google.code.gson:gson:2.10.1'

// Unified Push
implementation 'com.github.UnifiedPush:android-connector:2.5.0'

// Desugaring
coreLibraryDesugaring("com.android.tools:desugar_jdk_libs:2.0.4")
```

A resolution strategy forces specific versions for Cloud Firestore compatibility:

```groovy
force 'com.squareup.okhttp:okhttp:2.7.5'
force 'com.squareup.okio:okio:1.17.5'
```

### 1.9 Android Manifest -- Permissions

Declared in `android/app/src/main/AndroidManifest.xml`:

**Core permissions:**

| Permission | Purpose |
|---|---|
| `CAMERA` | Photo/video capture for attachments |
| `ACCESS_NETWORK_STATE` | Network connectivity checks |
| `INTERNET` | Server communication |
| `VIBRATE` | Notification haptics |
| `READ_CONTACTS` | Contact display and sharing |
| `WRITE_CONTACTS` | Contact creation from the app |
| `RECORD_AUDIO` | Voice message recording |

**Optional permissions:**

| Permission | Purpose |
|---|---|
| `CALL_PHONE` | Initiating phone calls |
| `USE_BIOMETRIC` | Biometric authentication lock |

**Notification and background permissions:**

| Permission | Purpose |
|---|---|
| `RECEIVE_BOOT_COMPLETED` | Auto-start foreground service after device reboot |
| `FOREGROUND_SERVICE` | Persistent background Socket.IO connection |
| `FOREGROUND_SERVICE_REMOTE_MESSAGING` | Android 14+ typed foreground service |
| `WAKE_LOCK` | Prevent CPU sleep during background tasks |
| `ACCESS_NOTIFICATION_POLICY` | DND policy access |
| `POST_NOTIFICATIONS` | Android 13+ notification posting |
| `SYSTEM_ALERT_WINDOW` | Overlay windows (bubble notifications) |
| `SCHEDULE_EXACT_ALARM` | Precise scheduled notifications |

**Location permissions:**

| Permission | Purpose |
|---|---|
| `ACCESS_COARSE_LOCATION` | Approximate location sharing |
| `ACCESS_FINE_LOCATION` | Precise location sharing |

**Storage permissions:**

| Permission | Max SDK | Purpose |
|---|---|---|
| `READ_MEDIA_IMAGES` | -- | Android 13+ image access |
| `READ_MEDIA_VIDEO` | -- | Android 13+ video access |
| `READ_MEDIA_AUDIO` | -- | Android 13+ audio access |
| `READ_EXTERNAL_STORAGE` | 32 | Legacy storage read (pre-Android 13) |
| `WRITE_EXTERNAL_STORAGE` | 29 | Legacy storage write (pre-Android 10) |

**Removed permissions (explicitly stripped via `tools:node="remove"`):**

| Permission | Reason |
|---|---|
| `REQUEST_INSTALL_PACKAGES` | Added by an external package; not needed |
| `com.google.android.gms.permission.AD_ID` | Ad tracking identifier not used |

### 1.10 Hardware Features

All declared as `android:required="false"` so the app can install on devices without these hardware features:

- `android.hardware.touchscreen`
- `android.hardware.camera`
- `android.hardware.camera.autofocus`
- `android.hardware.telephony`

### 1.11 Application Tag Attributes

| Attribute | Value | Purpose |
|---|---|---|
| `allowBackup` | `false` | Disables Android auto-backup |
| `appCategory` | `social` | Categorizes for OS optimization |
| `requestLegacyExternalStorage` | `true` | Android 10 scoped storage bypass |
| `usesCleartextTraffic` | `true` | Allows HTTP connections to local BlueBubbles servers |
| `extractNativeLibs` | `true` | Extracts native libraries from APK at install time |

### 1.12 Intent Filters (MainActivity)

The main activity declares the following intent filters:

1. **Launcher** -- `android.intent.action.MAIN` + `LAUNCHER` category
2. **Share intents** -- Accepts `SEND` and `SEND_MULTIPLE` actions for these MIME types:
   - `image/*`
   - `video/*`
   - `text/*`
   - `text/vcard`, `text/x-vcard`
   - `application/*`
   - `audio/*`
3. **imessage:// URL scheme** -- `android.intent.action.VIEW` with `android:scheme="imessage"`, `BROWSABLE` category

### 1.13 Broadcast Receivers

| Receiver | Exported | Purpose |
|---|---|---|
| `InternalIntentReceiver` | No | Handles internal notification actions (mark read, reply, delete notification) |
| `ExternalIntentReceiver` | Yes | Exposes `com.bluebubbles.external.GET_SERVER_URL` action for external apps |
| `ForegroundServiceBroadcastReceiver` | Yes | Listens for `restartservice` action to restart the foreground service |
| `AutoStartReceiver` | Yes | Starts the foreground service on `BOOT_COMPLETED` |
| `UnifiedPushReceiver` | Yes | Handles UnifiedPush `MESSAGE`, `UNREGISTERED`, `NEW_ENDPOINT`, `REGISTRATION_FAILED` |
| `ActionBroadcastReceiver` (flutter_local_notifications) | No | Notification action callbacks |
| `ScheduledNotificationReceiver` (flutter_local_notifications) | No | Scheduled notification triggers |
| `ScheduledNotificationBootReceiver` (flutter_local_notifications) | No | Re-schedules notifications after boot/package replace |

### 1.14 Services

| Service | Type | Exported | Purpose |
|---|---|---|---|
| `BlueBubblesFirebaseMessagingService` | -- | No | Receives FCM push notifications; `directBootAware=true`, `stopWithTask=false` |
| `NotificationListener` | -- | No | `NotificationListenerService` for reading notification dismissals; requires `BIND_NOTIFICATION_LISTENER_SERVICE` permission |
| `SocketIOForegroundService` | `remoteMessaging` | Yes | Persistent Socket.IO connection to the BlueBubbles server |

### 1.15 Content Providers

| Provider | Purpose |
|---|---|
| `FileProvider` (androidx.core) | Shares files with external apps via content:// URIs. Authority uses per-flavor `file_provider` string. Paths defined in `res/xml/filepaths.xml` expose root, external, cache, and files directories. |
| `FirebaseInitProvider` | **Removed** via `tools:node="remove"`. Firebase is initialized manually, not through auto-init. |

### 1.16 Notification Channels

Created programmatically by `NotificationChannelHandler`:

| Channel ID | Name | Importance | Special Behavior |
|---|---|---|---|
| `com.bluebubbles.new_messages` | (dynamic) | `IMPORTANCE_HIGH` | Allows bubbles (Android Q+), bypasses DND, shows badge |
| `com.bluebubbles.foreground_service` | (dynamic) | `IMPORTANCE_LOW` | Low importance to avoid heads-up notifications |
| `com.bluebubbles.messaging.foreground_service` | BlueBubbles Foreground Service | `IMPORTANCE_MIN` | Created by SocketIOForegroundService; no sound, no vibration |

### 1.17 Android Auto Support

The manifest includes a `com.google.android.gms.car.application` metadata entry pointing to `res/xml/automotive_app_desc.xml`, which declares:

```xml
<automotiveApp>
    <uses name="notification" />
</automotiveApp>
```

This enables notification display on Android Auto.

### 1.18 Direct Share / Share Targets

Defined in `res/xml/shortcuts.xml`:

```xml
<share-target android:targetClass="com.bluebubbles.messaging.MainActivity">
    <data android:mimeType="*/*" />
    <category android:name="com.bluebubbles.messaging.directshare.category.TEXT_SHARE_TARGET" />
</share-target>
```

Share targets are dynamically pushed via `PushShareTargetsHandler` using `ShortcutManagerCompat`.

### 1.19 Window Manager Preferences

The manifest sets ChromeOS/freeform window preferences:

| Preference | Value | Purpose |
|---|---|---|
| `SuppressWindowControlNavigationButton` | `true` | Hides ChromeOS navigation buttons |
| `FreeformWindowSize` | `tablet` | Default freeform window size on ChromeOS |
| `FreeformWindowOrientation` | `landscape` | Preferred freeform orientation |

### 1.20 Bubble Activity

A separate `BubbleActivity` extends `FlutterFragmentActivity` with:
- Custom Dart entrypoint: `"bubble"` (not the default `main`)
- Used for Android notification bubbles; launched from bubble notification intents
- Shares the same method channel setup as `MainActivity`

### 1.21 ProGuard Rules

No custom ProGuard/R8 rules are defined. `minifyEnabled` is `false` for all build types.

---

## 2. iOS Configuration

Source files: `ios/Runner/Info.plist`, `ios/Runner/AppDelegate.swift`, `ios/Flutter/AppFrameworkInfo.plist`

### 2.1 Deployment Target

The `AppFrameworkInfo.plist` specifies a `MinimumOSVersion` of `8.0`, though the actual effective minimum is determined by Flutter SDK and plugin compatibility (likely iOS 12+ in practice).

### 2.2 Info.plist Keys

| Key | Value | Purpose |
|---|---|---|
| `CFBundleName` | `bluebubbles` | App bundle display name |
| `CFBundlePackageType` | `APPL` | Application bundle |
| `CFBundleShortVersionString` | `$(FLUTTER_BUILD_NAME)` | From pubspec.yaml version |
| `CFBundleVersion` | `$(FLUTTER_BUILD_NUMBER)` | From pubspec.yaml build number |
| `LSRequiresIPhoneOS` | `true` | Requires iOS (not macOS via Catalyst) |
| `UILaunchStoryboardName` | `LaunchScreen` | Launch screen storyboard |
| `UIMainStoryboardFile` | `Main` | Main storyboard |
| `UIViewControllerBasedStatusBarAppearance` | `false` | Global status bar style |
| `UIStatusBarHidden` | `false` | Status bar is visible |

### 2.3 Supported Orientations

**iPhone:**
- Portrait
- Landscape Left
- Landscape Right

**iPad (additional):**
- Portrait Upside Down

### 2.4 Native Code (AppDelegate.swift)

The iOS `AppDelegate` is minimal -- a standard `FlutterAppDelegate` subclass that calls `GeneratedPluginRegistrant.register(with: self)` in `application(_:didFinishLaunchingWithOptions:)`. No custom native iOS code beyond standard Flutter plugin registration.

### 2.5 CocoaPods / Podfile

No `Podfile` is present in the repository. CocoaPods dependencies are managed entirely through Flutter plugins, which generate the Podfile at build time via `flutter pub get`.

### 2.6 Entitlements

No custom `.entitlements` files exist in the iOS runner directory. Capabilities like push notifications or background modes are not configured in the checked-in iOS project files.

---

## 3. Windows Configuration

Source files: `windows/CMakeLists.txt`, `windows/runner/CMakeLists.txt`, `windows/runner/main.cpp`, `windows/runner/flutter_window.cpp`, `windows/bluebubbles_installer_script.iss`

### 3.1 CMake Setup

| Property | Value |
|---|---|
| CMake minimum | 3.14 |
| Project name | `bluebubbles_app` |
| Binary name | `bluebubbles_app` |
| C++ standard | C++17 |
| Unicode | Enabled (`-DUNICODE -D_UNICODE`) |
| DPI awareness | PerMonitorV2 (via manifest) |

The runner links against:
- `flutter` and `flutter_wrapper_app` (Flutter engine)
- `dwmapi.lib` (Desktop Window Manager API, for window effects like acrylic/mica)

Compiler flags: `/W4 /WX /wd"4100" /EHsc` with `_HAS_EXCEPTIONS=0` and `NOMINMAX` defined.

### 3.2 Windows Plugins (generated_plugins.cmake)

The following native plugins are compiled and linked:

`app_links`, `bitsdojo_window_windows`, `connectivity_plus`, `desktop_webview_auth`, `dynamic_color`, `emoji_picker_flutter`, `file_selector_windows`, `flutter_acrylic`, `flutter_timezone`, `geolocator_windows`, `irondash_engine_context`, `local_auth_windows`, `local_notifier`, `maps_launcher`, `media_kit_libs_windows_video`, `media_kit_video`, `objectbox_flutter_libs`, `pasteboard`, `permission_handler_windows`, `printing`, `record_windows`, `screen_brightness_windows`, `screen_retriever`, `secure_application`, `share_plus`, `super_native_extensions`, `system_tray`, `tray_manager`, `url_launcher_windows`, `window_manager`, `windows_taskbar`

FFI plugin: `media_kit_native_event_loop`

### 3.3 Native Code (main.cpp)

The Windows entry point (`wWinMain`):

1. Attaches to the parent console or creates a debug console
2. Initializes COM with `COINIT_APARTMENTTHREADED`
3. Creates a `DartProject` pointing to the `data` directory
4. Passes command-line arguments to the Dart entrypoint
5. Creates the Flutter window titled `"BlueBubbles"` at origin `(10, 10)` with size `1280x720`
6. Integrates `bitsdojo_window` with flags `BDW_CUSTOM_FRAME | BDW_HIDE_ON_STARTUP` for custom window chrome

### 3.4 Native Code (flutter_window.cpp)

The `FlutterWindow` class:
- Creates a `FlutterViewController` matching the client area dimensions
- Registers all plugins with the engine
- Uses `SetNextFrameCallback` to show the window once Flutter renders the first frame
- Handles `WM_FONTCHANGE` messages to trigger `ReloadSystemFonts()`
- Delegates all other messages to `Win32Window::MessageHandler` and `HandleTopLevelWindowProc`

### 3.5 Application Manifest (runner.exe.manifest)

Declares compatibility with:
- Windows 10 and Windows 11
- Windows 8.1
- Windows 8
- Windows 7

DPI awareness: `PerMonitorV2`

### 3.6 Inno Setup Installer

Defined in `windows/bluebubbles_installer_script.iss`:

| Property | Value |
|---|---|
| App name | BlueBubbles |
| App version | 1.15.0.0 |
| Publisher | BlueBubbles |
| URL | https://bluebubbles.app/ |
| Executable | `bluebubbles_app.exe` |
| AppId (GUID) | `{6129D070-FCBC-4167-8C1F-9A4B18263EFF}` |
| Install directory | `{autopf}\BlueBubbles` |
| Compression | LZMA, solid |
| Privileges | Requires admin by default; user can override via dialog |

**Runtime dependency:** The installer checks for and auto-downloads the Visual C++ 2015-2022 Redistributable (x64) if not already installed. The dependency system is powered by `CodeDependencies.iss` (an InnoDependencyInstaller library).

**Files installed:**
- `bluebubbles_app.exe`
- All `.dll`, `.lib`, `.exp` files from the build output
- The `data/` directory (Flutter assets, ICU data, AOT library)

**Registry entries for imessage:// protocol:**

```
HKCU\Software\Classes\imessage     -> "URL:iMessage Protocol"
HKCU\Software\Classes\imessage     -> URL Protocol = ""
HKCU\Software\Classes\imessage\DefaultIcon -> "{app}\bluebubbles_app.exe,0"
HKCU\Software\Classes\imessage\shell\open\command -> ""{app}\bluebubbles_app.exe" "%1""
```

**Post-install:** Optionally launches the application.

**Shortcuts:**
- Start menu program entry
- Optional desktop icon (unchecked by default)

---

## 4. Linux Configuration

Source files: `linux/CMakeLists.txt`, `linux/main.cc`, `linux/my_application.cc`, `linux/build.sh`, `snap/snapcraft.yaml`, `snap/gui/bluebubbles.desktop`

### 4.1 CMake Setup

| Property | Value |
|---|---|
| CMake minimum | 3.10 |
| Binary name | `bluebubbles` |
| Application ID | `app.bluebubbles.BlueBubbles` |
| C++ standard | C++14 |
| System dependency | GTK+ 3.0 (via pkg-config) |
| Linker | Also links `mimalloc` library |

Compiler flags: `-Wall -Werror`, `-O3` for non-debug builds.

### 4.2 Linux Plugins (generated_plugins.cmake)

`bitsdojo_window_linux`, `desktop_webview_auth`, `dynamic_color`, `emoji_picker_flutter`, `file_selector_linux`, `flutter_acrylic`, `gtk`, `irondash_engine_context`, `local_notifier`, `maps_launcher`, `media_kit_libs_linux`, `media_kit_video`, `objectbox_flutter_libs`, `pasteboard`, `printing`, `record_linux`, `screen_retriever`, `super_native_extensions`, `system_tray`, `tray_manager`, `url_launcher_linux`, `window_manager`

FFI plugin: `media_kit_native_event_loop`

### 4.3 Native Code (my_application.cc)

The GTK application:
- Creates a `GtkApplicationWindow`
- On GNOME (X11), uses a `GtkHeaderBar` with close button
- On non-GNOME X11 window managers, uses `bitsdojo_window` custom frame with `setCustomFrame(true)`
- Default window size: `1280x720`
- Title: `"bluebubbles_app"`
- Application flags: `G_APPLICATION_NON_UNIQUE` (allows multiple instances)

### 4.4 Build Script (linux/build.sh)

The `build.sh` script:

1. Runs `flutter pub get`
2. Runs `flutter build linux --release -v`
3. Detects architecture (`x86_64` or `aarch64`) to determine output folder
4. Injects version `"1.15.0.0"` into `version.json` in the flutter assets using `jq`
5. Sets the binary as executable
6. Creates a `.tar` archive of the release bundle
7. Outputs the SHA-256 checksum

### 4.5 Snap Package (snapcraft.yaml)

| Property | Value |
|---|---|
| Name | `bluebubbles` |
| Title | BlueBubbles |
| Version | 1.14.0.0 |
| Confinement | `strict` |
| Base | `core24` |
| Grade | `stable` |
| Platforms | `amd64`, `arm64` |

**Snap plugs (permissions):**

| Plug | Purpose |
|---|---|
| `network` | Internet access |
| `network-manager` | Network configuration |
| `camera` | Camera access |
| `desktop` | Desktop integration |
| `desktop-legacy` | Legacy desktop APIs |
| `wayland` | Wayland display server |
| `x11` | X11 display server |
| `home` | Home directory access |
| `opengl` | GPU rendering |
| `alsa` | ALSA audio |
| `audio-playback` | Audio output |
| `audio-record` | Microphone input |

**Snap slots:**

- `dbus-bluebubbles`: D-Bus session bus with name `app.bluebubbles.BlueBubbles`

**Build parts:**

1. **desktop-glib-only** -- GLib desktop helpers from snapcraft-desktop-helpers
2. **alsa-mixin** -- ALSA audio support from snapcraft-alsa, with PulseAudio plugins
3. **fmedia** -- `fmedia` v1.31 audio tool (amd64 only; skipped on arm64)
4. **bluebubbles** -- Pre-built release tarball from GitHub releases; stage packages include `libglu1-mesa`, `libmpv2`, `libnotify4`, `net-tools`, `wmctrl`, `zenity`, `gir1.2-appindicator3-0.1`
5. **gpu-2404** -- GPU driver wrapper from canonical/gpu-snap
6. **cleanup** -- Removes duplicate files from core22 and gtk-common-themes

**Environment variables:**
- `ALWAYS_USE_PULSEAUDIO=1`
- Extended `LD_LIBRARY_PATH` for blas, lapack, samba, vdpau, dri libraries

**Layouts** -- Bind mounts for:
- `/usr/share/libdrm`
- `/usr/share/drirc.d`
- `/usr/share/X11/XErrorDB` and `/usr/share/X11/locale`
- `webkit2gtk-4.1` library path
- ALSA library and data paths

### 4.6 Desktop Entry

File: `snap/gui/bluebubbles.desktop`

```ini
[Desktop Entry]
Version=1.0
Name=BlueBubbles
Comment=BlueBubbles client for Linux
Exec=bluebubbles
Icon=${SNAP}/meta/gui/bluebubbles.png
Terminal=false
Type=Application
Categories=Network;InstantMessaging;Chat;
StartupWMClass=bluebubbles
```

---

## 5. macOS Configuration

Source files: `macos/Runner/AppDelegate.swift`, `macos/Runner/MainFlutterWindow.swift`, `macos/Runner/Info.plist`, `macos/Runner/Configs/AppInfo.xcconfig`, `macos/Runner/DebugProfile.entitlements`, `macos/Runner/Release.entitlements`

### 5.1 App Identity

Defined in `macos/Runner/Configs/AppInfo.xcconfig`:

| Property | Value |
|---|---|
| `PRODUCT_NAME` | `bluebubbles` |
| `PRODUCT_BUNDLE_IDENTIFIER` | `app.bluebubbles.BlueBubbles` |
| `PRODUCT_COPYRIGHT` | Copyright 2021 BlueBubbles. All rights reserved. |

### 5.2 Info.plist

| Key | Value |
|---|---|
| `CFBundleName` | `$(PRODUCT_NAME)` (resolves to `bluebubbles`) |
| `CFBundlePackageType` | `APPL` |
| `LSMinimumSystemVersion` | `$(MACOSX_DEPLOYMENT_TARGET)` |
| `NSHumanReadableCopyright` | `$(PRODUCT_COPYRIGHT)` |
| `NSMainNibFile` | `MainMenu` |
| `NSPrincipalClass` | `NSApplication` |

### 5.3 Entitlements

**Debug/Profile entitlements** (`DebugProfile.entitlements`):

| Entitlement | Value | Purpose |
|---|---|---|
| `com.apple.security.app-sandbox` | `true` | App Sandbox enabled |
| `com.apple.security.cs.allow-jit` | `true` | JIT compilation (for Dart VM in debug) |
| `com.apple.security.network.server` | `true` | Incoming network connections (for Flutter DevTools) |

**Release entitlements** (`Release.entitlements`):

| Entitlement | Value | Purpose |
|---|---|---|
| `com.apple.security.app-sandbox` | `true` | App Sandbox enabled |

The release build has more restrictive entitlements: no JIT, no network server capability.

### 5.4 Native Code

**AppDelegate.swift:**
- Subclasses `FlutterAppDelegate`
- Sets `applicationShouldTerminateAfterLastWindowClosed` to `true` (app quits when last window closes)

**MainFlutterWindow.swift:**
- Subclasses `BitsdojoWindow` (from `bitsdojo_window_macos`)
- Configures `BDW_CUSTOM_FRAME | BDW_HIDE_ON_STARTUP` for custom window chrome
- Creates `FlutterViewController`, registers generated plugins, and sets it as the content view controller

---

## 6. Web Configuration

Source files: `web/index.html`, `web/manifest.json`, `web/splash/`

### 6.1 index.html

- **Base href:** `/web/` (configurable via `--base-href` during `flutter build web`)
- **Viewport:** `width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no`
- **Description:** "Send iMessages on Web using BlueBubbles!"
- **Google Sign-In client ID:** `500464701389-8trcdkcj7ni5l4dn6n7l670ab5c060592935203.apps.googleusercontent.com` (for web-based Firebase Auth)
- **Service worker:** Enabled via Flutter's `serviceWorkerVersion` injection
- **Color emoji:** Enabled via `useColorEmoji: true` in `engineInitializer.initializeEngine()`
- **iOS web app meta tags:**
  - `apple-mobile-web-app-capable`: yes
  - `apple-mobile-web-app-status-bar-style`: black
  - `apple-mobile-web-app-title`: BlueBubbles

### 6.2 Splash Screen

The web build includes a splash screen with:
- Light/dark theme variants (`splash/img/light-{1-4}x.png`, `splash/img/dark-{1-4}x.png`)
- Responsive image sizing via `srcset` with 1x, 2x, 3x, and 4x densities
- `<picture>` element with `(prefers-color-scheme: light)` and `(prefers-color-scheme: dark)` media queries
- `removeSplashFromWeb()` function removes the splash elements and sets transparent background once Flutter loads

### 6.3 Web App Manifest (manifest.json)

```json
{
  "name": "BlueBubbles",
  "short_name": "BlueBubbles",
  "start_url": ".",
  "display": "standalone",
  "background_color": "#4990de",
  "theme_color": "#4990de",
  "description": "Send iMessages on Web using BlueBubbles!",
  "orientation": "portrait-primary",
  "prefer_related_applications": false,
  "icons": [
    { "src": "icons/Icon-128.png", "sizes": "128x128", "type": "image/png" },
    { "src": "icons/Icon-256.png", "sizes": "256x256", "type": "image/png" }
  ]
}
```

### 6.4 Icons

- `icons/Icon-128.png` (128x128)
- `icons/Icon-256.png` (256x256)
- `icons/Icon-maskable-128.png` (128x128, maskable)
- `icons/Icon-maskable-256.png` (256x256, maskable)
- `icons/favicon.ico`

---

## 7. Native Code (Kotlin)

All Android native code is written in Kotlin under `android/app/src/main/kotlin/com/bluebubbles/messaging/`. This section documents each native class and its purpose.

### 7.1 Activity Layer

**`MainActivity.kt`** -- Primary Flutter activity.
- Extends `FlutterFragmentActivity`
- Sets up a `MethodChannel` on `"com.bluebubbles.messaging"` to receive calls from Dart
- Stores a static reference to the `FlutterEngine` for background workers to use
- On destroy: if the system is killing the activity (not user-initiated) and `keepAppAlive` preference is enabled, sends a broadcast to `ForegroundServiceBroadcastReceiver` to restart the foreground service
- Handles `onActivityResult` for the notification listener permission request

**`BubbleActivity.kt`** -- Notification bubble activity.
- Extends `FlutterFragmentActivity`
- Overrides `getDartEntrypointFunctionName()` to return `"bubble"`, launching a separate Dart entrypoint for the bubble UI
- Same method channel setup as `MainActivity`

### 7.2 Backend/UI Interop Layer

**`MethodCallHandler.kt`** -- Central routing class for all Dart-to-native method calls.
- Dispatches `MethodCall` objects by `call.method` string to the appropriate handler class
- Also provides a static `invokeMethod()` function to call Dart from native code via the engine's method channel
- Registered handlers (22 total):

| Method Tag | Handler Class | Purpose |
|---|---|---|
| `unified-push-register` | `UnifiedPushHandler` | Register/unregister UnifiedPush |
| `firebase-auth` | `FirebaseAuthHandler` | Firebase authentication |
| `firebase-delete-token` | `FirebaseDeleteTokenHandler` | Delete FCM token |
| `create-notification-channel` | `NotificationChannelHandler` | Create Android notification channels |
| `get-server-url` | `ServerUrlRequestHandler` | Read server URL from preferences |
| `update-next-restart` | `UpdateNextRestartHandler` | Update next restart timestamp |
| `open-browser` | `BrowserLaunchRequestHandler` | Launch URL in Chrome Custom Tab |
| `push-share-targets` | `PushShareTargetsHandler` | Push dynamic share targets |
| `open-contact-form` | `NewContactFormRequestHandler` | Open system new contact form |
| `open-contact` | `OpenExistingContactRequestHandler` | Open existing contact in system contacts |
| `open-calendar` | `OpenCalendarRequestHandler` | Open system calendar |
| `start-google-duo` | `StartGoogleDuoRequestHandler` | Launch Google Duo call |
| `check-chromeos` | `CheckChromeOsHandler` | Detect if running on ChromeOS |
| `get-notification-listener-permission` | `NotificationListenerPermissionRequestHandler` | Request notification listener access |
| `start-notification-listener` | `StartNotificationListenerHandler` | Start the notification listener service |
| `open-conversation-notification-settings` | `OpenConversationNotificationSettingsHandler` | Open per-conversation notification settings |
| `get-content-uri-path` | `GetContentUriPathHandler` | Resolve content:// URI to a file path |
| `create-incoming-message-notification` | `CreateIncomingMessageNotification` | Build and post message notifications |
| `create-incoming-facetime-notification` | `CreateIncomingFaceTimeNotification` | Build and post FaceTime call notifications |
| `delete-notification` | `DeleteNotificationHandler` | Cancel a specific notification |
| `start-foreground-service` | `StartForegroundServiceHandler` | Start the Socket.IO foreground service |
| `stop-foreground-service` | `StopForegroundServiceHandler` | Stop the Socket.IO foreground service |

**`DartWorkManager.kt`** -- Background work orchestration.
- Uses AndroidX `WorkManager` to schedule `OneTimeWorkRequest` items
- Each work request is tagged with `"DartWorker"`
- Uses `setExpedited(OutOfQuotaPolicy.RUN_AS_NON_EXPEDITED_WORK_REQUEST)` for time-sensitive tasks
- Serializes method name and arguments to JSON via Gson
- Observes work completion using `WorkInfo` LiveData and runs a callback on the main thread

**`DartWorker.kt`** -- Headless Flutter engine worker.
- Extends `ListenableWorker` (not `Worker`) for asynchronous work
- If `MainActivity.engine` is available, uses it directly to send events to Dart
- If no activity engine is available, initializes a new headless `FlutterEngine`:
  1. Calls `FlutterMain.startInitialization()` and `ensureInitializationComplete()`
  2. Looks up the background callback handle stored in `FlutterSharedPreferences` under `flutter.backgroundCallbackHandle`
  3. Executes the Dart callback via `dartExecutor.executeDartCallback()`
  4. Waits for Dart to signal readiness via a `"ready"` method call
- After work completes, waits 5 seconds then checks if more work is queued; if not, destroys the headless engine
- Provides a `ForegroundInfo` notification for Android 11 and below (required for expedited work)

### 7.3 Firebase Services

**`BlueBubblesFirebaseMessagingService.kt`** -- FCM message handler.
- Extends `FirebaseMessagingService`
- On message received: extracts the `type` field from the data payload, then creates a `DartWorker` via `DartWorkManager`
- If the `sendEventsToTasker` preference is enabled, broadcasts an intent with action `net.dinglisch.android.taskerm.BB_EVENT` containing the server URL, event type, and all message data fields

**Other Firebase handlers** (invoked via method channel from Dart):
- `FirebaseAuthHandler` -- Handles Firebase authentication flows
- `FirebaseCloudMessagingTokenHandler` -- Retrieves the FCM registration token
- `FirebaseDeleteTokenHandler` -- Deletes the FCM token
- `FirebaseDatabaseListener` -- Listens to Firebase Realtime Database for server URL changes
- `ServerUrlRequestHandler` -- Reads the server URL from shared preferences or Firebase
- `UpdateNextRestartHandler` -- Updates the next restart timestamp in preferences

### 7.4 Foreground Service

**`SocketIOForegroundService.kt`** -- Persistent background connection service.
- Extends Android `Service`
- Reads configuration from `FlutterSharedPreferences`:
  - `serverAddress` -- BlueBubbles server URL
  - `guidAuthKey` -- Server password
  - `keepAppAlive` -- Whether to keep the service running
  - `customHeaders` -- JSON string of custom HTTP headers
- Creates a `IMPORTANCE_MIN` notification channel with no sound or vibration
- Starts as a foreground service with type `FOREGROUND_SERVICE_TYPE_REMOTE_MESSAGING`
- Establishes a Socket.IO connection to the server with:
  - Password sent as URL query parameter
  - Custom headers parsed from JSON and added to the Socket.IO options
- Listens for Socket.IO events and creates `DartWorker` tasks for each incoming event
- Blacklisted events that do not trigger workers: `typing-indicator`, `new-findmy-location`, `connect`, `disconnect`
- Updates the foreground notification with connection status messages
- On disconnect, attempts reconnection after a 30-second delay

**`ForegroundServiceBroadcastReceiver.kt`** -- Restarts the foreground service when receiving the `restartservice` broadcast.

**`StartForegroundServiceHandler.kt` / `StopForegroundServiceHandler.kt`** -- Method channel handlers to start/stop the foreground service from Dart.

### 7.5 Notification System

**`CreateIncomingMessageNotification.kt`** -- Builds rich message notifications.
- Uses `NotificationCompat.MessagingStyle` for conversation-style notifications
- Features:
  - Per-contact avatar icons via `Person.Builder`
  - Group conversation titles
  - Stacking messages in existing notifications using `extractMessagingStyleFromNotification()`
  - "Mark As Read" action with semantic action `SEMANTIC_ACTION_MARK_AS_READ`
  - Inline "Reply" action with `RemoteInput` for quick reply
  - Bubble metadata with `BubbleMetadata.Builder` pointing to `BubbleActivity`
  - Notification grouping via `NOTIFICATION_GROUP_NEW_MESSAGES` key
  - Wearable extensions for smartwatch actions
  - `ShortcutId` set to `chatGuid` for linking to direct share targets
  - Summary notification for the group
  - Duplicate notification detection by `chatGuid` and `messageGuid`

**`CreateIncomingFaceTimeNotification.kt`** -- Builds FaceTime call notifications.
- Uses `CATEGORY_CALL` with `PRIORITY_MAX`
- "Answer" action opens `MainActivity` with the FaceTime link
- "Ignore" action dismisses the notification
- `setOngoing(true)` makes it persistent
- 30-second auto-timeout via `setTimeoutAfter(30000)` when a call UUID is present
- Wearable extensions for smartwatch

**`NotificationChannelHandler.kt`** -- Creates notification channels from Dart-side requests.

**`DeleteNotificationHandler.kt`** -- Cancels notifications by ID.

**`NotificationListener.kt`** -- `NotificationListenerService` implementation for detecting when the user dismisses notifications.

### 7.6 Intent Receivers

**`AutoStartReceiver.kt`** -- Boot-completed receiver.
- Checks the `keepAppAlive` preference
- Starts `SocketIOForegroundService` via `startForegroundService()` if enabled

**`InternalIntentReceiver.kt`** -- Handles intents from notification actions (mark as read, reply, delete notification).

**`ExternalIntentReceiver.kt`** -- Handles the `com.bluebubbles.external.GET_SERVER_URL` intent for external app integrations (e.g., Tasker).

### 7.7 UnifiedPush

**`UnifiedPushReceiver.kt`** -- Handles UnifiedPush protocol messages.
- Extends `MessagingReceiver` from the UnifiedPush Android connector library
- `onNewEndpoint`: Sends the new endpoint URL to Dart via a `DartWorker`
- `onRegistrationFailed` / `onUnregistered`: Clears the endpoint
- `onMessage`: Parses the JSON payload, extracts the event `type`, and creates a `DartWorker`; also broadcasts to Tasker if configured

### 7.8 Utility Classes

**`Constants.kt`** -- Central constants including:
- Method channel name: `"com.bluebubbles.messaging"`
- Notification tags and channel IDs
- Pending intent offsets for different notification actions (ensuring unique request codes)
- Log tag: `"BlueBubblesApp"`

**`Utils.kt`** -- Utility functions for bitmap processing and server URL retrieval.

**`FilesystemUtils.kt`** -- Filesystem utility functions.

**`GetContentUriPathHandler.kt`** -- Resolves `content://` URIs to filesystem paths.

---

## 8. Protocol Handling

The `imessage://` URL scheme is registered on every platform that supports it.

### 8.1 Android

Declared in `AndroidManifest.xml` on `MainActivity`:

```xml
<intent-filter>
    <action android:name="android.intent.action.VIEW" />
    <category android:name="android.intent.category.DEFAULT" />
    <category android:name="android.intent.category.BROWSABLE" />
    <data android:scheme="imessage" />
</intent-filter>
```

The `app_links` Flutter plugin handles the incoming URL in Dart.

### 8.2 Windows

Registered via the Inno Setup installer in the Windows registry:

```
HKCU\Software\Classes\imessage                        = "URL:iMessage Protocol"
HKCU\Software\Classes\imessage\URL Protocol            = ""
HKCU\Software\Classes\imessage\DefaultIcon             = "<install_path>\bluebubbles_app.exe,0"
HKCU\Software\Classes\imessage\shell\open\command      = ""<install_path>\bluebubbles_app.exe" "%1""
```

Registry entries are cleaned up on uninstall (`Flags: uninsdeletekey`).

### 8.3 Linux

Not registered at the OS level in the checked-in code. The `app_links` plugin may handle it via D-Bus or the desktop file, but no explicit `MimeType` or `x-scheme-handler/imessage` entry is present in the `.desktop` file.

### 8.4 macOS

No explicit `CFBundleURLTypes` entry for `imessage://` is present in the macOS `Info.plist`. The `app_links` plugin may register it at the Flutter layer.

### 8.5 iOS

No `CFBundleURLTypes` for `imessage://` is declared in the iOS `Info.plist`.

### 8.6 Web

Not applicable. Web browsers handle `imessage://` links natively.

---

## 9. Permissions Matrix

A comprehensive table of all permissions requested per platform.

### 9.1 Full Matrix

| Permission Category | Android | iOS | Windows | macOS | Linux (Snap) | Web |
|---|---|---|---|---|---|---|
| **Internet / Network** | `INTERNET`, `ACCESS_NETWORK_STATE` | Implicit | Implicit | Implicit | `network`, `network-manager` | Implicit |
| **Camera** | `CAMERA` | Via plugin | Via plugin | -- | `camera` | Via getUserMedia |
| **Microphone** | `RECORD_AUDIO` | Via plugin | Via plugin | -- | `audio-record` | Via getUserMedia |
| **Contacts (Read)** | `READ_CONTACTS` | Via plugin | -- | -- | -- | -- |
| **Contacts (Write)** | `WRITE_CONTACTS` | Via plugin | -- | -- | -- | -- |
| **Phone Call** | `CALL_PHONE` | -- | -- | -- | -- | -- |
| **Biometrics** | `USE_BIOMETRIC` | Via plugin | `local_auth_windows` | -- | -- | -- |
| **Location (Coarse)** | `ACCESS_COARSE_LOCATION` | Via plugin | `geolocator_windows` | -- | -- | Via Geolocation API |
| **Location (Fine)** | `ACCESS_FINE_LOCATION` | Via plugin | `geolocator_windows` | -- | -- | Via Geolocation API |
| **Storage (Read)** | `READ_MEDIA_*` / `READ_EXTERNAL_STORAGE` | Via plugin | Implicit | -- | `home` | -- |
| **Storage (Write)** | `WRITE_EXTERNAL_STORAGE` (max 29) | Via plugin | Implicit | -- | `home` | -- |
| **Notifications** | `POST_NOTIFICATIONS`, `ACCESS_NOTIFICATION_POLICY` | Via plugin | `local_notifier` | -- | -- | Via Notification API |
| **Background Execution** | `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_REMOTE_MESSAGING`, `WAKE_LOCK` | -- | -- | -- | -- | Service Worker |
| **Boot Start** | `RECEIVE_BOOT_COMPLETED` | -- | -- | -- | -- | -- |
| **Vibration** | `VIBRATE` | -- | -- | -- | -- | Via Vibration API |
| **Overlay** | `SYSTEM_ALERT_WINDOW` | -- | -- | -- | -- | -- |
| **Exact Alarm** | `SCHEDULE_EXACT_ALARM` | -- | -- | -- | -- | -- |
| **Display** | -- | -- | `dwmapi.lib` | -- | `opengl`, `desktop`, `x11`, `wayland` | -- |
| **Audio Playback** | -- | -- | -- | -- | `audio-playback`, `alsa` | -- |
| **App Sandbox** | -- | -- | -- | Enabled | `strict` confinement | -- |
| **JIT (Debug)** | -- | -- | -- | `allow-jit` (debug only) | -- | -- |
| **Network Server (Debug)** | -- | -- | -- | `network.server` (debug only) | -- | -- |

### 9.2 Android Permissions Removed

| Permission | Source | Reason |
|---|---|---|
| `REQUEST_INSTALL_PACKAGES` | External plugin | Not needed; explicitly removed |
| `com.google.android.gms.permission.AD_ID` | Google Play Services | Ad tracking not used; explicitly removed |

---

## 10. Build Instructions

### 10.1 Prerequisites (All Platforms)

- **Flutter SDK** >= 3.1.3, < 4.0.0 (per `pubspec.yaml` constraint)
- **Dart SDK** (bundled with Flutter)

### 10.2 Android Build

**Additional tools:**
- Android SDK with API level 34
- Android NDK (required by some plugins)
- Java 17 (JDK)
- Kotlin 1.9.23

**Steps:**

```bash
cd bluebubbles-app-ELECTRON

# Get dependencies
flutter pub get

# Build APK (production flavor, release mode)
flutter build apk --flavor prod --release

# Build App Bundle (for Play Store)
flutter build appbundle --flavor prod --release
```

**Signing:** Place a `key.properties` file in `android/` with:

```properties
storeFile=<path-to-keystore>
storePassword=<keystore-password>
keyAlias=<key-alias>
keyPassword=<key-password>
```

**Firebase:** Place `google-services.json` in `android/app/src/<flavor>/` for each flavor that needs Firebase.

### 10.3 iOS Build

**Additional tools:**
- Xcode (latest stable recommended)
- CocoaPods

**Steps:**

```bash
cd bluebubbles-app-ELECTRON

flutter pub get

# Generate CocoaPods dependencies
cd ios && pod install && cd ..

# Build for device
flutter build ios --release

# Or build IPA for distribution
flutter build ipa --release
```

### 10.4 Windows Build

**Additional tools:**
- Visual Studio 2022 with "Desktop development with C++" workload
- CMake 3.14+ (included with Visual Studio)

**Steps:**

```bash
cd bluebubbles-app-ELECTRON

flutter pub get

# Build release
flutter build windows --release
```

The output is at `build/windows/x64/runner/Release/`.

**Creating the installer:**

1. Install [Inno Setup](https://jrsoftware.org/isinfo.php)
2. Open `windows/bluebubbles_installer_script.iss` in Inno Setup
3. Compile to produce `bluebubbles-windows.exe`

The installer auto-downloads the VC++ 2015-2022 Redistributable if needed.

### 10.5 Linux Build

**Additional tools:**
- GCC/G++ toolchain
- GTK 3 development headers (`libgtk-3-dev`)
- Additional dependencies for plugins: `libmpv-dev`, `libasound2-dev`, etc.
- `jq` (for the build script's version injection)

**Steps:**

```bash
cd bluebubbles-app-ELECTRON

flutter pub get

# Build release
flutter build linux --release

# Or use the provided build script
chmod +x linux/build.sh
./linux/build.sh
```

The build script produces a `.tar` archive named `bluebubbles-linux-<arch>.tar`.

**Building the Snap:**

```bash
cd bluebubbles-app-ELECTRON
snapcraft
```

This builds from pre-compiled release tarballs hosted on GitHub.

### 10.6 macOS Build

**Additional tools:**
- Xcode (latest stable)
- CocoaPods (if plugins require it)

**Steps:**

```bash
cd bluebubbles-app-ELECTRON

flutter pub get

# Build release
flutter build macos --release
```

### 10.7 Web Build

**Steps:**

```bash
cd bluebubbles-app-ELECTRON

flutter pub get

# Build for web
flutter build web --release --base-href /web/
```

The output is at `build/web/`. The `--base-href` flag should match the deployment path (defaults to `/web/` per the checked-in `index.html`).

**Deployment:** The web build can be served from any static file host. The `manifest.json` and service worker enable PWA installation. The Google Sign-In client ID in `index.html` must match the Firebase project configuration for authentication to work.

---

## Appendix A: File Provider Paths

Defined in `android/app/src/main/res/xml/filepaths.xml`:

```xml
<paths>
    <root-path name="root" path="." />
    <external-path name="external" path="." />
    <external-files-path name="external_files" path="." />
    <cache-path name="cache" path="." />
    <external-cache-path name="external_cache" path="." />
    <files-path name="files" path="." />
</paths>
```

This grants the `FileProvider` access to all standard Android storage locations, enabling file sharing with external apps via `content://` URIs.

## Appendix B: Constants Reference

From `Constants.kt`:

| Constant | Value | Usage |
|---|---|---|
| `logTag` | `"BlueBubblesApp"` | Android log tag |
| `methodChannel` | `"com.bluebubbles.messaging"` | Flutter MethodChannel name |
| `categoryTextShareTarget` | `"com.bluebubbles.messaging.directshare.category.TEXT_SHARE_TARGET"` | Direct share category |
| `googleDuoPackageName` | `"com.google.android.apps.tachyon"` | Google Duo package for call intents |
| `newMessageNotificationTag` | `"com.bluebubbles.messaging.NEW_MESSAGE_NOTIFICATION"` | Notification tag for messages |
| `newFaceTimeNotificationTag` | `"com.bluebubbles.messaging.NEW_FACETIME_NOTIFICATION"` | Notification tag for FaceTime |
| `notificationGroupKey` | `"com.bluebubbles.messaging.NOTIFICATION_GROUP_NEW_MESSAGES"` | Notification group key |
| `foregroundServiceNotificationChannel` | `"com.bluebubbles.messaging.foreground_service"` | Foreground service channel ID |
| `foregroundServiceNotificationId` | `1` | Foreground service notification ID |
| `dartWorkerTag` | `"DartWorker"` | WorkManager tag for background workers |
| `pendingIntentOpenChatOffset` | `0` | Base offset for open-chat pending intents |
| `pendingIntentMarkReadOffset` | `100000` | Offset for mark-read pending intents |
| `pendingIntentOpenBubbleOffset` | `200000` | Offset for bubble pending intents |
| `pendingIntentDeleteNotificationOffset` | `300000` | Offset for delete-notification intents |
| `pendingIntentAnswerFaceTimeOffset` | `-100000` | Offset for FaceTime answer intents |
| `pendingIntentDeclineFaceTimeOffset` | `-200000` | Offset for FaceTime decline intents |
| `notificationListenerRequestCode` | `1000` | Request code for notification listener |
| `dartWorkerNotificationId` | `1000000` | Notification ID for DartWorker foreground info |

## Appendix C: Platform Plugin Summary

Plugins that are platform-specific native code (compiled per platform):

| Plugin | Windows | Linux | Purpose |
|---|---|---|---|
| `bitsdojo_window` | Yes | Yes | Custom window frame and title bar |
| `app_links` | Yes | -- | Deep link / URL scheme handling |
| `connectivity_plus` | Yes | -- | Network connectivity monitoring |
| `desktop_webview_auth` | Yes | Yes | Web-based OAuth authentication |
| `dynamic_color` | Yes | Yes | Material You dynamic theming |
| `emoji_picker_flutter` | Yes | Yes | Emoji keyboard |
| `file_selector` | Yes (Windows) | Yes (Linux) | Native file picker |
| `flutter_acrylic` | Yes | Yes | Window transparency/acrylic effects |
| `geolocator_windows` | Yes | -- | Location services |
| `local_auth_windows` | Yes | -- | Biometric/PIN authentication |
| `local_notifier` | Yes | Yes | Desktop notifications |
| `media_kit` | Yes | Yes | Video/audio playback |
| `objectbox_flutter_libs` | Yes | Yes | ObjectBox native database |
| `pasteboard` | Yes | Yes | Clipboard access |
| `screen_retriever` | Yes | Yes | Multi-monitor detection |
| `system_tray` / `tray_manager` | Yes | Yes | System tray icon |
| `window_manager` | Yes | Yes | Window size/position control |
| `windows_taskbar` | Yes | -- | Taskbar progress/overlay |
| `super_native_extensions` | Yes | Yes | Native drag and drop |
