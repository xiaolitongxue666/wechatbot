(function () {
  'use strict';

  // ===================================================================
  // Reference types
  // ===================================================================
  const TYPE_BOOKMARK = 'bookmark';
  const TYPE_FOLDER = 'folder';

  // ===================================================================
  // SVG Icons
  // ===================================================================
  const FOLDER_ICON = `<svg viewBox="0 0 16 16"><path d="M1.5 3.5h5l1.5 1.5H14.5v7a1 1 0 0 1-1 1H2.5a1 1 0 0 1-1-1V4.5a1 1 0 0 1 1-1z"/></svg>`;
  const FOLDER_OPEN_ICON = `<svg viewBox="0 0 16 16"><path d="M1.5 3.5h5l1.5 1.5H14.5v7a1 1 0 0 1-1 1H2.5a1 1 0 0 1-1-1v-7l.586-.586c.187-.188.442-.293.708-.293L1.5 3.5z"/></svg>`;
  const BOOKMARK_ICON = `<svg viewBox="0 0 16 16"><path d="M3 1h10l1 1v12l-6-3-6 3V2z"/></svg>`;

  // ===================================================================
  // State
  // ===================================================================
  let bookmarkTree = [];          // root children (the full user-visible tree)
  let allNodes = new Map();       // id -> node for fast lookup
  let selectedIds = new Set();    // currently selected node ids
  let lastClickedId = null;       // last clicked (for shift-select range)
  let searchQuery = '';
  let expandedIds = new Set();    // expanded folder ids
  let draggableNodeId = null;     // currently being dragged
  let dialogMode = null;          // 'add-bookmark' | 'add-folder' | 'edit'
  let dialogTargetId = null;      // target bookmark/folder id for edit, or parent for add
  let contextMenuNodeId = null;   // node for context menu

  // ===================================================================
  // DOM refs
  // ===================================================================
  const treeEl = document.getElementById('tree');
  const searchInput = document.getElementById('search-input');
  const btnClearSearch = document.getElementById('btn-clear-search');
  const dialogOverlay = document.getElementById('dialog-overlay');
  const dialogTitle = document.getElementById('dialog-title');
  const dialogName = document.getElementById('dialog-name');
  const dialogUrl = document.getElementById('dialog-url');
  const dialogUrlField = document.getElementById('dialog-url-field');
  const dialogParent = document.getElementById('dialog-parent');
  const contextMenu = document.getElementById('context-menu');

  // ===================================================================
  // Utility functions
  // ===================================================================
  function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }

  function highlightText(text, query) {
    if (!query) return escapeHtml(text);
    const escaped = escapeHtml(text);
    const escapedQuery = escapeHtml(query);
    const regex = new RegExp(`(${escapedQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
    return escaped.replace(regex, '<span class="highlight">$1</span>');
  }

  function flattenTree(nodes, result = []) {
    for (const node of nodes) {
      result.push(node);
      if (node.children && node.children.length > 0) {
        flattenTree(node.children, result);
      }
    }
    return result;
  }

  // ===================================================================
  // Load bookmarks from Chrome API
  // ===================================================================
  function loadBookmarks() {
    chrome.bookmarks.getTree(function (tree) {
      // tree[0] is the root node with children: "Bookmarks Bar", "Other Bookmarks", "Mobile Bookmarks"
      bookmarkTree = tree[0].children || [];
      allNodes.clear();
      function indexNodes(nodes) {
        for (const node of nodes) {
          allNodes.set(node.id, node);
          if (node.children) indexNodes(node.children);
        }
      }
      indexNodes(bookmarkTree);
      renderTree();
    });
  }

  // ===================================================================
  // Render tree
  // ===================================================================
  function renderTree() {
    treeEl.innerHTML = '';
    if (bookmarkTree.length === 0) {
      treeEl.innerHTML = '<div class="empty-state">No bookmarks found</div>';
      return;
    }

    function matchesSearch(node) {
      if (!searchQuery) return true;
      const q = searchQuery.toLowerCase();
      if (node.title && node.title.toLowerCase().includes(q)) return true;
      if (node.url && node.url.toLowerCase().includes(q)) return true;
      if (node.children) {
        return node.children.some(matchesSearch);
      }
      return false;
    }

    function renderNode(node, depth) {
      if (searchQuery && !matchesSearch(node)) return null;

      const isFolder = !node.url;
      const isExpanded = expandedIds.has(node.id);
      const isSelected = selectedIds.has(node.id);
      const hasChildren = isFolder && node.children && node.children.length > 0;
      const isDragging = draggableNodeId === node.id;

      const row = document.createElement('div');
      row.className = 'tree-node' + (isExpanded ? ' expanded' : '') + (isSelected ? ' selected' : '') + (isDragging ? ' dragging' : '');
      row.dataset.id = node.id;
      row.dataset.type = isFolder ? TYPE_FOLDER : TYPE_BOOKMARK;
      row.draggable = true;

      // Indent
      const indent = document.createElement('span');
      indent.className = 'indent';
      indent.style.width = (depth * 16) + 'px';
      row.appendChild(indent);

      // Toggle
      const toggle = document.createElement('span');
      toggle.className = 'toggle' + (hasChildren ? '' : ' empty');
      toggle.textContent = hasChildren ? (isExpanded ? '▼' : '▶') : '';
      toggle.addEventListener('click', function (e) {
        e.stopPropagation();
        if (!hasChildren) return;
        if (expandedIds.has(node.id)) {
          expandedIds.delete(node.id);
        } else {
          expandedIds.add(node.id);
        }
        renderTree();
      });
      row.appendChild(toggle);

      // Icon
      const icon = document.createElement('span');
      icon.className = 'icon';
      if (isFolder) {
        icon.innerHTML = isExpanded ? FOLDER_OPEN_ICON : FOLDER_ICON;
      } else {
        icon.innerHTML = BOOKMARK_ICON;
      }
      row.appendChild(icon);

      // Name + URL wrapper
      const content = document.createElement('span');
      content.className = 'node-content';

      const nameSpan = document.createElement('span');
      nameSpan.className = 'name';
      nameSpan.innerHTML = highlightText(node.title || (isFolder ? 'Untitled Folder' : 'Untitled'), searchQuery);
      content.appendChild(nameSpan);

      if (!isFolder && node.url) {
        const urlSpan = document.createElement('span');
        urlSpan.className = 'url-text';
        urlSpan.innerHTML = highlightText(node.url, searchQuery);
        urlSpan.title = node.url;
        urlSpan.addEventListener('click', function (e) {
          e.stopPropagation();
        });
        content.appendChild(urlSpan);
      }

      row.appendChild(content);

      // Click handler for selection
      row.addEventListener('click', function (e) {
        handleNodeClick(e, node.id);
      });

      // Context menu
      row.addEventListener('contextmenu', function (e) {
        e.preventDefault();
        e.stopPropagation();
        showContextMenu(e.clientX, e.clientY, node.id);
      });

      // Double-click to edit
      row.addEventListener('dblclick', function (e) {
        e.stopPropagation();
        if (isFolder) {
          openDialog('edit', node.id);
        } else {
          openDialog('edit', node.id);
        }
      });

      // F2 to rename (handled globally)

      // Drag events
      row.addEventListener('dragstart', function (e) {
        if (selectedIds.has(node.id)) {
          // If dragging a selected node, drag all selected
          e.dataTransfer.setData('text/plain', JSON.stringify([...selectedIds]));
        } else {
          e.dataTransfer.setData('text/plain', JSON.stringify([node.id]));
        }
        e.dataTransfer.effectAllowed = 'move';
        draggableNodeId = node.id;
        row.classList.add('dragging');
      });

      row.addEventListener('dragend', function (e) {
        draggableNodeId = null;
        renderTree();
      });

      row.addEventListener('dragover', function (e) {
        e.preventDefault();
        if (!draggableNodeId) return;
        // Determine if dropping onto folder or between
        const rect = row.getBoundingClientRect();
        const y = e.clientY - rect.top;

        // Remove all drag-over classes first
        treeEl.querySelectorAll('.drag-over, .drop-target-folder, .drop-target-between').forEach(el => {
          el.classList.remove('drag-over', 'drop-target-folder', 'drop-target-between');
        });

        if (isFolder && y > rect.height * 0.25 && y < rect.height * 0.75) {
          row.classList.add('drop-target-folder');
          e.dataTransfer.dropEffect = 'move';
        } else if (y <= rect.height * 0.5) {
          row.classList.add('drop-target-between');
          row.style.borderTop = '2px solid #007acc';
          e.dataTransfer.dropEffect = 'move';
        }
      });

      row.addEventListener('dragleave', function (e) {
        row.classList.remove('drop-target-folder', 'drop-target-between');
        row.style.borderTop = '';
      });

      row.addEventListener('drop', function (e) {
        e.preventDefault();
        e.stopPropagation();
        row.classList.remove('drop-target-folder', 'drop-target-between');
        row.style.borderTop = '';

        const ids = JSON.parse(e.dataTransfer.getData('text/plain') || '[]');
        if (ids.length === 0) return;

        const rect = row.getBoundingClientRect();
        const y = e.clientY - rect.top;

        if (isFolder && y > rect.height * 0.25 && y < rect.height * 0.75) {
          // Drop into folder
          moveNodes(ids, node.id);
        } else if (y <= rect.height * 0.5) {
          // Drop before this node
          moveNodesBefore(ids, node.id);
        } else {
          // Drop after this node
          moveNodesAfter(ids, node.id);
        }
      });

      // Render children
      if (isFolder && node.children && node.children.length > 0) {
        const childrenContainer = document.createElement('div');
        childrenContainer.className = 'children';
        for (const child of node.children) {
          const childRow = renderNode(child, depth + 1);
          if (childRow) childrenContainer.appendChild(childRow);
        }
        row.appendChild(childrenContainer);
      } else if (isFolder && searchQuery) {
        // In search mode, still render matching descendants as if flattened
        const childrenContainer = document.createElement('div');
        childrenContainer.className = 'children';
        for (const child of (node.children || [])) {
          const childRow = renderNode(child, depth + 1);
          if (childRow) childrenContainer.appendChild(childRow);
        }
        if (childrenContainer.children.length > 0) {
          row.appendChild(childrenContainer);
          row.classList.add('expanded');
        }
      }

      return row;
    }

    for (const node of bookmarkTree) {
      const row = renderNode(node, 0);
      if (row) treeEl.appendChild(row);
    }
  }

  // ===================================================================
  // Selection handling
  // ===================================================================
  function handleNodeClick(e, nodeId) {
    if (e.ctrlKey || e.metaKey) {
      // Toggle individual selection
      if (selectedIds.has(nodeId)) {
        selectedIds.delete(nodeId);
      } else {
        selectedIds.add(nodeId);
      }
      lastClickedId = nodeId;
    } else if (e.shiftKey && lastClickedId) {
      // Range selection
      const flatNodes = flattenTree(bookmarkTree);
      const idxA = flatNodes.findIndex(n => n.id === lastClickedId);
      const idxB = flatNodes.findIndex(n => n.id === nodeId);
      if (idxA !== -1 && idxB !== -1) {
        const start = Math.min(idxA, idxB);
        const end = Math.max(idxA, idxB);
        selectedIds.clear();
        for (let i = start; i <= end; i++) {
          selectedIds.add(flatNodes[i].id);
        }
      }
    } else {
      // Single selection
      selectedIds.clear();
      selectedIds.add(nodeId);
      lastClickedId = nodeId;
    }
    renderTree();
    updateDeleteButton();
  }

  function updateDeleteButton() {
    const btn = document.getElementById('btn-delete-selected');
    btn.textContent = selectedIds.size > 0 ? `Delete (${selectedIds.size})` : 'Delete';
  }

  // ===================================================================
  // Move operations
  // ===================================================================
  function moveNodes(ids, parentId) {
    if (ids.includes(parentId)) return; // Can't move into self
    let moved = 0;
    for (const id of ids) {
      chrome.bookmarks.move(id, { parentId: parentId }, function () {
        moved++;
        if (moved === ids.length) {
          expandedIds.add(parentId);
          refreshTree();
        }
      });
    }
  }

  function moveNodesBefore(ids, targetId) {
    const target = allNodes.get(targetId);
    if (!target || !target.parentId) return;
    if (ids.includes(targetId)) return;

    chrome.bookmarks.getChildren(target.parentId, function (siblings) {
      const targetIdx = siblings.findIndex(s => s.id === targetId);
      const toMove = ids.filter(id => {
        const srcIdx = siblings.findIndex(s => s.id === id);
        return srcIdx !== targetIdx;
      });
      if (toMove.length === 0) return;

      let insertIdx = targetIdx;
      let moved = 0;
      for (const id of toMove) {
        chrome.bookmarks.move(id, { parentId: target.parentId, index: insertIdx }, function () {
          insertIdx++;
          moved++;
          if (moved === toMove.length) refreshTree();
        });
      }
    });
  }

  function moveNodesAfter(ids, targetId) {
    const target = allNodes.get(targetId);
    if (!target || !target.parentId) return;
    if (ids.includes(targetId)) return;

    chrome.bookmarks.getChildren(target.parentId, function (siblings) {
      const targetIdx = siblings.findIndex(s => s.id === targetId);
      const toMove = ids.filter(id => {
        const srcIdx = siblings.findIndex(s => s.id === id);
        return srcIdx !== targetIdx;
      });
      if (toMove.length === 0) return;

      let insertIdx = targetIdx + 1;
      let moved = 0;
      for (const id of toMove) {
        chrome.bookmarks.move(id, { parentId: target.parentId, index: insertIdx }, function () {
          insertIdx++;
          moved++;
          if (moved === toMove.length) refreshTree();
        });
      }
    });
  }

  // ===================================================================
  // Refresh tree after mutation
  // ===================================================================
  function refreshTree() {
    chrome.bookmarks.getTree(function (tree) {
      bookmarkTree = tree[0].children || [];
      allNodes.clear();
      function indexNodes(nodes) {
        for (const node of nodes) {
          allNodes.set(node.id, node);
          if (node.children) indexNodes(node.children);
        }
      }
      indexNodes(bookmarkTree);
      renderTree();
    });
  }

  // ===================================================================
  // Dialog handling
  // ===================================================================
  function openDialog(mode, targetId) {
    dialogMode = mode;
    dialogTargetId = targetId;

    if (mode === 'edit') {
      const node = allNodes.get(targetId);
      if (!node) return;
      const isFolder = !node.url;
      dialogTitle.textContent = isFolder ? 'Edit Folder' : 'Edit Bookmark';
      dialogName.value = node.title || '';
      dialogUrl.value = node.url || '';
      dialogUrlField.style.display = isFolder ? 'none' : 'block';
    } else if (mode === 'add-bookmark') {
      dialogTitle.textContent = 'Add Bookmark';
      dialogName.value = '';
      dialogUrl.value = '';
      dialogUrlField.style.display = 'block';
    } else if (mode === 'add-folder') {
      dialogTitle.textContent = 'Add Folder';
      dialogName.value = '';
      dialogUrl.value = '';
      dialogUrlField.style.display = 'none';
    }

    // Populate parent selector
    dialogParent.innerHTML = '';
    function addFolderOptions(nodes, depth) {
      for (const node of nodes) {
        if (!node.url) {
          const opt = document.createElement('option');
          opt.value = node.id;
          opt.textContent = '\u00A0\u00A0'.repeat(depth) + (node.title || 'Untitled Folder');
          if (node.id === targetId && mode === 'edit') opt.selected = true;
          if (node.id === targetId && mode !== 'edit') opt.selected = true;
          dialogParent.appendChild(opt);
          if (node.children) addFolderOptions(node.children, depth + 1);
        }
      }
    }
    addFolderOptions(bookmarkTree, 0);

    if (mode === 'edit') {
      const node = allNodes.get(targetId);
      if (node && node.parentId) {
        dialogParent.value = node.parentId;
      }
    } else {
      // Default to target folder (or first folder)
      if (targetId) {
        dialogParent.value = targetId;
      }
    }

    dialogOverlay.classList.remove('hidden');
    dialogName.focus();
    dialogName.select();
  }

  function closeDialog() {
    dialogOverlay.classList.add('hidden');
    dialogMode = null;
    dialogTargetId = null;
  }

  function saveDialog() {
    const name = dialogName.value.trim();
    if (!name) {
      dialogName.focus();
      return;
    }

    const parentId = dialogParent.value;
    const url = dialogUrl.value.trim();

    if (dialogMode === 'edit') {
      const node = allNodes.get(dialogTargetId);
      if (!node) return;
      const isFolder = !node.url;
      const changes = { title: name };
      if (!isFolder && url) changes.url = url;
      chrome.bookmarks.update(dialogTargetId, changes, function () {
        if (parentId && parentId !== node.parentId) {
          chrome.bookmarks.move(dialogTargetId, { parentId: parentId }, function () {
            closeDialog();
            refreshTree();
          });
        } else {
          closeDialog();
          refreshTree();
        }
      });
    } else if (dialogMode === 'add-bookmark') {
      chrome.bookmarks.create({ parentId: parentId, title: name, url: url || 'about:blank' }, function () {
        closeDialog();
        refreshTree();
      });
    } else if (dialogMode === 'add-folder') {
      chrome.bookmarks.create({ parentId: parentId, title: name }, function () {
        closeDialog();
        refreshTree();
      });
    }
  }

  // ===================================================================
  // Context menu
  // ===================================================================
  function showContextMenu(x, y, nodeId) {
    contextMenuNodeId = nodeId;

    // If right-clicked node is not selected, select only it
    if (!selectedIds.has(nodeId)) {
      selectedIds.clear();
      selectedIds.add(nodeId);
      lastClickedId = nodeId;
      renderTree();
      updateDeleteButton();
    }

    const node = allNodes.get(nodeId);
    const isFolder = node && !node.url;

    contextMenu.querySelector('[data-action="add-bookmark"]').style.display = isFolder ? 'block' : 'none';
    contextMenu.querySelector('[data-action="add-folder"]').style.display = isFolder ? 'block' : 'none';

    contextMenu.style.left = x + 'px';
    contextMenu.style.top = y + 'px';
    contextMenu.classList.remove('hidden');

    // Ensure menu fits in viewport
    const rect = contextMenu.getBoundingClientRect();
    if (rect.bottom > window.innerHeight) {
      contextMenu.style.top = (y - rect.height) + 'px';
    }
    if (rect.right > window.innerWidth) {
      contextMenu.style.left = (x - rect.width) + 'px';
    }
  }

  function hideContextMenu() {
    contextMenu.classList.add('hidden');
    contextMenuNodeId = null;
  }

  // ===================================================================
  // Delete bookmarks
  // ===================================================================
  function deleteSelected() {
    if (selectedIds.size === 0) return;

    const ids = [...selectedIds];
    const names = ids.map(id => {
      const node = allNodes.get(id);
      return node ? (node.title || 'Untitled') : 'Unknown';
    });

    const confirmMsg = ids.length === 1
      ? `Delete "${names[0]}"?`
      : `Delete ${ids.length} items?\n\n${names.slice(0, 5).join('\n')}${ids.length > 5 ? '\n...and ' + (ids.length - 5) + ' more' : ''}`;

    if (!confirm(confirmMsg)) return;

    let removed = 0;
    for (const id of ids) {
      chrome.bookmarks.remove(id, function () {
        removed++;
        if (removed === ids.length) {
          selectedIds.clear();
          refreshTree();
          updateDeleteButton();
        }
      });
    }
  }

  // ===================================================================
  // Search
  // ===================================================================
  function doSearch() {
    searchQuery = searchInput.value.trim();
    if (searchQuery) {
      btnClearSearch.classList.remove('hidden');
      // Auto-expand all folders in search mode
    } else {
      btnClearSearch.classList.add('hidden');
    }
    renderTree();
  }

  // ===================================================================
  // Expand / Collapse all
  // ===================================================================
  function expandAll() {
    function collectFolders(nodes) {
      for (const node of nodes) {
        if (!node.url) {
          expandedIds.add(node.id);
          if (node.children) collectFolders(node.children);
        }
      }
    }
    collectFolders(bookmarkTree);
    renderTree();
  }

  function collapseAll() {
    expandedIds.clear();
    renderTree();
  }

  // ===================================================================
  // Global keyboard shortcuts
  // ===================================================================
  document.addEventListener('keydown', function (e) {
    // Ignore if in input/dialog
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT' || e.target.tagName === 'TEXTAREA') {
      if (e.key === 'Escape' && dialogOverlay && !dialogOverlay.classList.contains('hidden')) {
        closeDialog();
        return;
      }
      if (e.key === 'Enter' && dialogOverlay && !dialogOverlay.classList.contains('hidden')) {
        saveDialog();
        return;
      }
      return;
    }

    // Ctrl+B: Add bookmark
    if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
      e.preventDefault();
      const parentId = selectedIds.size === 1 ? [...selectedIds][0] : bookmarkTree[0]?.id;
      openDialog('add-bookmark', parentId);
      return;
    }

    // Ctrl+Shift+F: Add folder
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'F') {
      e.preventDefault();
      const parentId = selectedIds.size === 1 ? [...selectedIds][0] : bookmarkTree[0]?.id;
      openDialog('add-folder', parentId);
      return;
    }

    // Delete key: delete selected
    if (e.key === 'Delete') {
      e.preventDefault();
      deleteSelected();
      return;
    }

    // F2: Rename
    if (e.key === 'F2' && selectedIds.size === 1) {
      e.preventDefault();
      const id = [...selectedIds][0];
      openDialog('edit', id);
      return;
    }

    // F5: Refresh
    if (e.key === 'F5') {
      e.preventDefault();
      refreshTree();
      return;
    }

    // Escape: clear selection, close context menu
    if (e.key === 'Escape') {
      hideContextMenu();
      if (selectedIds.size > 0) {
        selectedIds.clear();
        lastClickedId = null;
        renderTree();
        updateDeleteButton();
      }
      return;
    }

    // Ctrl+A: select all
    if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
      e.preventDefault();
      const flatNodes = flattenTree(bookmarkTree);
      selectedIds.clear();
      for (const node of flatNodes) {
        selectedIds.add(node.id);
      }
      lastClickedId = flatNodes.length > 0 ? flatNodes[flatNodes.length - 1].id : null;
      renderTree();
      updateDeleteButton();
      return;
    }

    // Arrow keys for navigation
    if (['ArrowUp', 'ArrowDown', 'ArrowRight', 'ArrowLeft'].includes(e.key)) {
      e.preventDefault();
      navigateTree(e.key, e.shiftKey);
    }
  });

  function navigateTree(key, shiftKey) {
    const flatNodes = flattenTree(bookmarkTree);

    if (flatNodes.length === 0) return;

    let currentIdx = -1;
    if (selectedIds.size === 1) {
      const currentId = [...selectedIds][0];
      currentIdx = flatNodes.findIndex(n => n.id === currentId);
    }

    if (key === 'ArrowDown') {
      const nextIdx = Math.min(currentIdx + 1, flatNodes.length - 1);
      if (!shiftKey) selectedIds.clear();
      selectedIds.add(flatNodes[nextIdx].id);
      lastClickedId = flatNodes[nextIdx].id;
    } else if (key === 'ArrowUp') {
      const prevIdx = Math.max(currentIdx - 1, 0);
      if (!shiftKey) selectedIds.clear();
      selectedIds.add(flatNodes[prevIdx].id);
      lastClickedId = flatNodes[prevIdx].id;
    } else if (key === 'ArrowRight') {
      if (currentIdx >= 0) {
        const node = flatNodes[currentIdx];
        if (!node.url) {
          expandedIds.add(node.id);
        }
      }
    } else if (key === 'ArrowLeft') {
      if (currentIdx >= 0) {
        const node = flatNodes[currentIdx];
        if (!node.url && expandedIds.has(node.id)) {
          expandedIds.delete(node.id);
        }
      }
    }

    renderTree();
    updateDeleteButton();

    // Scroll to selected node
    if (selectedIds.size === 1) {
      const id = [...selectedIds][0];
      const el = treeEl.querySelector(`[data-id="${id}"]`);
      if (el) {
        el.scrollIntoView({ block: 'nearest' });
      }
    }
  }

  // ===================================================================
  // Button event handlers
  // ===================================================================
  document.getElementById('btn-add-bookmark').addEventListener('click', function () {
    const parentId = selectedIds.size === 1 ? [...selectedIds][0] : bookmarkTree[0]?.id;
    openDialog('add-bookmark', parentId);
  });

  document.getElementById('btn-add-folder').addEventListener('click', function () {
    const parentId = selectedIds.size === 1 ? [...selectedIds][0] : bookmarkTree[0]?.id;
    openDialog('add-folder', parentId);
  });

  document.getElementById('btn-delete-selected').addEventListener('click', deleteSelected);
  document.getElementById('btn-refresh').addEventListener('click', refreshTree);
  document.getElementById('btn-expand-all').addEventListener('click', expandAll);
  document.getElementById('btn-collapse-all').addEventListener('click', collapseAll);

  searchInput.addEventListener('input', doSearch);
  searchInput.addEventListener('keydown', function (e) {
    if (e.key === 'Escape') {
      searchInput.value = '';
      doSearch();
    }
  });

  btnClearSearch.addEventListener('click', function () {
    searchInput.value = '';
    doSearch();
    searchInput.focus();
  });

  // Dialog
  document.getElementById('dialog-save').addEventListener('click', saveDialog);
  document.getElementById('dialog-cancel').addEventListener('click', closeDialog);
  dialogOverlay.addEventListener('click', function (e) {
    if (e.target === dialogOverlay) closeDialog();
  });

  // Context menu actions
  contextMenu.querySelectorAll('.context-item').forEach(item => {
    item.addEventListener('click', function () {
      const action = this.dataset.action;
      const nodeId = contextMenuNodeId;

      if (action === 'edit') {
        openDialog('edit', nodeId);
      } else if (action === 'add-bookmark') {
        openDialog('add-bookmark', nodeId);
      } else if (action === 'add-folder') {
        openDialog('add-folder', nodeId);
      } else if (action === 'delete') {
        deleteSelected();
      }

      hideContextMenu();
    });
  });

  // Hide context menu on click outside
  document.addEventListener('click', function (e) {
    if (!contextMenu.contains(e.target)) {
      hideContextMenu();
    }
  });

  // Click on tree background to deselect
  treeEl.addEventListener('click', function (e) {
    if (e.target === treeEl) {
      selectedIds.clear();
      lastClickedId = null;
      renderTree();
      updateDeleteButton();
    }
  });

  // Allow drop on tree background (move to root)
  treeEl.addEventListener('dragover', function (e) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  });

  treeEl.addEventListener('drop', function (e) {
    e.preventDefault();
    if (e.target === treeEl) {
      const ids = JSON.parse(e.dataTransfer.getData('text/plain') || '[]');
      if (ids.length > 0) {
        // Drop on background: move to last available folder (e.g. "Other Bookmarks")
        const lastFolder = findLastFolder(bookmarkTree);
        if (lastFolder) {
          moveNodes(ids, lastFolder.id);
        }
      }
    }
  });

  function findLastFolder(nodes) {
    for (let i = nodes.length - 1; i >= 0; i--) {
      if (!nodes[i].url) return nodes[i];
    }
    return null;
  }

  // ===================================================================
  // Listen for bookmark changes from Chrome
  // ===================================================================
  chrome.bookmarks.onCreated.addListener(function () { refreshTree(); });
  chrome.bookmarks.onRemoved.addListener(function () { refreshTree(); });
  chrome.bookmarks.onChanged.addListener(function () { refreshTree(); });
  chrome.bookmarks.onMoved.addListener(function () { refreshTree(); });

  // ===================================================================
  // Initial load
  // ===================================================================
  loadBookmarks();
})();
