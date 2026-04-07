import assert from "node:assert/strict";
import test from "node:test";

import { localizeMenuModel } from "./menu-model.mjs";
import {
  findMenuCommandByShortcut,
  shortcutMatchesEvent,
  shouldHandleShortcutEvent
} from "./keyboard-shortcuts.mjs";

test("shortcutMatchesEvent matches modifier shortcuts and function keys", () => {
  assert.equal(
    shortcutMatchesEvent("Ctrl+Shift+N", {
      key: "N",
      ctrlKey: true,
      shiftKey: true,
      altKey: false,
      metaKey: false
    }),
    true
  );

  assert.equal(
    shortcutMatchesEvent("F4", {
      key: "F4",
      ctrlKey: false,
      shiftKey: false,
      altKey: false,
      metaKey: false
    }),
    true
  );
});

test("shortcutMatchesEvent ignores unsupported chord shortcuts", () => {
  assert.equal(
    shortcutMatchesEvent("Ctrl+R,A", {
      key: "a",
      ctrlKey: true,
      shiftKey: false,
      altKey: false,
      metaKey: false
    }),
    false
  );
});

test("findMenuCommandByShortcut resolves the matching menu command", () => {
  const menuModel = localizeMenuModel("fr");
  const match = findMenuCommandByShortcut(menuModel, {
    key: "o",
    ctrlKey: true,
    shiftKey: false,
    altKey: false,
    metaKey: false
  });

  assert.deepEqual(match, {
    menuId: "file",
    commandId: "project.open",
    shortcut: "Ctrl+O"
  });
});

test("shouldHandleShortcutEvent keeps function keys and modified keys inside editable controls", () => {
  const textareaTarget = {
    tagName: "TEXTAREA",
    isContentEditable: false
  };

  assert.equal(
    shouldHandleShortcutEvent({
      key: "s",
      ctrlKey: true,
      altKey: false,
      metaKey: false,
      repeat: false,
      defaultPrevented: false,
      target: textareaTarget
    }),
    true
  );

  assert.equal(
    shouldHandleShortcutEvent({
      key: "F4",
      ctrlKey: false,
      altKey: false,
      metaKey: false,
      repeat: false,
      defaultPrevented: false,
      target: textareaTarget
    }),
    true
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
        isContentEditable: false
      }
    }),
    false
  );
});
