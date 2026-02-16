# 08 - Animations and Effects

> Complete specification for all animations, transitions, screen effects, message effects, gradient backgrounds, loading animations, hover states, and window effects.

---

## 1. Standard Animation Timing Reference

| Context | Duration | Easing Curve |
|---------|----------|-------------|
| Send animation (message fly-up) | 450ms | Custom tweens |
| Gentle bubble effect | 1800ms total | easeInOut |
| Gradient background cycle | 3000ms (3s) | fastOutSlowIn |
| Setup page transition (forward) | 500ms | easeInOut |
| Setup page transition (back) | 300ms | easeIn |
| List item size animation | 150ms | default (linear) |
| Chip/tag animation | 250ms | easeIn |
| Search debounce | 250ms | -- |
| Fullscreen media swipe | 300ms | easeIn |
| Cupertino dialog inset | 100ms | decelerate |
| Splash screen delay | 100ms | -- |
| Fullscreen video controls auto-hide | 100ms | default |
| New message insertion | 500ms | SizeTransition + SlideTransition |
| Circle progress bar | Dynamic | easeInOut |
| Typing indicator debounce | 3000ms | -- |

---

## 2. Screen Transitions

### Page Route Transitions by Skin

#### iOS Skin: CustomCupertinoPageTransition
- **Forward:** Slide from right edge with parallax effect on the existing page
  - New page slides in from right (0% to 100% of width)
  - Existing page shifts left slightly (parallax, ~30% of width)
  - Forward curve: `Curves.linearToEaseOut`
  - Reverse curve: `Curves.easeInToLinear`
- **Back:** Reverse of forward animation
  - Current page slides out to right
  - Previous page slides back in from left parallax position

#### Material Skin: MaterialPageRoute
- Standard Material Design page transition
- **Forward:** Slide up from bottom + fade in
- **Back:** Slide down + fade out

#### Samsung Skin: MaterialPageRoute
- Same as Material skin transition

### Setup Wizard Transitions
- Forward: 500ms, `Curves.easeInOut`
- Back: 300ms, `Curves.easeIn`
- Mechanism: `PageView` with programmatic `animateToPage`

### Zero-Duration Transitions
- ChatCreator to ConversationView: Uses `customRoute` with `Duration.zero` for instant switch
- Used when creating a new chat (no visual transition needed)

---

## 3. Message Send Animation

**File:** `widgets/message/send_animation.dart`

### Behavior
- Duration: 450ms
- Animates the message widget from the text field position to its final location in the message list
- Visual: Message appears to "fly up" from the input area
- Text field height is measured dynamically for accurate start position

### Implementation Notes
- Start position: Calculated from text field component bounds
- End position: Message's final slot in the reversed scroll list
- Opacity: Fades in during first portion of animation
- Scale: May include slight scale-up effect

---

## 4. iMessage Screen Effects

**File:** `layouts/conversation_view/widgets/effects/screen_effects_widget.dart`

All screen effects are rendered using custom `Canvas` painting on a fullscreen overlay widget.

### Balloon Effect
| Property | Value |
|----------|-------|
| Controller | `BalloonController` |
| Renderer | `BalloonRendering` |
| Auto-launch delay | 100ms |
| Description | Rising balloon particles that float upward |
| Canvas | Custom painted balloons with physics |

### Fireworks Effect
| Property | Value |
|----------|-------|
| Controller | `FireworkController` |
| Renderer | `FireworkRendering` |
| Auto-launch delay | 100ms |
| Description | Exploding firework particles with trails |
| Canvas | Particle system with explosion physics |

### Love Effect
| Property | Value |
|----------|-------|
| Controller | `LoveController` |
| Renderer | `LoveRendering` |
| Auto-launch delay | 100ms |
| Description | Floating heart animations rising from bottom |
| Canvas | Heart shapes with gentle drift |

### Laser Effect
| Property | Value |
|----------|-------|
| Controller | `LaserController` |
| Renderer | `LaserRendering` |
| Auto-launch delay | 500ms (longer than others) |
| Description | Laser beam effects across screen |
| Canvas | Animated beam lines |

### Spotlight Effect
| Property | Value |
|----------|-------|
| Controller | `SpotlightController` |
| Renderer | `SpotlightRendering` |
| Auto-launch delay | 100ms |
| Description | Spotlight/focus illumination effect |
| Canvas | Radial gradient spotlight overlay |

