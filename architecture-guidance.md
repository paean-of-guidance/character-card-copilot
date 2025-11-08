# Character Card Copilot – Architecture Guidance

> **Living Document**: This document provides a comprehensive overview of the application's architecture, data flows, and development guidelines. It is maintained to help developers understand the codebase structure and make informed technical decisions.
>
> **Last Updated**: 2025-11-08

## Table of Contents
1. [System Overview](#system-overview)
2. [Frontend Architecture & Data Flows](#frontend-architecture--data-flows)
3. [Backend Architecture & Data Flows](#backend-architecture--data-flows)
4. [Event Bridge Highlights](#event-bridge-highlights)
5. [Command Mapping](#frontend--backend-command-mapping)
6. [Data Flow Patterns](#data-flow-patterns)
7. [Partially Implemented Areas](#partially-implemented--in-progress-areas)
8. [Code Health & Maintenance](#code-health--maintenance-status)
9. [Improvement Roadmap](#improvement-roadmap)
10. [Architecture Strengths](#architecture-strengths)
11. [Development Guidelines](#development-guidelines)

---

## System Overview

Character Card Copilot is a **desktop application** for AI-assisted character card editing, built with:
- **Frontend**: Vue 3 + TypeScript + TailwindCSS 4.0 + Pinia + vue-router
- **Backend**: Rust + Tauri
- **Communication**: Tauri event system + invoke calls
- **Design Pattern**: Event-driven architecture with centralized state management

### Key Design Principles
1. **Event-driven**: All state changes flow through the event system
2. **Service layer mapping**: One-to-one mapping between services and Tauri commands
3. **Session-based**: Character sessions provide context isolation
4. **Tool-extensible**: AI tools follow OpenAI function calling schema
5. **Token-aware**: Smart token budget management across all AI operations

---

## Frontend Architecture & Data Flows
### Bootstrapping & Layout
- `src/main.ts:1` creates the Vue app, mounts the router/Pinia stack and pulls in global styles, so every view/component shares the same stores and services for backend communication.
- `src/App.vue:1-76` provides the global toast + modal services via Vue `provide`, meaning any descendant (e.g., Editor and AIPanel) can surface backend results/errors without duplicating transport logic.

### Primary Views
- `src/views/Home.vue:51` calls `getAllCharacters()` from `src/services/characterStorage.ts:23` on mount to pull the complete character list via Tauri, populates the card grid, and wires click handlers to push the editor route with the chosen UUID. `handleNewCharacter()` at `src/views/Home.vue:71` issues `createCharacter()` and navigates to the new editor view once the backend returns.
- `src/views/Editor.vue:288-310` loads the target character record with `getCharacterByUUID` and immediately invokes `load_character_session` (`invoke('load_character_session', { uuid })` at line 296) so that the Rust `CharacterSession` state is hydrated before the AI panel starts listening. On first load the editor mirrors the Tavern card fields into local state (`characterData.value = …`), and whenever `route.params.uuid` changes the same pipeline reruns.
- `src/views/Editor.vue:84-139` registers Tauri event listeners (`character-loaded`, `character-updated`, `session-unloaded`, `error`) so the form can react to backend-side mutations. Each listener double-checks the payload UUID before patching local form state and raising notifications, so edits that originate in the AI tools or commands remain in sync.
- `src/views/Editor.vue:400` persists the active character in `CharacterStateService.setActiveCharacter`, ensuring the backend command/AI subsystems know which session context they should operate on. On unmount `CharacterStateService.clearActiveCharacter()` (`src/views/Editor.vue:589`) tears the association down.
- `src/views/Settings.vue:35-186` orchestrates API profile editing. `updateApiList()` (`line 35`) fetches `get_all_api_configs`, while `autoSave()` (`line 60`) immediately forwards every change to the matching `update_api_config` command. Actions such as "Set default" (`line 106`) and "Test connection" (`line 166`) call the dedicated service helpers, and successful operations refresh the list via another `updateApiList()` call.

### Key Components
- **AIPanel** (`src/components/AIPanel.vue`):
  - Mount lifecycle (`initializeBackendEventListeners()` at line 334) subscribes to eleven backend events (`character-loaded`, `chat-history-loaded`, `message-sent`, `message-received`, `context-built`, `character-updated`, `tool-executed`, `session-unloaded`, `error`, `token-stats`, `progress`—lines 338‑476) so UI state, token counters, and progress bars react in real time. Payloads are mirrored into the Pinia chat store (`chatStore.setChatHistory` and `setActiveCharacter` at lines 363‑364).
  - AIPanel directly invokes `load_chat_history` command when initializing sessions (line 296), then exclusively relies on backend events (`message-sent`, `message-received`, `chat-history-loaded`) and the Pinia chat store for all subsequent state management.
  - Message submission funnels through `sendMessage()` (`line 226`) → `sendMessageViaBackend()` (`line 522`). That helper lazily reloads the session if it has expired (`load_character_session` at line 545), then uses `invoke('send_chat_message', { message })` (`lines 549 & 563`), letting the Rust `CharacterSession` handle context building, API calls, and event emission. Delete/edit/regenerate actions call the matching commands (`edit_chat_message` at line 642, `delete_chat_message` at line 682, `regenerate_last_message` at line 709).
  - The built-in command palette (lines 734‑833) sources its catalog via `backendCommandService.getCommands()`/`searchCommands()` (lines 736‑750) and executes selections through `backendCommandService.executeCommand()` (line 823). Confirmation prompts leverage the global modal service, and TODOs at lines 835/839 remind us that toast notifications still need to wrap command results.
- **WorldBookEditor** (`src/components/WorldBookEditor.vue:94-170`) loads character books through the Pinia world-book store and registers `world-book-entry-created` and `tool-executed` listeners (`lines 104-134`). Whenever the backend AI tools emit one of those events, the editor automatically reloads entries so the UI reflects AI-authored additions/removals without manual refreshes.

### Stores & Client Services
- `src/stores/chat.ts:5-78` holds chat histories keyed by character UUID, tracks whether the backend session is active, and exposes helpers that the AI panel calls whenever events come in. Because the store mirrors all message arrays the application survives route changes without re-querying the backend immediately.
- `src/stores/worldBook.ts:19-197` wraps CRUD operations in computed views (filtered lists, selection, statistics). Every mutation reuses the functions in `src/services/worldBookService.ts:103-236`, which in turn read/patch the local Tavern card and call `update_character` so persistence happens in Rust.
- The service layer is deliberately thin and maps one-to-one onto Tauri commands:
  - `src/services/characterStorage.ts:23-178` wraps `get_all_characters`, `get_character_by_uuid`, `create_character`, `update_character`, `delete_character`, `upload_background_image`, `export/import_character_card`, etc., all of which are implemented by `CharacterStorage` inside `src-tauri/src/character_storage.rs:172-429`.
  - `src/services/apiConfig.ts:13-142` mirrors API profile CRUD + testing; each helper invokes the commands defined in `src-tauri/src/lib.rs:95-147` backed by `ApiConfigService` (`src-tauri/src/api_config.rs:58-320`).
  - `src/services/aiConfig.ts:13-58` surfaces AI role CRUD against the commands in `src-tauri/src/lib.rs:152-189`/`src-tauri/src/ai_config.rs:75-172`.
  - `src/services/aiTools.ts:66-137` exposes the tool registry endpoints (`get_available_tools`, `get_tools_by_category`, `execute_tool_call`, `get_tool_categories`) that ultimately reach `ToolRegistry` in `src-tauri/src/tools/registry.rs:1-83`.
  - `src/services/backendCommandService.ts:8-85` is the only command gateway currently used by the UI; it memoizes `/command_system/tauri_commands` results for responsive palette searches.

## Backend Architecture & Data Flows
### Character & Storage Layer
- `src-tauri/src/character_storage.rs:172-429` owns the Tavern card lifecycle. It stores each character under `AppData/character-cards/<uuid>/card.json`, converts background images to base64 (so the frontend can render them without separate file reads), handles import/export (JSON or PNG via `png_utils.rs`), and responds to all character CRUD commands registered in `src-tauri/src/lib.rs:15-120`.
- `src-tauri/src/character_state.rs:5-56` keeps track of the “current” character UUID inside a global `CharacterStateManager`. The `set_active_character/get_active_character/clear_active_character/has_active_character` commands are invoked from the Editor view so that downstream modules (sessions, commands, tools) always know which conversation they should apply a side effect to.
- `src-tauri/src/chat_history.rs:1-159` persists every message as JSONL per character. Helpers like `save_message`, `load_history`, `clear_history`, `delete_message`, `update_message`, `get_last_message`, and `get_recent_messages` back the commands wired into `lib.rs:206-292`. This file-based log is what session restarts hydrate from.

### Session & AI Pipeline
- `src-tauri/src/character_session.rs:1-1056` is the heart of the runtime. A process-wide `SESSION_MANAGER` (`lazy_static` at line 457) stores up to 10 live `CharacterSession` structs, each carrying the loaded Tavern card, chat history, configurable token budget, and timestamps. The exported commands (`load_character_session` at line 475, `send_chat_message` at line 497, `unload_character_session` at line 700, `get_session_info` at line 733, `get_all_sessions` at line 743, `save_all_sessions` at line 749, `cleanup_expired_sessions` at line 767, `delete_chat_message` at line 791, `edit_chat_message` at line 814, `regenerate_last_message` at line 839) coordinate everything the AIPanel can trigger.
- Every `send_chat_message` call builds a context via `ContextBuilder::build_full_context` (line 514), emits `context-built` + progress/token events, then constructs a canonical OpenAI request: system prompts, worldbook info, truncated history, and the latest user message (lines 525‑606). It picks an API profile, calls `AIChatService::create_chat_completion` (`src-tauri/src/ai_chat.rs:405-618`), and appends the response to both in-memory history and the on-disk log before firing `message-received`.
- `regenerate_last_message` follows the same path (lines 875‑1006) but reuses the most recent user utterance after deleting the stale assistant reply, which mirrors what the frontend’s “regenerate” button does.
- `src-tauri/src/context_builder.rs:1-320` encapsulates how system/assistant/history slices are constructed and counted. `build_full_context` assembles token allocations per bucket, and helper methods format the Tavern card + world book entries. The exported `build_context` command (line 483) is still a TODO; right now only `CharacterSession` calls the builder directly.
- `src-tauri/src/ai_chat.rs:165-671` manages outbound HTTP calls. `AIChatService::create_chat_completion` (line 405) builds a Reqwest client per API profile, injects headers, and normalizes the OpenAI-compatible response. `create_streaming_chat_completion` (line 619) is a synchronous fallback that converts a full response into fake SSE chunks until a proper streaming pipeline is wired.
- `src-tauri/src/token_counter.rs:1-74` offers precise token calculations backed by `tiktoken-rs`. The commands (`count_tokens`, `count_tokens_batch`, `check_token_limit`, `truncate_to_token_limit`) are exposed via `lib.rs:300-340` so both frontend utilities and backend context builders stay consistent.

### Configuration, Tools & Commands
- `src-tauri/src/api_config.rs:58-320` stores user API profiles inside `AppData/api/apis.json`, allows only one default, and offers `test_api_connection`/`fetch_models` implementations using `reqwest` (lines 231 and 290). Those routines are what the Settings view relies on when users validate credentials.
- `src-tauri/src/ai_config.rs:5-172` manages AI roles in YAML (`ai_config.yml`). Commands exposed at `lib.rs:152-189` let the UI list, mutate, and switch defaults, and the `CharacterSession` module can load whichever role is configured as default when constructing system prompts.
- `src-tauri/src/tools/registry.rs:1-83` plus `character_editor.rs` and `world_book_creator.rs` define the available AI tools. Each tool advertises itself as an OpenAI “function” (see the schema definitions in `src-tauri/src/ai_tools.rs:1-78`) and implements `AIToolTrait::execute` to mutate character cards or character books. Upon success they emit domain-specific events such as `character-updated` or `world-book-entry-created`.
- `src-tauri/src/command_system` introduces an async command bus. `command.rs` defines `CommandMetadata` and `CommandContext`, `registry.rs` stores executors inside a Tokio `RwLock`, and `tauri_commands.rs:19-90` exposes `get_available_commands`, `search_commands`, and `execute_command`. During app setup (`src-tauri/src/lib.rs:214-252`) the builder spawns `initialize_command_system()` which registers `ClearCommand` (`src-tauri/src/command_system/builtin/clear_command.rs:9-79`). The `/clear` executor wipes chat history, updates the session manager, and raises the `chat-history-loaded` event so the UI instantly reflects the mutation.
- `src-tauri/src/events.rs:1-340` defines every payload struct plus `EventEmitter` helpers. All major backend modules use it to push real-time updates (character/session lifecycle, tool runs, token stats, long-running progress, structured errors).

### Tauri Entry Point
- `src-tauri/src/lib.rs:1-341` wires each command into `tauri::Builder::default().invoke_handler(...)`, attaches the `dialog`, `fs`, and `opener` plugins, and spawns the command-system initialization task inside `.setup`. `src-tauri/src/main.rs:1-13` simply delegates to `run()`.

## Event Bridge Highlights
- Backend events originate from `EventEmitter` (e.g., `send_character_loaded`, `send_chat_history_loaded`, `send_message_sent`, `send_message_received`, `send_context_built`, `send_tool_executed`, `send_session_unloaded`, `send_error`, `send_token_stats`, `send_progress` in `src-tauri/src/events.rs:140-339`).
- The `Editor` view listens to `character-loaded`, `character-updated`, `session-unloaded`, and `error` (`src/views/Editor.vue:84-139`) to keep form state consistent.
- `AIPanel` subscribes to the full set of chat + telemetry events (`src/components/AIPanel.vue:338-476`) so the chat transcript, tool responses, token gauges, and progress indicators stay live without polling.
- `WorldBookEditor` listens to `world-book-entry-created` and `tool-executed` (`src/components/WorldBookEditor.vue:104-134`) to refresh the entry list whenever AI tools modify the book.

## Front ↔ Backend Command Mapping
| Frontend hook | Backend command(s) | Backend implementation |
| --- | --- | --- |
| `characterStorage.getAllCharacters()` (`src/services/characterStorage.ts:23`) | `get_all_characters` | `CharacterStorage::get_all_characters` (`src-tauri/src/character_storage.rs:172`) |
| `characterStorage.updateCharacter()` (`src/services/characterStorage.ts:67`) & world book helpers | `update_character` | `CharacterStorage::update_character` (`src-tauri/src/character_storage.rs:262`) |
| `apiConfig` service methods (`src/services/apiConfig.ts:13-142`) | `get_all_api_configs`, `create_api_config`, `update_api_config`, `delete_api_config`, `set_default_api_config`, `toggle_api_config`, `test_api_connection`, `fetch_models` | `ApiConfigService` (`src-tauri/src/api_config.rs:84-320`) |
| `CharacterStateService` (`src/services/characterState.ts:8-30`) | `set_active_character`, `get_active_character`, `clear_active_character`, `has_active_character` | `CharacterStateManager` (`src-tauri/src/character_state.rs:5-56`) |
| AI chat session actions from `AIPanel` (`invoke('load_character_session')`, `send_chat_message`, `delete_chat_message`, etc.) | `load_character_session`, `send_chat_message`, `unload_character_session`, `delete_chat_message`, `edit_chat_message`, `regenerate_last_message` | `SessionManager` & `CharacterSession` (`src-tauri/src/character_session.rs:475-905`) |
| Command palette (`backendCommandService` at `src/services/backendCommandService.ts:8-83`) | `get_available_commands`, `search_commands`, `execute_command` | Command registry (`src-tauri/src/command_system/tauri_commands.rs:19-90`) |
| Tool APIs (`src/services/aiTools.ts:66-115`) | `get_available_tools`, `get_tools_by_category`, `execute_tool_call`, `get_tool_categories` | Tool registry + executors (`src-tauri/src/tools/registry.rs:1-83`, `character_editor.rs`, `world_book_creator.rs`) |
| AIPanel chat operations (`src/components/AIPanel.vue`) | `load_chat_history`, `delete_chat_message` (other operations handled via event system + Pinia Store) | File-backed history (`src-tauri/src/chat_history.rs:1-159` via handlers in `lib.rs:206-292`) |
| Token utilities (`src/utils/tokenCounter.ts:6-57` & any UI invoking backend) | `count_tokens`, `count_tokens_batch`, `check_token_limit`, `truncate_to_token_limit` | `src-tauri/src/token_counter.rs:1-74` |

---

## Data Flow Patterns

### Pattern 1: Character Editing Flow
```
Editor.vue (form input)
  → updateField(field, value)
  → invoke('update_character_field', { uuid, field, value })
  → CharacterSession.update_character_field()
  → emit('character-updated')
  → Editor.vue (listen event)
  → updateEditorFromCharacterData(payload.character_data)
  → sync to form state
```

### Pattern 2: AI Chat Message Flow
```
AIPanel.vue (user types message)
  → sendMessageViaBackend()
  → invoke('send_chat_message', { message })
  → CharacterSession.build_context()
  → emit('context-built')
  → AIChatService.create_chat_completion()
    → OpenAI API call
    → Tool call detection
    → Tool execution (if any)
    → emit('tool-executed')
  → emit('message-received')
  → AIPanel.vue (listen events)
  → groupedMessages computed
  → ToolExecutionCard render (if tool calls)
  → MarkdownRenderer render (if text)
```

### Pattern 3: WorldBook Management Flow
```
WorldBookEditor.vue (create new entry)
  → worldBookStore.createEntry()
  → invoke('create_world_book_entry', { ...params })
  → WorldBookCreator tool execute()
  → emit('world-book-entry-created')
  → WorldBookEditor.vue (listen event)
  → loadWorldBook(uuid)
  → refresh local state
```

### Pattern 4: Command Execution Flow
```
CommandPalette.vue (user selects command)
  → executeCommand(command)
  → backendCommandService.executeCommand(command.id, userInput)
  → invoke('execute_command', { command_id, user_input })
  → CommandRegistry.execute_command()
  → emit('progress')
  → emit('command-complete')
  → AIPanel.vue (listen event)
  → show notification
```

## Partially Implemented / In-Progress Areas
- `context_builder::build_context` (`src-tauri/src/context_builder.rs:483-489`) is a stub that currently returns an error. The AI panel already listens for `context-built`, but only `CharacterSession` emits that event internally; exposing a direct build endpoint will unblock any “preview context” UI.
- UI TODOs: `src/components/ApiList.vue:38` still lacks the final delete-confirm dialog; `src/components/AIPanel.vue:835-839` earmarks notification hooks for command success/error states in the palette modal.
- `AIChatService::create_streaming_chat_completion` (`src-tauri/src/ai_chat.rs:619-663`) synthesizes SSE output by recycling regular completions, so true streaming is not yet available even though the frontend exposes a streaming hook.
- `src/services/aiTools.ts:43` documents that the legacy `AITool` interface is deprecated in favour of the OpenAI `ChatTool` schema but both types coexist, which can confuse contributors until the old type is removed.
- The Settings view’s edit form autosaves on every blur via `autoSave()` (`src/views/Settings.vue:60-142`), but there is no form-level validation or error surfacing beyond console logs; the TODO in `ApiList.vue` demonstrates that UX polish is still underway.

## Code Health & Maintenance Status

### ✅ Completed Cleanup (2025-11-07)
The following legacy code has been successfully removed (1,387 lines):
- ~~`src/services/aiChat.ts`~~ – Removed unused `AIChatService` and `BackendSessionService` classes (312 lines)
- ~~`src/services/commandService.ts`~~ – Removed deprecated frontend command system (250 lines)
- ~~`src/services/builtinCommands.ts`~~ – Removed obsolete built-in commands (112 lines)
- ~~`src/types/command.ts`~~ – Removed deprecated command type definitions (99 lines)
- ~~`src/services/COMMAND_DEVELOPMENT_GUIDE.md`~~ – Removed outdated documentation (483 lines)
- ~~`src/services/chatHistory.ts`~~ – Removed redundant `ChatHistoryManager` class (124 lines) + cleaned up AIPanel.vue imports (7 lines)
  - AIPanel now directly invokes `load_chat_history` command and relies exclusively on backend event system + Pinia Store
  - All 7 methods eliminated: only `loadHistory()` was in use, replaced by direct `invoke()` call
  - Zero breaking changes: functionality preserved through event-driven architecture

**Impact**: All removed code was verified to have zero references in the active codebase. The application now exclusively uses the backend command system (`backendCommandService` + `src-tauri/src/command_system`) and event-driven state management.

---

## Improvement Roadmap

### High Priority

#### 1. Context Preview Feature
- **Issue**: `context_builder::build_context` is a stub that returns an error
- **Goal**: Enable users to inspect the assembled prompt before sending a message
- **Implementation**:
  - Complete `build_context` command in `src-tauri/src/context_builder.rs:483-489`
  - Add UI component to display context breakdown
  - Add token allocation visualization
- **Files**: `src-tauri/src/context_builder.rs`, new component `ContextPreview.vue`

#### 2. True Streaming Support
- **Issue**: `AIChatService::create_streaming_chat_completion` is a placeholder that converts full responses to fake SSE
- **Goal**: Implement real-time streaming for long AI responses
- **Implementation**:
  - Replace placeholder in `src-tauri/src/ai_chat.rs:619-663`
  - Add proper SSE bridge using Reqwest streaming
  - Update AIPanel to handle incremental updates
- **Files**: `src-tauri/src/ai_chat.rs`, `src/components/AIPanel.vue`

### Medium Priority

#### 3. State Management Unification
- **Issue**: Mixed usage of Pinia stores and direct service calls
- **Goal**: Consolidate state management into Pinia for consistency
- **Implementation**:
  - Migrate `backendCommandService` to a Pinia store
  - Migrate `characterStorage` operations to store actions
  - Create composables for common patterns
- **Files**: New store `commandStore.ts`, updated `characterStore.ts`

#### 4. Performance Optimizations
- **Issue**: No debouncing, pagination, or virtual scrolling
- **Goal**: Improve performance for large datasets
- **Implementation**:
  - Add debounce to form updates (Editor.vue)
  - Implement virtual scrolling for WorldBook list
  - Add pagination for chat history
  - Implement smart caching for character data
- **Files**: All form inputs, `WorldBookEditor.vue`, `AIPanel.vue`

#### 5. Type Safety Improvements
- **Issue**: Multiple `any` types and legacy interfaces
- **Goal**: Improve type safety across the codebase
- **Implementation**:
  - Replace all `any` types with specific interfaces
  - Remove legacy `AITool` interface from `src/services/aiTools.ts:43`
  - Add Zod schemas for runtime validation
  - Create centralized type definitions
- **Files**: All `.ts` files, new `types/schema.ts`

### Low Priority

#### 6. UX Enhancements
- **Issue**: Missing loading states, keyboard shortcuts, offline support
- **Goal**: Improve user experience
- **Implementation**:
  - Add loading skeletons for all async operations
  - Implement keyboard shortcuts (Ctrl+S, Ctrl+Enter, etc.)
  - Add offline mode with local caching
  - Add drag-and-drop for world book entries
  - Implement theme system
- **Files**: Global components, new composables

#### 7. Developer Experience
- **Issue**: Limited tooling, logging, and debugging
- **Goal**: Improve developer productivity
- **Implementation**:
  - Add structured logging (Winston/Loguru)
  - Create VS Code debugging configuration
  - Add API documentation generation
  - Create migration scripts for data updates
  - Add performance monitoring
- **Files**: New `src/utils/logger.ts`, `scripts/`, `.vscode/`

#### 8. Extensibility
- **Issue**: Limited plugin system and customization
- **Goal**: Enable third-party extensions
- **Implementation**:
  - Design plugin API for tools
  - Create theme extension points
  - Add custom command registration
  - Implement hot-reload for development
- **Files**: New `src/plugin-system/`, documentation

---

## Architecture Strengths

### ✅ Well-Designed Components

1. **Event-Driven Architecture**
   - Real-time UI updates without polling
   - Loose coupling between components
   - Easy to trace data flow
   - 11 distinct event types for different operations

2. **Session Management**
   - Clean separation of character contexts
   - Automatic cleanup of expired sessions
   - Support for up to 10 concurrent sessions
   - State persistence across route changes

3. **Token Budget Management**
   - Precise token counting with `tiktoken-rs`
   - Smart allocation across system/character/history buckets
   - Automatic truncation when needed
   - Real-time token statistics

4. **Tool System**
   - OpenAI function calling compatibility
   - Extensible tool registry
   - Automatic tool discovery
   - Event-driven tool execution feedback

5. **Command System**
   - Async command bus pattern
   - Searchable command palette
   - Confirmation workflows
   - Progress tracking

### 📊 Code Quality Metrics

- **Lines Removed**: 1,387 (2025-11-07 cleanup)
- **Event Types**: 11 well-defined backend events
- **Command Categories**: 6+ distinct backend subsystems
- **Session Limit**: 10 active sessions (configurable)
- **Tool Categories**: 2+ (character, worldbook)

### 🔄 Data Flow Advantages

1. **Predictable**: All flows follow documented patterns
2. **Debuggable**: Events provide clear audit trail
3. **Scalable**: Session-based isolation enables growth
4. **Maintainable**: One-to-one service command mapping

---

## Development Guidelines

### When Adding New Features

1. **Follow the Event Pattern**
   - Emit events for all state changes
   - Listen to events instead of direct calls
   - Use EventEmitter helpers in `src-tauri/src/events.rs`

2. **Service Layer Rules**
   - Services map 1:1 to Tauri commands
   - No business logic in services
   - Return promises/async results
   - Handle errors consistently

3. **State Management**
   - Use Pinia for shared state
   - Use local `ref` for component-specific state
   - Sync via events, not direct updates
   - Store persists across route changes

4. **Type Safety**
   - Avoid `any` types
   - Define interfaces in `src/types/`
   - Use TypeScript strict mode
   - Add runtime validation (Zod) for external data

5. **Performance**
   - Add debouncing for form inputs (300ms)
   - Implement pagination for lists (>50 items)
   - Use virtual scrolling for large datasets
   - Cache expensive computations

6. **Error Handling**
   - Emit structured error events
   - Show user-friendly notifications
   - Log errors with context
   - Provide retry mechanisms

7. **Testing Strategy**
   - Unit test business logic (Rust)
   - Integration test event flows
   - E2E test critical user journeys
   - Mock external API calls

### Code Review Checklist

- [ ] Follows event-driven pattern
- [ ] Uses existing events or defines new ones
- [ ] Type-safe (no `any` types)
- [ ] Error handling implemented
- [ ] Performance considered (debouncing, pagination)
- [ ] Documentation added
- [ ] No hardcoded strings (use i18n keys)
- [ ] Accessible (keyboard navigation, ARIA labels)
- [ ] Mobile responsive (if UI component)
- [ ] Token usage optimized
