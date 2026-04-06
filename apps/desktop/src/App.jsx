import { useEffect, useRef, useState } from "react";

import {
  calculateResizedDockWidths,
  defaultWorkspaceDockWidths,
  defaultWorkspacePanels,
  defaultLocale,
  getWorkspaceColumnState,
  getVisibleSidebarWidth,
  localizeMenuModel,
  panelIdFromCommand,
  supportedLocales,
  toggleWorkspacePanel,
  WORKSPACE_RESIZER_WIDTH,
  translate
} from "@futureaero/ui";

const FALLBACK_FIXTURES = [
  { id: "pick-and-place-demo.faero", projectName: "Pick And Place Demo" },
  { id: "wireless-integration-demo.faero", projectName: "Wireless Integration Demo" },
  { id: "empty-project.faero", projectName: "Empty Project" }
];

const FALLBACK_STATUS = {
  runtime: "web-preview",
  fixtureId: "pick-and-place-demo.faero",
  projectName: "Shell Preview",
  entityCount: 0,
  endpointCount: 0,
  streamCount: 0,
  pluginCount: 0
};

const FALLBACK_SNAPSHOT = {
  status: FALLBACK_STATUS,
  details: {
    projectId: "prj_preview",
    formatVersion: "0.1.0",
    defaultFrame: "world",
    rootSceneId: null,
    activeConfigurationId: "cfg_default"
  },
  entities: [],
  endpoints: [],
  streams: [],
  plugins: [],
  recentActivity: []
};

const FALLBACK_AI_STATUS = {
  available: false,
  provider: "ollama",
  endpoint: "http://127.0.0.1:11434",
  mode: "web-preview",
  localOnly: true,
  activeModel: null,
  availableModels: [],
  warning: "Local AI runtime unavailable in web preview."
};

function MenuBar({ menus, activeMenuId, onSelect }) {
  return (
    <nav className="menu-bar" aria-label="Application menu">
      {menus.map((menu) => (
        <button
          key={menu.id}
          className={menu.id === activeMenuId ? "menu-button active" : "menu-button"}
          type="button"
          onClick={() => onSelect(menu.id)}
        >
          {menu.label}
        </button>
      ))}
    </nav>
  );
}

function Panel({ title, children, accent, collapsed = false, onToggle, toggleLabel }) {
  return (
    <section className={collapsed ? "panel panel-collapsed" : "panel"}>
      <header className="panel-header">
        <div className="panel-header-main">
          <span className="panel-title">{title}</span>
          {accent ? <span className="panel-accent">{accent}</span> : null}
        </div>
        {onToggle ? (
          <button
            className="panel-toggle"
            type="button"
            onClick={onToggle}
            aria-expanded={!collapsed}
            aria-label={toggleLabel}
            title={toggleLabel}
          >
            {collapsed ? "+" : "-"}
          </button>
        ) : null}
      </header>
      {collapsed ? null : <div className="panel-body">{children}</div>}
    </section>
  );
}

async function invokeBackend(command, payload) {
  try {
    const tauriCore = await import("@tauri-apps/api/core");
    return await tauriCore.invoke(command, payload);
  } catch {
    return null;
  }
}

function buildFallbackSnapshot(projectId = FALLBACK_STATUS.fixtureId) {
  const fixture = FALLBACK_FIXTURES.find((entry) => entry.id === projectId) ?? FALLBACK_FIXTURES[0];
  return {
    ...FALLBACK_SNAPSHOT,
    status: {
      ...FALLBACK_STATUS,
      fixtureId: fixture.id,
      projectName: fixture.projectName
    },
    details: {
      ...FALLBACK_SNAPSHOT.details,
      projectId: `preview:${fixture.id}`
    }
  };
}

function appendFallbackActivity(snapshot, channel, kind, targetId) {
  const entry = {
    id: `web_${snapshot.recentActivity.length + 1}`,
    channel,
    kind,
    timestamp: "2026-04-06T12:59:59Z",
    targetId
  };

  return {
    ...snapshot,
    recentActivity: [entry, ...snapshot.recentActivity].slice(0, 12)
  };
}

