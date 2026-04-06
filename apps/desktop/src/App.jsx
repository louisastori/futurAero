import { useEffect, useState } from "react";

import {
  defaultLocale,
  localizeMenuModel,
  supportedLocales,
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

function MenuBar({ menus }) {
  return (
    <nav className="menu-bar" aria-label="Application menu">
      {menus.map((menu) => (
        <button key={menu.id} className="menu-button" type="button">
          {menu.label}
        </button>
      ))}
    </nav>
  );
}

function Panel({ title, children, accent }) {
  return (
    <section className="panel">
      <header className="panel-header">
        <span className="panel-title">{title}</span>
        {accent ? <span className="panel-accent">{accent}</span> : null}
      </header>
      <div className="panel-body">{children}</div>
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

async function fetchBackendStatus() {
  return (await invokeBackend("backend_status")) ?? FALLBACK_STATUS;
}

async function fetchFixtureProjects() {
  return (await invokeBackend("available_fixture_projects")) ?? FALLBACK_FIXTURES;
}

async function fetchProjectSnapshot(projectId) {
  return (await invokeBackend("load_project_snapshot", { projectId })) ?? buildFallbackSnapshot(projectId);
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

  return translate(locale, "ui.activity.event", channel);
}

export default function App() {
  const [locale, setLocale] = useState(defaultLocale);
  const [projectSnapshot, setProjectSnapshot] = useState(FALLBACK_SNAPSHOT);
  const [fixtureProjects, setFixtureProjects] = useState(FALLBACK_FIXTURES);
  const [selectedFixtureId, setSelectedFixtureId] = useState(FALLBACK_STATUS.fixtureId);
  const [activeMenuId, setActiveMenuId] = useState("file");
  const [fixtureLoading, setFixtureLoading] = useState(false);

  const menus = localizeMenuModel(locale);
  const menu = menus.find((entry) => entry.id === activeMenuId) ?? menus[0];
  const currentStatus = projectSnapshot.status;
  const t = (key, fallback = key) => translate(locale, key, fallback);

  useEffect(() => {
    let mounted = true;

    async function bootstrapWorkspace() {
      try {
        const fixtures = await fetchFixtureProjects();
        const nextFixtures = fixtures.length > 0 ? fixtures : FALLBACK_FIXTURES;
        const status = await fetchBackendStatus();
        const nextFixtureId = status.fixtureId ?? nextFixtures[0].id;
        const snapshot = await fetchProjectSnapshot(nextFixtureId);

        if (!mounted) {
          return;
        }

        setFixtureProjects(nextFixtures);
        setProjectSnapshot(snapshot);
        setSelectedFixtureId(nextFixtureId);
      } catch {
        if (!mounted) {
          return;
        }

        setFixtureProjects(FALLBACK_FIXTURES);
        setProjectSnapshot(buildFallbackSnapshot());
        setSelectedFixtureId(FALLBACK_STATUS.fixtureId);
      }
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
      const snapshot = await fetchProjectSnapshot(nextFixtureId);
      setProjectSnapshot(snapshot);
    } finally {
      setFixtureLoading(false);
    }
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
                disabled={fixtureProjects.length === 0}
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
            <span className="status-pill">
              {fixtureLoading
                ? t("ui.fixture.loading", "Chargement...")
                : fixtureLabel(fixtureProjects, selectedFixtureId)}
            </span>
          </div>
        </div>
      </header>

      <MenuBar menus={menus} />

      <div className="toolbar">
        {menus.map((entry) => (
          <button
            key={entry.id}
            className={entry.id === activeMenuId ? "tool-button active" : "tool-button"}
            type="button"
            onClick={() => setActiveMenuId(entry.id)}
          >
            {entry.label}
          </button>
        ))}
      </div>

      <main className="workspace">
        <aside className="workspace-left">
          <Panel
            title={t("ui.panel.project_explorer", "Explorateur de projet")}
            accent={`${currentStatus.entityCount} ${t("ui.workspace.entities", "entites")}`}
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

          <Panel title={t("ui.panel.properties", "Proprietes")} accent="F4">
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

        <section className="workspace-center">
          <Panel title={t("ui.panel.command_surface", "Surface de commandes")} accent={menu.label}>
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
                    <span className="shortcut">{item.shortcut ?? ""}</span>
                  </li>
                )
              )}
            </ul>
          </Panel>

          <Panel
            title={t("ui.panel.viewport", "Viewport 3D")}
            accent={projectSnapshot.details.rootSceneId ?? t("ui.panel.scene_host", "Hote de scene")}
          >
            <div className="viewport-card">
              <div className="viewport-wireframe" />
              <div className="viewport-caption">{t("ui.viewport.caption", "Shell React/Tauri")}</div>
            </div>
          </Panel>
        </section>

        <aside className="workspace-right">
          <Panel title={t("ui.panel.output", "Sortie")} accent={t("ui.panel.live", "Actif")}>
            <div className="stack-block">
              <div className="subsection-label">{t("ui.output.recent_activity", "Activite recente")}</div>
              {projectSnapshot.recentActivity.length > 0 ? (
                <ul className="command-list">
                  {projectSnapshot.recentActivity.map((entry) => (
                    <li key={entry.id} className="command-row">
                      <div>
                        <strong>{entry.kind}</strong>
                        <div className="command-id">
                          {activityChannelLabel(locale, entry.channel)} · {entry.targetId ?? currentStatus.projectName}
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
          >
            <p className="muted">{t("ui.output.problems", "Les checks critiques remontent ici.")}</p>
          </Panel>
        </aside>
      </main>
    </div>
  );
}
