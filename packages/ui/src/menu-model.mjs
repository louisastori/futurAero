import { defaultLocale, translate } from "./i18n.mjs";

const menuDefinitions = [
  {
    id: "file",
    label: "File",
    translationKey: "menu.file",
    items: [
      { id: "file.new_project", label: "New Project", translationKey: "menu_item.file.new_project", command: "project.create", shortcut: "Ctrl+Shift+N" },
      { id: "file.open_project", label: "Open Project", translationKey: "menu_item.file.open_project", command: "project.open", shortcut: "Ctrl+O" },
      { id: "file.open_recent", label: "Open Recent", translationKey: "menu_item.file.open_recent", command: "project.open_recent" },
      { type: "separator" },
      { id: "file.save", label: "Save", translationKey: "menu_item.file.save", command: "project.save", shortcut: "Ctrl+S" },
      { id: "file.save_all", label: "Save All", translationKey: "menu_item.file.save_all", command: "project.save_all", shortcut: "Ctrl+Shift+S" },
      { type: "separator" },
      { id: "file.import", label: "Import", translationKey: "menu_item.file.import", command: "project.import" },
      { id: "file.export", label: "Export", translationKey: "menu_item.file.export", command: "project.export" },
      { type: "separator" },
      { id: "file.settings", label: "Settings", translationKey: "menu_item.file.settings", command: "app.settings" },
      { id: "file.exit", label: "Exit", translationKey: "menu_item.file.exit", command: "app.exit" }
    ]
  },
  {
    id: "edit",
    label: "Edit",
    translationKey: "menu.edit",
    items: [
      { id: "edit.undo", label: "Undo", translationKey: "menu_item.edit.undo", command: "history.undo", shortcut: "Ctrl+Z" },
      { id: "edit.redo", label: "Redo", translationKey: "menu_item.edit.redo", command: "history.redo", shortcut: "Ctrl+Y" },
      { type: "separator" },
      { id: "edit.cut", label: "Cut", translationKey: "menu_item.edit.cut", command: "selection.cut", shortcut: "Ctrl+X" },
      { id: "edit.copy", label: "Copy", translationKey: "menu_item.edit.copy", command: "selection.copy", shortcut: "Ctrl+C" },
      { id: "edit.paste", label: "Paste", translationKey: "menu_item.edit.paste", command: "selection.paste", shortcut: "Ctrl+V" },
      { id: "edit.delete", label: "Delete", translationKey: "menu_item.edit.delete", command: "selection.delete", shortcut: "Del" },
      { type: "separator" },
      { id: "edit.find", label: "Find", translationKey: "menu_item.edit.find", command: "workspace.find", shortcut: "Ctrl+F" },
      { id: "edit.command_palette", label: "Command Palette", translationKey: "menu_item.edit.command_palette", command: "workspace.command_palette", shortcut: "Ctrl+Shift+P" }
    ]
  },
  {
    id: "view",
    label: "View",
    translationKey: "menu.view",
    items: [
      { id: "view.project_explorer", label: "Project Explorer", translationKey: "menu_item.view.project_explorer", command: "view.project_explorer" },
      { id: "view.properties", label: "Properties", translationKey: "menu_item.view.properties", command: "view.properties", shortcut: "F4" },
      { id: "view.output", label: "Output", translationKey: "menu_item.view.output", command: "view.output" },
      { id: "view.problems", label: "Problems", translationKey: "menu_item.view.problems", command: "view.problems" },
      { id: "view.jobs", label: "Jobs", translationKey: "menu_item.view.jobs", command: "view.jobs" },
      { id: "view.ai_assistant", label: "AI Assistant", translationKey: "menu_item.view.ai_assistant", command: "view.ai_assistant" },
      { type: "separator" },
      { id: "view.viewport_3d", label: "3D Viewport", translationKey: "menu_item.view.viewport_3d", command: "view.viewport_3d" },
      { id: "view.simulation_timeline", label: "Simulation Timeline", translationKey: "menu_item.view.simulation_timeline", command: "view.simulation_timeline" },
      { id: "view.telemetry_monitor", label: "Telemetry Monitor", translationKey: "menu_item.view.telemetry_monitor", command: "view.telemetry_monitor" }
    ]
  },
  {
    id: "git",
    label: "Git",
    translationKey: "menu.git",
    items: [
      { id: "git.commit", label: "Commit", translationKey: "menu_item.git.commit", command: "git.commit" },
      { id: "git.push", label: "Push", translationKey: "menu_item.git.push", command: "git.push" },
      { id: "git.pull", label: "Pull", translationKey: "menu_item.git.pull", command: "git.pull" },
      { id: "git.branch", label: "Branches", translationKey: "menu_item.git.branch", command: "git.branches" }
    ]
  },
  {
    id: "project",
    label: "Project",
    translationKey: "menu.project",
    items: [
      { id: "project.add_part", label: "Add Part", translationKey: "menu_item.project.add_part", command: "entity.create.part" },
      { id: "project.add_assembly", label: "Add Assembly", translationKey: "menu_item.project.add_assembly", command: "entity.create.assembly" },
      { id: "project.add_robot_cell", label: "Add Robot Cell", translationKey: "menu_item.project.add_robot_cell", command: "entity.create.robot_cell" },
      { id: "project.add_sensor_rig", label: "Add Sensor Rig", translationKey: "menu_item.project.add_sensor_rig", command: "entity.create.sensor_rig" },
      { id: "project.add_external_endpoint", label: "Add External Endpoint", translationKey: "menu_item.project.add_external_endpoint", command: "entity.create.external_endpoint" },
      { type: "separator" },
      { id: "project.properties", label: "Project Properties", translationKey: "menu_item.project.properties", command: "project.properties", shortcut: "Alt+Enter" }
    ]
  },
  {
    id: "build",
    label: "Build",
    translationKey: "menu.build",
    items: [
      { id: "build.regenerate_part", label: "Regenerate Part", translationKey: "menu_item.build.regenerate_part", command: "build.regenerate_part", shortcut: "Ctrl+B" },
      { id: "build.rebuild_assembly", label: "Rebuild Assembly", translationKey: "menu_item.build.rebuild_assembly", command: "build.rebuild_assembly", shortcut: "Ctrl+Shift+B" },
      { id: "build.build_robot_cell", label: "Build Robot Cell", translationKey: "menu_item.build.build_robot_cell", command: "build.robot_cell" },
      { id: "build.prepare_commissioning", label: "Prepare Commissioning Package", translationKey: "menu_item.build.prepare_commissioning", command: "build.commissioning_package" }
    ]
  },
  {
    id: "debug",
    label: "Debug",
    translationKey: "menu.debug",
    items: [
      { id: "debug.start_simulation", label: "Start Simulation", translationKey: "menu_item.debug.start_simulation", command: "simulation.run.start", shortcut: "F5" },
      { id: "debug.start_without_debugging", label: "Start Without Debugging", translationKey: "menu_item.debug.start_without_debugging", command: "simulation.run.start_without_debugging", shortcut: "Ctrl+F5" },
      { id: "debug.stop", label: "Stop", translationKey: "menu_item.debug.stop", command: "simulation.run.cancel", shortcut: "Shift+F5" },
      { type: "separator" },
      { id: "debug.step_timeline", label: "Step Timeline", translationKey: "menu_item.debug.step_timeline", command: "simulation.timeline.step", shortcut: "F10" },
      { id: "debug.step_into_logic", label: "Step Into Logic", translationKey: "menu_item.debug.step_into_logic", command: "simulation.logic.step_into", shortcut: "F11" },
      { id: "debug.breakpoints", label: "Breakpoints", translationKey: "menu_item.debug.breakpoints", command: "simulation.breakpoints" }
    ]
  },
  {
    id: "test",
    label: "Test",
    translationKey: "menu.test",
    items: [
      { id: "test.run_all", label: "Run All Tests", translationKey: "menu_item.test.run_all", command: "test.run_all", shortcut: "Ctrl+R,A" },
      { id: "test.run_current_fixture", label: "Run Current Fixture", translationKey: "menu_item.test.run_current_fixture", command: "test.run_fixture" },
      { id: "test.coverage", label: "Coverage Report", translationKey: "menu_item.test.coverage", command: "test.coverage_report" },
      { id: "test.replay", label: "Replay Scenario", translationKey: "menu_item.test.replay", command: "test.replay_scenario" }
    ]
  },
  {
    id: "analyze",
    label: "Analyze",
    translationKey: "menu.analyze",
    items: [
      { id: "analyze.validation", label: "Validation Report", translationKey: "menu_item.analyze.validation", command: "analyze.validation_report" },
      { id: "analyze.as_built", label: "As-Built vs As-Designed", translationKey: "menu_item.analyze.as_built", command: "analyze.as_built" },
      { id: "analyze.safety", label: "Safety Analysis", translationKey: "menu_item.analyze.safety", command: "analyze.safety" },
      { id: "analyze.optimization", label: "Optimization Study", translationKey: "menu_item.analyze.optimization", command: "analyze.optimization" },
      { id: "analyze.ai_explain", label: "AI Deep Explain", translationKey: "menu_item.analyze.ai_explain", command: "ai.deep_explain.request" }
    ]
  },
  {
    id: "tools",
    label: "Tools",
    translationKey: "menu.tools",
    items: [
      { id: "tools.extensions", label: "Extensions and Plugins", translationKey: "menu_item.tools.extensions", command: "plugin.manage" },
      { id: "tools.device_manager", label: "Device Manager", translationKey: "menu_item.tools.device_manager", command: "integration.device_manager" },
      { id: "tools.telemetry", label: "Telemetry Streams", translationKey: "menu_item.tools.telemetry", command: "integration.telemetry_streams" },
      { id: "tools.options", label: "Options", translationKey: "menu_item.tools.options", command: "app.options" }
    ]
  },
  {
    id: "window",
    label: "Window",
    translationKey: "menu.window",
    items: [
      { id: "window.new_window", label: "New Window", translationKey: "menu_item.window.new_window", command: "window.new" },
      { id: "window.split", label: "Split View", translationKey: "menu_item.window.split", command: "window.split" },
      { id: "window.reset_layout", label: "Reset Layout", translationKey: "menu_item.window.reset_layout", command: "window.reset_layout" },
      { id: "window.close_all_documents", label: "Close All Documents", translationKey: "menu_item.window.close_all_documents", command: "window.close_all_documents" }
    ]
  },
  {
    id: "help",
    label: "Help",
    translationKey: "menu.help",
    items: [
      { id: "help.documentation", label: "Documentation", translationKey: "menu_item.help.documentation", command: "help.documentation" },
      { id: "help.openspec", label: "OpenSpec", translationKey: "menu_item.help.openspec", command: "help.openspec" },
      { id: "help.keyboard_shortcuts", label: "Keyboard Shortcuts", translationKey: "menu_item.help.keyboard_shortcuts", command: "help.keyboard_shortcuts" },
      { id: "help.about", label: "About FutureAero", translationKey: "menu_item.help.about", command: "help.about" }
    ]
  }
];

function localizeMenuItem(locale, item) {
  if (item.type === "separator") {
    return item;
  }

  return {
    ...item,
    label: translate(locale, item.translationKey, item.label)
  };
}

export function localizeMenuModel(locale = defaultLocale) {
  return menuDefinitions.map((menu) => ({
    ...menu,
    label: translate(locale, menu.translationKey, menu.label),
    items: menu.items.map((item) => localizeMenuItem(locale, item))
  }));
}

export const visualStudioInspiredMenus = localizeMenuModel(defaultLocale);

export function getTopLevelMenuLabels(locale = defaultLocale) {
  return localizeMenuModel(locale).map((menu) => menu.label);
}

export function getAllMenuCommands(menuModel = visualStudioInspiredMenus) {
  return menuModel.flatMap((menu) =>
    menu.items.filter((item) => item.type !== "separator").map((item) => item.command)
  );
}
