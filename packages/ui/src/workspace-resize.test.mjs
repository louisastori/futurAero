import test from "node:test";
import assert from "node:assert/strict";

import {
  calculateResizedDockWidths,
  defaultWorkspaceDockWidths,
  getVisibleSidebarWidth,
  WORKSPACE_COLLAPSED_WIDTH,
  workspaceDockWidthLimits
} from "./workspace-resize.mjs";

test("left dock width grows and stays within configured limits", () => {
  const result = calculateResizedDockWidths({
    side: "left",
    startWidths: defaultWorkspaceDockWidths,
    deltaX: 80,
    layoutWidth: 1600,
    leftExpanded: true,
    rightExpanded: true
  });

  assert.equal(result.left, defaultWorkspaceDockWidths.left + 80);
  assert.equal(result.right, defaultWorkspaceDockWidths.right);
});

test("left dock width is clamped by layout center minimum", () => {
  const result = calculateResizedDockWidths({
    side: "left",
    startWidths: defaultWorkspaceDockWidths,
    deltaX: 1000,
    layoutWidth: 1100,
    leftExpanded: true,
    rightExpanded: true
  });

  assert.equal(result.left, 280);
});

test("right dock width expands when dragging the separator to the left", () => {
  const result = calculateResizedDockWidths({
    side: "right",
    startWidths: defaultWorkspaceDockWidths,
    deltaX: -90,
    layoutWidth: 1600,
    leftExpanded: true,
    rightExpanded: true
  });

  assert.equal(result.right, defaultWorkspaceDockWidths.right + 90);
  assert.equal(result.left, defaultWorkspaceDockWidths.left);
});

test("right dock width is clamped to its configured max when layout is wide", () => {
  const result = calculateResizedDockWidths({
    side: "right",
    startWidths: defaultWorkspaceDockWidths,
    deltaX: -1000,
    layoutWidth: 2200,
    leftExpanded: true,
    rightExpanded: true
  });

  assert.equal(result.right, workspaceDockWidthLimits.right.max);
});

test("collapsed docks use the compact width", () => {
  assert.equal(getVisibleSidebarWidth(320, true), 320);
  assert.equal(getVisibleSidebarWidth(320, false), WORKSPACE_COLLAPSED_WIDTH);
});
