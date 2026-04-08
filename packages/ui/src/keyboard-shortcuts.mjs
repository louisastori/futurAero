function normalizeShortcutToken(token) {
  const normalized = String(token ?? "")
    .trim()
    .toLowerCase();

  if (normalized === "control") {
    return "ctrl";
  }

  if (normalized === "commandorcontrol" || normalized === "cmdorctrl") {
    return "mod";
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
  const raw = String(key ?? "");
  if (raw === " ") {
    return "space";
  }

  const normalized = raw.trim().toLowerCase();

  if (normalized === "") {
    return "";
  }

  if (normalized === "spacebar") {
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

function detectShortcutPlatform(platformOverride) {
  const platform =
    platformOverride ??
    globalThis.navigator?.userAgentData?.platform ??
    globalThis.navigator?.platform ??
    "";
  return String(platform).toLowerCase().includes("mac") ? "mac" : "default";
}

function matchSingleShortcut(shortcut, event) {
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
  let expectedMod = false;
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

    if (token === "mod") {
      expectedMod = true;
      continue;
    }

    expectedKey = token;
  }

  if (!expectedKey) {
    return false;
  }

  const ctrlMatches = expectedMod
    ? Boolean(event?.ctrlKey || event?.metaKey)
    : Boolean(event?.ctrlKey) === expectedCtrl &&
      Boolean(event?.metaKey) === expectedMeta;

  return (
    ctrlMatches &&
    Boolean(event?.shiftKey) === expectedShift &&
    Boolean(event?.altKey) === expectedAlt &&
    normalizeEventKey(event?.key) === expectedKey
  );
}

export function formatShortcutLabel(shortcut, platformOverride) {
  if (!shortcut) {
    return "";
  }

  const platform = detectShortcutPlatform(platformOverride);

  return shortcut
    .split("/")
    .map((candidate) =>
      candidate
        .split("+")
        .map((token) => {
          const normalized = normalizeShortcutToken(token);
          if (normalized === "mod") {
            return platform === "mac" ? "Cmd" : "Ctrl";
          }

          if (normalized === "ctrl") {
            return "Ctrl";
          }

          if (normalized === "cmd" || normalized === "meta") {
            return "Cmd";
          }

          if (normalized === "alt") {
            return "Alt";
          }

          if (normalized === "shift") {
            return "Shift";
          }

          if (normalized === "delete") {
            return "Del";
          }

          if (normalized === "space") {
            return "Space";
          }

          if (/^f\d{1,2}$/.test(normalized)) {
            return normalized.toUpperCase();
          }

          return normalized.length === 1
            ? normalized.toUpperCase()
            : normalized.charAt(0).toUpperCase() + normalized.slice(1);
        })
        .join("+"),
    )
    .join(" / ");
}

export function shortcutMatchesEvent(shortcut, event) {
  if (!shortcut || shortcut.includes(",")) {
    return false;
  }

  const candidates = shortcut
    .split("/")
    .map((candidate) => candidate.trim())
    .filter(Boolean);

  return candidates.some((candidate) => matchSingleShortcut(candidate, event));
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
          shortcut: item.shortcut,
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
