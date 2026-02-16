# BlueBubbles App - Documentation Index

> Comprehensive documentation for rebuilding the BlueBubbles Flutter/Dart cross-platform iMessage client from scratch. Optimized for LLM ingestion and reference.

## Quick Reference

- **Framework:** Flutter (Dart 3.1.3+)
- **Platforms:** Android, iOS, Windows, macOS, Linux, Web
- **Architecture:** Layered (UI > Services > Database > Platform)
- **State Management:** GetX (reactive singletons)
- **Database:** ObjectBox (local), Firebase (remote sync)
- **Networking:** Dio (HTTP), Socket.IO (real-time)
- **Source Code:** `bluebubbles-app-ELECTRON/`

---

## Documentation Map

### Core Architecture
| # | Document | Description | Key Topics |
|---|----------|-------------|------------|
| 01 | [Architecture Overview](01-architecture-overview.md) | System design, tech stack, project structure | Layered architecture, dependency map (100+ packages), state management, build system, entry points, init sequence, mermaid diagrams |
| 02 | [Services & Business Logic](02-services-and-business-logic.md) | All services, handlers, and business logic | 25+ service singletons, action handler, event dispatcher, sync system, queue system, chat lifecycle, notification services |

### Data Layer
| # | Document | Description | Key Topics |
|---|----------|-------------|------------|
| 03 | [Database & Models](03-database-and-models.md) | ObjectBox schema, all entity/model definitions | 9 ObjectBox entities, 17 global models, ER diagrams, 120+ settings keys, theme model, platform-specific models, query patterns |
| 04 | [API & Networking](04-api-and-networking.md) | REST endpoints, Socket.IO protocol, auth | 12 REST endpoint categories, 11 socket events, auth flow, SSL handling, file transfer, sync protocol |

### User Interface
| # | Document | Description | Key Topics |
|---|----------|-------------|------------|
| 05 | [UI Layouts & Components](05-ui-layouts-and-components.md) | Every screen, widget, and navigation flow | 60+ screens, 3 skin systems (iOS/Material/Samsung), conversation view, settings panels, setup wizard, responsive design |
| 06 | [Theming & Design Language](06-theming-and-design-language.md) | Colors, typography, animations, styling | 27 color tokens, 3 UI skins, color engine (Oklab/Oklch), 13+ built-in themes, typography scale, animation patterns, window effects |

### Implementation Details
| # | Document | Description | Key Topics |
|---|----------|-------------|------------|
| 07 | [Utilities & Helpers](07-utilities-and-helpers.md) | Crypto, parsers, file utils, extensions | AES-256-CBC crypto, emoji system, logger, color engine, type extensions, share utils, UI helpers |
| 08 | [Platform-Specific Code](08-platform-specific-code.md) | Native code per platform, build configs | Android (40+ Kotlin files, 25+ permissions), iOS, Windows (MSIX/Inno), Linux (Snap), macOS, Web, build instructions |

---

## Reading Order

**For rebuilding from scratch:**
1. `01-architecture-overview.md` - Understand the overall system design
2. `03-database-and-models.md` - Set up the data layer first
3. `04-api-and-networking.md` - Implement server communication
4. `02-services-and-business-logic.md` - Build the business logic layer
5. `06-theming-and-design-language.md` - Establish the design system
6. `05-ui-layouts-and-components.md` - Build the UI
7. `07-utilities-and-helpers.md` - Add utilities as needed
8. `08-platform-specific-code.md` - Platform-specific integration

**For understanding a specific feature:**
- Messaging flow: `04` (API) -> `02` (Services) -> `05` (UI)
- Theming: `06` (Design) -> `03` (Theme models) -> `05` (UI styling)
- Push notifications: `08` (Platform native) -> `02` (Services) -> `04` (FCM endpoints)
- Database: `03` (Models) -> `02` (Query patterns in services)

---

## Architecture at a Glance

```
bluebubbles-app-ELECTRON/
├── lib/
│   ├── main.dart                    # Entry point (main + bubble)
│   ├── app/                         # UI Layer
│   │   ├── layouts/                 # 12 major screen modules
│   │   ├── components/              # Reusable widgets
│   │   ├── wrappers/                # Layout wrappers
│   │   └── animations/              # Animation utilities
│   ├── services/                    # Business Logic Layer
│   │   ├── backend/                 # Action handler
│   │   ├── backend_ui_interop/      # Event dispatcher + intents
│   │   ├── network/                 # HTTP, Socket, Downloads
│   │   └── ui/                      # Attachments, Contacts, Push
│   ├── database/                    # Data Layer
│   │   ├── global/                  # Cross-platform models
│   │   ├── io/                      # Mobile/Desktop ObjectBox entities
│   │   └── html/                    # Web platform stubs
│   ├── helpers/                     # Backend, Network, Type, UI helpers
│   └── utils/                       # Crypto, Logger, Parsers, Color Engine
├── android/                         # Android native (Kotlin)
├── ios/                             # iOS native (Swift)
├── windows/                         # Windows native (C++)
├── linux/                           # Linux native (GTK/C)
├── macos/                           # macOS native (Swift)
└── web/                             # Web (HTML/JS)
```

---

## Key Patterns for LLM Reference

### Service Access Pattern
All services are GetX singletons accessed via short global accessors:
- `ss` - SettingsService
- `http` - HttpService
- `socket` - SocketService
- `cm` - ChatManager
- `cs` - ChatsService
- `fs` - FileSystemService
- `ls` - LifecycleService
- `ns` - NotificationService
- `ts` - ThemesService

### Platform Conditional Import Pattern
```dart
// Shared interface in global/
// Platform implementation in io/ (mobile/desktop) or html/ (web)
export 'package:bluebubbles/database/io/chat.dart'
    if (dart.library.html) 'package:bluebubbles/database/html/chat.dart';
```

### Authentication
- HTTP: `guid` query parameter on every request (server password hash)
- Socket: `guid` in auth payload on connection
- Firebase: OAuth per platform for web/desktop real-time sync fallback

### Message Flow
```
Incoming: Server -> Socket.IO/FCM -> ActionHandler -> ObjectBox -> EventDispatcher -> UI
Outgoing: UI -> QueueService -> HTTP POST -> Server -> Socket ACK -> DB Update -> UI
```
