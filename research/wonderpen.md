# WonderPen Feature Research

## Overview

WonderPen is a professional writing application developed by Yingjie Wu (TominLab, China) focused on providing a distraction-free, fluid writing experience for long-form content. It targets novelists, academic writers, short story authors, and diarists. Available on Windows, macOS, Linux, iOS, Android, and Apple Vision. The software uses a freemium model with a free Basic tier, a one-time Pro purchase (~CNY 128 / ~$10 USD), and an annual Pro+ subscription (CNY 128/year) for expanded cloud features. The editor is Markdown-based with WYSIWYG rendering. Data is stored locally in JSON format with optional cloud sync.

## Core Features

- **Offline-first architecture** -- All library data stored completely locally. Cloud libraries also maintain a full local copy, so network failures do not interrupt writing.
- **Cross-platform** -- Windows, macOS, Linux (including Flathub), iOS, Android, Apple Vision (visionOS 1.0+).
- **Markdown-based editor** with WYSIWYG preview mode.
- **Freemium pricing** -- Free tier is functional; Pro is a one-time purchase; Pro+ is an annual subscription for cloud expansion.
- **Lightweight** -- Minimal system resource consumption, ad-free.

## Writing/Editing Features

- **WYSIWYG editor** -- Markdown source with rendered preview; "what you see is what you get" layout editing.
- **One-click formatting (typesetting)** -- Automatic typography cleanup via Ctrl+G / Cmd+G. Organizes text layout in one action.
- **Auto Replace** -- Custom character/punctuation replacement rules (e.g., auto-convert straight quotes to curly quotes, abbreviation expansion). Free tier limited to 5 rules; Pro/Pro+ unlimited.
- **Typewriter Mode** -- Fixes the cursor at a configurable vertical position (10%-90% of screen height). Includes optional current-line highlighting and customizable scroll timing.
- **Composition Mode (Focus Mode)** -- Full-screen mode hiding all UI except the editor (Ctrl+Alt+F / Cmd+Opt+F). Exit with Esc.
- **Minimal Mode** -- Windowed mode that hides toolbar and non-editor elements to maximize editing area without going full-screen.
- **Darkroom Mode** (Pro+) -- Full-screen distraction-free mode that locks the user in until a preset writing target is reached. Cannot be exited until the goal is met. Enabled via Settings > Advanced.
- **Dark Mode** -- System-wide dark color theme for low-light writing.
- **Editor Split** -- Two-column dual-pane view for viewing/editing two documents side-by-side (Shift+Ctrl+\ / Shift+Cmd+\). One pane can be "pinned" to stay fixed while navigating in the other.
- **Highlighted Dialogue** -- Automatic visual highlighting of dialogue/quoted speech in the editor.
- **Find and Replace** -- Standard search and replace functionality (Ctrl+F / Cmd+F).
- **Mathematical Formula & LaTeX** -- Support for rendering math formulas and LaTeX within documents.
- **Document Templates** -- Pre-built and custom document templates (3 in free tier, unlimited in Pro/Pro+).
- **Typesetting Schemes** -- Multiple formatting/layout presets (1 in free tier, unlimited in Pro/Pro+).
- **Custom CSS** (Pro/Pro+) -- Full CSS customization of the editor appearance.
- **Background Image** -- Custom editor background images (Pro/Pro+).
- **Style Customization** -- Adjustable font, line height, margin, editor width, and optional horizontal line backgrounds via style panel.
- **Multiple Themes** -- Light and Dark in free tier; unlimited built-in themes in Pro/Pro+.
- **Browser-style navigation** -- Back/forward buttons for document history navigation.
- **Favorite/Star documents** -- Quick-access starring system for frequently used documents.
- **Floating Window** -- Open a document in a separate floating window for reference while working.

## Organization/Project Management Features

- **Tree Directory (Document Tree)** -- Unlimited nesting levels for hierarchical document organization. Supports drag-and-drop reordering, indentation-based hierarchy, and keyboard navigation (arrow keys, Enter to create new doc).
- **Library System** -- Projects organized as separate libraries. Free tier: 1 cloud library; Pro+: up to 1,000 cloud libraries.
- **Document Splitting and Merging** -- Split a single document into multiple chapters or merge multiple documents into one.
- **Table of Contents** -- Auto-generated TOC for navigation within documents.
- **Outline View** -- Tabular outline mode providing an overview of each chapter/section.
- **Memo System** -- Per-document side-panel notes that do not appear in exported output. Useful for chapter notes, TODOs, and reminders.
- **Whiteboard** -- Global planning/reference workspace with five card types:
  - **Notes** -- Text cards with customizable background colors for categorization.
  - **Images** -- Reference images (portraits, maps, etc.).
  - **Tables** -- Structured data (character lists, inventories, skill tables).
  - **Checklists** -- Trackable to-do items for planning or foreshadowing.
  - **Kanban** -- Kanban-style boards for information management.
  - Tables, checklists, and kanban cards are interconvertible.
  - Cards organized via a Contents directory with three sort modes (name ascending, name descending, manual drag-and-drop).
  - Free tier: 10 cards per whiteboard. Pro/Pro+: unlimited.
