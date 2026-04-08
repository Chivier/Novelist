# Scrivener - Competitive Research

> Literature & Latte's Scrivener is the gold standard writing tool for long-form authors.
> Current version: Scrivener 3 (macOS, Windows, iOS). One-time license fee (~$49 USD per platform).
> Research compiled April 2026.

---

## 1. Core Editing Features

### Text Editor
- Rich text editing: bold, italics, underline, strikethrough, highlighting
- Comments and inline annotations (two separate systems)
- Inline footnotes and inspector-linked footnotes (two streams)
- Endnote support (placement configurable during compile)
- Lists (bulleted, numbered), tables, images embedded in text
- Styles/formatting presets: named styles that apply multiple formatting rules at once (e.g., "Block Quote" = indent + smaller text + italic)
- Page View mode: visual representation of pages filling as you type
- Find & Replace with regex support
- Text color and highlight color

### Scrivenings Mode
- View and edit multiple Binder documents as a **single continuous text** in the editor
- Allows treating fragmented scenes/chapters as one long document when needed
- Dividers shown between sections (configurable)

### Split Editor
- Up to **four documents simultaneously** visible in one window (Scrivener 3 Mac)
- Horizontal or vertical split
- Each pane can independently show editor, corkboard, or outliner
- Lock a pane to a specific document while navigating in the other

### Composition Mode (Distraction-Free)
- Full-screen writing environment
- Customizable background color, text width, opacity
- Background image/texture options
- Paper position (center, left, right)
- Typewriter scrolling (keeps current line centered)
- Minimal UI — hides binder, inspector, toolbar

### Dark Mode
- System-wide dark mode support
- Editor can have independent theme from the rest of the UI

---

## 2. Project Organization

### The Binder (Left Sidebar)
- Hierarchical tree structure for the entire project
- **Three root-level folders** by default:
  - **Draft/Manuscript** — the actual text to be compiled into output
  - **Research** — reference materials (can hold any file type)
  - **Trash** — deleted items
- Unlimited nesting depth (folders within folders)
- Drag-and-drop reordering
- Documents and folders are interchangeable (a document can contain sub-documents)
- Custom icons assignable to any item
- Color-coded icons via Labels
- Items can be included/excluded from compile individually

### Corkboard View
- Virtual index cards laid out on a cork-textured board
- Each card represents a Binder item (scene, chapter, etc.)
- Card shows: **title** + **synopsis** (short summary)
- Two layout modes:
  - **Grid mode**: ordered rows and columns, configurable cards-per-row
  - **Freeform mode**: place cards anywhere on the board (spatial arrangement)
- Cards can display label colors as pin colors or card tint
- Cards can show status stamps
- Keyword colored chips visible on cards
- Drag-and-drop to reorder (reorders Binder and manuscript simultaneously)
- Adjustable card size and spacing

### Outliner View
- Spreadsheet-like hierarchical view of the manuscript
- Configurable columns:
  - Title, Synopsis, Label, Status
  - Word Count, Target Word Count, Progress bar
  - Keywords, Custom Metadata fields
  - Creation Date, Modified Date
  - Section Type, Compile inclusion
- Drag-and-drop reordering
- Expand/collapse hierarchy
- Sort by any column
- All three views (Binder, Corkboard, Outliner) are **always synchronized** — reorder in one, all update

### Collections
- Saved groups/lists of documents drawn from anywhere in the project
- **Static Collections**: manually curated lists
- **Dynamic/Smart Collections**: auto-populated based on search criteria
- Appear as tabs above the Binder
- Useful for tracking POV characters, subplots, revision passes

---

## 3. The Compile/Export System

### Philosophy: WYSINWYG
- "What You See Is **Not** What You Get" — intentionally separates writing formatting from output formatting
- Writers can use whatever font/size/spacing they prefer while drafting
- Compile transforms everything to the required output format
- This is Scrivener's most powerful and most criticized feature

### Compile Architecture (Three-Column Interface)
1. **Formats** (left column): Preset formatting recipes (e.g., "Manuscript (Courier)", "Paperback", "Ebook", "Default")
   - Built-in formats + user-created custom formats
   - Project-level or system-wide formats
