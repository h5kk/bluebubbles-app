# 01 - Design Tokens

> Complete design token specification for colors, typography, spacing, borders, shadows, animations, themes, and dark/light/OLED modes.

---

## 1. Color Palette - Material 3 ColorScheme Tokens

Every theme defines all 27 Material 3 `ColorScheme` tokens. These are the canonical color roles used throughout the application.

### Token Definitions and Usage

| # | Token | Usage in BlueBubbles |
|---|-------|---------------------|
| 1 | `primary` | Main accent: buttons, sliders, chips, switches, FABs, active toggles |
| 2 | `onPrimary` | Text/icons on primary-colored elements |
| 3 | `primaryContainer` | Container fill for buttons, switches, cards |
| 4 | `onPrimaryContainer` | Text/icons on primaryContainer elements |
| 5 | `secondary` | Attention-drawing accent buttons |
| 6 | `onSecondary` | Text/icons on secondary elements |
| 7 | `secondaryContainer` | Secondary container fills |
| 8 | `onSecondaryContainer` | Text/icons on secondary containers |
| 9 | `tertiary` | Tertiary accent |
| 10 | `onTertiary` | Text/icons on tertiary elements |
| 11 | `tertiaryContainer` | Pinned chat mute/unmute indicator, selection highlight |
| 12 | `onTertiaryContainer` | Text/icons on tertiary container elements |
| 13 | `error` | Error indicators (failed message icon) |
| 14 | `onError` | Text/icons on error elements |
| 15 | `errorContainer` | Desktop X-button hover color |
| 16 | `onErrorContainer` | Desktop X-button icon color |
| 17 | `background` | Main app background color |
| 18 | `onBackground` | Text/icons on background |
| 19 | `surface` | Alternate background color |
| 20 | `onSurface` | Text/icons on surface elements |
| 21 | `surfaceVariant` | Alternate background, settings divider between tiles |
| 22 | `onSurfaceVariant` | Text/icons on surfaceVariant |
| 23 | `outline` | Outlined elements, small label-style text |
| 24 | `shadow` | Shadow color |
| 25 | `inverseSurface` | Attention-grabbing background (snackbars, toasts) |
| 26 | `onInverseSurface` | Text/icons on inverse surface |
| 27 | `inversePrimary` | Inverse primary accent |

### Proper Surface Selection Algorithm

When `surface` and `background` are too similar, use `surfaceVariant` instead:

```
properSurface = (surface.computeDifference(background) < 8) ? surfaceVariant : surface
```

The threshold `8` represents 8% of the maximum possible Euclidean RGB distance.

---

## 2. Bubble Color Tokens

Six additional color tokens for message bubbles, defined via the `BubbleColors` ThemeExtension:

| Token | Description |
|-------|-------------|
| `iMessageBubbleColor` | Sent iMessage bubble background |
| `oniMessageBubbleColor` | Text/icons on iMessage bubble |
| `smsBubbleColor` | Sent SMS/Text Forwarding bubble background |
| `onSmsBubbleColor` | Text/icons on SMS bubble |
| `receivedBubbleColor` | Received message bubble background |
| `onReceivedBubbleColor` | Text/icons on received bubbles |

### Default Bubble Colors (OLED Dark and Bright White themes)

| Color Token | OLED Dark | Bright White |
|-------------|-----------|--------------|
| iMessage bubble | `#1982FC` | `#1982FC` |
| On iMessage bubble | `#FFFFFF` | `#FFFFFF` |
| SMS bubble | `#43CC47` | `#43CC47` |
| On SMS bubble | `#FFFFFF` | `#FFFFFF` |
| Received bubble | `#323332` | `#E9E9E8` |
| On received bubble | `#FFFFFF` | `#000000` |

### Algorithmic Bubble Color Derivation

For non-default themes, bubble colors are derived from the ColorScheme:

