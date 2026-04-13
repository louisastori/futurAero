export const defaultWorkspacePanels = {
  projectExplorer: false,
  properties: false,
  commandSurface: false,
  viewport: true,
  simulationTimeline: true,
  aiAssistant: true,
  output: true,
  problems: true
};

const viewCommandToPanelId = {
  "view.project_explorer": "projectExplorer",
  "view.properties": "properties",
  "view.output": "output",
  "view.problems": "problems",
  "view.ai_assistant": "aiAssistant",
  "view.viewport_3d": "viewport",
  "view.simulation_timeline": "simulationTimeline"
};

export function panelIdFromCommand(commandId) {
  return viewCommandToPanelId[commandId] ?? null;
}

export function toggleWorkspacePanel(panelState, panelId) {
  if (!(panelId in panelState)) {
    return panelState;
  }

  return {
    ...panelState,
    [panelId]: !panelState[panelId]
  };
}

export function setWorkspacePanel(panelState, panelId, visible) {
  if (!(panelId in panelState)) {
    return panelState;
  }

  return {
    ...panelState,
    [panelId]: visible
  };
}

export function getWorkspaceColumnState(panelState) {
  return {
    leftExpanded: false,
    rightExpanded:
      panelState.simulationTimeline ||
      panelState.aiAssistant ||
      panelState.output ||
      panelState.problems
  };
}
