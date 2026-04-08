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
  const collapsed = toggleWorkspacePanel(defaultWorkspacePanels, "projectExplorer");

  assert.equal(collapsed.projectExplorer, false);
  assert.equal(collapsed.output, true);
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
  const leftClosed = getWorkspaceColumnState({
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

  assert.deepEqual(bothOpen, { leftExpanded: true, rightExpanded: true });
  assert.deepEqual(leftClosed, { leftExpanded: false, rightExpanded: true });
  assert.deepEqual(rightClosed, { leftExpanded: true, rightExpanded: false });
});
