# 06 - Theming and Design Language

## Table of Contents

- [Design System Overview](#design-system-overview)
- [Theme Architecture](#theme-architecture)
- [Color System](#color-system)
- [Color Engine](#color-engine)
- [Built-in Themes](#built-in-themes)
- [Custom Theme Creation](#custom-theme-creation)
- [Typography](#typography)
- [Spacing and Layout](#spacing-and-layout)
- [Component Styling](#component-styling)
- [Dark and Light Mode](#dark-and-light-mode)
- [Platform-Specific Styling](#platform-specific-styling)
- [Window Effects](#window-effects)
- [Animation Patterns](#animation-patterns)

---

## Design System Overview

BlueBubbles uses Material Design 3 (Material You) as its foundation, built on top of the Flutter framework with `FlexColorScheme` for advanced color scheme generation and `adaptive_theme` for runtime theme switching. The application offers three distinct UI skins that dramatically alter the visual presentation:

- **iOS skin** -- Mimics native Apple iMessage aesthetics with Cupertino-style widgets, bouncing scroll physics, and bubble tails.
- **Material skin** -- Standard Material Design 3 with ink sparkle effects, Material-style headers, and clamping scroll physics.
- **Samsung skin** -- Samsung One UI-inspired layout with a large collapsible header area, bottom navigation footer, and Samsung-style conversation tiles.

### Design philosophy

The theming system prioritizes deep user customization. Users can select from 50+ built-in themes, create fully custom themes with per-token color editing, swap Google Fonts at runtime, enable Material You (Monet) dynamic color, and apply desktop window transparency effects. Every color role in the Material 3 `ColorScheme` is individually editable through the Advanced Theming panel.

### Key dependencies

| Package | Version | Purpose |
|---|---|---|
| `adaptive_theme` | ^3.6.0 | Runtime light/dark/system theme switching |
| `flex_color_scheme` | ^7.3.1 | Color scheme generation, FlexScheme presets, Material 3 theming |
| `dynamic_color` | ^1.7.0 | Android Monet palette extraction, Windows accent color |
| `material_color_utilities` | ^0.11.1 | `CorePalette` tonal palette generation for Monet |
| `google_fonts` | ^6.2.1 | Runtime Google Font loading and application |
| `flutter_acrylic` | ^1.1.3 | Desktop window transparency effects (Mica, Acrylic, Aero) |
| `simple_animations` | ^5.0.2 | Gradient background animation tweens |
| `flex_color_picker` | ^3.4.1 | Color picker in avatar and theme customization dialogs |

---

## Theme Architecture

### ThemeStruct

`ThemeStruct` is the central data model for a complete theme. It wraps a Flutter `ThemeData` object with additional metadata and is persisted to the ObjectBox database.

**Source:** `lib/database/io/theme.dart` (native platforms), `lib/database/html/theme.dart` (web)

```
ThemeStruct
  +-- id: int?                    // ObjectBox primary key
  +-- name: String                // Unique theme name (e.g. "OLED Dark", "Nord Theme")
  +-- gradientBg: bool            // Whether to show animated gradient conversation background
  +-- googleFont: String          // Google Font family name, or "Default"
  +-- data: ThemeData             // The full Flutter ThemeData object
```

Key behaviors:

- `name` has a `@Unique()` constraint in ObjectBox, preventing duplicate theme names.
- `isPreset` returns `true` if the theme name matches any of the default themes in `ThemesService.defaultThemes`.
- Preset themes cannot be deleted. Custom themes can be deleted via `delete()`.
- The `data` field is serialized to JSON through `dbThemeData` getter/setter, encoding the `textTheme` (six text styles plus bubble text) and the full `colorScheme` (27 color tokens plus brightness).

### ThemeStruct serialization format

When serialized to JSON (for ObjectBox storage), a ThemeStruct produces a map with this shape:

```json
{
  "ROWID": 1,
  "name": "OLED Dark",
  "gradientBg": 0,
  "data": {
    "textTheme": {
      "font": "Default",
      "titleLarge": { "color": 4294967295, "fontWeight": 3, "fontSize": 22.0 },
      "bodyLarge": { "color": ..., "fontWeight": ..., "fontSize": 16.0 },
      "bodyMedium": { "color": ..., "fontWeight": ..., "fontSize": 14.0 },
      "bodySmall": { "color": ..., "fontWeight": ..., "fontSize": 12.0 },
      "labelLarge": { "color": ..., "fontWeight": ..., "fontSize": 14.0 },
      "labelSmall": { "color": ..., "fontWeight": ..., "fontSize": 11.0 },
      "bubbleText": { "fontSize": 15.0 }
    },
    "colorScheme": {
      "primary": 4278221567,
      "onPrimary": 4294967295,
      "primaryContainer": ...,
      "onPrimaryContainer": ...,
      "secondary": ...,
      "...": "... (27 total color tokens)",
      "brightness": 1
    }
  }
}
```

Color values are stored as integer ARGB values (e.g., `4294967295` = `0xFFFFFFFF` = white). Brightness is an index into `Brightness.values` (0 = light, 1 = dark).

### ThemeColors

`ThemeColors` in `lib/database/global/theme_colors.dart` defines string constants for legacy theme color token names:

| Constant | Value |
|---|---|
| `Headline1` | `"Headline1"` |
| `Headline2` | `"Headline2"` |
| `Bodytext1` | `"Bodytext1"` |
| `Bodytext2` | `"BodyText2"` |
| `Subtitle1` | `"Subtitle1"` |
| `Subtitle2` | `"Subtitle2"` |
| `AccentColor` | `"AccentColor"` |
| `DividerColor` | `"DividerColor"` |
| `BackgroundColor` | `"BackgroundColor"` |
| `PrimaryColor` | `"PrimaryColor"` |

These constants are used in the legacy `ThemeObject`/`ThemeEntry` system. The current system uses the Material 3 `ColorScheme` tokens instead.

### ThemesService

`ThemesService` (`lib/services/ui/theme/themes_service.dart`) is a `GetxService` singleton accessed globally via the `ts` shorthand. It manages:

- **Default theme construction** -- Builds three base themes (`oledDarkTheme`, `whiteLightTheme`, `nordDarkTheme`) plus all `FlexScheme` light/dark variants.
- **Material You (Monet) integration** -- Retrieves the system `CorePalette` via `DynamicColorPlugin` and applies it in harmonize or full mode.
- **Windows accent color** -- On Windows, retrieves the system accent color and applies it similarly to Monet.
- **Music theme** -- Dynamically generates a `ColorScheme` from album art using `ColorScheme.fromImageProvider`.
- **Theme switching** -- `changeTheme()` persists the selected theme name to `SharedPreferences` and calls `AdaptiveTheme.of(context).setTheme()`.
- **Dark mode detection** -- `inDarkMode(context)` checks `AdaptiveTheme.of(context).mode` against system brightness.
- **Gradient background** -- `isGradientBg(context)` checks the current theme's `gradientBg` flag.
- **Skin-aware scroll physics** -- iOS skin gets `CustomBouncingScrollPhysics`, all others get `ClampingScrollPhysics`.

### Theme loading flow

1. On app startup, `ThemesService.init()` fetches the Monet palette and Windows accent color.
2. When a theme is loaded (`_loadTheme`), the selected light and dark `ThemeStruct` objects are retrieved from the database.
3. Their `ThemeData` objects are passed through `getStructsFromData()`, which on Windows calls `_applyWindowsAccent()` and on other platforms calls `_applyMonet()`.
4. The Monet/accent integration either harmonizes or fully replaces color scheme tokens.
5. The resulting `ThemeData` pair is set via `AdaptiveTheme.of(context).setTheme(light: ..., dark: ...)`.

### Theme storage

- **Native platforms (Android, Windows, Linux, macOS):** Themes are stored in ObjectBox as `ThemeStruct` entities. The `data` field is serialized to/from JSON.
- **Web:** The web implementation in `lib/database/html/theme.dart` provides a simplified `ThemeStruct` that does not persist to a database. It returns `ts.defaultThemes` directly from `getThemes()`.

### Theme selection persistence

The selected light and dark theme names are stored in `SharedPreferences`:

- `"selected-light"` -- Name of the active light theme (default: `"Bright White"`)
- `"selected-dark"` -- Name of the active dark theme (default: `"OLED Dark"`)
- `"previous-light"` / `"previous-dark"` -- Used to revert after temporary theme changes (e.g., Music Theme)

---

## Color System

### Material 3 ColorScheme tokens

Every theme defines all 27 Material 3 `ColorScheme` tokens. These are the tokens stored and edited in the theming system:

| Token | Usage in BlueBubbles |
|---|---|
| `primary` | Main accent on buttons, sliders, chips, switches, FABs, active toggle elements |
| `onPrimary` | Text/icons on primary-colored elements |
| `primaryContainer` | Container fill for buttons, switches, cards |
| `onPrimaryContainer` | Text/icons on primaryContainer elements |
| `secondary` | Attention-drawing accent buttons |
| `onSecondary` | Text/icons on secondary elements |
| `secondaryContainer` | Secondary container fills |
| `onSecondaryContainer` | Text/icons on secondary containers |
| `tertiary` | Tertiary accent |
| `onTertiary` | Text/icons on tertiary elements |
| `tertiaryContainer` | Pinned chat mute/unmute status indicator, selection highlight |
| `onTertiaryContainer` | Text/icons on tertiary container elements |
| `error` | Error indicators (e.g., failed message icon) |
| `onError` | Text/icons on error elements |
| `errorContainer` | Desktop X-button hover color |
| `onErrorContainer` | Desktop X-button icon color |
| `background` | Main app background color |
| `onBackground` | Text/icons on background |
| `surface` | Alternate background color |
| `onSurface` | Text/icons on surface elements |
| `surfaceVariant` | Alternate background, settings divider color between tiles |
| `onSurfaceVariant` | Text/icons on surfaceVariant |
| `outline` | Outlined elements, small label-style text |
| `shadow` | Shadow color |
| `inverseSurface` | Attention-grabbing background (snackbars, toasts) |
| `onInverseSurface` | Text/icons on inverse surface |
| `inversePrimary` | Inverse primary |

### Proper surface selection

The app uses an algorithm to decide which surface color provides sufficient contrast against the background:

```dart
Color get properSurface =>
    surface.computeDifference(background) < 8 ? surfaceVariant : surface;
```

If `surface` and `background` are too similar (Euclidean RGB distance < 8% of max), `surfaceVariant` is used instead. The threshold value `8` represents 8% of the maximum possible RGB distance.

### Theme extension: BubbleColors

BlueBubbles extends `ThemeData` with a `BubbleColors` `ThemeExtension` containing six optional color overrides for message bubbles:

| Field | Description |
|---|---|
| `iMessageBubbleColor` | Sent iMessage bubble background |
| `oniMessageBubbleColor` | Text/icons on iMessage bubble |
| `smsBubbleColor` | Sent SMS/Text Forwarding bubble background |
| `onSmsBubbleColor` | Text/icons on SMS bubble |
| `receivedBubbleColor` | Received message bubble background |
| `onReceivedBubbleColor` | Text/icons on received bubbles |

For the two default themes (OLED Dark and Bright White), explicit bubble colors are set:

| Color | OLED Dark | Bright White |
|---|---|---|
| iMessage bubble | `#1982FC` (blue) | `#1982FC` (blue) |
| On iMessage bubble | `#FFFFFF` (white) | `#FFFFFF` (white) |
| SMS bubble | `#43CC47` (green) | `#43CC47` (green) |
| On SMS bubble | `#FFFFFF` (white) | `#FFFFFF` (white) |
| Received bubble | `#323332` (dark gray) | `#E9E9E8` (light gray) |
| On received bubble | `#FFFFFF` (white) | `#000000` (black) |

For all other themes, bubble colors are derived algorithmically from the `ColorScheme`:

- **iMessage bubble:** Whichever of `primary` or `primaryContainer` is more "colorful" (lower distance from peak saturation and mid-lightness in HSL space).
- **SMS bubble:** The opposite of the iMessage bubble selection.
- **Colorfulness formula:** `sqrt((saturation - 1)^2 + (lightness - 0.5)^2)` -- lower values are more colorful.

When Material You (Monet) is active, the algorithm-derived colors are always used. Otherwise, the `BubbleColors` extension values take priority if present.

### Theme extension: BubbleText

`BubbleText` is a `ThemeExtension` that holds a single `TextStyle` for message bubble text. It defaults to `bodyMedium` with:

- `fontSize`: 15.0 (independent of the `bodyMedium` theme font size)
- `height`: `bodyMedium.height * 0.85` (tighter line height for chat bubbles)

### Colorful avatars and bubbles

Two independent settings control per-contact color assignment:

- **`colorfulAvatars`** -- When enabled, each contact avatar displays a unique gradient color derived from their address. When disabled, all avatars use a gray gradient (`#686868` to `#928E8E`).
- **`colorfulBubbles`** -- When enabled, received message bubbles display the contact's assigned color gradient instead of the theme's `properSurface` color.

The color gradient for a contact is deterministically generated from their address string:

```dart
// Seed from character codes
int total = 0;
for (int i = 0; i < str.length; i++) {
  total += str.codeUnitAt(i);
}
Random random = Random(total);
int seed = random.nextInt(7);
```

Seven gradient palettes are available:

| Seed | Color | Gradient start | Gradient end |
|---|---|---|---|
| 0 | Pink | `#FD678D` | `#FF8AA8` |
| 1 | Blue | `#6BCFF6` | `#94DDFD` |
| 2 | Orange | `#FEA21C` | `#FEB854` |
| 3 | Green | `#5EDE79` | `#8DE798` |
| 4 | Yellow | `#FFCA1C` | `#FCD752` |
| 5 | Red | `#FF534D` | `#FD726A` |
| 6 | Purple | `#A78DF3` | `#BCABFC` |

Users can override any contact's color via a color picker dialog. Custom colors are stored on the `Handle` model's `color` field.

### Color helper utilities

`lib/helpers/ui/theme_helpers.dart` provides extension methods on `Color`:

| Method | Description |
|---|---|
| `darkenPercent(double percent)` | Darkens by multiplying RGB channels by `(1 - percent/100)` |
| `lightenPercent(double percent)` | Lightens by interpolating RGB toward 255 |
| `lightenOrDarken(double percent)` | Darkens if close to black (distance <= 50), otherwise lightens |
| `oppositeLightenOrDarken(double percent)` | Inverse of `lightenOrDarken` |
| `themeLightenOrDarken(context, percent)` | Darkens in light mode, lightens in dark mode |
| `themeOpacity(context)` | Returns full opacity when window effects disabled, otherwise custom opacity |
| `darkenAmount(double amount)` | Adjusts HSL lightness down by amount (0.0--1.0) |
| `lightenAmount(double amount)` | Adjusts HSL lightness up by amount (0.0--1.0) |
| `computeDifference(Color other)` | Euclidean RGB distance as percentage (0--100) |

### HexColor

`HexColor` extends Flutter's `Color` class and parses hex strings:

```dart
HexColor("1982FC")   // Prepends "FF" for full opacity
HexColor("#323332")  // Strips the "#" prefix
HexColor("FF43CC47") // 8-character ARGB
```

---

## Color Engine

BlueBubbles includes a ported implementation of the kdrag0n Monet color engine in `lib/utils/color_engine/`. This engine is used as a fallback when the platform does not support wallpaper-based Monet (Material You) natively.

### Color spaces

The engine implements full conversion paths between these color spaces:

**Source:** `lib/utils/color_engine/colors.dart`

| Class | Color Space | Description |
|---|---|---|
| `Srgb` | sRGB | Standard 8-bit per channel RGB. Quantizes to integer via `quantize8()`. |
| `LinearSrgb` | Linear sRGB | Linearized sRGB (gamma removed). Central hub for conversions. |
| `Oklab` | Oklab | Perceptually uniform color space by Bjorn Ottosson. Used for palette generation. |
| `Oklch` | Oklch | Cylindrical form of Oklab (Lightness, Chroma, Hue). Primary working space for the engine. |
| `CieXyz` | CIE XYZ | CIE 1931 color space. Uses D65 illuminant: `(0.95047, 1.0, 0.108883)`. |
| `CieLab` | CIE L*a*b* | Perceptually uniform space used for L* lightness targeting. |
| `CieLch` | CIE LCH | Cylindrical form of CIE L*a*b*. |

### Transfer functions

sRGB gamma encoding/decoding uses the standard IEC 61966-2-1 transfer functions:

- **OETF (linear to sRGB):** `x >= 0.0031308 ? 1.055 * pow(x, 1/2.4) - 0.055 : 12.92 * x`
- **EOTF (sRGB to linear):** `x >= 0.04045 ? pow((x + 0.055) / 1.055, 2.4) : x / 12.92`

### Palette generation

**Source:** `lib/utils/color_engine/theme.dart`

#### TargetColors

`TargetColors` defines the target chroma and lightness values for the five Monet palettes:

**Chroma values (Oklch):**

| Palette | Chroma |
|---|---|
| `neutral1` | 0.0132 (from Google CAM16) |
| `neutral2` | 0.0066 (half of neutral1) |
| `accent1` | 0.1212 (from Pixel defaults) |
| `accent2` | 0.04 |
| `accent3` | 0.06 |

**Lightness map (13 shades, CIELAB L\*):**

| Shade | Oklch Lightness | CIELAB L* Target |
|---|---|---|
| 0 | 1.000 | 100.0 |
| 10 | 0.988 | 99.0 |
| 50 | 0.955 | 95.0 |
| 100 | 0.913 | 90.0 |
| 200 | 0.827 | 80.0 |
| 300 | 0.741 | 70.0 |
| 400 | 0.653 | 60.0 |
| 500 | 0.562 | 49.6 |
| 600 | 0.482 | 40.0 |
| 700 | 0.394 | 30.0 |
| 800 | 0.309 | 20.0 |
| 900 | 0.222 | 10.0 |
| 1000 | 0.000 | 0.0 |

#### DynamicColorScheme

`DynamicColorScheme` takes a seed `primaryColor` and generates all five palettes:

1. Converts the primary color to Oklch.
2. For each palette, applies a chroma multiplier (default 1.0) and uses the primary hue.
3. For `accent3`, shifts the hue by +60 degrees (secondary color on the color wheel).
4. For each shade, performs a binary search to find the Oklch lightness value that produces the target CIELAB L* after quantization to 8-bit sRGB. This "accurate shades" mode ensures perceptual accuracy despite quantization.

**Binary search parameters:**
- Search range: L = -0.5 to 1.5 (allows for imperfect blacks and overexposed whites)
- Target threshold: 0.01 L* (considered a match)
- Epsilon: 0.001 (minimum search interval before terminating)

#### MonetColors and MonetPalette

The engine output is a `MonetColors` object containing five `MonetPalette` instances:

- `accent1` -- Main accent, close to primary color
- `accent2` -- Secondary accent, darker shades
- `accent3` -- Tertiary accent, hue-shifted by 60 degrees
- `neutral1` -- Main background, tinted with primary
- `neutral2` -- Secondary background, slightly tinted

Each `MonetPalette` extends `ColorSwatch<int>` and contains 13 shades (0, 10, 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000) accessible via `shade0` through `shade1000`.

---

## Built-in Themes

BlueBubbles ships with three hand-crafted base themes plus all `FlexScheme` variants in both light and dark modes.

### Core themes

#### OLED Dark (default dark theme)

- **Brightness:** Dark
- **Seed color:** `Colors.blue`
- **Background:** `#000000` (pure black, OLED-friendly)
- **Error:** `Colors.red`
- **iMessage bubble:** `#1982FC`
- **SMS bubble:** `#43CC47`
- **Received bubble:** `#323332`
- **Generated via:** `ColorScheme.fromSeed(seedColor: Colors.blue, background: Colors.black, brightness: Brightness.dark)`

#### Bright White (default light theme)

- **Brightness:** Light
- **Seed color:** `Colors.blue`
- **Background:** `#FFFFFF` (pure white)
- **Surface variant:** `#F3F3F6`
- **Error:** `Colors.red`
- **iMessage bubble:** `#1982FC`
- **SMS bubble:** `#43CC47`
- **Received bubble:** `#E9E9E8`
- **Generated via:** `ColorScheme.fromSeed(seedColor: Colors.blue, background: Colors.white, surfaceVariant: HexColor('F3F3F6'), brightness: Brightness.light)`

#### Nord Theme (dark)

- **Brightness:** Dark
- **Primary swatch seed:** `#5E81AC` (Nord blue)
- **Accent color:** `#88C0D0` (Nord frost)
- **Background:** `#3B4252` (Nord polar night)
- **Card color:** `#4C566A` (Nord polar night lighter)
- **Primary container:** `#49688E`
- **Outline:** `Colors.grey`
- **Error:** `Colors.red`
- **Generated via:** `ColorScheme.fromSwatch()` with Nord palette colors

### Music themes

Two special themes exist that dynamically change their color schemes based on currently playing music album art:

- **Music Theme (sun symbol)** -- Light variant, initialized from `whiteLightTheme`
- **Music Theme (moon symbol)** -- Dark variant, initialized from `oledDarkTheme`

When music playback is detected, `ThemesService.updateMusicTheme()` calls `ColorScheme.fromImageProvider()` with the album art bytes to generate both light and dark color schemes, then applies them immediately.

### FlexScheme themes

Every `FlexScheme` enum value (excluding `FlexScheme.custom`) generates two `ThemeStruct` entries:

- A light variant with a sun symbol suffix
- A dark variant with a moon symbol suffix

These are generated with:
- `FlexThemeData.light(scheme: e, surfaceMode: FlexSurfaceMode.highSurfaceLowScaffold, blendLevel: 40)`
- `FlexThemeData.dark(scheme: e, surfaceMode: FlexSurfaceMode.highSurfaceLowScaffold, blendLevel: 40)`

The `highSurfaceLowScaffold` surface mode creates stronger surface color blending while keeping the scaffold (background) closer to neutral. A blend level of 40 provides moderate color blending.

FlexScheme includes over 50 color schemes such as: Material, Material HC, Blue, Indigo, Hippie Blue, Aqua Blue, Brandeis Blue, Deep Blue, Sakura, Mandarin Red, Red, Red & Blue, Pink, Rosewood, Money, Jungle, Grey Law, Wasabi, Gold, Mango, Amber, Vesuvius Burn, Deep Purple, Ebony Clay, Barossa, Shark, Big Stone, Damask, Bahama & Trinidad, Mallard & Valencia, Espresso & Crema, Outer Space, Blue Whale, San Juan Blue, Baross, Lipstick, Flutter Dash, M3 Baseline, Verdun Hemlock, Dell & Genoa Green, Red M3, Pink M3, Purple M3, Indigo M3, Blue M3, Cyan M3, Teal M3, Green M3, Lime M3, Yellow M3, Orange M3, Deep Orange M3, and more.

The theme name is formatted by splitting on capital letters and capitalizing: e.g., `FlexScheme.hippieBlue` becomes `"hippie Blue"`.

---

## Custom Theme Creation

Users create custom themes through the Advanced Theming panel (`lib/app/layouts/settings/pages/theming/advanced/`).

### What is customizable

**Colors (21 paired tokens + 1 standalone):**
All `ColorScheme` tokens are editable in pairs (color + "on" variant). The SMS bubble color pair is also editable as a separate `BubbleColors` extension. The `outline` token is edited standalone.

**Text sizes:**
All six `TextTheme` slots plus `bubbleText` have individually adjustable font sizes.

**Font family:**
Any Google Font can be selected via a searchable dropdown. The font applies to all text styles globally. The default is `"Default"` which uses the platform's standard font.

**Gradient background:**
Per-theme toggle for an animated gradient background in conversation views.

### Customization constraints

- Preset (built-in) themes cannot have their colors or fonts edited. Users must create a new custom theme to modify colors.
- When Material You (Monet) is enabled (`Monet.harmonize` or `Monet.full`), the color editor is disabled because Monet overrides the color scheme at runtime.
- Custom themes that are not presets can be deleted. Presets are permanent.

### Theme selection UI

The Advanced Theming panel has two tabs (Light Theme / Dark Theme) with a `NavigationBar` at the bottom. Each tab shows:

1. A theme selector dropdown listing all themes, divided into three groups by dividers:
   - Custom/base themes (no sun/moon suffix)
   - Themes matching the current tab's brightness
   - Themes for the opposite brightness
2. A grid of color pair cards, each showing two 12x12 color swatches for quick preview.
3. Text size adjustment controls.
4. Font family selector.
5. A "Create New" FAB that opens a dialog to clone and name a new theme.

### Import/export

Themes are serialized to JSON via `ThemeStruct.toMap()`. The old `ThemeObject`/`ThemeEntry` system is legacy but still supported for import via the "View Old" action in the app bar.

---

## Typography

### Font system

BlueBubbles uses `google_fonts` for dynamic font loading. The base typography foundation is:

- **Light mode:** `Typography.englishLike2021.merge(Typography.blackMountainView)`
- **Dark mode:** `Typography.englishLike2021.merge(Typography.whiteMountainView)`

`Typography.englishLike2021` provides Material Design 3 English text geometry. `blackMountainView` and `whiteMountainView` provide Roboto-based styles with black or white text colors for respective brightnesses.

All letter spacing is explicitly zeroed out (`letterSpacingFactor: 0`) for a cleaner, tighter appearance.

### Default text sizes

| TextTheme Slot | Default Size | Usage |
|---|---|---|
| `titleLarge` | 22.0 sp | Page titles, app bar titles, section headings |
| `bodyLarge` | 16.0 sp | Primary body text, conversation subtitles |
| `bodyMedium` | 14.0 sp | Standard body text, list item text |
| `bodySmall` | 12.0 sp | Secondary information, timestamps, captions |
| `labelLarge` | 14.0 sp | Button text, setting subtitles, list tile subtitles |
| `labelSmall` | 11.0 sp | Overline text, smallest labels |
| `bubbleText` | 15.0 sp | Message bubble text (independent of bodyMedium) |

### Bubble text specifics

Bubble text uses a dedicated `BubbleText` `ThemeExtension` rather than a standard `TextTheme` slot. This allows message text to have an independent font size from the rest of the UI. The line height multiplier is `0.85` of the standard `bodyMedium` height, producing tighter vertical spacing in chat bubbles.

### Font weight assignments

Font weights are stored as indices into `FontWeight.values` (0 = w100 through 8 = w900). The default weights follow Material Design 3 conventions:

- `titleLarge`: `FontWeight.w400` (index 3)
- `bodyLarge`: `FontWeight.w400` (index 3)
- `bodyMedium`: `FontWeight.w400` (index 3)
- `bodySmall`: `FontWeight.w400` (index 3)
- `labelLarge`: `FontWeight.w500` (index 4)
- `labelSmall`: `FontWeight.w400` (index 3)

### Non-themeable text styles

The following `TextTheme` slots are not individually customizable via the advanced theming UI. They still receive the selected Google Font but use default Material Design geometry:

`displayLarge`, `displayMedium`, `displaySmall`, `headlineLarge`, `headlineMedium`, `headlineSmall`, `titleMedium`, `titleSmall`, `labelMedium`

### Platform-specific text styling

- **iOS skin subtitle:** `labelLarge` with `FontWeight.w300`, color varies based on dark mode and window effect state.
- **Material/Samsung skin subtitle:** `labelLarge` with `FontWeight.bold`, colored with `primary`.

---

## Spacing and Layout

### Message bubble dimensions

| Property | Value |
|---|---|
| Max bubble width factor | `0.75` of screen width (75%) |
| Max bubble width | `screenWidth * 0.75 - 40` (minus padding) |
| Min bubble height | 40.0 |
| Bubble internal padding | `EdgeInsets.symmetric(vertical: 10, horizontal: 15)` |
| From-me right padding | +10 additional right |
| Received left padding | +10 additional left |
| Big emoji max width | Full screen width |

### Bubble shape geometry

Bubble corners use a `TailClipper` custom clipper with specific radius values:

| Element | Radius |
|---|---|
| Standard corner radius | 20.0 |
| iOS tail arc radius | 10.0 (inner), 20.0 (outer) |
| Connected bubble radius | 5.0 (smaller for tighter grouping) |
| Tail intersection angle | Slightly more than 45 degrees |
| Tail offset | Offset(6.547, 5.201) from bottom edge |

The tail shape differs by skin:
- **iOS skin:** Curved iMessage-style tail at the bottom corner, arcing out and back.
- **Material/Samsung skin:** Simple rounded rectangle, no tail. Connected bubbles have smaller corner radii (5.0 vs 20.0) on the shared edge.

### Reply bubble constraints

| Property | Value |
|---|---|
| Max width | `screenWidth * 0.75 - 30` |
| Min height | 30.0 |

### Avatar dimensions

| Property | Default Value |
|---|---|
| Default avatar size | Determined by parent context |
| Border thickness | 2.0 |
| Color picker wheel diameter | 165 |
| Color picker min dialog height | 480 |
| Color picker dialog padding | 70 from each side |

### Settings layout

| Property | Value |
|---|---|
| App bar height | 50 |
| Color swatch preview size | 12 x 12 with 4px border radius, 3px padding |
| Tablet mode threshold | Screen width > 600, aspect ratio > 0.8 |
| Samsung header scroll snap distance | `screenHeight / 3 - 57` |
| Scrolled-under elevation | 3.0 |

### General spacing patterns

- Standard card border radius: 4.0 for small color swatches
- Icon padding in headers: Consistent with Material 3 specs
- List tile subtitle padding: Follows skin conventions
- Window effect alpha for tile backgrounds: 100 (of 255) when effects enabled, 255 when disabled
- Window effect alpha for header backgrounds: 20 (of 255) when effects enabled, 255 when disabled

---

## Component Styling

### Message bubbles

**Sent messages (isFromMe):**
- Background color: `primary` (or `primary.darkenAmount(0.2)` for pending/temp messages)
- Selected state: `tertiaryContainer`
- Text color: Theme's on-primary or on-primary-container (from `BubbleColors` extension when not using Monet)

**Received messages:**
- Background: `properSurface` (or contact's color gradient when `colorfulBubbles` is enabled)
- Gradient direction: `bottomCenter` to `topCenter`
- When colorful bubbles active and no custom handle color: Deterministic gradient from address hash
- When colorful bubbles active with custom handle color: Solid color to `color.lightenAmount(0.075)`
- Text color on colorful bubbles: `getBubbleColors().first.oppositeLightenOrDarken(75)`

**iMessage vs SMS distinction:**
- iMessage bubbles use the more "colorful" of primary/primaryContainer
- SMS bubbles use the less "colorful" of primary/primaryContainer
- Default explicit colors: iMessage = `#1982FC`, SMS = `#43CC47`

### Avatars

Avatars display a circular gradient background with the contact's initials overlaid:

- When `colorfulAvatars` enabled: Gradient from the contact's assigned colors
- When `colorfulAvatars` disabled: Gray gradient from `#928E8E` to `#686868`
- iOS skin uses reversed gradient order (colors[1], colors[0])
- Material/Samsung uses standard order (colors[0], colors[0])

### App bars

- Background: `headerColor` (skin-dependent, see Platform-Specific Styling)
- Title style: `titleLarge` from theme text theme
- Elevation: 0 (flat)
- Scrolled-under elevation: 3.0
- Surface tint: `primary`
- Center title: Only on iOS skin
- System overlay style: Inverted brightness (dark status bar icons on light backgrounds)

### Settings tiles

- **iOS skin:** Subtitle uses `labelLarge` at `FontWeight.w300`
- **Material/Samsung skin:** Subtitle uses `labelLarge` at `FontWeight.bold` with `primary` color
- Header/tile color mapping reverses in Material dark mode

### Conversation list tiles

- Different widget implementations per skin: `CupertinoConversationTile`, `MaterialConversationTile`, `SamsungConversationTile`
- Pinned tiles use `tertiaryContainer` for mute/unmute status
- Pinned tile text bubbles respect `colorfulBubbles` and `colorfulAvatars` settings

### Input fields

- Send sound: Configurable via settings
- Text field height is measured dynamically and used for send animation positioning
- Attachment picker uses the theme's surface colors

### Floating Action Buttons

- Background: `primary`
- Icon/label color: `onPrimary`
- Label style: `labelLarge`

---

## Dark and Light Mode

### AdaptiveTheme integration

BlueBubbles uses the `adaptive_theme` package to manage light/dark/system mode. Three modes are available:

| Mode | Behavior |
|---|---|
| `AdaptiveThemeMode.light` | Always uses the selected light theme |
| `AdaptiveThemeMode.dark` | Always uses the selected dark theme |
| `AdaptiveThemeMode.system` | Follows `PlatformDispatcher.instance.platformBrightness` |

### Dark mode detection

```dart
bool inDarkMode(BuildContext context) =>
    (AdaptiveTheme.of(context).mode == AdaptiveThemeMode.dark ||
      (AdaptiveTheme.of(context).mode == AdaptiveThemeMode.system &&
          PlatformDispatcher.instance.platformBrightness == Brightness.dark));
```

### OLED Dark mode

The default dark theme uses `Colors.black` (`#000000`) as the background, which enables true black on OLED displays for power savings. This is achieved by passing `background: Colors.black` to `ColorScheme.fromSeed()`.

### Color mapping inversion

In settings pages, the header and tile background colors swap based on skin and dark mode:

```dart
bool get reverseMapping => ss.settings.skin.value == Skins.Material && ts.inDarkMode(context);
```

- **Material skin, dark mode:** Tile color becomes the lighter surface, header becomes darker background.
- **All other combinations:** Header uses the lighter color, tile uses the darker color.

The base colors are:
- Header base: `background` (dark mode) or `properSurface` (light mode)
- Tile base: `properSurface` (dark mode) or `background` (light mode)

### Gradient background

When a theme has `gradientBg: true`, conversation views display an animated gradient overlay:

- Gradient direction: `topRight` to `bottomLeft`
- Color 1: Bubble color (iMessage or SMS based on chat type) at 50% opacity
- Color 2: Background color at full opacity
- Animation: `MirrorAnimationBuilder` with `Curves.fastOutSlowIn`, 3-second duration
- Gradient stops animate between `[0.0, 0.8]` and `[0.2, 1.0]`

The `GradientBackground` widget wraps the conversation view and responds to platform brightness changes.

---

## Platform-Specific Styling

### Skins enum

```dart
enum Skins {
  iOS,
  Material,
  Samsung,
}
```

### iOS skin

- **Scroll physics:** `CustomBouncingScrollPhysics` (always scrollable, bouncing overscroll)
- **Page transitions:** Custom Cupertino page transitions with `Curves.linearToEaseOut` (forward) and `Curves.easeInToLinear` (reverse)
- **Conversation list:** `CupertinoConversationList` with a Cupertino-style sliver header
- **Message bubbles:** Display iOS-style tails via `TailClipper` with curved arcs
- **Icons:** Uses `CupertinoIcons` (e.g., `CupertinoIcons.pencil`, `CupertinoIcons.sun_max`, `CupertinoIcons.moon`)
- **App bar:** Center-aligned title
- **Subtitle text:** `FontWeight.w300`, color adapts to window effects

### Material skin

- **Scroll physics:** `ClampingScrollPhysics` (standard Material overscroll)
- **Conversation list:** `MaterialConversationList` with Material-style header
- **Message bubbles:** Rounded rectangles without tails; connected bubbles have smaller radii
- **Icons:** Uses Material `Icons` (e.g., `Icons.edit`, `Icons.brightness_high`, `Icons.brightness_3`)
- **Splash effect:** `InkSparkle.splashFactory` (Material 3 sparkle ripple)
- **App bar:** Left-aligned title
- **Subtitle text:** `FontWeight.bold`, colored with `primary`
- **Color mapping:** Reverses header/tile colors in dark mode

### Samsung skin

- **Scroll physics:** `ClampingScrollPhysics`
- **Conversation list:** `SamsungConversationList` with a large collapsible header area
- **Header:** Scroll-snapping Samsung One UI style header that collapses at `screenHeight / 3 - 57` distance
- **Footer:** `SamsungFooter` bottom navigation bar
- **Background color:** Always uses `headerColor` (uses background color for header, not surface)
- **Tile color:** Uses tile color pattern (same as Material but without dark-mode reversal)

### Immersive mode

When `ss.settings.immersiveMode.value` is true, the system navigation bar is set to `Colors.transparent`. Otherwise, it uses `context.theme.colorScheme.background`.

Status bar is always transparent. Status bar icon brightness is set to the opposite of the theme's color scheme brightness.

---

## Window Effects

**Source:** `lib/utils/window_effects.dart`

Window effects are available only on Windows desktop builds. They use the `flutter_acrylic` package to apply system-level visual effects to the window background.

### Available effects

| Effect | Min Build | Max Build | Dependencies | Dark Opacity | Light Opacity |
|---|---|---|---|---|---|
| `WindowEffect.tabbed` | 22523 | -- | brightness | 0.0 | 0.0 |
| `WindowEffect.mica` | 22000 | -- | brightness | 0.0 | 0.0 |
| `WindowEffect.aero` | 0 | 22523 | color | 0.6 | 0.75 |
| `WindowEffect.acrylic` | 17134 | -- | color | 0.0 | 0.6 |
| `WindowEffect.transparent` | 0 | -- | color | 0.7 | 0.7 |
| `WindowEffect.disabled` | 0 | -- | none | 1.0 | 1.0 |

### Effect descriptions

- **Tabbed:** Mica-like material that incorporates theme and desktop wallpaper, sensitive to wallpaper color. Requires Windows 11 build 22523+.
- **Mica:** Opaque dynamic material incorporating theme and wallpaper. Requires Windows 11 build 22000+.
- **Aero:** Windows Vista/7-style glossy blur effect. Available on builds before 22523.
- **Acrylic:** Translucent texture for depth and visual hierarchy. Requires Windows 10 version 1803 (build 17134+).
- **Transparent:** Simple transparent window background.
- **Disabled:** Standard opaque window background.

### Effect dependencies

Effects have two types of dependencies:
- **brightness:** The effect adapts to light/dark mode automatically (Tabbed, Mica)
- **color:** The effect uses a background color (Aero, Acrylic, Transparent)

### Opacity handling

Users can customize opacity per mode via:
- `ss.settings.windowEffectCustomOpacityDark.value`
- `ss.settings.windowEffectCustomOpacityLight.value`

Default opacities vary by effect (see table above). When effects are active, UI elements reduce their alpha:
- Header backgrounds: alpha = 20 (of 255)
- Tile backgrounds: alpha = 100 (of 255)

### Acrylic on Windows 11

On Windows 11 (build >= 22000), Acrylic supports fully transparent backgrounds. On older Windows 10 builds, a minimum non-zero alpha of `1/255` is applied to prevent rendering artifacts.

### Version detection

```dart
int parsedWindowsVersion() {
  String raw = Platform.operatingSystemVersion;
  return int.tryParse(raw.substring(raw.indexOf("Build") + 6, raw.length - 1)) ?? 0;
}
```

---

## Animation Patterns

### Standard durations

| Context | Duration | Curve |
|---|---|---|
| Send animation | 450 ms | Custom tweens |
| Gentle message effect | 1800 ms total | `Curves.easeInOut` |
| Gradient background cycle | 3000 ms (3 s) | `Curves.fastOutSlowIn` |
| Page transition (setup) | 500 ms | `Curves.easeInOut` |
| Page transition (back) | 300 ms | `Curves.easeIn` |
| List item size animation | 150 ms | default |
| Chip/tag animation | 250 ms | `Curves.easeIn` |
| Search debounce | 250 ms | -- |
| Fullscreen media swipe | 300 ms | `Curves.easeIn` |
| Cupertino dialog inset | 100 ms | `Curves.decelerate` |
| Splash screen delay | 100 ms | -- |
| Fullscreen video controls | 100 ms | default |
| Cupertino page forward | -- | `Curves.linearToEaseOut` / `Curves.easeInToLinear` (reverse) |
| Circle progress bar | Dynamic | `Curves.easeInOut` |

### iMessage screen effects

BlueBubbles implements several iMessage screen effects with custom particle/animation systems:

| Effect | Launch Duration | Description |
|---|---|---|
| Balloon | 100 ms auto-launch | Rising balloon particles |
| Fireworks | 100 ms auto-launch | Exploding firework particles |
| Love | 100 ms auto-launch | Floating heart animations |
| Laser | 500 ms auto-launch | Laser beam effects |
| Spotlight | 100 ms auto-launch | Spotlight/focus effect |
| Celebration | 100 ms auto-launch | Confetti-style celebration |

### Gentle send effect

The "gentle" bubble effect uses a three-phase `MovieTween`:
1. Phase 1 (0--1 ms): Hold at scale 1.0
2. Phase 2 (1--500 ms): Shrink from scale 1.0 to 0.0 (partial, targets 0.5)
3. Phase 3 (1000--1800 ms): Grow from scale 0.5 back to 1.0

The animation uses `Curves.easeInOut` for both shrink and grow phases.

### Send animation

The send animation (message appearing in the conversation) has a 450 ms duration and animates the message widget from the text field position to its final location in the message list.

### Gradient background animation

Uses `MirrorAnimationBuilder` (from `simple_animations`) to create a continuously oscillating gradient:

```dart
MovieTween()
  ..scene(begin: Duration.zero, duration: Duration(seconds: 3))
      .tween("color1", Tween<double>(begin: 0, end: 0.2))
  ..scene(begin: Duration.zero, duration: Duration(seconds: 3))
      .tween("color2", Tween<double>(begin: 0.8, end: 1))
```

The two gradient stops oscillate between `[0.0, 0.8]` and `[0.2, 1.0]`, creating a subtle breathing effect. The mirror builder reverses the animation at each endpoint for infinite seamless looping.

### Splash factory

All themes use `InkSparkle.splashFactory` as their splash factory, providing the Material 3 sparkle ripple effect instead of the default ink splash or ink ripple.

---

## Key Source Files Reference

| File | Purpose |
|---|---|
| `lib/database/io/theme.dart` | `ThemeStruct` entity (native platforms) |
| `lib/database/html/theme.dart` | `ThemeStruct` entity (web) |
| `lib/database/global/theme_colors.dart` | Legacy color token constants |
| `lib/services/ui/theme/themes_service.dart` | Theme service, defaults, Monet, switching |
| `lib/helpers/ui/theme_helpers.dart` | HexColor, BubbleColors, BubbleText, color/theme extensions |
| `lib/helpers/types/constants.dart` | Skins, Monet enums |
| `lib/utils/color_engine/colors.dart` | Color space implementations (sRGB, Oklab, Oklch, CIE) |
| `lib/utils/color_engine/theme.dart` | Monet palette generation (TargetColors, DynamicColorScheme) |
| `lib/utils/color_engine/engine.dart` | Engine library exports |
| `lib/utils/window_effects.dart` | Desktop window transparency effects |
| `lib/app/wrappers/gradient_background_wrapper.dart` | Animated gradient background |
| `lib/app/layouts/conversation_view/widgets/message/text/text_bubble.dart` | Message bubble rendering |
| `lib/app/layouts/conversation_view/widgets/message/misc/tail_clipper.dart` | Bubble tail shape clipping |
| `lib/app/components/avatars/contact_avatar_widget.dart` | Avatar rendering and color picker |
| `lib/app/layouts/settings/pages/theming/advanced/advanced_theming_panel.dart` | Advanced theming UI |
| `lib/app/layouts/settings/pages/theming/advanced/advanced_theming_content.dart` | Theme editor content |