2. **Section Layouts** (center): Define how each document type looks in output
   - Controls: title visibility, numbering, text formatting, page breaks, separators
   - Can override editor text formatting entirely or use "As-Is" to preserve it
   - Examples: "Chapter Heading", "Scene", "Scene with Title", "Numbered Chapter"
3. **Section Types** (right/mapping): Assign Section Types to Binder structure
   - Section Types are project-level labels (e.g., "Chapter", "Scene", "Front Matter")
   - Can be auto-assigned by hierarchy level or manually set per document
   - Compile maps Section Types -> Section Layouts

### Output Formats
- **Word Processing**: DOCX, RTF, ODT, TXT, HTML
- **Print**: PDF (with headers, footers, page numbers)
- **Ebook**: EPUB, Kindle/Mobi
- **Screenwriting**: Final Draft (.fdx), Fountain
- **Academic**: MultiMarkdown -> LaTeX -> PDF pipeline
- **Outline**: OPML export
- **Plain text / Markdown**

### Compile Capabilities
- Front matter / back matter inclusion (selectable per compile)
- Chapter numbering (auto-generated, Roman, Arabic, words)
- Running headers/footers
- Table of contents generation
- Page break control
- Separator control between sections (empty line, custom text, scene break character)
- Strip comments/annotations on export
- Strip inline footnotes or convert to endnotes
- Font/size/spacing override at output level
- Placeholder tags (e.g., `<$title>`, `<$date>`, `<$wc>` for word count)

### Export (Distinct from Compile)
- Export individual documents or selections
- Preserves Binder structure as folder hierarchy
- Can export notes, metadata, snapshots alongside
- OPML export for outliner interoperability

---

## 4. Research Management

### Research Folder
- Dedicated Binder section for non-manuscript materials
- Stores **any file type**: PDFs, images, web pages, audio, video, text files
- Files viewable within Scrivener (built-in viewers for common formats)
- Web pages can be imported and stored as web archives
- Split-screen: view research in one pane, write in the other

### Bookmarks
- Project Bookmarks: key reference documents for the whole project
- Document Bookmarks: references specific to a particular scene/chapter
- Can link to internal documents or external files/URLs
- Replaced the older "Project Notes" feature in Scrivener 3

### Import Capabilities
- Microsoft Word (.doc, .docx)
- OpenOffice/LibreOffice (.odt)
- RTF, TXT, HTML
- Final Draft (.fdx)
- PDFs
- Images (PNG, JPG, etc.)
- Multimedia (audio, video)
- Web pages (as web archives)
- OPML (outline files)
- Markdown / MultiMarkdown
- Import and Split: import a large document and split it by separator (e.g., `#` or `***`)

---

## 5. Writing Tools & Tracking

### Word Count Targets
- **Project Target**: total word count goal for the manuscript
  - Visual progress bar
  - Options to count only documents included in compile
  - Deadline setting with calculated daily target
- **Session Target**: daily word count goal
  - Resets automatically (configurable reset time)
  - Notification on reaching goal
- **Document Target**: individual word count goal per document/scene
  - Progress bar visible in Outliner and Footer

### Writing History
- Tracks daily word count over time
- Historical log of words written per day
- Useful for maintaining writing streaks and analyzing productivity

### Snapshots
- Save a point-in-time copy of any document
- Named snapshots (e.g., "Before major revision")
- **Compare** snapshots to current version (diff view with additions/deletions highlighted)
- Roll back to any snapshot version
- Snapshots preserved independently of main text
- Titled snapshots can be taken automatically before sync

### Linguistic Focus (Mac only)
- Highlights specific parts of speech in the text:
  - Nouns, Pronouns, Verbs, Adjectives, Adverbs, Prepositions, Conjunctions
- Fades other text so the chosen part of speech stands out
- Useful for detecting overuse of adverbs, passive constructions, etc.

### Dialogue Focus
- Dims all non-dialogue text
- Only quoted speech remains fully visible
- Fade level adjustable via slider
- Helps assess dialogue flow and naturalness
- Available on Mac and Windows

