import test from "node:test";
import assert from "node:assert/strict";

import { getAllMenuCommands, getTopLevelMenuLabels, visualStudioInspiredMenus } from "./menu-model.mjs";

test("top level menu order stays close to Visual Studio", () => {
  assert.deepEqual(getTopLevelMenuLabels(), [
    "File",
    "Edit",
    "View",
    "Git",
    "Project",
    "Build",
    "Debug",
    "Test",
    "Analyze",
    "Tools",
    "Window",
    "Help"
  ]);
});

test("all actionable menu items expose a command id", () => {
  for (const menu of visualStudioInspiredMenus) {
    for (const item of menu.items) {
      if (item.type === "separator") {
        continue;
      }

      assert.equal(typeof item.command, "string");
      assert.ok(item.command.length > 0);
    }
  }
});

test("menu commands stay unique to avoid ambiguous routing", () => {
  const commands = getAllMenuCommands();
  const uniqueCommands = new Set(commands);

  assert.equal(commands.length, uniqueCommands.size);
});

