export const visualStudioInspiredMenus = [
  {
    id: "file",
    label: "File",
    items: [
      { id: "file.new_project", label: "New Project", command: "project.create", shortcut: "Ctrl+Shift+N" },
      { id: "file.open_project", label: "Open Project", command: "project.open", shortcut: "Ctrl+O" },
      { id: "file.open_recent", label: "Open Recent", command: "project.open_recent" },
      { type: "separator" },
      { id: "file.save", label: "Save", command: "project.save", shortcut: "Ctrl+S" },
      { id: "file.save_all", label: "Save All", command: "project.save_all", shortcut: "Ctrl+Shift+S" },
      { type: "separator" },
      { id: "file.import", label: "Import", command: "project.import" },
      { id: "file.export", label: "Export", command: "project.export" },
      { type: "separator" },
      { id: "file.settings", label: "Settings", command: "app.settings" },
      { id: "file.exit", label: "Exit", command: "app.exit" }
    ]
  },
  {
    id: "edit",
    label: "Edit",
    items: [
      { id: "edit.undo", label: "Undo", command: "history.undo", shortcut: "Ctrl+Z" },
      { id: "edit.redo", label: "Redo", command: "history.redo", shortcut: "Ctrl+Y" },
      { type: "separator" },
      { id: "edit.cut", label: "Cut", command: "selection.cut", shortcut: "Ctrl+X" },
      { id: "edit.copy", label: "Copy", command: "selection.copy", shortcut: "Ctrl+C" },
      { id: "edit.paste", label: "Paste", command: "selection.paste", shortcut: "Ctrl+V" },
      { id: "edit.delete", label: "Delete", command: "selection.delete", shortcut: "Del" },
      { type: "separator" },
      { id: "edit.find", label: "Find", command: "workspace.find", shortcut: "Ctrl+F" },
      { id: "edit.command_palette", label: "Command Palette", command: "workspace.command_palette", shortcut: "Ctrl+Shift+P" }
    ]
  },
  {
    id: "view",
    label: "View",
    items: [
      { id: "view.project_explorer", label: "Project Explorer", command: "view.project_explorer" },
      { id: "view.properties", label: "Properties", command: "view.properties", shortcut: "F4" },
      { id: "view.output", label: "Output", command: "view.output" },
      { id: "view.problems", label: "Problems", command: "view.problems" },
      { id: "view.jobs", label: "Jobs", command: "view.jobs" },
      { id: "view.ai_assistant", label: "AI Assistant", command: "view.ai_assistant" },
      { type: "separator" },
      { id: "view.viewport_3d", label: "3D Viewport", command: "view.viewport_3d" },
      { id: "view.simulation_timeline", label: "Simulation Timeline", command: "view.simulation_timeline" },
      { id: "view.telemetry_monitor", label: "Telemetry Monitor", command: "view.telemetry_monitor" }
    ]
  },
  {
    id: "git",
    label: "Git",
    items: [
      { id: "git.commit", label: "Commit", command: "git.commit" },
      { id: "git.push", label: "Push", command: "git.push" },
      { id: "git.pull", label: "Pull", command: "git.pull" },
      { id: "git.branch", label: "Branches", command: "git.branches" }
    ]
  },
  {
    id: "project",
    label: "Project",
    items: [
      { id: "project.add_part", label: "Add Part", command: "entity.create.part" },
      { id: "project.add_assembly", label: "Add Assembly", command: "entity.create.assembly" },
      { id: "project.add_robot_cell", label: "Add Robot Cell", command: "entity.create.robot_cell" },
      { id: "project.add_sensor_rig", label: "Add Sensor Rig", command: "entity.create.sensor_rig" },
      { id: "project.add_external_endpoint", label: "Add External Endpoint", command: "entity.create.external_endpoint" },
      { type: "separator" },
      { id: "project.properties", label: "Project Properties", command: "project.properties", shortcut: "Alt+Enter" }
    ]
  },
  {
    id: "build",
    label: "Build",
    items: [
      { id: "build.regenerate_part", label: "Regenerate Part", command: "build.regenerate_part", shortcut: "Ctrl+B" },
      { id: "build.rebuild_assembly", label: "Rebuild Assembly", command: "build.rebuild_assembly", shortcut: "Ctrl+Shift+B" },
      { id: "build.build_robot_cell", label: "Build Robot Cell", command: "build.robot_cell" },
      { id: "build.prepare_commissioning", label: "Prepare Commissioning Package", command: "build.commissioning_package" }
    ]
  },
  {
    id: "debug",
    label: "Debug",
    items: [
      { id: "debug.start_simulation", label: "Start Simulation", command: "simulation.run.start", shortcut: "F5" },
      { id: "debug.start_without_debugging", label: "Start Without Debugging", command: "simulation.run.start_without_debugging", shortcut: "Ctrl+F5" },
      { id: "debug.stop", label: "Stop", command: "simulation.run.cancel", shortcut: "Shift+F5" },
      { type: "separator" },
      { id: "debug.step_timeline", label: "Step Timeline", command: "simulation.timeline.step", shortcut: "F10" },
      { id: "debug.step_into_logic", label: "Step Into Logic", command: "simulation.logic.step_into", shortcut: "F11" },
      { id: "debug.breakpoints", label: "Breakpoints", command: "simulation.breakpoints" }
    ]
  },
  {
    id: "test",
    label: "Test",
    items: [
      { id: "test.run_all", label: "Run All Tests", command: "test.run_all", shortcut: "Ctrl+R,A" },
      { id: "test.run_current_fixture", label: "Run Current Fixture", command: "test.run_fixture" },
      { id: "test.coverage", label: "Coverage Report", command: "test.coverage_report" },
      { id: "test.replay", label: "Replay Scenario", command: "test.replay_scenario" }
    ]
  },
  {
    id: "analyze",
    label: "Analyze",
    items: [
      { id: "analyze.validation", label: "Validation Report", command: "analyze.validation_report" },
      { id: "analyze.as_built", label: "As-Built vs As-Designed", command: "analyze.as_built" },
      { id: "analyze.safety", label: "Safety Analysis", command: "analyze.safety" },
      { id: "analyze.optimization", label: "Optimization Study", command: "analyze.optimization" },
      { id: "analyze.ai_explain", label: "AI Deep Explain", command: "ai.deep_explain.request" }
    ]
  },
  {
    id: "tools",
    label: "Tools",
    items: [
      { id: "tools.extensions", label: "Extensions and Plugins", command: "plugin.manage" },
      { id: "tools.device_manager", label: "Device Manager", command: "integration.device_manager" },
      { id: "tools.telemetry", label: "Telemetry Streams", command: "integration.telemetry_streams" },
      { id: "tools.options", label: "Options", command: "app.options" }
    ]
  },
  {
    id: "window",
    label: "Window",
    items: [
      { id: "window.new_window", label: "New Window", command: "window.new" },
      { id: "window.split", label: "Split View", command: "window.split" },
      { id: "window.reset_layout", label: "Reset Layout", command: "window.reset_layout" },
      { id: "window.close_all_documents", label: "Close All Documents", command: "window.close_all_documents" }
    ]
  },
  {
    id: "help",
    label: "Help",
    items: [
      { id: "help.documentation", label: "Documentation", command: "help.documentation" },
      { id: "help.openspec", label: "OpenSpec", command: "help.openspec" },
      { id: "help.keyboard_shortcuts", label: "Keyboard Shortcuts", command: "help.keyboard_shortcuts" },
      { id: "help.about", label: "About FutureAero", command: "help.about" }
    ]
  }
];

export function getTopLevelMenuLabels() {
  return visualStudioInspiredMenus.map((menu) => menu.label);
}

export function getAllMenuCommands() {
  return visualStudioInspiredMenus.flatMap((menu) =>
    menu.items.filter((item) => item.type !== "separator").map((item) => item.command)
  );
}