### Celebration Effect
| Property | Value |
|----------|-------|
| Controller | `CelebrationController` |
| Renderer | `CelebrationRendering` |
| Auto-launch delay | 100ms |
| Description | Confetti-style celebration particles |
| Canvas | Mixed confetti particles falling/floating |

### Confetti Effect
| Property | Value |
|----------|-------|
| Controller | `ConfettiController` |
| Renderer | Uses `confetti` package |
| Auto-launch delay | 100ms |
| Description | Standard confetti burst |

### Send Effect Picker (`SendEffectPicker`)
- UI for selecting effect before sending
- Triggered by long-press on send button
- Grid layout of effect options
- Each option shows a mini preview/icon
- Selecting an effect attaches it to the outgoing message

### Auto-Effect Detection
| Trigger Phrase | Applied Effect |
|---------------|---------------|
| "Congratulations" | Confetti/Celebration |
| "Happy Birthday" | Balloons |
| "Happy New Year" | Fireworks |
| "Pew Pew" | Lasers |

---

## 5. Bubble-Level Send Effects

**File:** `widgets/message/misc/bubble_effects.dart`

### Slam Effect
- Message appears with a "slam down" impact
- Scale + position tween creating a drop-and-bounce feel

### Loud Effect
- Message grows larger and shakes
- Scale oscillation animation
- Conveys emphasis/volume

### Gentle Effect
Three-phase `MovieTween`:

| Phase | Time Range | Scale | Curve |
|-------|-----------|-------|-------|
| Phase 1 (hold) | 0 - 1ms | 1.0 -> 1.0 | -- |
| Phase 2 (shrink) | 1 - 500ms | 1.0 -> 0.5 | easeInOut |
| Phase 3 (grow) | 1000 - 1800ms | 0.5 -> 1.0 | easeInOut |

Total duration: 1800ms

### Invisible Ink Effect
- Message text obscured by animated noise/particle overlay
- Tap to reveal: Noise clears to show message
- Re-obscures after a delay or on next view

---

## 6. Gradient Background Animation

**File:** `wrappers/gradient_background_wrapper.dart`

### Configuration
| Property | Value |
|----------|-------|
| Direction | topRight to bottomLeft |
| Color 1 | Bubble color (iMessage or SMS) at 50% opacity |
| Color 2 | Background color at 100% opacity |
| Duration | 3000ms (3 seconds) per cycle |
| Curve | `Curves.fastOutSlowIn` |
| Loop | Infinite via MirrorAnimationBuilder |

### Animation Details
```
MovieTween
  scene(0s - 3s): "color1" stop: 0.0 -> 0.2
  scene(0s - 3s): "color2" stop: 0.8 -> 1.0
```

The gradient stops oscillate between `[0.0, 0.8]` and `[0.2, 1.0]`, creating a subtle breathing/pulsing effect. The `MirrorAnimationBuilder` reverses at each endpoint for seamless infinite looping.

### Activation
- Only active when current theme has `gradientBg: true`
- Checked via `ts.isGradientBg(context)`
- Responds to platform brightness changes

---

## 7. Loading Animations

### Typing Indicator
- Three dots with staggered bounce animation
- Each dot bounces with slight delay offset
- Contained in bubble shape via `TypingClipper`
- Appears at bottom of message list

### Splash Screen
- Animated BlueBubbles icon
- 100ms initial delay
- Transitions to either Setup Wizard or Conversation List

### Circle Progress Bar
**File:** `components/circle_progress_bar.dart`
- Circular arc that fills clockwise
- Animation curve: `Curves.easeInOut`
- Dynamic duration (matches data load progress)
- Supports determinate (percentage) and indeterminate modes

### List Loading
- New messages appear with 500ms SizeTransition + SlideTransition
- Smooth insertion into the animated list via `SliverAnimatedList`

### Message Pagination Loading
- Spinner shown at the top of the message list while fetching older messages
- Disappears when `fetching` flag is cleared

---

## 8. Hover States (Desktop)

### Conversation Tile Hover
- `hoverHighlight: RxBool` triggers subtle background color change
- Uses `MouseRegion` for hover detection
- Instant transition (no animation delay)

### Window Buttons Hover
| Button | Normal | Hover |
|--------|--------|-------|
| Minimize | Default icon color | Subtle background highlight |
| Maximize | Default icon color | Subtle background highlight |
| Close | Default icon color | `errorContainer` background, `onErrorContainer` icon |

