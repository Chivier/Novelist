# Notesnook -- Competitive Research

**Last updated:** 2026-04-06
**Website:** https://notesnook.com/
**GitHub:** https://github.com/streetwriters/notesnook (13.9k stars, 939 forks)
**License:** GPL-3.0
**Founded:** 2021
**Organization:** Streetwriters (small team)

---

## 1. Core Editing Features

Notesnook uses **TipTap** (built on **ProseMirror**) as its rich-text editor framework. The editor package lives at `@notesnook/editor` in their monorepo, with a separate `@notesnook/editor-mobile` wrapper for mobile platforms.

### Text Formatting
- Bold, italic, underline, strikethrough, superscript, subscript
- Text alignment (left, center, right, justified)
- Font selection and font size adjustment
- Text color and background/highlight color
- Clear formatting option

### Structural Elements
- Headings (H1-H6)
- Bullet lists (nested), numbered lists (nested), checklists/task lists, outline lists
- Block quotes
- Code blocks with language-specific syntax highlighting and auto-indentation
- Inline code
- Line dividers / horizontal rules
- Indentation controls
- Foldable headers/outline mode
- Foldable and nested bullet lists
- Table of contents generation

### Advanced Editor Features
- Full WYSIWYG editing with markdown shortcuts supported
- Markdown formatting symbols can be hidden while editing
- Focus mode (distraction-free, full-screen editor)
- Multiple editing panes for simultaneous note viewing
- Slash command palette for quick formatting
- Selection formatting menu (inline popup toolbar)
- Fixed formatting bar
- RTL language support
- LaTeX/KaTeX math and chemistry formula support
- Mermaid diagram support
- Bidirectional note linking (backlinks)
- Cross-note linking with autosuggestions (including block-level)
- Note link auto-update on title changes
- Word and character count (including selection count)
- Configurable keyboard shortcuts

### Tables
- Basic table creation and editing
- Cell background coloring
- Row/column formatting
- Column resizing
- Per-cell/row/column alignment
- CSV import/export for tables
- Rich content in cells (images, files, videos)
- Searchable/filterable rows
- Sortable rows with persistent headers
- Customizable borders

### Media & Attachments
- Image upload, resize, rotate, crop
- Image gallery with grid layout
- Text wrapping around images
- Image captions/annotations
- OCR text extraction (searchable in full-text search)
- Audio upload and playback with embedded controls
- Video upload and playback
- YouTube embeds, Twitter/tweet embeds
- PDF viewing and annotation within notes (PDF content indexed in search)
- Google Docs/Sheets embedding
- Arbitrary file attachments
- Drag-and-drop file import to create notes

---

## 2. Note Organization

### Notebooks & Folders
- Hierarchical notebook system (3+ levels deep)
- Multiple hierarchies can be applied to a single note
- Hierarchy coloring, moving, and merging
- Auto-normalized casing/spacing in hierarchy names

### Tags
- Tag-based categorization
- Inline tags for line-level tagging
- Auto-normalized tag naming

### Other Organization
- Pin notes to top
- Shortcuts/Starred favorites section
- Archive section with filtering
- Trash with restoration capability
- Version history for all notes with revert capability
- Note colors for visual organization

### Search
- Full-text search with quoted/literal search support
- Boolean operators (AND, NOT, OR)
- Search filtering by author, date range, hierarchy
- Quick-open note lookup with hotkey
- Search result snippets with highlighting
- Search within attachments
- Saved search capability
- Search within notes on mobile
- Filters in search (added January 2026)
- Editor stats (added January 2026)

---

## 3. Privacy & Encryption Approach

### Encryption Stack
- **Encryption algorithm:** XChaCha20-Poly1305-IETF (authenticated encryption, stronger than AES-256)
- **Password hashing:** Argon2id
- **Key derivation (PKDF):** Argon2i
- **Cryptographic library:** libsodium (C-based with cross-platform wrappers via `@notesnook/sodium` and `@notesnook/crypto` packages)

