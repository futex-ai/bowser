(() => {
  const INCLUDE_HIDDEN = __BOWSER_INCLUDE_HIDDEN__;
  const state = { nextId: 1, metadata: [] };
  window.__bowserElements = new Map();
  window.__bowserMetadata = new Map();
  const visibilityCache = new WeakMap();
  let activeModalRoots = [];

  const isHiddenInput = (el) => el.tagName === 'INPUT' && (el.getAttribute('type') || '').toLowerCase() === 'hidden';
  const isFocused = (el) => {
    const doc = el.ownerDocument || document;
    return doc.activeElement === el;
  };
  const viewportFor = (doc) => {
    const view = doc.defaultView || window;
    return { width: view.innerWidth, height: view.innerHeight, view };
  };
  const renderedRect = (rect) => ({
    left: rect.left,
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
    width: Math.max(0, rect.width),
    height: Math.max(0, rect.height)
  });
  const viewportRect = (rect, doc) => {
    const viewport = viewportFor(doc);
    const left = Math.max(rect.left, 0);
    const top = Math.max(rect.top, 0);
    const right = Math.min(rect.right, viewport.width);
    const bottom = Math.min(rect.bottom, viewport.height);
    return {
      left,
      top,
      right,
      bottom,
      width: Math.max(0, right - left),
      height: Math.max(0, bottom - top)
    };
  };
  const hasPositiveArea = (rect) => rect.width > 0 && rect.height > 0;
  const samplePoints = (rect) => {
    if (rect.width <= 0 || rect.height <= 0) return [];
    const x1 = rect.left + Math.min(rect.width / 2, Math.max(rect.width * 0.2, 1));
    const x2 = rect.right - Math.min(rect.width / 2, Math.max(rect.width * 0.2, 1));
    const y1 = rect.top + Math.min(rect.height / 2, Math.max(rect.height * 0.2, 1));
    const y2 = rect.bottom - Math.min(rect.height / 2, Math.max(rect.height * 0.2, 1));
    return [
      { x: (rect.left + rect.right) / 2, y: (rect.top + rect.bottom) / 2 },
      { x: x1, y: y1 },
      { x: x2, y: y1 },
      { x: x1, y: y2 },
      { x: x2, y: y2 }
    ];
  };
  const isTopMatch = (owner, top) =>
    !!top && (top === owner || owner.contains(top) || top.contains(owner));
  const isInsideModalRoot = (el) =>
    activeModalRoots.some((root) => root === el || root.contains(el));
  const isModalObscured = (el) =>
    activeModalRoots.length > 0 && !isInsideModalRoot(el);
  const shouldCaptureElement = (el, visibleSelf, mode) =>
    mode === 'visible' ? visibleSelf : (!visibleSelf || isModalObscured(el));
  const hasReachablePoint = (owner, rects, doc) => {
    const hitTest = typeof doc.elementFromPoint === 'function'
      ? doc.elementFromPoint.bind(doc)
      : document.elementFromPoint.bind(document);
    return rects.some((rect) =>
      samplePoints(rect).some(({ x, y }) => {
        const top = hitTest(x, y);
        return isTopMatch(owner, top);
      })
    );
  };
  const hasReachablePointInCurrentDocument = (owner, rects) =>
    rects.some((rect) =>
      samplePoints(rect).some(({ x, y }) => {
        const top = document.elementFromPoint(x, y);
        return isTopMatch(owner, top);
      })
    );
  const hasReachablePointInViewport = (owner, rects, doc) => {
    const clipped = rects
      .map((rect) => viewportRect(rect, doc))
      .filter(hasPositiveArea);
    if (clipped.length === 0) return false;
    return doc === document
      ? hasReachablePointInCurrentDocument(owner, clipped)
      : hasReachablePoint(owner, clipped, doc);
  };
  const isStyleSuppressed = (el) => {
    const doc = el.ownerDocument || document;
    const style = viewportFor(doc).view.getComputedStyle(el);
    return style.display === 'none' || style.visibility === 'hidden' || Number(style.opacity || '1') === 0;
  };
  const isRendered = (el) => {
    if (visibilityCache.has(el)) return visibilityCache.get(el);
    if (isHiddenInput(el)) return true;
    if (isStyleSuppressed(el)) {
      visibilityCache.set(el, false);
      return false;
    }
    const rect = renderedRect(el.getBoundingClientRect());
    const visible = hasPositiveArea(rect);
    visibilityCache.set(el, visible);
    return visible;
  };
  const topLevelModalRoots = () => {
    const candidates = Array.from(
      document.querySelectorAll('dialog, [role="dialog"], [role="alertdialog"], [aria-modal="true"]')
    );
    return candidates.filter((el) => {
      if (!isRendered(el)) return false;
      const doc = el.ownerDocument || document;
      return hasReachablePointInViewport(el, [renderedRect(el.getBoundingClientRect())], doc);
    }).filter((el, index, list) =>
      !list.some((other, otherIndex) => otherIndex !== index && other.contains(el))
    );
  };
  const isTextNodeVisible = (node) => {
    const parent = node.parentElement;
    if (!parent || !isRendered(parent)) return false;
    const doc = parent.ownerDocument || document;
    const range = doc.createRange();
    range.selectNodeContents(node);
    const rects = Array.from(range.getClientRects())
      .map(renderedRect)
      .filter(hasPositiveArea);
    return rects.length > 0;
  };
  const shouldCaptureTextNode = (node, mode) => {
    const parent = node.parentElement;
    if (!parent) return false;
    const visible = isTextNodeVisible(node);
    return mode === 'visible' ? visible : (!visible || isModalObscured(parent));
  };
  const nextId = (el, metadataFactory) => {
    const id = state.nextId++;
    el.__bowserCaptureId = id;
    window.__bowserElements.set(id, el);
    if (metadataFactory) {
      storeMetadata(id, metadataFactory(id));
    }
    return id;
  };
  const storeMetadata = (id, metadata) => {
    if (!metadata) return;
    window.__bowserMetadata.set(id, metadata);
    state.metadata.push(metadata);
  };
  const textNode = (text) => {
    const trimmed = text.replace(/\s+/g, ' ').trim();
    if (!trimmed) return null;
    return { kind: 'text', text: trimmed };
  };
  const mergeText = (children) => {
    const merged = [];
    for (const child of children) {
      if (!child) continue;
      const last = merged[merged.length - 1];
      if (last && last.kind === 'text' && child.kind === 'text') {
        last.text = `${last.text} ${child.text}`.trim();
      } else {
        merged.push(child);
      }
    }
    return merged;
  };
  const simpleText = (el) => textNode(typeof el.innerText === 'string' ? el.innerText : '');
  const labelFor = (el) => {
    const ariaLabel = (el.getAttribute('aria-label') || '').replace(/\s+/g, ' ').trim();
    if (ariaLabel) return ariaLabel;
    const labelledBy = (el.getAttribute('aria-labelledby') || '').trim();
    if (labelledBy) {
      const doc = el.ownerDocument || document;
      const text = labelledBy
        .split(/\s+/)
        .map((id) => {
          const label = doc.getElementById(id);
          return label && typeof label.innerText === 'string' ? label.innerText : '';
        })
        .join(' ')
        .replace(/\s+/g, ' ')
        .trim();
      if (text) return text;
    }
    return null;
  };
  const captureChildren = (el, mode) => {
    const children = [];
    for (const node of el.childNodes) {
      const captured = captureNode(node, mode);
      if (Array.isArray(captured)) children.push(...captured);
      else if (captured) children.push(captured);
    }
    return mergeText(children);
  };
  const captureCell = (cell, mode) => ({ children: captureChildren(cell, mode) });
  const captureNode = (node, mode) => {
    if (node.nodeType === Node.TEXT_NODE) {
      return shouldCaptureTextNode(node, mode) ? textNode(node.textContent || '') : null;
    }
    if (node.nodeType !== Node.ELEMENT_NODE) return null;
    const el = node;
    const tag = el.tagName.toLowerCase();
    const role = (el.getAttribute('role') || '').toLowerCase();
    if (['script', 'style', 'noscript', 'template'].includes(tag)) return null;
    if (mode === 'obscured' && isInsideModalRoot(el)) return null;
    if (mode === 'visible' && isStyleSuppressed(el) && !isHiddenInput(el)) return null;
    const visibleSelf = isRendered(el);
    const captureSelf = shouldCaptureElement(el, visibleSelf, mode);

    if (/^h[1-6]$/.test(tag)) {
      if (!captureSelf) return null;
      const text = simpleText(el);
      return text ? { kind: 'heading', level: Number(tag[1]), text: text.text } : null;
    }
    if (tag === 'a') {
      if (!captureSelf) return null;
      const text = (simpleText(el)?.text) || (el.getAttribute('aria-label') || '').trim() || '[link]';
      const href = el.href || el.getAttribute('href') || '';
      const focused = isFocused(el);
      const id = nextId(el, (id) => ({ kind: 'link', element_id: id, text, href, focused, visibility: null }));
      return { kind: 'link', id, text, href, focused };
    }
    if (tag === 'button' || role === 'button' || (tag === 'input' && ['submit', 'button'].includes((el.getAttribute('type') || '').toLowerCase()))) {
      if (!captureSelf) return null;
      const text = (simpleText(el)?.text) || el.getAttribute('value') || labelFor(el) || '[button]';
      const focused = isFocused(el);
      const id = nextId(el, (id) => ({ kind: 'button', element_id: id, text: text.trim(), focused, visibility: null }));
      return { kind: 'button', id, text: text.trim(), focused };
    }
    if (tag === 'input' || tag === 'textarea' || tag === 'select') {
      if (!captureSelf && !isHiddenInput(el)) return null;
      if (mode === 'obscured' && isHiddenInput(el) && !isModalObscured(el) && !isStyleSuppressed(el.parentElement || el)) return null;
      const rawType = tag === 'textarea' ? 'textarea' : (tag === 'select' ? 'select' : ((el.getAttribute('type') || 'text').toLowerCase()));
      const typeMap = new Set(['text', 'password', 'email', 'number', 'tel', 'url', 'search', 'textarea', 'select', 'checkbox', 'radio', 'date', 'file', 'hidden']);
      const inputType = typeMap.has(rawType) ? rawType : rawType;
      const focused = isFocused(el);
      const id = rawType === 'hidden'
        ? null
        : nextId(el, (id) => ({
            kind: 'input',
            element_id: id,
            name: el.getAttribute('name'),
            input_type: inputType,
            placeholder: el.getAttribute('placeholder'),
            value: tag === 'select'
              ? ((el.selectedOptions && el.selectedOptions[0] && el.selectedOptions[0].textContent) || '')
              : (el.value || ''),
            label: labelFor(el),
            options: tag === 'select' ? Array.from(el.options).map((option) => option.textContent || '') : [],
            focused,
            visibility: null
          }));
      return {
        kind: 'input',
        id,
        name: el.getAttribute('name'),
        input_type: inputType,
        placeholder: el.getAttribute('placeholder'),
        value: tag === 'select'
          ? ((el.selectedOptions && el.selectedOptions[0] && el.selectedOptions[0].textContent) || '')
          : (el.value || ''),
        label: labelFor(el),
        options: tag === 'select' ? Array.from(el.options).map((option) => option.textContent || '') : [],
        focused
      };
    }
    if (['checkbox', 'radio', 'switch'].includes(role)) {
      if (!captureSelf) return null;
      const inputType = role === 'radio' ? 'radio' : 'checkbox';
      const label = labelFor(el) || (simpleText(el)?.text) || null;
      const value = el.getAttribute('aria-checked') || '';
      const focused = isFocused(el);
      const id = nextId(el, (id) => ({
        kind: 'input',
        element_id: id,
        name: el.getAttribute('name'),
        input_type: inputType,
        placeholder: null,
        value,
        label,
        options: [],
        focused,
        visibility: null
      }));
      return {
        kind: 'input',
        id,
        name: el.getAttribute('name'),
        input_type: inputType,
        placeholder: null,
        value,
        label,
        options: [],
        focused
      };
    }
    if (tag === 'img' || tag === 'svg' || tag === 'picture' || el.getAttribute('role') === 'img') {
      if (!captureSelf) return null;
      const alt = el.getAttribute('alt') || el.getAttribute('aria-label') || '';
      const src = el.currentSrc || el.getAttribute('src') || '';
      const focused = isFocused(el);
      const id = nextId(el, (id) => ({ kind: 'image', element_id: id, alt, src, description: null, focused, visibility: null }));
      return { kind: 'image', id, alt, src, description: null, focused };
    }
    if (tag === 'table') {
      if (!captureSelf) return captureChildren(el, mode);
      const focused = isFocused(el);
      const id = nextId(el);
      const headerCells = Array.from(el.querySelectorAll('thead th')).map((cell) => captureCell(cell, mode));
      const bodyRows = Array.from(el.querySelectorAll('tbody tr'));
      const rows = (bodyRows.length ? bodyRows : Array.from(el.querySelectorAll('tr')))
        .map((row) => ({ cells: Array.from(row.children).filter((cell) => ['td', 'th'].includes(cell.tagName.toLowerCase())).map((cell) => captureCell(cell, mode)) }))
        .filter((row) => row.cells.length > 0);
      if (headerCells.length === 0 && rows.length === 0) return null;
      storeMetadata(id, {
        kind: 'table',
        element_id: id,
        headers: headerCells.length,
        rows: rows.length,
        focused,
        visibility: null
      });
      return { kind: 'table', id, headers: headerCells, rows, truncation: null, focused };
    }
    if (tag === 'ul' || tag === 'ol' || tag === 'dl') {
      if (!captureSelf) return captureChildren(el, mode);
      const focused = isFocused(el);
      const id = nextId(el);
      const items = Array.from(el.children)
        .filter((child) => ['li', 'dt', 'dd'].includes(child.tagName.toLowerCase()))
        .map((child) => ({ children: captureChildren(child, mode) }));
      if (items.length === 0) return null;
      const listType = tag === 'ol' ? 'ordered' : (tag === 'dl' ? 'description' : 'unordered');
      storeMetadata(id, {
        kind: 'list',
        element_id: id,
        list_type: listType,
        items: items.length,
        focused,
        visibility: null
      });
      return {
        kind: 'list',
        id,
        list_type: listType,
        items,
        truncation: null,
        focused
      };
    }
    if (tag === 'nav') {
      if (!captureSelf) return captureChildren(el, mode);
      const id = nextId(el);
      const children = captureChildren(el, mode);
      if (children.length === 0) return null;
      const focused = isFocused(el);
      storeMetadata(id, { kind: 'nav', element_id: id, children: children.length, focused, visibility: null });
      return { kind: 'nav', id, children, truncation: null, focused };
    }
    if (tag === 'form') {
      if (!captureSelf) return captureChildren(el, mode);
      const id = nextId(el);
      const children = captureChildren(el, mode);
      if (children.length === 0) return null;
      const focused = isFocused(el);
      const action = el.getAttribute('action');
      storeMetadata(id, { kind: 'form', element_id: id, action, children: children.length, focused, visibility: null });
      return {
        kind: 'form',
        id,
        action,
        children,
        truncation: null,
        focused
      };
    }
    if (
      tag === 'dialog' ||
      ['dialog', 'alertdialog'].includes((el.getAttribute('role') || '').toLowerCase()) ||
      el.getAttribute('aria-modal') === 'true'
    ) {
      if (!captureSelf) return captureChildren(el, mode);
      const id = nextId(el);
      const children = captureChildren(el, mode);
      if (children.length === 0) return null;
      const focused = isFocused(el);
      storeMetadata(id, { kind: 'section', element_id: id, tag: 'dialog', children: children.length, focused, visibility: null });
      return {
        kind: 'section',
        id,
        tag: 'dialog',
        children,
        truncation: null,
        focused
      };
    }
    if (['section', 'article', 'aside', 'main', 'header', 'footer'].includes(tag)) {
      if (!captureSelf) return captureChildren(el, mode);
      const id = nextId(el);
      const children = captureChildren(el, mode);
      if (children.length === 0) return null;
      const focused = isFocused(el);
      storeMetadata(id, { kind: 'section', element_id: id, tag, children: children.length, focused, visibility: null });
      return { kind: 'section', id, tag, children, truncation: null, focused };
    }
    if (tag === 'iframe') {
      if (!captureSelf) return captureChildren(el, mode);
      const id = nextId(el);
      let children = [];
      try {
        if (el.contentDocument && el.contentDocument.body) {
          children = captureChildren(el.contentDocument.body, mode);
        }
      } catch (_) {}
      if (children.length === 0 && !(el.getAttribute('src') || '')) return null;
      const focused = isFocused(el);
      const src = el.getAttribute('src') || '';
      storeMetadata(id, { kind: 'iframe', element_id: id, src, children: children.length, focused, visibility: null });
      return {
        kind: 'iframe',
        id,
        src,
        children,
        truncation: null,
        focused
      };
    }

    const children = captureChildren(el, mode);
    if (children.length === 0) {
      return captureSelf ? simpleText(el) : null;
    }
    if (!captureSelf && children.length === 1) {
      return children[0];
    }
    if (!captureSelf) return children;
    return children;
  };

  activeModalRoots = topLevelModalRoots();
  const captureRoot = (mode) => {
    const root = [];
    const sourceNodes = mode === 'visible' && activeModalRoots.length > 0
      ? activeModalRoots
      : Array.from(document.body ? document.body.childNodes : document.childNodes);
    for (const child of sourceNodes) {
      const captured = captureNode(child, mode);
      if (Array.isArray(captured)) root.push(...captured);
      else if (captured) root.push(captured);
    }
    return mergeText(root);
  };
  const visibleRoot = captureRoot('visible');
  const obscuredRoot = captureRoot('obscured');
  let bodyId = null;
  let obscuredBodyId = null;
  if (document.body) {
    bodyId = state.nextId++;
    obscuredBodyId = state.nextId++;
    window.__bowserElements.set(bodyId, document.body);
    window.__bowserElements.set(obscuredBodyId, document.body);
    storeMetadata(bodyId, {
      kind: 'section',
      element_id: bodyId,
      tag: 'body',
      children: visibleRoot.length,
      focused: isFocused(document.body),
      visibility: null
    });
    storeMetadata(obscuredBodyId, {
      kind: 'section',
      element_id: obscuredBodyId,
      tag: 'body',
      children: obscuredRoot.length,
      focused: isFocused(document.body),
      visibility: null
    });
  }

  return {
    url: window.location.href,
    title: document.title || '',
    body_id: bodyId,
    obscured_body_id: obscuredBodyId,
    content: {
      visible: visibleRoot,
      obscured: obscuredRoot
    },
    metadata: state.metadata
  };
})()