### Find & Replace
- Project-wide search
- Search by formatting, metadata, or text
- Project Search results shown as a Collection

### Text Statistics
- Word count, character count (with/without spaces)
- Paragraph and sentence count
- Average words per sentence
- Reading time estimate
- Available per document or for the entire manuscript

---

## 6. Metadata System

### Built-in Metadata
- **Title**: document name (shown in Binder)
- **Synopsis**: short summary (shown on index cards)
- **Label**: color-coded tag (e.g., POV character, subplot) — colors visible throughout UI
- **Status**: workflow state (To Do, First Draft, Revised Draft, Final Draft, Done)
- **Keywords**: multiple keyword tags per document, each with associated color
- **Section Type**: structural role for compile (Chapter, Scene, etc.)

### Custom Metadata Fields
Four types available:
1. **Text**: freeform text
2. **Checkbox**: boolean toggle
3. **List**: dropdown with predefined values
4. **Date**: date/time with configurable format

Custom metadata fields appear in:
- The Inspector panel
- Outliner columns
- Can be used for searching and filtering

### Inspector Panel (Right Sidebar)
The Inspector has multiple tabs:
1. **Synopsis & Notes**: index card synopsis + document notes
2. **Bookmarks**: document-level reference links
3. **Metadata**: labels, status, keywords, custom fields
4. **Snapshots**: version history for the document
5. **Comments & Footnotes**: linked annotations

---

## 7. UI/UX Design Decisions

### Core Design Philosophy
- **The "Binder" metaphor**: project as a ring binder holding all materials
- **Everything in one place**: manuscript, research, notes, outlines all in one project
- **Non-linear writing**: write scenes in any order, assemble later
- **Separation of content from presentation**: format during compile, not while writing
- **Flexibility over prescription**: multiple ways to do everything

### Layout Architecture
```
+--------------------------------------------------+
|  Toolbar                                         |
+----------+---------------------------+-----------+
|          |                           |           |
|  Binder  |    Main Editor            | Inspector |
|  (tree)  |  (text / corkboard /      |  (meta-  |
|          |   outliner)               |   data)   |
|          |                           |           |
|          |                           |           |
+----------+---------------------------+-----------+
|  Footer bar (word count, zoom, view toggles)     |
+--------------------------------------------------+
```

- Three-panel layout: Binder | Editor | Inspector
- Binder and Inspector can be hidden independently
- Editor can split into 2-4 panes
- Footer bar shows word count, target progress, zoom controls
- Toolbar is customizable

### Key UX Decisions
- **No auto-formatting of output while writing** — deliberately avoids WYSIWYG for final format
- **Documents and Folders are interchangeable** — both can contain text and children
- **Every item has a synopsis** — drives the corkboard and outliner views
- **No page numbers while writing** — text flows continuously, pages only matter at compile
- **Drag-and-drop everywhere** — Binder, corkboard, outliner all support drag reorder
- **Project-based** — one project = one .scriv package, not individual files

### Keyboard Shortcuts
- Heavily keyboard-driven for power users
- Customizable shortcuts
- Quick Search (project search from keyboard)

---

## 8. File Format & Data Storage

### The .scriv Package
On macOS, a `.scriv` "file" is actually a **macOS Document Package** (a folder that Finder displays as a single file). On Windows, it is a regular folder with a `.scriv` extension.

### Internal Structure
```
MyProject.scriv/
├── MyProject.scrivx          # XML project manifest (the "brain")
├── Files/
│   ├── Data/
│   │   ├── [UUID]/           # One folder per Binder item
│   │   │   ├── content.rtf   # Document text in RTF format
│   │   │   ├── notes.rtf     # Document notes
│   │   │   └── snapshots/    # Snapshot versions
│   │   ├── [UUID]/
│   │   │   └── ...
│   │   └── ...
│   ├── docs.checksum          # Integrity checksums
│   ├── binder.autosave        # Auto-save state
│   ├── binder.backup          # Binder backup
│   ├── search.indexes         # Full-text search index
│   ├── styles.xml             # Document styles
│   ├── version.txt            # Format version
│   └── writing.history        # Writing history log
├── Settings/
│   ├── recents.txt            # Recently opened documents
│   ├── ui-common.xml          # Shared UI settings
│   └── ui.ini                 # Platform-specific UI state
└── [imported files]           # PDFs, images, media in research folder
```

