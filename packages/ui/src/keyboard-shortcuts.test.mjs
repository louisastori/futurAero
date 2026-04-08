import assert from "node:assert/strict";
import test from "node:test";

import { localizeMenuModel } from "./menu-model.mjs";
import {
  formatShortcutLabel,
  findMenuCommandByShortcut,
  shortcutMatchesEvent,
  shouldHandleShortcutEvent,
} from "./keyboard-shortcuts.mjs";

test("shortcutMatchesEvent matches Mod shortcuts on Windows and function keys", () => {
  assert.equal(
    shortcutMatchesEvent("Mod+Shift+N", {
      key: "N",
      ctrlKey: true,
      shiftKey: true,
      altKey: false,
      metaKey: false,
    }),
    true,
  );

  assert.equal(
    shortcutMatchesEvent("F4", {
      key: "F4",
      ctrlKey: false,
      shiftKey: false,
      altKey: false,
      metaKey: false,
    }),
    true,
  );
});

test("shortcutMatchesEvent also matches Mod shortcuts on macOS", () => {
  assert.equal(
    shortcutMatchesEvent("Mod+S", {
      key: "s",
      ctrlKey: false,
      shiftKey: false,
      altKey: false,
      metaKey: true,
    }),
    true,
  );
});

test("shortcutMatchesEvent ignores unsupported chord shortcuts", () => {
  assert.equal(
    shortcutMatchesEvent("Ctrl+R,A", {
      key: "a",
      ctrlKey: true,
      shiftKey: false,
      altKey: false,
      metaKey: false,
    }),
    false,
  );
});

test("findMenuCommandByShortcut resolves the matching menu command", () => {
  const menuModel = localizeMenuModel("fr");
  const match = findMenuCommandByShortcut(menuModel, {
    key: "o",
    ctrlKey: true,
    shiftKey: false,
    altKey: false,
    metaKey: false,
  });

  assert.deepEqual(match, {
    menuId: "file",
    commandId: "project.open",
    shortcut: "Mod+O",
  });
});

test("findMenuCommandByShortcut resolves ctrl space for the AI input command", () => {
  const menuModel = localizeMenuModel("fr");
  const match = findMenuCommandByShortcut(menuModel, {
    key: " ",
    ctrlKey: true,
    shiftKey: false,
    altKey: false,
    metaKey: false,
  });

  assert.deepEqual(match, {
    menuId: "ai",
    commandId: "ai.focus_input",
    shortcut: "Ctrl+Space",
  });
});

test("formatShortcutLabel renders platform aware labels", () => {
  assert.equal(formatShortcutLabel("Mod+S", "Win32"), "Ctrl+S");
  assert.equal(formatShortcutLabel("Mod+S", "MacIntel"), "Cmd+S");
  assert.equal(formatShortcutLabel("Ctrl+Space"), "Ctrl+Space");
});

test("shouldHandleShortcutEvent keeps function keys and modified keys inside editable controls", () => {
  const textareaTarget = {
    tagName: "TEXTAREA",
    isContentEditable: false,
  };

  assert.equal(
    shouldHandleShortcutEvent({
      key: "s",
      ctrlKey: true,
      altKey: false,
      metaKey: false,
      repeat: false,
      defaultPrevented: false,
      target: textareaTarget,
    }),
    true,
  );

  assert.equal(
    shouldHandleShortcutEvent({
      key: "F4",
      ctrlKey: false,
      altKey: false,
      metaKey: false,
      repeat: false,
      defaultPrevented: false,
      target: textareaTarget,
    }),
    true,
  );
});

test("shouldHandleShortcutEvent ignores plain typing inside editable controls", () => {
  assert.equal(
    shouldHandleShortcutEvent({
      key: "a",
      ctrlKey: false,
      altKey: false,
      metaKey: false,
      repeat: false,
      defaultPrevented: false,
      target: {
        tagName: "INPUT",
        isContentEditable: false,
      },
    }),
    false,
  );
});