- **iMessage bubble**: Whichever of `primary` or `primaryContainer` is more "colorful"
- **SMS bubble**: The opposite (less colorful) of the two
- **Colorfulness formula**: `sqrt((saturation - 1)^2 + (lightness - 0.5)^2)` -- lower value = more colorful

When Material You (Monet) is active, algorithmic colors always take priority over BubbleColors extension values.

---

## 3. Avatar Color Palettes

Seven deterministic gradient palettes assigned to contacts based on address string hash:

| Seed | Name | Gradient Start | Gradient End |
|------|------|---------------|--------------|
| 0 | Pink | `#FD678D` | `#FF8AA8` |
| 1 | Blue | `#6BCFF6` | `#94DDFD` |
| 2 | Orange | `#FEA21C` | `#FEB854` |
| 3 | Green | `#5EDE79` | `#8DE798` |
| 4 | Yellow | `#FFCA1C` | `#FCD752` |
| 5 | Red | `#FF534D` | `#FD726A` |
| 6 | Purple | `#A78DF3` | `#BCABFC` |

### Hash Algorithm
```
total = sum of all character codes in address string
seed = Random(total).nextInt(7)
```

### Disabled State
When `colorfulAvatars` is disabled, all avatars use gray gradient: `#928E8E` to `#686868`.

### Gradient Direction
- iOS skin: reversed order (end color, start color)
- Material/Samsung skin: standard order (start color, start color -- solid)

---

## 4. Typography Scale

### Base Typography
- Light mode foundation: `Typography.englishLike2021.merge(Typography.blackMountainView)`
- Dark mode foundation: `Typography.englishLike2021.merge(Typography.whiteMountainView)`
- Letter spacing: **0** globally (zeroed out via `letterSpacingFactor: 0`)

### Text Style Slots

| Slot | Default Size | Default Weight | Weight Index | Usage |
|------|-------------|----------------|--------------|-------|
| `titleLarge` | 22.0 sp | w400 | 3 | Page titles, app bar titles, section headings |
| `bodyLarge` | 16.0 sp | w400 | 3 | Primary body text, conversation subtitles |
| `bodyMedium` | 14.0 sp | w400 | 3 | Standard body text, list item text |
| `bodySmall` | 12.0 sp | w400 | 3 | Secondary info, timestamps, captions |
| `labelLarge` | 14.0 sp | w500 | 4 | Button text, setting subtitles |
| `labelSmall` | 11.0 sp | w400 | 3 | Overline text, smallest labels |
| `bubbleText` | 15.0 sp | (inherits bodyMedium) | -- | Message bubble text (independent size) |

### Bubble Text Specifics
- Font size: 15.0 sp (independent of bodyMedium)
- Line height multiplier: `bodyMedium.height * 0.85` (tighter vertical spacing)
- Defined via `BubbleText` ThemeExtension

### Platform-Specific Text Styling

| Context | iOS Skin | Material/Samsung Skin |
|---------|----------|----------------------|
| Subtitle | `labelLarge`, w300, muted color | `labelLarge`, bold, `primary` color |

### Font System
- Default font: Platform standard ("Default")
- Custom fonts: Any Google Font, selected via searchable dropdown
- Font applies globally to all text styles
- Non-customizable slots (receive font but use default geometry): `displayLarge`, `displayMedium`, `displaySmall`, `headlineLarge`, `headlineMedium`, `headlineSmall`, `titleMedium`, `titleSmall`, `labelMedium`

---

## 5. Spacing Scale

### Message Bubble Dimensions

| Property | Value |
|----------|-------|
| Max bubble width factor | 75% of screen width |
| Max bubble width | `screenWidth * 0.75 - 40` (minus padding) |
| Min bubble height | 40.0px |
| Bubble internal padding | 10px vertical, 15px horizontal |
| From-me extra right padding | +10px |
| Received extra left padding | +10px |
| Big emoji max width | Full screen width |

### Reply Bubble Constraints

| Property | Value |
|----------|-------|
| Max width | `screenWidth * 0.75 - 30` |
| Min height | 30.0px |

