# DevTools Bookmarks

A Chrome extension that adds a **Bookmarks** panel to Chrome DevTools for managing your bookmarks in a tree view.

## Features

- Tree view of all bookmarks and folders
- Add, edit, delete bookmarks and folders
- Search and filter bookmarks
- Drag-and-drop to move bookmarks between folders
- Multi-select with Ctrl/Shift keys
- Batch delete of multiple selected bookmarks
- Keyboard shortcuts for all operations
- Dark theme matching Chrome DevTools

## Installation

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable **Developer mode** (toggle in top-right corner)
3. Click **Load unpacked** and select the `devtools-bookmark` directory
4. Open Chrome DevTools (F12) — you'll see a new **Bookmarks** panel

## Usage

### Toolbar

| Button | Action |
|--------|--------|
| **+ Bookmark** | Add a new bookmark (Ctrl+B) |
| **+ Folder** | Add a new folder (Ctrl+Shift+F) |
| **Delete** | Delete selected items (Del) |
| **Search** | Filter bookmarks by name or URL |
| **Collapse / Expand** | Collapse or expand all folders |
| **Refresh** | Reload bookmarks from Chrome (F5) |

### Tree View

- **Click** a node to select it
- **Ctrl+Click** to toggle individual selection
- **Shift+Click** to select a range
- **Double-click** to edit a bookmark/folder
- **Right-click** for context menu
- **Drag-and-drop** to move bookmarks between folders

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+B` | Add bookmark |
| `Ctrl+Shift+F` | Add folder |
| `Delete` | Delete selected |
| `F2` | Rename selected |
| `F5` | Refresh |
| `Ctrl+A` | Select all |
| `Arrow Up/Down` | Navigate tree |
| `Shift+Arrow Up/Down` | Extend selection |
| `Arrow Right` | Expand folder |
| `Arrow Left` | Collapse folder |
| `Escape` | Clear selection / close dialog |

### Drag and Drop

- Drag bookmarks and folders to reorder them within the same parent
- Drop onto a folder to move items into it
- Drag multiple selected items together