- **Card List (Highlight Cards)** -- Right-column reference cards for characters, locations, or concepts. When enabled, matching card titles are automatically highlighted in the editor text; clicking a highlight shows the card. Cards can be created directly from selected editor text via right-click. Disabled by default for performance (many highlights can slow the editor). Free tier: 1 card list, 10 cards. Pro/Pro+: unlimited.
- **Custom Metadata** -- User-defined metadata fields per document (3 in free tier, unlimited in Pro/Pro+).
- **Document Icons** -- Customizable icons for documents in the tree.
- **Auto Numbering** -- Automatic chapter/section numbering.
- **Word Count in Tree** -- Optional display of word counts next to document titles in the directory tree (Settings > Advanced > Left Panel).
- **Multi-document selection** -- Select multiple sibling documents to see combined word count.
- **Hierarchical word count** -- Selecting a parent document shows total count for it and all children.

## Export/Publishing Features

- **Export formats** -- PNG image, PDF, Word (docx), EPUB (e-book), HTML (web pages), plain text, Markdown.
- **Batch export** -- Export multiple documents/chapters at once.
- **Flexible output structure** -- When batch exporting text/HTML, choose between a single consolidated file or folder structure mirroring the library's directory tree.
- **Export as ZIP** -- Bundle exported documents into a ZIP archive.
- **Pandoc integration** -- Advanced document conversion via Pandoc for additional format support.
- **Import** -- Import text files into the library.
- **Data portability** -- Internal storage is JSON; text export ensures no vendor lock-in.
- **Performance caveat** -- Exporting very long documents as images may fail.

## UI/UX Features

- **Three-pane interface** -- Left sidebar (document tree), center (editor), right panel (memo/cards).
- **Togglable panels** -- Tree panel (Ctrl+1 / Cmd+1), info panel (Ctrl+2 / Cmd+2) can be shown/hidden.
- **Zoom controls** -- Adjustable zoom levels via keyboard shortcuts.
- **Interface Lock** -- Password-protect the interface for temporary absences.
- **Writing Targets/Goals** -- Set word count goals with real-time progress monitoring.
- **Daily Word Count Statistics** -- Track daily writing output with progress visualization.
- **Typing Statistics Sharing** -- Generate shareable image cards of daily writing stats/results.
- **Keyboard shortcuts** -- Comprehensive hotkey system covering interface control, directory tree navigation, document management, and editing. Platform-aware (Ctrl on Win/Linux, Cmd on Mac).
- **Built-in Document Styles** -- Pre-defined visual styles for documents.
- **Built-in Theme Styles** -- Pre-defined color themes for the application.
- **Design Resources** -- Bundled design assets.
- **Synchronization Conflict Resolution** -- Built-in handling for cloud sync conflicts.
- **Automatic Backup** -- Full database backup every 15 minutes.
- **Manual Backup** -- On-demand backup accessible through preferences.
- **Document Snapshots** -- Save and restore historical document versions. Retention: 7 days (free), 30 days (Pro), 90 days (Pro+).
- **Backup Space** -- Dedicated backup storage management.

## Notable Design Decisions

1. **Offline-first with local-priority cloud sync** -- Cloud libraries always maintain a full local copy. This is a strong trust-building decision: users never lose access due to network issues, and it reduces anxiety about data loss. Worth emulating.

2. **JSON internal storage** -- Documents stored as JSON rather than proprietary binary format. Enables data portability and reduces lock-in concerns, though it is not a standard writing format.

3. **Darkroom Mode as a "commitment device"** -- The mode that locks the writer in until a word count target is met is a distinctive behavioral design feature targeting writers who struggle with procrastination. Novel concept worth considering.

4. **Highlight Cards with performance trade-off** -- The automatic text highlighting for character/concept cards is disabled by default because it degrades editor performance with many cards. This reveals a technical limitation in their rendering approach -- a competing product should solve this more efficiently.

5. **Freemium with one-time Pro purchase** -- The pricing separates local features (one-time buy) from cloud features (subscription). This is writer-friendly: the core tool works forever without recurring costs, and only cloud/sync is subscription-based.

6. **Deliberately limited scope** -- WonderPen explicitly does NOT support complex rich text typography or multi-person collaborative writing. This focus keeps the product simple but leaves significant competitive openings in collaboration and advanced formatting.

7. **Whiteboard with interconvertible card types** -- Tables, checklists, and kanban cards can convert between each other. Flexible approach to planning tools.

8. **Per-document memos separate from content** -- Notes that are attached to a document but excluded from export. Clean separation of process notes from output.

9. **Weak points identified in reviews:**
   - Poor English documentation (primarily Chinese).
   - Limited Markdown support beyond basics (no auto-continuation for lists, no bracket auto-completion, no URL link assistance).
   - No spell-checking integration with OS-level tools (macOS).
   - No macOS text expansion support (auto-capitalization, period shortcuts).
   - No macOS Services integration.
   - Described as lacking "polish" compared to competitors like Ulysses or Bear.
   - Developer communication/support noted as slow.

10. **Target market positioning** -- Budget-friendly ($10 one-time) alternative to Scrivener (~$49), Ulysses (subscription), and other premium writing tools. Competes on price but sacrifices polish and ecosystem integration.

## Competitive Landscape Notes

- **vs. Scrivener** -- WonderPen is simpler and cheaper but lacks Scrivener's compile system, corkboard, and research folder depth.
- **vs. Ulysses** -- WonderPen avoids subscription-only pricing for core features. Ulysses has superior Markdown editing, publishing integration, and Apple ecosystem polish.
- **vs. Bear** -- Bear is more polished as a note-taking/writing app with better Markdown handling, but lacks WonderPen's novel-specific organization features (whiteboard, highlight cards, outline view).
- **Key opportunity** -- No competitor combines strong novel-writing organization tools with modern collaborative features, excellent Markdown editing, and cross-platform support at an accessible price point.