### Avatar Dimensions

| Property | Value |
|----------|-------|
| Default size | Context-dependent (commonly 40px) |
| Border thickness | 2.0px |
| Color picker wheel diameter | 165px |
| Color picker min dialog height | 480px |
| Color picker dialog side padding | 70px |

### Settings Layout

| Property | Value |
|----------|-------|
| App bar height | 50px |
| Color swatch preview | 12 x 12px |
| Color swatch border radius | 4.0px |
| Color swatch padding | 3.0px |

### General Spacing

| Property | Value |
|----------|-------|
| Tablet mode width threshold | 600px |
| Tablet mode aspect ratio threshold | 0.8 |
| Samsung header snap distance | `screenHeight / 3 - 57` |
| Scrolled-under elevation | 3.0 |
| TabletModeWrapper divider width | 7.0px |
| TabletModeWrapper default split ratio | 0.5 |

### Window Effect Alpha Values

| Element | Effects Enabled | Effects Disabled |
|---------|----------------|-----------------|
| Header backgrounds | alpha 20/255 | alpha 255/255 |
| Tile backgrounds | alpha 100/255 | alpha 255/255 |

---

## 6. Border Radius

| Element | Radius |
|---------|--------|
| Standard bubble corner | 20.0px |
| Connected bubble corner (shared edge) | 5.0px |
| iOS tail arc (inner) | 10.0px |
| iOS tail arc (outer) | 20.0px |
| Color swatch preview | 4.0px |
| Tail offset from bottom edge | Offset(6.547, 5.201) |

---

## 7. Shadow / Elevation

| Context | Elevation |
|---------|-----------|
| App bar (default) | 0 (flat) |
| App bar (scrolled under) | 3.0 |
| App bar surface tint | `primary` color |

---

## 8. Animation Timing

### Standard Durations

| Context | Duration | Easing Curve |
|---------|----------|-------------|
| Send animation | 450ms | Custom tweens |
| Gentle message effect | 1800ms total | easeInOut |
| Gradient background cycle | 3000ms | fastOutSlowIn |
| Setup page transition (forward) | 500ms | easeInOut |
| Setup page transition (back) | 300ms | easeIn |
| List item size animation | 150ms | default (linear) |
| Chip/tag animation | 250ms | easeIn |
| Search debounce | 250ms | -- |
| Fullscreen media swipe | 300ms | easeIn |
| Cupertino dialog inset | 100ms | decelerate |
| Splash screen delay | 100ms | -- |
| Fullscreen video controls | 100ms | default |
| Circle progress bar | Dynamic | easeInOut |

### Page Transition Curves

| Skin | Forward Curve | Reverse Curve |
|------|--------------|---------------|
| iOS | linearToEaseOut | easeInToLinear |
| Material | Standard MaterialPageRoute | Standard MaterialPageRoute |
| Samsung | Standard MaterialPageRoute | Standard MaterialPageRoute |

### iMessage Screen Effects Auto-Launch Delays

| Effect | Delay |
|--------|-------|
| Balloon | 100ms |
| Fireworks | 100ms |
| Love | 100ms |
| Laser | 500ms |
| Spotlight | 100ms |
| Celebration | 100ms |

### Typing Indicator Debounce
- Typing event sends: 3-second debounce between `started-typing` / `stopped-typing` socket events

### Message Pagination
- Fetch chunk size: 25 messages
- New message insertion animation: 500ms SizeTransition + SlideTransition

---

## 9. Built-in Theme Definitions

### Core Themes

#### OLED Dark (Default Dark Theme)
| Property | Value |
|----------|-------|
| Brightness | Dark |
| Seed color | `Colors.blue` |
| Background | `#000000` (pure black) |
| Error | `Colors.red` |
| iMessage bubble | `#1982FC` |
| SMS bubble | `#43CC47` |
| Received bubble | `#323332` |
| On iMessage bubble | `#FFFFFF` |
| On SMS bubble | `#FFFFFF` |
| On received bubble | `#FFFFFF` |
| Generation | `ColorScheme.fromSeed(seedColor: Colors.blue, background: Colors.black, brightness: Brightness.dark)` |

