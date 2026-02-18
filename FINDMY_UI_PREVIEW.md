# Find My UI Integration - Visual Preview

## Chat Header with Location

```
┌─────────────────────────────────────────────────┐
│  [Avatar]  John Smith              ▼    [Video] │
│            iMessage                              │
│            New Orleans, LA         ← NEW!        │
├─────────────────────────────────────────────────┤
│                                                  │
│  Message bubbles...                              │
```

## Contact Sidebar - Info Tab

### With Location Available

```
┌────────────────────────────────────┐
│  [✕]                          Edit  │
├────────────────────────────────────┤
│            [Avatar]                │
│          John Smith                │
│                                    │
│  [📞] [📹] [✉️] [💬]              │
├────────────────────────────────────┤
│  [ Info | Backgrounds | Photos ]  │
├────────────────────────────────────┤
│  ┌──────────────────────────────┐ │
│  │ ● Live                    ▶  │ │ ← Location Card
│  │ 123 Main St, New Orleans, LA │ │
│  │ Updated 2m ago               │ │
│  │                              │ │
│  │  ┌────────────────────────┐ │ │
│  │  │    [Map Preview]       │ │ │ ← Leaflet Map
│  │  │       📍               │ │ │   with avatar
│  │  │                        │ │ │   marker
│  │  └────────────────────────┘ │ │
│  └──────────────────────────────┘ │
│                                    │
│  ┌──────────────────────────────┐ │
│  │ Contact Details              │ │
│  │ 📱 +1 (555) 123-4567        │ │
│  └──────────────────────────────┘ │
└────────────────────────────────────┘
```

### Without Location

```
┌────────────────────────────────────┐
│  [✕]                          Edit  │
├────────────────────────────────────┤
│            [Avatar]                │
│          John Smith                │
│                                    │
│  [📞] [📹] [✉️] [💬]              │
├────────────────────────────────────┤
│  [ Info | Backgrounds | Photos ]  │
├────────────────────────────────────┤
│  ┌──────────────────────────────┐ │
│  │ Location                     │ │
│  │ Location sharing not         │ │
│  │ available                    │ │
│  └──────────────────────────────┘ │
│                                    │
│  ┌──────────────────────────────┐ │
│  │ Contact Details              │ │
│  │ 📱 +1 (555) 123-4567        │ │
│  └──────────────────────────────┘ │
└────────────────────────────────────┘
```

## Full-Screen Map Popover

Click the location card to open:

```
╔═══════════════════════════════════════════════════════════╗
║                                                  [✕]       ║
║  Location                                                  ║
╠═══════════════════════════════════════════════════════════╣
║  [Avatar] John Smith                                       ║
║           ● Live                                          ║
║           123 Main St, New Orleans, LA 70112              ║
║           Updated 2m ago                                  ║
╠═══════════════════════════════════════════════════════════╣
║                                                            ║
║         ┌──────────────────────────────────┐              ║
║         │                                  │              ║
║         │     Interactive Leaflet Map      │              ║
║         │                                  │              ║
║         │            📍 Avatar             │              ║
║         │                                  │              ║
║         │    [+] [-] (Zoom controls)       │              ║
║         │                                  │              ║
║         └──────────────────────────────────┘              ║
║                                                            ║
╠═══════════════════════════════════════════════════════════╣
║  ┌─────────────────────────────────────────────────────┐  ║
║  │   📍 Open in Find My                                │  ║
║  └─────────────────────────────────────────────────────┘  ║
╚═══════════════════════════════════════════════════════════╝
```

## Status Indicators

### Live Status (Green Dot)
```
● Live
```
- Contact is actively sharing real-time location
- Updates frequently
- High accuracy

### Last Known Location (Gray Dot)
```
○ Last known location
```
- Contact's location is not currently updating
- Shows last recorded position
- May be outdated

## Map Markers

### Avatar Marker Design
```
   ╭───╮
   │ J │  ← First letter of name
   ╰───╯
     ▼
```

Styled with:
- Circular shape
- White border (3-4px)
- Shadow for depth
- Gradient background
- First letter of contact's name

## Theme Support

### Light Theme
- OpenStreetMap tiles (standard)
- Light gray background
- Dark text
- Colored markers

### Dark Themes
```
oled-dark
blue-dark
indigo-dark
nord
green-dark
purple-dark
```

Uses:
- CartoDB Dark tiles
- Dark background
- Light text
- High contrast markers

## Interaction Flow

1. **View Contact Chat**
   ```
   Chat opens → Location text appears in header (if available)
   ```

2. **Open Contact Details**
   ```
   Click header → Sidebar slides in → Map card visible in Info tab
   ```

3. **Expand Map**
   ```
   Click map card → Popover fades in → Full interactive map
   ```

4. **Open in Find My**
   ```
   Click button → System Find My app launches → Closes popover
   ```

5. **Close Popover**
   ```
   Click [X] or outside → Popover fades out → Back to sidebar
   ```

## Responsive Design

### Map Card in Sidebar
- Width: 100% of sidebar (320px)
- Height: 120px preview + 80px info
- Fits perfectly in scroll area

### Full Popover
- Width: 90% of screen (max 800px)
- Height: 85% of screen (max 700px)
- Centered with overlay backdrop
- Scrollable content if needed

## Animation Details

### Sidebar Entry
- Slides in from right
- Duration: 300ms
- Spring animation (damping: 28, stiffness: 300)

### Popover Entry
- Fades in overlay (opacity 0 → 1)
- Scales up content (0.9 → 1.0)
- Duration: 250ms
- Spring animation (damping: 25, stiffness: 300)

### Status Dot Pulse (Live mode)
Could be enhanced with:
```css
animation: pulse 2s ease-in-out infinite;
```

## Accessibility

- ARIA labels on all buttons
- Keyboard navigation support
- Screen reader friendly
- High contrast status indicators
- Descriptive alt text
- Focus management

## Color Palette

### Status Colors
- Live: `#34C759` (green)
- Offline: `var(--color-outline)` (gray)
- Error: `#FF453A` (red)

### UI Colors
- Primary: `var(--color-primary)` (app theme)
- Surface: `var(--color-surface)`
- Surface Variant: `var(--color-surface-variant)`
- On Surface: `var(--color-on-surface)`
- On Surface Variant: `var(--color-on-surface-variant)`

## Text Hierarchy

### Chat Header Location
- Font size: 10px
- Weight: 400
- Color: On Surface Variant
- Opacity: 0.8

### Map Card Header
- Font size: 13px (title)
- Font size: 11px (address)
- Font size: 10px (timestamp)
- Weight: 500 (title), 400 (body)

### Popover Header
- Font size: 18px (title)
- Font size: 15px (name)
- Font size: 13px (status)
- Weight: 600 (titles), 400 (body)