### The .scrivx File (XML Manifest)
- XML format with root element `<ScrivenerProject>`
- Contains:
  - Full Binder hierarchy (folders, documents, ordering)
  - UUID identifiers for each item (maps to Data/ subfolders)
  - Document titles, labels, status values
  - Compile inclusion flags
  - Creation and modification timestamps
  - Target word counts per document
  - Section type assignments
  - Keyword associations
  - Project settings and print/search configuration

### Document Storage
- Individual documents stored as **RTF** (Rich Text Format) files
- UUID-based filenames (not human-readable)
- One RTF file per Binder document
- Research files (PDFs, images, etc.) stored as-is in their UUID folders
- The RTF format is an open standard, so text is recoverable even without Scrivener

### Key Format Characteristics
- **Not a single monolithic file** — data spread across many small files
- **RTF as base text format** — open, recoverable, but not modern (no native Markdown)
- **XML manifest is the glue** — without the .scrivx, files are just orphaned RTFs
- **Checksums for integrity** — helps detect corruption
- **No database** — pure file-system storage, no SQLite or similar
- **Sync-friendly design** — small individual files mean only changed documents sync
- **Platform differences**: macOS uses Apple Document Package (appears as single file); Windows shows as regular folder

### Backup System
- Automatic backups on project open and/or close
- Backups stored as zipped .scriv packages
- Configurable backup count and location
- Manual backup ("Back Up To..." any location)

---

## 9. Project Templates

### Built-in Template Categories
- **Blank**: empty project
- **Fiction**: Novel, Novel (with Parts), Short Story
- **Non-Fiction**: General Nonfiction, Research Proposal, Undergraduate Essay
- **Scriptwriting**: Screenplay, Comic Script, Stage Play (UK/US), BBC Radio Drama
- **Poetry & Lyrics**: Poetry Collection
- **Miscellaneous**: Lecture, Recipe Collection

### Template Features
- Templates pre-populate Binder structure, compile settings, and formatting
- Users can create **custom project templates** from any project
- **Document Templates**: reusable document types within a project (e.g., Character Sheet, Setting Profile)
  - Accessed via "New from Template" menu
  - Can have custom icons and default content

---

## 10. Cross-Platform & Sync

### Platform Availability
- **macOS**: Full-featured, primary development platform
- **Windows**: Full-featured (reached parity with Scrivener 3 for Windows)
- **iOS** (iPad/iPhone): Capable mobile version with compile support
- **No Android version** (announced but long-delayed; Scrivener Android status uncertain)
- **No web version** — entirely native/local application
- **No Linux version** (old unofficial WINE-based workarounds exist)

### Syncing
- **Dropbox** is the only officially supported sync method for iOS
- Sync is **manual** on iOS (must tap sync button) — not real-time background sync
- Desktop versions can use any cloud storage (Dropbox, iCloud, Google Drive, OneDrive) but with caveats:
  - Must wait for full sync before opening on another machine
  - File-level sync can cause corruption if two machines edit simultaneously
- **No built-in real-time sync or collaboration**
- Separate licenses required per platform (Mac + Windows = two purchases; iOS is a separate purchase)

---

## 11. What Users Love

### Organization & Structure
- "All my materials in one place — outline, character profiles, location info, research and more"
- The Binder is universally praised as the killer feature
- Non-linear writing: write scenes out of order, rearrange freely
- Corkboard gives a "bird's eye view" that other tools lack

### Writing Flexibility
- Modular approach makes revisions feel like "puzzle-solving rather than a chore"
- Write in comfortable formatting, compile to submission standards
- Scrivenings mode bridges the gap between fragmented and continuous writing