### TabletModeWrapper Divider Hover
- `MouseRegion` with `SystemMouseCursors.resizeLeftRight`
- Visual: Three dots on divider become more prominent on hover

### General Interactive Elements
- Buttons: Material InkSparkle ripple on Material/Samsung skins
- Links: Underline on hover (desktop)
- List items: Background tint change

---

## 9. Window Effects (Desktop Only)

**File:** `lib/utils/window_effects.dart`

### Available Effects

#### Tabbed (Windows 11 Build 22523+)
- Mica-like material incorporating theme and desktop wallpaper
- Sensitive to wallpaper color changes
- Dependencies: brightness only (auto-adapts to light/dark)
- Opacity: 0.0 dark / 0.0 light (fully transparent to effect)

#### Mica (Windows 11 Build 22000+)
- Opaque dynamic material with theme and wallpaper integration
- More subtle than Tabbed
- Dependencies: brightness only
- Opacity: 0.0 dark / 0.0 light

#### Aero (Windows Builds 0 - 22523)
- Windows Vista/7-style glossy blur effect
- Dependencies: background color
- Opacity: 0.6 dark / 0.75 light

#### Acrylic (Windows 10 Build 17134+)
- Translucent texture for depth and visual hierarchy
- Dependencies: background color
- Opacity: 0.0 dark / 0.6 light
- On Windows 11 (build >= 22000): Fully transparent backgrounds supported
- On older Windows 10: Minimum alpha of 1/255 to prevent rendering artifacts

#### Transparent
- Simple transparent window background
- Dependencies: background color
- Opacity: 0.7 dark / 0.7 light
- Available on all Windows builds

#### Disabled
- Standard opaque window background
- No dependencies
- Opacity: 1.0 dark / 1.0 light

### UI Alpha Adjustments When Effects Active
| Element | Alpha (effects on) | Alpha (effects off) |
|---------|--------------------|---------------------|
| Header backgrounds | 20/255 | 255/255 |
| Tile backgrounds | 100/255 | 255/255 |

### Custom Opacity
- Users can override default opacity per mode:
  - `windowEffectCustomOpacityDark` (0.0 - 1.0)
  - `windowEffectCustomOpacityLight` (0.0 - 1.0)

### Windows Version Detection
Parses `Platform.operatingSystemVersion` to extract build number for feature gating.

---

## 10. Material Splash Effect

All themes use `InkSparkle.splashFactory` as their splash factory:
- Material 3 sparkle ripple effect
- Replaces default ink splash or ink ripple
- Active on Material and Samsung skins
- Not used on iOS skin (iOS uses no splash/ripple)

---

## 11. Scroll Animations

### iOS Skin: Bouncing Scroll
- `CustomBouncingScrollPhysics`
- Always scrollable (even when content fits)
- Elastic overscroll at both ends
- Natural iOS feel

### Material/Samsung: Clamping Scroll
- `ClampingScrollPhysics`
- Standard Material overscroll glow effect
- Stops at content bounds

### Scroll-to-Bottom FAB
- `AnimatedOpacity` transition
- Appears when user scrolls up from the bottom
- Hides when at the bottom of the message list

### Material FAB Collapse
- FAB text fades out on scroll down (chat list)
- FAB text fades in on scroll up
- Tracks `materialScrollStartPosition` for direction detection

---

## 12. Swipe Animations

### iOS Timestamp Reveal
- Horizontal drag gesture on message
- Slides messages to reveal per-message timestamps
- Spring-back animation when released

### Swipe-to-Reply
- Horizontal swipe on message bubble
- `SlideToReply` arrow indicator animates in during swipe
- Threshold: Complete swipe triggers reply mode
- Incomplete swipe: Spring-back animation

### iOS Conversation List Swipe Actions
- `Dismissible` widget per tile
- Swipe reveals action buttons beneath
- Full swipe executes primary action (archive/delete)
- Partial swipe reveals all action buttons

---

## 13. Samsung Header Animation

- Expanding/collapsing large title header
- Scroll-driven animation
- Snap points: Fully expanded or fully collapsed
- Snap threshold: `screenHeight / 3 - 57`
- Uses `SliverAppBar` with custom `flexibleSpace`
- Title text scales during transition
