(() => {
  const pageStartedAtMs = Date.now();
  const stage = document.getElementById("stage");
  const eventCount = document.getElementById("event-count");
  const targetCount = document.getElementById("target-count");
  const pointerPosition = document.getElementById("pointer-position");
  const syncState = document.getElementById("sync-state");
  const queue = [];
  let totalEvents = 0;
  let totalBatches = 0;
  let totalTargets = 0;
  let flushTimer = null;
  let lastMoveAt = 0;
  const visualLayer = document.createElement("div");
  visualLayer.className = "cursor-visuals";
  visualLayer.setAttribute("aria-hidden", "true");
  const cursorVisual = document.createElement("div");
  cursorVisual.className = "cursor-visual";
  visualLayer.appendChild(cursorVisual);
  document.body.appendChild(visualLayer);
  const trailPoints = [];
  const trailSegments = [];
  const maxTrailPoints = 22;

  const trim = (value, max = 160) => {
    if (!value) return null;
    const normalized = String(value).replace(/\s+/g, " ").trim();
    return normalized ? normalized.slice(0, max) : null;
  };

  const viewport = () => ({
    width: Math.max(window.innerWidth || 0, 1),
    height: Math.max(window.innerHeight || 0, 1),
    device_pixel_ratio: window.devicePixelRatio || 1
  });

  const targetInfo = (target) => {
    const element = target && target.nodeType === Node.ELEMENT_NODE
      ? target.closest("[data-pointer-target], button, input, select, textarea, [role], a, main")
      : document.body;
    return {
      tag: element ? element.tagName.toLowerCase() : "unknown",
      id: element ? trim(element.id) : null,
      role: element ? trim(element.getAttribute("role")) : null,
      label: element ? trim(element.innerText || element.value || element.getAttribute("aria-label")) : null,
      classes: element ? trim(element.className) : null
    };
  };

  const targetRectInfo = (target) => {
    const element = target && target.nodeType === Node.ELEMENT_NODE
      ? target.closest("[data-pointer-target], button, input, select, textarea, [role], a, main")
      : null;
    if (!element) return null;
    const rect = element.getBoundingClientRect();
    return {
      left: Math.round(rect.left),
      top: Math.round(rect.top),
      width: Math.round(rect.width),
      height: Math.round(rect.height)
    };
  };

  const scheduleFlush = () => {
    if (flushTimer) return;
    flushTimer = window.setTimeout(() => flush(false), 900);
  };

  const enqueue = (record) => {
    queue.push(record);
    totalEvents += 1;
    eventCount.textContent = String(totalEvents);
    scheduleFlush();
    if (queue.length >= 128) flush(false);
  };

  const redrawTrail = () => {
    const required = Math.max(trailPoints.length - 1, 0);
    while (trailSegments.length < required) {
      const segment = document.createElement("div");
      segment.className = "cursor-trail-segment";
      visualLayer.appendChild(segment);
      trailSegments.push(segment);
    }
    trailSegments.forEach((segment, index) => {
      if (index >= required) {
        segment.style.display = "none";
        return;
      }
      const start = trailPoints[index];
      const end = trailPoints[index + 1];
      const dx = end.x - start.x;
      const dy = end.y - start.y;
      const length = Math.hypot(dx, dy);
      const angle = Math.atan2(dy, dx);
      const age = index / Math.max(required - 1, 1);
      segment.style.display = "block";
      segment.style.width = `${length}px`;
      segment.style.opacity = String(0.18 + age * 0.72);
      segment.style.transform = `translate(${start.x}px, ${start.y}px) rotate(${angle}rad)`;
    });
  };

  const updatePointerVisual = (event) => {
    if (!Number.isFinite(event.clientX) || !Number.isFinite(event.clientY)) return;
    const point = { x: event.clientX, y: event.clientY };
    const last = trailPoints[trailPoints.length - 1];
    cursorVisual.style.opacity = "1";
    cursorVisual.style.transform = `translate(${point.x}px, ${point.y}px) translate(-50%, -50%)`;
    if (!last || Math.hypot(point.x - last.x, point.y - last.y) >= 2) {
      trailPoints.push(point);
      while (trailPoints.length > maxTrailPoints) trailPoints.shift();
      redrawTrail();
    }
  };

  const animateClick = (event) => {
    if (!Number.isFinite(event.clientX) || !Number.isFinite(event.clientY)) return;
    const dot = document.createElement("div");
    dot.className = "click-dot";
    dot.style.transform = `translate(${event.clientX}px, ${event.clientY}px) translate(-50%, -50%)`;
    visualLayer.appendChild(dot);
    dot.addEventListener("animationend", () => dot.remove(), { once: true });
  };

  const baseRecord = (event, kind) => ({
    kind,
    time_ms: Math.round(performance.timeOrigin + event.timeStamp),
    elapsed_ms: Math.round(performance.now()),
    x: Number.isFinite(event.clientX) ? event.clientX : null,
    y: Number.isFinite(event.clientY) ? event.clientY : null,
    movement_x: Number.isFinite(event.movementX) ? event.movementX : null,
    movement_y: Number.isFinite(event.movementY) ? event.movementY : null,
    delta_x: Number.isFinite(event.deltaX) ? event.deltaX : null,
    delta_y: Number.isFinite(event.deltaY) ? event.deltaY : null,
    button: Number.isFinite(event.button) ? event.button : null,
    buttons: Number.isFinite(event.buttons) ? event.buttons : null,
    pointer_type: trim(event.pointerType),
    target: targetInfo(event.target),
    target_rect: targetRectInfo(event.target)
  });

  const recordPointer = (event, kind) => {
    updatePointerVisual(event);
    if (kind === "pointer_down") animateClick(event);
    if (kind === "pointer_move") {
      const now = performance.now();
      if (now - lastMoveAt < 8) return;
      lastMoveAt = now;
      pointerPosition.textContent = `${Math.round(event.clientX)}, ${Math.round(event.clientY)}`;
    }
    enqueue(baseRecord(event, kind));
  };

  const recordLifecycle = (kind) => {
    enqueue({
      kind,
      time_ms: Date.now(),
      elapsed_ms: Math.round(performance.now()),
      x: null,
      y: null,
      movement_x: null,
      movement_y: null,
      delta_x: null,
      delta_y: null,
      button: null,
      buttons: null,
      pointer_type: null,
      target: targetInfo(document.body),
      target_rect: null
    });
  };

  async function flush(useBeacon) {
    if (flushTimer) {
      window.clearTimeout(flushTimer);
      flushTimer = null;
    }
    if (!queue.length) return;
    const events = queue.splice(0, queue.length);
    const body = JSON.stringify({
      page_started_at_ms: pageStartedAtMs,
      viewport: viewport(),
      events
    });
    syncState.textContent = "syncing";
    if (useBeacon && navigator.sendBeacon) {
      navigator.sendBeacon("/events", new Blob([body], { type: "application/json" }));
      totalBatches += 1;
      syncState.textContent = `saved ${totalBatches}`;
      return;
    }
    try {
      const response = await fetch("/events", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body
      });
      if (!response.ok) throw new Error(String(response.status));
      totalBatches += 1;
      syncState.textContent = `saved ${totalBatches}`;
    } catch (_error) {
      queue.unshift(...events);
      syncState.textContent = "retry";
      scheduleFlush();
    }
  }
  window.__pointerLogFlush = () => flush(false);

  const randomItem = (items) => items[Math.floor(Math.random() * items.length)];

  const createTarget = () => {
    const kind = randomItem(["button", "input", "checkbox", "select", "textarea", "div", "link"]);
    const tag = kind === "div" ? "div" : kind === "link" ? "a" : kind === "checkbox" ? "input" : kind;
    const element = document.createElement(tag);
    element.className = "target";
    element.dataset.pointerTarget = "true";
    element.id = `target-${totalTargets + 1}`;
    element.setAttribute("aria-label", kind);

    if (kind === "button") {
      element.type = "button";
      element.textContent = "Button";
    } else if (kind === "input") {
      element.type = "text";
      element.value = "Input";
    } else if (kind === "checkbox") {
      element.type = "checkbox";
    } else if (kind === "select") {
      element.innerHTML = "<option>Select</option><option>Alpha</option><option>Beta</option>";
    } else if (kind === "textarea") {
      element.value = "Textarea";
    } else if (kind === "link") {
      element.href = "#";
      element.textContent = "Link";
    } else {
      element.role = "button";
      element.tabIndex = 0;
      element.textContent = "Element";
    }

    element.addEventListener("click", (event) => {
      event.preventDefault();
      window.setTimeout(spawnTarget, 0);
    });
    return element;
  };

  const placeTarget = (element) => {
    const stageRect = stage.getBoundingClientRect();
    const targetRect = element.getBoundingClientRect();
    const maxX = Math.max(stageRect.width - targetRect.width - 16, 0);
    const maxY = Math.max(stageRect.height - targetRect.height - 16, 0);
    element.style.left = `${8 + Math.round(Math.random() * maxX)}px`;
    element.style.top = `${8 + Math.round(Math.random() * maxY)}px`;
  };

  const spawnTarget = () => {
    stage.replaceChildren();
    const element = createTarget();
    stage.appendChild(element);
    totalTargets += 1;
    targetCount.textContent = String(totalTargets);
    requestAnimationFrame(() => {
      placeTarget(element);
      enqueue({
        kind: "target_spawn",
        time_ms: Date.now(),
        elapsed_ms: Math.round(performance.now()),
        x: null,
        y: null,
        movement_x: null,
        movement_y: null,
        delta_x: null,
        delta_y: null,
        button: null,
        buttons: null,
        pointer_type: null,
        target: targetInfo(element),
        target_rect: targetRectInfo(element)
      });
    });
  };

  document.addEventListener("pointermove", (event) => recordPointer(event, "pointer_move"), true);
  document.addEventListener("mousemove", updatePointerVisual, true);
  document.addEventListener("pointerdown", (event) => recordPointer(event, "pointer_down"), true);
  document.addEventListener("pointerup", (event) => recordPointer(event, "pointer_up"), true);
  document.addEventListener("click", (event) => recordPointer(event, "click"), true);
  document.addEventListener("wheel", (event) => recordPointer(event, "wheel"), { capture: true, passive: true });
  document.addEventListener("visibilitychange", () => recordLifecycle("visibility_change"), true);
  window.addEventListener("pagehide", () => {
    recordLifecycle("page_hide");
    flush(true);
  });
  window.addEventListener("resize", spawnTarget);
  spawnTarget();
})();