### Power & Depth
- "Sets the standard for tools in its category"
- One-time purchase (no subscription) is highly valued
- Snapshots eliminate fear of major revisions
- Word targets and writing history drive productivity
- Mature, stable software (15+ years of development)

### Research Integration
- Having research viewable alongside the manuscript is a top feature
- Import anything — PDFs, web pages, images, multimedia

---

## 12. What Users Hate / Common Complaints

### Steep Learning Curve
- Most frequently cited complaint
- Compile system is especially confusing for new users
- "If there's a persistent complaint against Scrivener, it's the steep learning curve"
- Many features are hidden or non-obvious
- The manual is extensive but itself daunting

### Compile System Complexity
- The three-column Section Types / Section Layouts mapping is confusing
- "Scrivener 3 will format it as it sees fit and you will like it"
- Formatting surprises when compile output doesn't match expectations
- Indents, tables, bullet points can misbehave
- Users report getting different results with identical settings

### No Real-Time Collaboration
- Single-user tool only
- No sharing, commenting, or co-authoring
- No way to send a project to an editor who doesn't own Scrivener
- Literature & Latte has stated they have no plans for real-time collaboration

### Sync Issues
- Dropbox-only iOS sync is limiting
- Sync is fragile — corruption possible if not done carefully
- Manual sync requirement on iOS is annoying
- No native cloud storage

### Platform & Licensing
- Separate licenses per platform frustrates multi-platform users
- iOS version is less capable than desktop
- No Android app (long-promised, not delivered)
- No web version
- No Linux version

### UI/UX Aging
- Interface can feel "crowded" and "visually confusing"
- Some users report "unnecessary mental fatigue" from the dense UI
- Not as modern-looking as newer competitors (Ulysses, iA Writer)
- Dark mode arrived late and still has rough edges

### Missing Modern Features
- No built-in grammar checking (only basic spell check)
- No AI writing assistance
- No Markdown-native editing (RTF-based internally)
- Limited table support
- No plugin/extension system
- No real-time word processor-style collaboration

---

## 13. Competitive Positioning Summary

| Strength | Weakness |
|----------|----------|
| Best-in-class manuscript organization | Steep learning curve |
| Powerful compile/export system | Compile is confusing |
| All-in-one project management | No collaboration |
| Research integration | No cloud/web version |
| Non-linear writing support | Aging UI |
| One-time purchase pricing | Per-platform licensing |
| Mature & stable (15+ years) | Slow to adopt modern features |
| Snapshots for revision safety | RTF-based (not Markdown-native) |
| Corkboard & Outliner views | No plugin ecosystem |
| Cross-platform (Mac/Win/iOS) | No Android/Linux/Web |

---

## 14. Key Takeaways for Building a Competitor

1. **The Binder is sacred** — hierarchical project organization is the #1 feature writers value. Any competitor must nail this.
2. **Corkboard view is beloved** — visual index card organization is a differentiator over plain word processors.
3. **Compile is the pain point** — a simpler, more intuitive export/format system would be a massive competitive advantage.
4. **Collaboration is the gap** — Scrivener explicitly refuses to build this. A competitor with real-time collaboration would capture a large unserved market.
5. **Cloud-native is expected now** — Scrivener's local-first, Dropbox-sync approach feels dated. Modern users expect seamless cloud sync.
6. **Markdown-native > RTF** — Scrivener's RTF internals feel anachronistic. Markdown is the modern standard for writing tools.
7. **Reduce the learning curve** — progressive disclosure of features would help. Don't show everything at once.
8. **Keep one-time pricing** — users consistently praise Scrivener's non-subscription model and resent tools that charge monthly.
9. **Snapshots/version control is valued** — writers want safety nets before major revisions. Git-like versioning done right for writers would be powerful.
10. **Research alongside writing** — the ability to store and view reference materials within the project is highly valued and must be preserved.

---

*Sources: Literature & Latte official site and blog, Literature & Latte forums, Capterra/G2/GetApp reviews, Obsolete Thor digital preservation analysis, various writer review sites (Jerry Jenkins, Kindlepreneur, The Creative Penn, Writer Gadgets, SoftwareHow, Squibler)*