### How It Works
1. **Client-side key derivation:** User password is hashed with Argon2 using a predictable salt (derived from a fixed client salt + email)
2. **Server-side secondary hash:** The resulting hash undergoes secondary hashing to prevent password passthrough attacks
3. **Encryption key generation:** User password + server-generated salt combined via Argon2 PKDF to derive the encryption key
4. **Per-item encryption:** Each database item is individually encrypted, producing a JSON object with base64-encoded ciphertext, 192-bit nonce, random salt, algorithm identifier, and item ID
5. **Server is blind:** Encrypted payloads are sent to servers which perform zero decryption operations

### Key Storage
- **Desktop/Web:** Stored as a `CryptoKey` in IndexedDB (browser prevents export/direct access)
- **Mobile:** Stored in device native keychain (iOS Keychain / Android Keystore)

### Zero-Knowledge Architecture
- All notes, notebooks, tags, and attachments encrypted on-device before leaving
- Server stores only ciphertext
- Employees cannot read note content (documented policy)
- **Vericrypt** tool allows independent verification of encryption claims

### Additional Security Features
- Two-factor authentication (2FA)
- App lock (auto-locks notes database) -- NOTE: now Pro-only, which has been controversial
- Password-protect individual notes (vault functionality)
- Local-only / unsynchronized notes option
- Anonymous login without email requirement
- Self-hosted sync server option for full control

---

## 4. Export Capabilities

### Export Formats
- **PDF**
- **HTML**
- **Markdown**
- **Plain text**
- **CSV** (for tables)

### Export Modes
- **Single note:** Right-click (desktop/web) or three-dot menu (mobile), choose format
- **Multiple notes:** Ctrl+click selection, exported as .zip
- **All notes:** Via Settings > Import & Export, requires password authentication, generates complete backup .zip

### What's Included
- Full note content in chosen format
- Images and attachments included in exports
- Note links preserved on export
- Hierarchy (tags/folders) metadata exported for import by other apps

### Export Destinations
- Desktop/Web: User chooses save location
- Mobile: Saved to "Notesnook/exported" folder
- Export note to email

### Import Support (for competitive context)
Notesnook Importer 2.0 supports importing from:
- Evernote (ENEX)
- HTML files
- Markdown files
- CSV
- Roam Research (preserves linked content)
- Simplenote
- Standard Notes
- Google Keep
- And others

---

## 5. Cross-Platform Support

| Platform | Technology | Notes |
|----------|-----------|-------|
| **Web** | React + PWA | Full browser-based access |
| **Windows** | Electron + React | Desktop app |
| **macOS** | Electron + React | Desktop app |
| **Linux** | Electron + React | Desktop app (Flatpak also available) |
| **iOS** | React Native | Native iPad app included |
| **Android** | React Native | Full mobile app |
| **Web Clipper** | Browser extension | Chrome, Firefox, Safari |

All platforms share the same `@notesnook/core` package for business logic, with platform-specific wrappers.

### Sync
- Real-time automatic sync across all platforms
- Offline-first: read, write, and search while offline
- Sync happens when connection is restored
- Attachments viewable offline (after initial download)

---

## 6. Technical Architecture

### Monorepo Structure (TypeScript/JavaScript)

| Package | Purpose |
|---------|---------|
| `@notesnook/web` | Web client (React) |
| `@notesnook/desktop` | Desktop app (Electron) |
| `@notesnook/mobile` | iOS & Android (React Native) |
| `@notesnook/core` | Shared business logic, data models, sync engine |
| `@notesnook/crypto` | Cryptographic wrapper around libsodium |
| `@notesnook/editor` | TipTap-based rich text editor + custom extensions |
| `@notesnook/editor-mobile` | Mobile editor wrapper (WebView-based) |
| `@notesnook/clipper` | Web clipper extension |
| `@notesnook/theme` | UI theming system |
| `@notesnook/sodium` | Cross-platform libsodium bindings |
| `@notesnook/streamable-fs` | IndexedDB-based virtual filesystem for large files |
| `@notesnook/logger` | Logging infrastructure |

