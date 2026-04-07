function normalizeShortcutToken(token) {
  const normalized = String(token ?? "")
    .trim()
    .toLowerCase();

  if (normalized === "control") {
    return "ctrl";
  }

  if (normalized === "del") {
    return "delete";
  }

  if (normalized === "esc") {
    return "escape";
  }

  if (normalized === "return") {
    return "enter";
  }

  return normalized;
}

function normalizeEventKey(key) {
  const normalized = String(key ?? "")
    .trim()
    .toLowerCase();

  if (normalized === "") {
    return "";
  }

  if (normalized === " ") {
    return "space";
  }

  return normalizeShortcutToken(normalized);
}

function isEditableShortcutTarget(target) {
  if (!target || typeof target !== "object") {
    return false;
  }

  const tagName = String(target.tagName ?? "").toLowerCase();
  return (
    target.isContentEditable === true ||
    tagName === "input" ||
    tagName === "textarea" ||
    tagName === "select"
  );
}

export function shortcutMatchesEvent(shortcut, event) {
  if (!shortcut || shortcut.includes(",")) {
    return false;
  }

  const tokens = shortcut
    .split("+")
    .map((token) => normalizeShortcutToken(token))
    .filter(Boolean);

  if (tokens.length === 0) {
    return false;
  }

  let expectedCtrl = false;
  let expectedShift = false;
  let expectedAlt = false;
  let expectedMeta = false;
  let expectedKey = "";

  for (const token of tokens) {
    if (token === "ctrl") {
      expectedCtrl = true;
      continue;
    }

    if (token === "shift") {
      expectedShift = true;
      continue;
    }

    if (token === "alt") {
      expectedAlt = true;
      continue;
    }

    if (token === "meta" || token === "cmd") {
      expectedMeta = true;
      continue;
    }

    expectedKey = token;
  }

  if (!expectedKey) {
    return false;
  }

  return (
    Boolean(event?.ctrlKey) === expectedCtrl &&
    Boolean(event?.shiftKey) === expectedShift &&
    Boolean(event?.altKey) === expectedAlt &&
    Boolean(event?.metaKey) === expectedMeta &&
    normalizeEventKey(event?.key) === expectedKey
  );
}

export function findMenuCommandByShortcut(menuModel, event) {
  for (const menu of menuModel ?? []) {
    for (const item of menu.items ?? []) {
      if (item.type === "separator" || !item.shortcut) {
        continue;
      }

      if (shortcutMatchesEvent(item.shortcut, event)) {
        return {
          menuId: menu.id,
          commandId: item.command,
          shortcut: item.shortcut
        };
      }
    }
  }

  return null;
}

export function shouldHandleShortcutEvent(event) {
  if (!event || event.defaultPrevented || event.repeat) {
    return false;
  }

  const eventKey = normalizeEventKey(event.key);
  const functionKey = /^f\d{1,2}$/.test(eventKey);
  const hasModifier = Boolean(event.ctrlKey || event.altKey || event.metaKey);

  if (isEditableShortcutTarget(event.target) && !hasModifier && !functionKey) {
    return false;
  }

  return true;
}