#### Bright White (Default Light Theme)
| Property | Value |
|----------|-------|
| Brightness | Light |
| Seed color | `Colors.blue` |
| Background | `#FFFFFF` (pure white) |
| Surface variant | `#F3F3F6` |
| Error | `Colors.red` |
| iMessage bubble | `#1982FC` |
| SMS bubble | `#43CC47` |
| Received bubble | `#E9E9E8` |
| On iMessage bubble | `#FFFFFF` |
| On SMS bubble | `#FFFFFF` |
| On received bubble | `#000000` |
| Generation | `ColorScheme.fromSeed(seedColor: Colors.blue, background: Colors.white, surfaceVariant: #F3F3F6, brightness: Brightness.light)` |

#### Nord Theme (Dark)
| Property | Value |
|----------|-------|
| Brightness | Dark |
| Primary swatch seed | `#5E81AC` (Nord blue) |
| Accent color | `#88C0D0` (Nord frost) |
| Background | `#3B4252` (Nord polar night) |
| Card color | `#4C566A` (Nord polar night lighter) |
| Primary container | `#49688E` |
| Outline | `Colors.grey` |
| Error | `Colors.red` |
| Generation | `ColorScheme.fromSwatch()` with Nord palette |

#### Music Theme (Light)
| Property | Value |
|----------|-------|
| Brightness | Light |
| Base | Initialized from `whiteLightTheme` |
| Behavior | Dynamic -- colors update from album art via `ColorScheme.fromImageProvider()` |

#### Music Theme (Dark)
| Property | Value |
|----------|-------|
| Brightness | Dark |
| Base | Initialized from `oledDarkTheme` |
| Behavior | Dynamic -- colors update from album art via `ColorScheme.fromImageProvider()` |

### FlexScheme Themes

Every `FlexScheme` enum value generates both a light and dark variant using:
- Light: `FlexThemeData.light(scheme: e, surfaceMode: FlexSurfaceMode.highSurfaceLowScaffold, blendLevel: 40)`
- Dark: `FlexThemeData.dark(scheme: e, surfaceMode: FlexSurfaceMode.highSurfaceLowScaffold, blendLevel: 40)`

Surface mode `highSurfaceLowScaffold` creates stronger surface color blending while keeping the scaffold closer to neutral. Blend level 40 provides moderate blending.

Available FlexScheme names (50+):
Material, Material HC, Blue, Indigo, Hippie Blue, Aqua Blue, Brandeis Blue, Deep Blue, Sakura, Mandarin Red, Red, Red & Blue, Pink, Rosewood, Money, Jungle, Grey Law, Wasabi, Gold, Mango, Amber, Vesuvius Burn, Deep Purple, Ebony Clay, Barossa, Shark, Big Stone, Damask, Bahama & Trinidad, Mallard & Valencia, Espresso & Crema, Outer Space, Blue Whale, San Juan Blue, Baross, Lipstick, Flutter Dash, M3 Baseline, Verdun Hemlock, Dell & Genoa Green, Red M3, Pink M3, Purple M3, Indigo M3, Blue M3, Cyan M3, Teal M3, Green M3, Lime M3, Yellow M3, Orange M3, Deep Orange M3

Theme name formatting: Split on capital letters and capitalize (e.g., `hippieBlue` becomes `"hippie Blue"`).

---

## 10. Dark / Light Mode Tokens

### Theme Mode Options

| Mode | Behavior |
|------|----------|
| Light | Always uses selected light theme |
| Dark | Always uses selected dark theme |
| System | Follows `PlatformDispatcher.instance.platformBrightness` |

### Theme Selection Persistence

