export {
  getAllMenuCommands,
  getTopLevelMenuLabels,
  localizeMenuModel,
  visualStudioInspiredMenus
} from "./menu-model.mjs";
export {
  defaultLocale,
  hasTranslation,
  supportedLocales,
  translate
} from "./i18n.mjs";
export {
  findMenuCommandByShortcut,
  shortcutMatchesEvent,
  shouldHandleShortcutEvent
} from "./keyboard-shortcuts.mjs";
export {
  defaultWorkspacePanels,
  getWorkspaceColumnState,
  panelIdFromCommand,
  setWorkspacePanel,
  toggleWorkspacePanel
} from "./workspace-layout.mjs";
export {
  calculateResizedDockWidths,
  defaultWorkspaceDockWidths,
  getVisibleSidebarWidth,
  WORKSPACE_COLLAPSED_WIDTH,
  WORKSPACE_RESIZER_WIDTH,
  workspaceDockWidthLimits
} from "./workspace-resize.mjs";

export const uiScaffoldStatus = "menu-model-ready";
