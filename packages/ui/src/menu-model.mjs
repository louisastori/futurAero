import { defaultLocale, translate } from "./i18n.mjs";

const menuDefinitions = [
  {
    id: "file",
    label: "File",
    translationKey: "menu.file",
    items: [
      {
        id: "file.new_project",
        label: "New Project",
        translationKey: "menu_item.file.new_project",
        command: "project.create",
        shortcut: "Mod+Shift+N",
      },
      {
        id: "file.open_project",
        label: "Open Project",
        translationKey: "menu_item.file.open_project",
        command: "project.open",
        shortcut: "Mod+O",
      },
      {
        id: "file.open_recent",
        label: "Open Recent",
        translationKey: "menu_item.file.open_recent",
        command: "project.open_recent",
      },
      { type: "separator" },
      {
        id: "file.save",
        label: "Save",
        translationKey: "menu_item.file.save",
        command: "project.save",
        shortcut: "Mod+S",
      },
      {
        id: "file.save_all",
        label: "Save All",
        translationKey: "menu_item.file.save_all",
        command: "project.save_all",
        shortcut: "Mod+Shift+S",
      },
      { type: "separator" },
      {
        id: "file.import",
        label: "Import",
        translationKey: "menu_item.file.import",
        command: "project.import",
      },
      {
        id: "file.export",
        label: "Export",
        translationKey: "menu_item.file.export",
        command: "project.export",
      },
      { type: "separator" },
      {
        id: "file.settings",
        label: "Settings",
        translationKey: "menu_item.file.settings",
        command: "app.settings",
      },
      {
        id: "file.exit",
        label: "Exit",
        translationKey: "menu_item.file.exit",
        command: "app.exit",
      },
    ],
  },
  {
    id: "edit",
    label: "Edit",
    translationKey: "menu.edit",
    items: [
      {
        id: "edit.undo",
        label: "Undo",
        translationKey: "menu_item.edit.undo",
        command: "history.undo",
        shortcut: "Mod+Z",
      },
      {
        id: "edit.redo",
        label: "Redo",
        translationKey: "menu_item.edit.redo",
        command: "history.redo",
        shortcut: "Mod+Y",
      },
      { type: "separator" },
      {
        id: "edit.cut",
        label: "Cut",
        translationKey: "menu_item.edit.cut",
        command: "selection.cut",
        shortcut: "Mod+X",
      },
      {
        id: "edit.copy",
        label: "Copy",
        translationKey: "menu_item.edit.copy",
        command: "selection.copy",
        shortcut: "Mod+C",
      },
      {
        id: "edit.paste",
        label: "Paste",
        translationKey: "menu_item.edit.paste",
        command: "selection.paste",
        shortcut: "Mod+V",
      },
      {
        id: "edit.delete",
        label: "Delete",
        translationKey: "menu_item.edit.delete",
        command: "selection.delete",
        shortcut: "Del",
      },
      { type: "separator" },
      {
        id: "edit.find",
        label: "Find",
        translationKey: "menu_item.edit.find",
        command: "workspace.find",
        shortcut: "Mod+F",
      },
      {
        id: "edit.command_palette",
        label: "Command Palette",
        translationKey: "menu_item.edit.command_palette",
        command: "workspace.command_palette",
        shortcut: "Mod+Shift+P",
      },
    ],
  },
  {
    id: "view",
    label: "View",
    translationKey: "menu.view",
    items: [
      {
        id: "view.project_explorer",
        label: "Project Explorer",
        translationKey: "menu_item.view.project_explorer",
        command: "view.project_explorer",
      },
      {
        id: "view.properties",
        label: "Properties",
        translationKey: "menu_item.view.properties",
        command: "view.properties",
        shortcut: "F4",
      },
      {
        id: "view.output",
        label: "Output",
        translationKey: "menu_item.view.output",
        command: "view.output",
      },
      {
        id: "view.problems",
        label: "Problems",
        translationKey: "menu_item.view.problems",
        command: "view.problems",
      },
      {
        id: "view.ai_assistant",
        label: "AI Assistant",
        translationKey: "menu_item.view.ai_assistant",
        command: "view.ai_assistant",
      },
      { type: "separator" },
      {
        id: "view.viewport_3d",
        label: "3D Viewport",
        translationKey: "menu_item.view.viewport_3d",
        command: "view.viewport_3d",
      },
      {
        id: "view.simulation_timeline",
        label: "Simulation Timeline",
        translationKey: "menu_item.view.simulation_timeline",
        command: "view.simulation_timeline",
      },
      {
        id: "view.telemetry_monitor",
        label: "Telemetry Monitor",
        translationKey: "menu_item.view.telemetry_monitor",
        command: "view.telemetry_monitor",
      },
    ],
  },
  {
    id: "insert",
    label: "Insert",
    translationKey: "menu.insert",
    items: [
      {
        id: "insert.add_part",
        label: "Add Part",
        translationKey: "menu_item.insert.add_part",
        command: "entity.create.part",
      },
      {
        id: "insert.add_assembly",
        label: "Add Assembly",
        translationKey: "menu_item.insert.add_assembly",
        command: "entity.create.assembly",
      },
      {
        id: "insert.add_robot_cell",
        label: "Add Robot Cell",
        translationKey: "menu_item.insert.add_robot_cell",
        command: "entity.create.robot_cell",
      },
      {
        id: "insert.add_sensor_rig",
        label: "Add Sensor Rig",
        translationKey: "menu_item.insert.add_sensor_rig",
        command: "entity.create.sensor_rig",
      },
      {
        id: "insert.add_external_endpoint",
        label: "Add External Endpoint",
        translationKey: "menu_item.insert.add_external_endpoint",
        command: "entity.create.external_endpoint",
      },
      { type: "separator" },
      {
        id: "insert.project_properties",
        label: "Project Properties",
        translationKey: "menu_item.insert.project_properties",
        command: "project.properties",
        shortcut: "Alt+Enter",
      },
    ],
  },
  {
    id: "simulation",
    label: "Simulation",
    translationKey: "menu.simulation",
    items: [
      {
        id: "simulation.start",
        label: "Start Simulation",
        translationKey: "menu_item.simulation.start",
        command: "simulation.run.start",
        shortcut: "F5",
      },
      {
        id: "simulation.stop",
        label: "Stop Simulation",
        translationKey: "menu_item.simulation.stop",
        command: "simulation.run.cancel",
        shortcut: "Shift+F5",
      },
      {
        id: "simulation.step",
        label: "Step Simulation",
        translationKey: "menu_item.simulation.step",
        command: "simulation.timeline.step",
        shortcut: "F10",
      },
      { type: "separator" },
      {
        id: "simulation.rebuild_geometry",
        label: "Rebuild Geometry",
        translationKey: "menu_item.simulation.rebuild_geometry",
        command: "build.regenerate_part",
        shortcut: "Mod+B",
      },
      {
        id: "simulation.safety_analysis",
        label: "Safety Analysis",
        translationKey: "menu_item.simulation.safety_analysis",
        command: "analyze.safety",
      },
      { type: "separator" },
      {
        id: "simulation.perception_run",
        label: "Run Perception",
        translationKey: "menu_item.simulation.perception_run",
        command: "perception.run.start",
      },
      {
        id: "simulation.integration_replay",
        label: "Replay Degraded Link",
        translationKey: "menu_item.simulation.integration_replay",
        command: "integration.replay.degraded",
      },
      {
        id: "simulation.commissioning_session",
        label: "Start Commissioning",
        translationKey: "menu_item.simulation.commissioning_session",
        command: "commissioning.session.start",
      },
      {
        id: "simulation.as_built_compare",
        label: "Compare As-Built",
        translationKey: "menu_item.simulation.as_built_compare",
        command: "commissioning.compare.as_built",
      },
      {
        id: "simulation.optimization_run",
        label: "Run Optimization",
        translationKey: "menu_item.simulation.optimization_run",
        command: "optimization.run.start",
      },
    ],
  },
  {
    id: "ai",
    label: "AI",
    translationKey: "menu.ai",
    items: [
      {
        id: "ai.focus_input",
        label: "Focus Local AI Chat",
        translationKey: "menu_item.ai.focus_input",
        command: "ai.focus_input",
        shortcut: "Ctrl+Space",
      },
      {
        id: "ai.show_panel",
        label: "Show AI Assistant",
        translationKey: "menu_item.ai.show_panel",
        command: "ai.show_panel",
      },
      {
        id: "ai.deep_explain",
        label: "AI Deep Explain",
        translationKey: "menu_item.ai.deep_explain",
        command: "ai.deep_explain.request",
      },
    ],
  },
  {
    id: "help",
    label: "Help",
    translationKey: "menu.help",
    items: [
      {
        id: "help.documentation",
        label: "Documentation",
        translationKey: "menu_item.help.documentation",
        command: "help.documentation",
      },
      {
        id: "help.openspec",
        label: "OpenSpec",
        translationKey: "menu_item.help.openspec",
        command: "help.openspec",
      },
      {
        id: "help.keyboard_shortcuts",
        label: "Keyboard Shortcuts",
        translationKey: "menu_item.help.keyboard_shortcuts",
        command: "help.keyboard_shortcuts",
      },
      {
        id: "help.about",
        label: "About FutureAero",
        translationKey: "menu_item.help.about",
        command: "help.about",
      },
    ],
  },
];