| Key | Default |
|-----|---------|
| `"selected-light"` | `"Bright White"` |
| `"selected-dark"` | `"OLED Dark"` |
| `"previous-light"` | Used for reverting after Music Theme |
| `"previous-dark"` | Used for reverting after Music Theme |

### OLED Black Mode
- Background color: `#000000` (pure black)
- Achieved via `background: Colors.black` in `ColorScheme.fromSeed()`
- Enables true black pixels on OLED displays for power savings

### Color Mapping Inversion (Settings Pages)

In Material skin dark mode, header and tile background colors swap:

| Condition | Header Color | Tile Color |
|-----------|-------------|------------|
| Material dark mode | darker `background` | lighter `properSurface` |
| All other combinations | lighter `properSurface` | darker `background` |

---

## 11. Material You (Monet) Integration

### Monet Modes

| Mode | Behavior |
|------|----------|
| Disabled | Theme colors used as-is |
| Harmonize | System palette harmonizes with selected theme colors |
| Full | System palette fully replaces theme colors |

### Monet Palette Structure (5 palettes, 13 shades each)

| Palette | Chroma (Oklch) | Description |
|---------|----------------|-------------|
| `neutral1` | 0.0132 | Main background, tinted with primary |
| `neutral2` | 0.0066 | Secondary background, slightly tinted |
| `accent1` | 0.1212 | Main accent, close to primary |
| `accent2` | 0.04 | Secondary accent |
| `accent3` | 0.06 | Tertiary accent, hue-shifted +60 degrees |

### Shade Map (CIELAB L* targets)

| Shade | Oklch Lightness | CIELAB L* |
|-------|----------------|-----------|
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

### Windows Accent Color
On Windows, the system accent color is retrieved and applied similarly to Monet, allowing the app to match the system theme.

---

## 12. Window Effect Tokens (Desktop Only)

| Effect | Min Windows Build | Max Windows Build | Dark Opacity | Light Opacity |
|--------|------------------|------------------|-------------|--------------|
| Tabbed | 22523 | -- | 0.0 | 0.0 |
| Mica | 22000 | -- | 0.0 | 0.0 |
| Aero | 0 | 22523 | 0.6 | 0.75 |
| Acrylic | 17134 | -- | 0.0 | 0.6 |
| Transparent | 0 | -- | 0.7 | 0.7 |
| Disabled | 0 | -- | 1.0 | 1.0 |

### Effect Dependencies
- **Brightness-dependent** (auto-adapt): Tabbed, Mica
- **Color-dependent** (use background color): Aero, Acrylic, Transparent

### Custom Opacity Settings
- `windowEffectCustomOpacityDark` -- User-configurable dark mode opacity
- `windowEffectCustomOpacityLight` -- User-configurable light mode opacity

---

## 13. Color Helper Utilities

| Method | Description |
|--------|-------------|
| `darkenPercent(percent)` | Multiplies RGB by `(1 - percent/100)` |
| `lightenPercent(percent)` | Interpolates RGB toward 255 |
| `lightenOrDarken(percent)` | Darkens if close to black (distance <= 50), else lightens |
| `oppositeLightenOrDarken(percent)` | Inverse of lightenOrDarken |
| `themeLightenOrDarken(context, percent)` | Darkens in light mode, lightens in dark mode |
| `themeOpacity(context)` | Full opacity when effects disabled, custom opacity when enabled |
| `darkenAmount(amount)` | Adjusts HSL lightness down (0.0-1.0) |
| `lightenAmount(amount)` | Adjusts HSL lightness up (0.0-1.0) |
| `computeDifference(other)` | Euclidean RGB distance as percentage (0-100) |

### HexColor Parsing
- 6-char: Prepends `FF` for full opacity (`"1982FC"` -> `0xFF1982FC`)
- 7-char with `#`: Strips prefix (`"#323332"` -> `0xFF323332`)
- 8-char: Direct ARGB (`"FF43CC47"` -> `0xFF43CC47`)