async function fetchWorkspaceBootstrap() {
  return (
    (await invokeBackend("workspace_bootstrap")) ?? {
      fixtures: FALLBACK_FIXTURES,
      snapshot: buildFallbackSnapshot()
    }
  );
}

async function loadWorkspaceFixture(projectId) {
  return (await invokeBackend("workspace_load_fixture", { projectId })) ?? buildFallbackSnapshot(projectId);
}

async function executeWorkspaceCommand(commandId, currentSnapshot) {
  const response = await invokeBackend("workspace_execute_command", { commandId });
  if (response) {
    return response;
  }

  return {
    snapshot: appendFallbackActivity(currentSnapshot, "system", "command.simulated", commandId),
    result: {
      commandId,
      status: "simulated",
      message: "commande simulee dans l apercu web"
    }
  };
}

async function fetchAiRuntimeStatus() {
  return (await invokeBackend("ai_runtime_status")) ?? FALLBACK_AI_STATUS;
}

function buildFallbackAiReferences(snapshot) {
  return [
    `project:${snapshot.details.projectId}`,
    ...snapshot.entities.slice(0, 3).map((entity) => `entity:${entity.id}`),
    ...snapshot.endpoints.slice(0, 2).map((endpoint) => `endpoint:${endpoint.id}`),
    ...snapshot.streams.slice(0, 2).map((stream) => `stream:${stream.id}`),
    ...snapshot.plugins.slice(0, 1).map((plugin) => `plugin:${plugin.pluginId}`)
  ].slice(0, 8);
}

function buildFallbackAiAnswer(locale, snapshot, message) {
  const summary = `${snapshot.status.projectName} | ${snapshot.status.entityCount} entites | ${snapshot.status.endpointCount} endpoints | ${snapshot.status.streamCount} flux`;

  if (locale === "en") {
    return `The local AI panel is running in web preview fallback mode. Current project: ${summary}. Your question was: "${message}". Start the Tauri shell with Ollama available on http://127.0.0.1:11434 to get a true local model-backed discussion.`;
  }

  if (locale === "es") {
    return `El panel de IA local esta en modo fallback de vista web. Proyecto actual: ${summary}. Tu pregunta fue: "${message}". Inicia el shell Tauri con Ollama disponible en http://127.0.0.1:11434 para obtener una conversacion local real con modelo.`;
  }

  return `Le panneau IA locale tourne en mode fallback d apercu web. Projet courant: ${summary}. Ta question etait: "${message}". Lance le shell Tauri avec Ollama disponible sur http://127.0.0.1:11434 pour obtenir une vraie discussion locale avec modele.`;
}

async function sendAiChatMessage(message, locale, history, snapshot) {
  const response = await invokeBackend("ai_chat_send_message", { message, locale, history });
  if (response) {
    return response;
  }

  return {
    answer: buildFallbackAiAnswer(locale, snapshot, message),
    runtime: FALLBACK_AI_STATUS,
    references: buildFallbackAiReferences(snapshot),
    warnings: [FALLBACK_AI_STATUS.warning],
    source: "web-preview"
  };
}

function runtimeLabel(locale, runtime) {
  if (runtime === "web-preview") {
    return translate(locale, "ui.status.web_preview", runtime);
  }

  return translate(locale, "ui.status.tauri", runtime);
}

function fixtureLabel(fixtures, fixtureId) {
  const currentFixture = fixtures.find((fixture) => fixture.id === fixtureId);
  return currentFixture?.projectName ?? fixtureId;
}

function activityChannelLabel(locale, channel) {
  if (channel === "command") {
    return translate(locale, "ui.activity.command", channel);
  }
  if (channel === "event") {
    return translate(locale, "ui.activity.event", channel);
  }

  return translate(locale, "ui.activity.system", channel);
}

function assistantRoleLabel(locale, role) {
  if (role === "assistant") {
    return translate(locale, "ui.ai.assistant_label", "Assistant IA");
  }

  return translate(locale, "ui.ai.user_label", "Utilisateur");
}