function localizeMenuItem(locale, item) {
  if (item.type === "separator") {
    return item;
  }

  return {
    ...item,
    label: translate(locale, item.translationKey, item.label),
  };
}

export function localizeMenuModel(locale = defaultLocale) {
  return menuDefinitions.map((menu) => ({
    ...menu,
    label: translate(locale, menu.translationKey, menu.label),
    items: menu.items.map((item) => localizeMenuItem(locale, item)),
  }));
}

export function findMenuEntryByCommand(
  commandId,
  menuModel = visualStudioInspiredMenus,
) {
  for (const menu of menuModel ?? []) {
    for (const item of menu.items ?? []) {
      if (item.type === "separator") {
        continue;
      }

      if (item.command === commandId) {
        return {
          menuId: menu.id,
          menu,
          item,
        };
      }
    }
  }

  return null;
}

export const visualStudioInspiredMenus = localizeMenuModel(defaultLocale);

export function getTopLevelMenuLabels(locale = defaultLocale) {
  return localizeMenuModel(locale).map((menu) => menu.label);
}

export function getAllMenuCommands(menuModel = visualStudioInspiredMenus) {
  return menuModel.flatMap((menu) =>
    menu.items
      .filter((item) => item.type !== "separator")
      .map((item) => item.command),
  );
}
