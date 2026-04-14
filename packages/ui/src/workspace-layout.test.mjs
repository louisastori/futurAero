import test from "node:test";
import assert from "node:assert/strict";

import {
  defaultWorkspacePanels,
  getWorkspaceColumnState,
  panelIdFromCommand,
  setWorkspacePanel,
  toggleWorkspacePanel
} from "./workspace-layout.mjs";

test("view commands map to the expected workspace panels", () => {
  assert.equal(panelIdFromCommand("view.project_explorer"), "projectExplorer");
  assert.equal(panelIdFromCommand("view.properties"), "properties");
  assert.equal(panelIdFromCommand("view.output"), "output");
  assert.equal(panelIdFromCommand("view.problems"), "problems");
  assert.equal(panelIdFromCommand("view.ai_assistant"), "aiAssistant");
  assert.equal(panelIdFromCommand("view.viewport_3d"), "viewport");
  assert.equal(
    panelIdFromCommand("view.simulation_timeline"),
    "simulationTimeline"
  );
  assert.equal(panelIdFromCommand("view.jobs"), null);
});

test("toggleWorkspacePanel flips only known panel ids", () => {
  const expanded = toggleWorkspacePanel(defaultWorkspacePanels, "projectExplorer");

  assert.equal(expanded.projectExplorer, true);
  assert.equal(expanded.output, true);
  assert.equal(
    toggleWorkspacePanel(defaultWorkspacePanels, "unknown"),
    defaultWorkspacePanels
  );
});

test("setWorkspacePanel can reopen a previously collapsed panel", () => {
  const collapsed = setWorkspacePanel(defaultWorkspacePanels, "aiAssistant", false);
  const reopened = setWorkspacePanel(collapsed, "aiAssistant", true);

  assert.equal(collapsed.aiAssistant, false);
  assert.equal(reopened.aiAssistant, true);
});

test("column state reports when left or right docks are fully collapsed", () => {
  const bothOpen = getWorkspaceColumnState(defaultWorkspacePanels);
  const leftOpen = getWorkspaceColumnState({
    ...defaultWorkspacePanels,
    projectExplorer: true
  });
  const leftStillClosed = getWorkspaceColumnState({
    ...defaultWorkspacePanels,
    projectExplorer: false,
    properties: false
  });
  const rightClosed = getWorkspaceColumnState({
    ...defaultWorkspacePanels,
    simulationTimeline: false,
    aiAssistant: false,
    output: false,
    problems: false
  });

  assert.deepEqual(bothOpen, { leftExpanded: false, rightExpanded: true });
  assert.deepEqual(leftOpen, { leftExpanded: false, rightExpanded: true });
  assert.deepEqual(leftStillClosed, { leftExpanded: false, rightExpanded: true });
  assert.deepEqual(rightClosed, { leftExpanded: false, rightExpanded: false });
});