function assistantAccent(locale, runtime) {
  if (runtime.available) {
    return `${translate(locale, "ui.ai.runtime_ready", "Pret")} | ${runtime.activeModel ?? translate(locale, "ui.ai.no_model", "aucun modele")}`;
  }

  return translate(locale, "ui.ai.runtime_fallback", "Fallback local");
}

function assistantBadge(locale, runtime) {
  if (runtime.available) {
    return `${translate(locale, "ui.ai.badge", "IA locale")} | ${runtime.activeModel ?? translate(locale, "ui.ai.no_model", "aucun modele")}`;
  }

  return `${translate(locale, "ui.ai.badge", "IA locale")} | ${translate(locale, "ui.ai.runtime_fallback", "Fallback local")}`;
}

export default function App() {
  const [locale, setLocale] = useState(defaultLocale);
  const [projectSnapshot, setProjectSnapshot] = useState(FALLBACK_SNAPSHOT);
  const [fixtureProjects, setFixtureProjects] = useState(FALLBACK_FIXTURES);
  const [selectedFixtureId, setSelectedFixtureId] = useState(FALLBACK_STATUS.fixtureId);
  const [activeMenuId, setActiveMenuId] = useState("file");
  const [fixtureLoading, setFixtureLoading] = useState(false);
  const [executingCommandId, setExecutingCommandId] = useState(null);
  const [commandResult, setCommandResult] = useState(null);
  const [aiRuntime, setAiRuntime] = useState(FALLBACK_AI_STATUS);
  const [aiMessages, setAiMessages] = useState([]);
  const [aiDraft, setAiDraft] = useState("");
  const [aiBusy, setAiBusy] = useState(false);
  const [panelState, setPanelState] = useState(defaultWorkspacePanels);
  const [dockWidths, setDockWidths] = useState(defaultWorkspaceDockWidths);
  const [dragSide, setDragSide] = useState(null);
  const aiInputRef = useRef(null);
  const workspaceRef = useRef(null);
  const dragStateRef = useRef(null);

  const menus = localizeMenuModel(locale);
  const menu = menus.find((entry) => entry.id === activeMenuId) ?? menus[0];
  const currentStatus = projectSnapshot.status;
  const t = (key, fallback = key) => translate(locale, key, fallback);
  const { leftExpanded, rightExpanded } = getWorkspaceColumnState(panelState);
  const workspaceStyle = {
    "--workspace-left-column": `${getVisibleSidebarWidth(dockWidths.left, leftExpanded)}px`,
    "--workspace-right-column": `${getVisibleSidebarWidth(dockWidths.right, rightExpanded)}px`,
    "--workspace-left-resizer": leftExpanded ? `${WORKSPACE_RESIZER_WIDTH}px` : "0px",
    "--workspace-right-resizer": rightExpanded ? `${WORKSPACE_RESIZER_WIDTH}px` : "0px"
  };
  const starterPrompts = [
    t("ui.ai.prompt.summary", "Resume le projet courant"),
    t("ui.ai.prompt.integration", "Quels endpoints et flux sont relies a ce projet ?"),
    t("ui.ai.prompt.next_step", "Quel est le prochain jalon technique concret ?")
  ];

  function togglePanel(panelId) {
    setPanelState((previous) => toggleWorkspacePanel(previous, panelId));
  }

  function panelToggleLabel(panelId) {
    return panelState[panelId]
      ? t("ui.panel.collapse", "Replier le panneau")
      : t("ui.panel.expand", "Rouvrir le panneau");
  }

  function resizeHandleLabel(side) {
    return side === "left"
      ? t("ui.workspace.resize_left", "Redimensionner le panneau gauche")
      : t("ui.workspace.resize_right", "Redimensionner le panneau droit");
  }

  function startDockResize(side, event) {
    if (event.button !== 0) {
      return;
    }

    event.preventDefault();
    dragStateRef.current = {
      side,
      startX: event.clientX,
      startWidths: dockWidths
    };
    setDragSide(side);
  }

  function resetDockWidth(side) {
    setDockWidths((previous) => ({
      ...previous,
      [side]: defaultWorkspaceDockWidths[side]
    }));
  }

  useEffect(() => {
    if (!dragSide) {
      return undefined;
    }

    function handlePointerMove(event) {
      if (!dragStateRef.current) {
        return;
      }

      const layoutWidth = workspaceRef.current?.clientWidth ?? 0;
      const deltaX = event.clientX - dragStateRef.current.startX;
      setDockWidths(
        calculateResizedDockWidths({
          side: dragStateRef.current.side,
          startWidths: dragStateRef.current.startWidths,
          deltaX,
          layoutWidth,
          leftExpanded,
          rightExpanded
        })
      );
    }

    function stopDragging() {
      dragStateRef.current = null;
      setDragSide(null);
      document.body.classList.remove("is-resizing");
    }

    document.body.classList.add("is-resizing");
    window.addEventListener("mousemove", handlePointerMove);
    window.addEventListener("mouseup", stopDragging);

    return () => {
      window.removeEventListener("mousemove", handlePointerMove);
      window.removeEventListener("mouseup", stopDragging);
      document.body.classList.remove("is-resizing");
    };
  }, [dragSide, leftExpanded, rightExpanded]);

  useEffect(() => {
    let mounted = true;

    async function bootstrapWorkspace() {
      const [bootstrap, runtime] = await Promise.all([
        fetchWorkspaceBootstrap(),
        fetchAiRuntimeStatus()
      ]);
      if (!mounted) {
        return;
      }

      setFixtureProjects(bootstrap.fixtures.length > 0 ? bootstrap.fixtures : FALLBACK_FIXTURES);
      setProjectSnapshot(bootstrap.snapshot);
      setSelectedFixtureId(bootstrap.snapshot.status.fixtureId ?? FALLBACK_STATUS.fixtureId);
      setAiRuntime(runtime);
    }

    bootstrapWorkspace();

    return () => {
      mounted = false;
    };
  }, []);

  async function handleFixtureChange(event) {
    const nextFixtureId = event.target.value;
    setSelectedFixtureId(nextFixtureId);
    setFixtureLoading(true);

    try {
      const [snapshot, runtime] = await Promise.all([
        loadWorkspaceFixture(nextFixtureId),
        fetchAiRuntimeStatus()
      ]);
      setProjectSnapshot(snapshot);
      setCommandResult(null);
      setAiRuntime(runtime);
      setAiMessages([]);
      setAiDraft("");
    } finally {
      setFixtureLoading(false);
    }
  }

  async function handleCommandExecute(commandId) {
    const panelId = panelIdFromCommand(commandId);
    if (panelId) {
      const willBeVisible = !panelState[panelId];
      setPanelState((previous) => toggleWorkspacePanel(previous, panelId));
      setCommandResult({
        commandId,
        status: "layout",
        message: panelState[panelId]
          ? t("ui.panel.collapsed_status", "Panneau replie.")
          : t("ui.panel.expanded_status", "Panneau rouvert.")
      });

      if (panelId === "aiAssistant" && willBeVisible) {
        window.setTimeout(() => aiInputRef.current?.focus(), 0);
      }
      return;
    }

    setExecutingCommandId(commandId);

    try {
      const response = await executeWorkspaceCommand(commandId, projectSnapshot);
      setProjectSnapshot(response.snapshot);
      setSelectedFixtureId(response.snapshot.status.fixtureId);
      setCommandResult(response.result);

      if (commandId === "project.create") {
        setAiMessages([]);
        setAiDraft("");
      }
    } finally {
      setExecutingCommandId(null);
    }
  }

  async function submitAiMessage(message) {
    const trimmedMessage = message.trim();
    if (!trimmedMessage || aiBusy) {
      return;
    }

    const history = aiMessages.map((entry) => ({
      role: entry.role,
      content: entry.content
    }));
    const userEntry = {
      role: "user",
      content: trimmedMessage,
      references: [],
      warnings: [],
      source: "user"
    };

    setAiMessages((previous) => [...previous, userEntry]);
    setAiDraft("");
    setAiBusy(true);

    try {
      const response = await sendAiChatMessage(trimmedMessage, locale, history, projectSnapshot);
      setAiRuntime(response.runtime);
      setAiMessages((previous) => [
        ...previous,
        {
          role: "assistant",
          content: response.answer,
          references: response.references ?? [],
          warnings: response.warnings ?? [],
          source: response.source
        }
      ]);
    } catch {
      setAiMessages((previous) => [
        ...previous,
        {
          role: "assistant",
          content: t("ui.ai.error", "Le runtime IA local a renvoye une erreur."),
          references: [],
          warnings: [],
          source: "error"
        }
      ]);
    } finally {
      setAiBusy(false);
      aiInputRef.current?.focus();
    }
  }

  async function handleAiSubmit(event) {
    event.preventDefault();
    await submitAiMessage(aiDraft);
  }

  return (
    <div className="shell">
      <header className="shell-header">
        <div className="brand-block">
          <div className="brand-mark">FA</div>
          <div>
            <div className="brand-title">FutureAero</div>
            <div className="brand-subtitle">{t("ui.brand.subtitle", "Desktop shell")}</div>
          </div>
        </div>

        <div className="header-right">
          <div className="header-controls">
            <label className="control-group">
              <span>{t("ui.locale.label", "Langue")}</span>
              <select
                className="shell-select"
                value={locale}
                onChange={(event) => setLocale(event.target.value)}
              >
                {supportedLocales.map((entry) => (
                  <option key={entry.id} value={entry.id}>
                    {entry.label}
                  </option>
                ))}
              </select>
            </label>

            <label className="control-group">
              <span>{t("ui.fixture.label", "Projet de demonstration")}</span>
              <select
                className="shell-select"
                value={selectedFixtureId}
                onChange={handleFixtureChange}
                disabled={fixtureProjects.length === 0 || fixtureLoading}
              >
                {fixtureProjects.length > 0 ? (
                  fixtureProjects.map((fixture) => (
                    <option key={fixture.id} value={fixture.id}>
                      {fixture.projectName}
                    </option>
                  ))
                ) : (
                  <option value="">{t("ui.fixture.empty", "Aucune fixture")}</option>
                )}
              </select>
            </label>
          </div>

          <div className="status-pills">
            <span className="status-pill">{runtimeLabel(locale, currentStatus.runtime)}</span>
            <span className="status-pill">{currentStatus.projectName}</span>
            <span className="status-pill">{assistantBadge(locale, aiRuntime)}</span>
            <span className="status-pill">
              {fixtureLoading
                ? t("ui.fixture.loading", "Chargement...")
                : fixtureLabel(fixtureProjects, selectedFixtureId)}
            </span>
          </div>
        </div>
      </header>

      <MenuBar menus={menus} activeMenuId={activeMenuId} onSelect={setActiveMenuId} />

      <div className="context-bar">
        <div className="context-title">{menu.label}</div>
        <div className="context-meta">
          {menu.items.filter((item) => item.type !== "separator").length} commandes
        </div>
      </div>

      <main className="workspace" style={workspaceStyle} ref={workspaceRef}>
        <aside className={leftExpanded ? "workspace-left" : "workspace-left workspace-column-collapsed"}>
          <Panel
            title={t("ui.panel.project_explorer", "Explorateur de projet")}
            accent={`${currentStatus.entityCount} ${t("ui.workspace.entities", "entites")}`}
            collapsed={!panelState.projectExplorer}
            onToggle={() => togglePanel("projectExplorer")}
            toggleLabel={panelToggleLabel("projectExplorer")}
          >
            <ul className="tree-list">
              <li className="tree-root">{currentStatus.projectName}</li>

              <li className="tree-section">
                <div className="tree-section-title">{t("ui.workspace.entities_section", "Entites")}</div>
                <ul className="tree-sublist">
                  {projectSnapshot.entities.length > 0 ? (
                    projectSnapshot.entities.map((entity) => (
                      <li key={entity.id} className="tree-row">
                        <span>{entity.name}</span>
                        <span className="tree-meta">{entity.entityType}</span>
                      </li>
                    ))
                  ) : (
                    <li className="tree-empty">{t("ui.workspace.empty_section", "Aucun element")}</li>
                  )}
                </ul>
              </li>

              <li className="tree-section">
                <div className="tree-section-title">{t("ui.workspace.endpoints_section", "Endpoints")}</div>
                <ul className="tree-sublist">
                  {projectSnapshot.endpoints.length > 0 ? (
                    projectSnapshot.endpoints.map((endpoint) => (
                      <li key={endpoint.id} className="tree-row">
                        <span>{endpoint.name}</span>
                        <span className="tree-meta">{endpoint.endpointType}</span>
                      </li>
                    ))
                  ) : (
                    <li className="tree-empty">{t("ui.workspace.empty_section", "Aucun element")}</li>
                  )}
                </ul>
              </li>

              <li className="tree-section">
                <div className="tree-section-title">{t("ui.workspace.streams_section", "Flux")}</div>
                <ul className="tree-sublist">
                  {projectSnapshot.streams.length > 0 ? (
                    projectSnapshot.streams.map((stream) => (
                      <li key={stream.id} className="tree-row">
                        <span>{stream.name}</span>
                        <span className="tree-meta">{stream.direction}</span>
                      </li>
                    ))
                  ) : (
                    <li className="tree-empty">{t("ui.workspace.empty_section", "Aucun element")}</li>
                  )}
                </ul>
              </li>

              <li className="tree-section">
                <div className="tree-section-title">{t("ui.workspace.plugins_section", "Plugins")}</div>
                <ul className="tree-sublist">
                  {projectSnapshot.plugins.length > 0 ? (
                    projectSnapshot.plugins.map((plugin) => (
                      <li key={plugin.pluginId} className="tree-row">
                        <span>{plugin.pluginId}</span>
                        <span className="tree-meta">
                          {plugin.enabled
                            ? t("ui.workspace.enabled", "active")
                            : t("ui.workspace.disabled", "inactive")}
                        </span>
                      </li>
                    ))
                  ) : (
                    <li className="tree-empty">{t("ui.workspace.empty_section", "Aucun element")}</li>
                  )}
                </ul>
              </li>
            </ul>
          </Panel>

          <Panel
            title={t("ui.panel.properties", "Proprietes")}
            accent="F4"
            collapsed={!panelState.properties}
            onToggle={() => togglePanel("properties")}
            toggleLabel={panelToggleLabel("properties")}
          >
            <dl className="property-grid">
              <dt>{t("ui.property.project", "Projet")}</dt>
              <dd>{currentStatus.projectName}</dd>
              <dt>{t("ui.property.project_id", "ID projet")}</dt>
              <dd>{projectSnapshot.details.projectId}</dd>
              <dt>{t("ui.property.runtime", "Runtime")}</dt>
              <dd>{runtimeLabel(locale, currentStatus.runtime)}</dd>
              <dt>{t("ui.property.default_frame", "Repere par defaut")}</dt>
              <dd>{projectSnapshot.details.defaultFrame}</dd>
              <dt>{t("ui.property.root_scene", "Scene racine")}</dt>
              <dd>{projectSnapshot.details.rootSceneId ?? "-"}</dd>
              <dt>{t("ui.property.endpoints", "Endpoints")}</dt>
              <dd>{currentStatus.endpointCount}</dd>
              <dt>{t("ui.property.streams", "Flux")}</dt>
              <dd>{currentStatus.streamCount}</dd>
              <dt>{t("ui.property.plugins", "Plugins")}</dt>
              <dd>{currentStatus.pluginCount}</dd>
              <dt>{t("ui.property.language", "Langue")}</dt>
              <dd>{supportedLocales.find((entry) => entry.id === locale)?.label ?? locale}</dd>
              <dt>{t("ui.property.fixture", "Fixture")}</dt>
              <dd>{fixtureLabel(fixtureProjects, selectedFixtureId)}</dd>
            </dl>
          </Panel>
        </aside>

        <div
          className={
            leftExpanded
              ? dragSide === "left"
                ? "workspace-resizer is-active"
                : "workspace-resizer"
              : "workspace-resizer workspace-resizer-hidden"
          }
          aria-hidden={!leftExpanded}
          aria-label={resizeHandleLabel("left")}
          title={resizeHandleLabel("left")}
          onMouseDown={(event) => startDockResize("left", event)}
          onDoubleClick={() => resetDockWidth("left")}
        />

        <section className="workspace-center">
          <Panel
            title={t("ui.panel.command_surface", "Surface de commandes")}
            accent={menu.label}
            collapsed={!panelState.commandSurface}
            onToggle={() => togglePanel("commandSurface")}
            toggleLabel={panelToggleLabel("commandSurface")}
          >
            <ul className="command-list">
              {menu.items.map((item, index) =>
                item.type === "separator" ? (
                  <li key={`${menu.id}-sep-${index}`} className="separator" />
                ) : (
                  <li key={item.id} className="command-row">
                    <div>
                      <strong>{item.label}</strong>
                      <div className="command-id">{item.command}</div>
                    </div>
                    <div className="command-actions">
                      <span className="shortcut">{item.shortcut ?? ""}</span>
                      <button
                        className="run-button"
                        type="button"
                        disabled={executingCommandId !== null}
                        onClick={() => handleCommandExecute(item.command)}
                      >
                        {executingCommandId === item.command
                          ? t("ui.command.running", "Execution...")
                          : t("ui.command.run", "Executer")}
                      </button>
                    </div>
                  </li>
                )
              )}
            </ul>
          </Panel>

          <Panel
            title={t("ui.panel.viewport", "Viewport 3D")}
            accent={projectSnapshot.details.rootSceneId ?? t("ui.panel.scene_host", "Hote de scene")}
            collapsed={!panelState.viewport}
            onToggle={() => togglePanel("viewport")}
            toggleLabel={panelToggleLabel("viewport")}
          >
            <div className="viewport-card">
              <div className="viewport-wireframe" />
              <div className="viewport-caption">{t("ui.viewport.caption", "Shell React/Tauri")}</div>
            </div>
          </Panel>
        </section>

        <div
          className={
            rightExpanded
              ? dragSide === "right"
                ? "workspace-resizer is-active"
                : "workspace-resizer"
              : "workspace-resizer workspace-resizer-hidden"
          }
          aria-hidden={!rightExpanded}
          aria-label={resizeHandleLabel("right")}
          title={resizeHandleLabel("right")}
          onMouseDown={(event) => startDockResize("right", event)}
          onDoubleClick={() => resetDockWidth("right")}
        />

        <aside className={rightExpanded ? "workspace-right" : "workspace-right workspace-column-collapsed"}>
          <Panel
            title={t("ui.panel.ai_assistant", "Assistant IA local")}
            accent={assistantAccent(locale, aiRuntime)}
            collapsed={!panelState.aiAssistant}
            onToggle={() => togglePanel("aiAssistant")}
            toggleLabel={panelToggleLabel("aiAssistant")}
          >
            <div className="stack-block">
              <div className={aiRuntime.available ? "assistant-runtime ready" : "assistant-runtime fallback"}>
                <div className="assistant-runtime-row">
                  <strong>
                    {aiRuntime.available
                      ? t("ui.ai.runtime_ready", "Pret")
                      : t("ui.ai.runtime_fallback", "Fallback local")}
                  </strong>
                  <span className="command-id">
                    {aiRuntime.provider} | {aiRuntime.activeModel ?? t("ui.ai.no_model", "aucun modele")}
                  </span>
                </div>
                <div className="muted">
                  {t("ui.ai.endpoint", "Endpoint")} {aiRuntime.endpoint}
                </div>
                {aiRuntime.warning ? <div className="muted">{aiRuntime.warning}</div> : null}
              </div>

              <div className="assistant-thread">
                {aiMessages.length > 0 ? (
                  aiMessages.map((entry, index) => (
                    <article
                      key={`${entry.role}-${index}`}
                      className={entry.role === "assistant" ? "assistant-message assistant" : "assistant-message user"}
                    >
                      <div className="assistant-message-header">
                        <strong>{assistantRoleLabel(locale, entry.role)}</strong>
                        <span className="command-id">{entry.source}</span>
                      </div>
                      <div className="assistant-message-body">{entry.content}</div>
                      {entry.references?.length > 0 ? (
                        <div className="assistant-message-tags">
                          {entry.references.map((reference) => (
                            <span key={reference} className="assistant-tag">
                              {reference}
                            </span>
                          ))}
                        </div>
                      ) : null}
                      {entry.warnings?.length > 0 ? (
                        <div className="assistant-warning-list">
                          {entry.warnings.map((warning) => (
                            <div key={warning} className="muted">
                              {warning}
                            </div>
                          ))}
                        </div>
                      ) : null}
                    </article>
                  ))
                ) : (
                  <div className="assistant-empty">
                    <p className="muted">
                      {t(
                        "ui.ai.no_messages",
                        "Aucune discussion pour l instant. Le chat s appuie sur le projet charge et reste local."
                      )}
                    </p>
                    <div className="assistant-starters">
                      {starterPrompts.map((prompt) => (
                        <button
                          key={prompt}
                          className="assistant-starter"
                          type="button"
                          disabled={aiBusy}
                          onClick={() => submitAiMessage(prompt)}
                        >
                          {prompt}
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>

              <form className="assistant-form" onSubmit={handleAiSubmit}>
                <textarea
                  ref={aiInputRef}
                  className="assistant-input"
                  value={aiDraft}
                  onChange={(event) => setAiDraft(event.target.value)}
                  placeholder={t(
                    "ui.ai.placeholder",
                    "Pose une question sur le projet courant, la simulation, l integration ou la safety..."
                  )}
                  rows={4}
                />
                <div className="assistant-form-footer">
                  <span className="muted">{t("ui.ai.local_only", "Mode local uniquement")}</span>
                  <button
                    className="run-button"
                    type="submit"
                    disabled={aiBusy || aiDraft.trim().length === 0}
                  >
                    {aiBusy
                      ? t("ui.ai.sending", "Generation...")
                      : t("ui.ai.send", "Envoyer")}
                  </button>
                </div>
              </form>
            </div>
          </Panel>

          <Panel
            title={t("ui.panel.output", "Sortie")}
            accent={t("ui.panel.live", "Actif")}
            collapsed={!panelState.output}
            onToggle={() => togglePanel("output")}
            toggleLabel={panelToggleLabel("output")}
          >
            <div className="stack-block">
              <div className="subsection-label">{t("ui.command.last_result", "Dernier resultat")}</div>
              {commandResult ? (
                <div className="result-card">
                  <strong>{commandResult.commandId}</strong>
                  <div className="command-id">{commandResult.status}</div>
                  <div className="muted">{commandResult.message}</div>
                </div>
              ) : (
                <p className="muted">{t("ui.command.no_result", "Aucune commande executee pendant cette session.")}</p>
              )}

              <div className="subsection-label">{t("ui.output.recent_activity", "Activite recente")}</div>
              {projectSnapshot.recentActivity.length > 0 ? (
                <ul className="command-list">
                  {projectSnapshot.recentActivity.map((entry) => (
                    <li key={entry.id} className="command-row">
                      <div>
                        <strong>{entry.kind}</strong>
                        <div className="command-id">
                          {activityChannelLabel(locale, entry.channel)} | {entry.targetId ?? currentStatus.projectName}
                        </div>
                      </div>
                      <span className="shortcut">{entry.timestamp}</span>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="muted">{t("ui.output.no_activity", "Aucune activite commande/evenement.")}</p>
              )}

              <div className="subsection-label">{t("ui.output.raw_status", "Etat brut")}</div>
              <pre className="output-box">
{JSON.stringify(currentStatus, null, 2)}
              </pre>
            </div>
          </Panel>

          <Panel
            title={t("ui.panel.problems", "Problemes")}
            accent={t("ui.problems.none_blocking", "0 bloquant")}
            collapsed={!panelState.problems}
            onToggle={() => togglePanel("problems")}
            toggleLabel={panelToggleLabel("problems")}
          >
            <p className="muted">{t("ui.output.problems", "Les checks critiques remontent ici.")}</p>
          </Panel>
        </aside>
      </main>
    </div>
  );
}