### Language Distribution
- TypeScript: 84.7%
- JavaScript: 13.2%
- Java: 0.7% (Android native)
- CSS: 0.7%
- HTML: 0.3%
- Swift: 0.2% (iOS native)

### Storage Architecture
- **Web/Desktop:** SQLite via OPFS (Origin Private File System) as primary, IndexedDB as fallback for older browsers
- **Multi-tab:** Shared service for OPFS backend (since OPFS doesn't support multi-tab access to the same DB)
- **Mobile:** SQLite (native)
- **Server-side:** MongoDB stores encrypted data
- **File storage:** Attachments stored encrypted on server, downloaded on demand

### Sync Architecture
- Data stored locally first (offline-first)
- Each item individually encrypted before sync
- Encrypted JSON payloads sent to sync server
- Server stores ciphertext in MongoDB
- Self-hostable sync server available (alpha) via Docker
- Self-hosted stack includes: database, auth server, sync server, file storage, monograph server

### Key Technical Decisions
- React everywhere (web, desktop via Electron, mobile via React Native)
- TipTap/ProseMirror for the editor (with heavy custom extensions)
- libsodium for cryptography (battle-tested C library with WASM/native bindings)
- SQLite on main thread caused performance issues; later moved to separate thread
- Logs stored in SQLite DB (10x faster web app loading in v3.0.7)

---

## 7. Editor Technology Deep Dive

### Stack: TipTap v2.x on ProseMirror

The editor lives at `@notesnook/editor` and wraps TipTap with extensive custom extensions. Evidence from the codebase:
- `apps/web/src/components/editor/tiptap.tsx` -- main TipTap integration
- PR #6872 updated TipTap to v2.9.1
- Custom extensions for encryption-aware content handling, math blocks, embeds, etc.

### Why This Matters for Competitive Analysis
- **ProseMirror** provides the document model, transaction system, and low-level editing primitives
- **TipTap** provides the React integration layer, extension system, and developer experience on top of ProseMirror
- This is the same foundation used by many modern editors (GitLab, Hacker News, many SaaS products)
- Notesnook has built significant custom extensions on top (math, embeds, checklists, etc.)
- The editor is shared across web, desktop, and mobile (mobile uses a WebView wrapper)

### Mobile Editor Approach
- `@notesnook/editor-mobile` wraps the web editor in a WebView
- This means mobile editing is NOT native -- it's the web editor running inside a WebView
- This is a common pattern but can have performance implications on lower-end devices

---

## 8. Performance Characteristics

### Known Issues (from GitHub issues and user reports)

**Large Note Performance:**
- Typing becomes slow and delayed in large notes (50k+ words)
- Search within very large notes can break/hang the app
- This is a TipTap/ProseMirror limitation -- DOM-based editors struggle with very large documents

**General Performance:**
- SQLite queries on main thread caused significant lag during note editing (later fixed by moving SQLite to its own thread)
- Linux desktop app (Electron/Flatpak) reports noticeable typing lag even in normal notes
- App can hang when adding many notes rapidly
- Attachments downloaded from server on each note open -- can fail, leading to lost attachments
- Wayland compatibility issues (freezing) reported on Linux

**Improvements Made:**
- v3.0.7: 10x faster web app loading by storing logs in SQLite
- v3.0.10: SQLite moved to its own thread for smoother editing
- Ongoing performance work visible in release notes

### Architectural Performance Implications
- **Electron for desktop:** Higher memory usage than native apps (~200-500MB typical)
- **React Native for mobile:** Generally good but not as smooth as fully native
- **WebView editor on mobile:** Additional overhead vs native text input
- **Offline-first with sync:** Good for responsiveness but sync conflicts can occur
- **Per-item encryption:** Adds CPU overhead to every read/write operation

---

## 9. What Users Love and Hate

### What Users Love

1. **Privacy-first without ugliness:** "Most privacy-focused apps either look like they were designed in 2005 or have a learning curve steeper than Mount Everest, but Notesnook actually feels modern without sacrificing the security fundamentals."
2. **Open source and transparent:** Fully auditable code, Vericrypt for independent encryption verification
3. **Cross-platform consistency:** Same experience on all platforms with seamless sync
4. **Modern, attractive UI:** Varied themes, dark mode, clean interface -- looks better than Standard Notes and most encrypted alternatives
5. **Rich editor:** Full formatting, tables, embeds, math support -- much more capable than minimalist encrypted note apps
6. **Affordable:** $49.99/year for Pro (or ~$1.67/month on Essential plan)
7. **Self-hostable:** Full control option for power users
8. **Good import/export:** Comprehensive importer supporting many source apps; export to standard formats
9. **Web clipper:** Privacy-respecting web clipping
10. **Active development:** Regular updates, responsive developers on GitHub

### What Users Hate

1. **Performance with large notes:** Severe lag/freezing with 50k+ word notes; search breaks in very large notes
2. **App lock paywalled:** Moving app lock to Pro-only drew significant criticism -- basic security feature behind paywall
3. **Attachment reliability:** Images/attachments downloaded from server on demand; some fail to download, effectively losing content
4. **No templates:** No premade templates system (though this may have been added recently)
5. **No web clipper initially / limited clipper:** Late addition, still basic compared to Evernote
6. **Electron bloat:** Desktop app uses significant memory for a note-taking app
7. **Limited collaboration:** No real-time collaborative editing between users (note sharing exists but is basic)
8. **Mobile editor in WebView:** Not native text editing on mobile; can feel sluggish
9. **Offline limitations:** "Requires prior app opening" for offline access -- not truly offline-first in all scenarios
10. **Sync issues:** Some users report note loss or sync conflicts, though rare
11. **Linux-specific issues:** Wayland freezing, Flatpak performance problems
12. **Limited free tier:** Many features locked behind Pro subscription

### Pricing (as of 2026)

| Plan | Price | Key Features |
|------|-------|-------------|
| **Free** | $0 | Basic note-taking, encryption, limited features |
| **Essential** | $1.67/mo (annual) | Extended features |
| **Pro** | $5.83/mo (annual) | All features including app lock, vault, etc. |
| **Believer** | $7.50/mo (annual) | Pro + support the project |

---

## 10. Key Takeaways for Novelist

### Strengths to Learn From
- **Privacy as a selling point works:** Notesnook carved a real niche by being the "encrypted Evernote"
- **Open source builds trust:** Especially for privacy-focused tools
- **TipTap/ProseMirror is a solid editor foundation:** Battle-tested, extensible, good ecosystem
- **Monorepo with shared core:** Efficient way to maintain cross-platform apps
- **Offline-first + encryption:** Users value both; doing both well is a differentiator

### Weaknesses to Exploit
- **Large document performance is poor:** A writing tool MUST handle long-form content well. This is Notesnook's Achilles' heel for novelists/long-form writers. ProseMirror struggles with very large documents without careful virtualization.
- **Not designed for long-form writing:** Notesnook is a note-taking app, not a writing tool. No chapters, manuscript structure, writing goals, or writing-specific features.
- **No native desktop editor:** Electron + TipTap adds overhead. A native editor or a more optimized approach could win on performance.
- **WebView editor on mobile is a compromise:** Native text editing would be smoother for heavy writing on tablets.
- **No collaboration designed for writing:** No editor/author workflows, comments for beta readers, track changes, etc.
- **No writing-specific organization:** No manuscript > chapter > scene hierarchy, no character/world-building databases, no storyboards.

### Technical Lessons
- TipTap/ProseMirror works well for note-length content but needs virtualization or a different approach for novel-length documents
- libsodium + XChaCha20 is a proven encryption stack worth considering
- SQLite + OPFS for web storage is the modern approach (over pure IndexedDB)
- Moving heavy operations off the main thread is essential
- Shared core package across platforms is the right architecture for cross-platform apps
