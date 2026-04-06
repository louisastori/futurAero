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
  defaultWorkspacePanels,
  getWorkspaceColumnState,
  panelIdFromCommand,
  setWorkspacePanel,
  toggleWorkspacePanel
} from "./workspace-layout.mjs";

export const uiScaffoldStatus = "menu-model-ready";
