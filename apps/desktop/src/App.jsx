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
  pluginCount: 0
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

async function fetchBackendStatus() {
  return (await invokeBackend("backend_status")) ?? FALLBACK_STATUS;
}

async function fetchFixtureProjects() {
  return (await invokeBackend("available_fixture_projects")) ?? FALLBACK_FIXTURES;
}

async function loadFixtureProject(projectId) {
  const status = await invokeBackend("load_fixture_project", { projectId });
  if (status) {
    return status;
  }

  const fixture = FALLBACK_FIXTURES.find((entry) => entry.id === projectId) ?? FALLBACK_FIXTURES[0];
  return {
    ...FALLBACK_STATUS,
    fixtureId: fixture.id,
    projectName: fixture.projectName
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

export default function App() {
  const [locale, setLocale] = useState(defaultLocale);
  const [backendStatus, setBackendStatus] = useState(FALLBACK_STATUS);
  const [fixtureProjects, setFixtureProjects] = useState(FALLBACK_FIXTURES);
  const [selectedFixtureId, setSelectedFixtureId] = useState(FALLBACK_STATUS.fixtureId);
  const [activeMenuId, setActiveMenuId] = useState("file");
  const [fixtureLoading, setFixtureLoading] = useState(false);

  const menus = localizeMenuModel(locale);
  const menu = menus.find((entry) => entry.id === activeMenuId) ?? menus[0];
  const t = (key, fallback = key) => translate(locale, key, fallback);

  useEffect(() => {
    let mounted = true;

    Promise.all([fetchFixtureProjects(), fetchBackendStatus()])
      .then(([fixtures, status]) => {
        if (!mounted) {
          return;
        }

        const nextFixtures = fixtures.length > 0 ? fixtures : FALLBACK_FIXTURES;
        const nextFixtureId = status.fixtureId ?? nextFixtures[0].id;

        setFixtureProjects(nextFixtures);
        setBackendStatus(status);
        setSelectedFixtureId(nextFixtureId);
      })
      .catch(() => {
        if (!mounted) {
          return;
        }

        setFixtureProjects(FALLBACK_FIXTURES);
        setBackendStatus(FALLBACK_STATUS);
        setSelectedFixtureId(FALLBACK_STATUS.fixtureId);
      });

    return () => {
      mounted = false;
    };
  }, []);

  async function handleFixtureChange(event) {
    const nextFixtureId = event.target.value;
    setSelectedFixtureId(nextFixtureId);
    setFixtureLoading(true);

    try {
      const status = await loadFixtureProject(nextFixtureId);
      setBackendStatus(status);
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
            <span className="status-pill">{runtimeLabel(locale, backendStatus.runtime)}</span>
            <span className="status-pill">{backendStatus.projectName}</span>
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
            accent={`${backendStatus.entityCount} ${t("ui.workspace.entities", "entites")}`}
          >
            <ul className="tree-list">
              <li>{t("ui.workspace.root", "Workspace FutureAero")}</li>
              <li>{t("ui.workspace.robot_cell", "Cellule robotique")}</li>
              <li>{t("ui.workspace.simulation", "Simulation")}</li>
              <li>{t("ui.workspace.integration", "Integration")}</li>
              <li>{t("ui.workspace.plugins", "Plugins")}</li>
            </ul>
          </Panel>

          <Panel title={t("ui.panel.properties", "Proprietes")} accent="F4">
            <dl className="property-grid">
              <dt>{t("ui.property.project", "Projet")}</dt>
              <dd>{backendStatus.projectName}</dd>
              <dt>{t("ui.property.runtime", "Runtime")}</dt>
              <dd>{runtimeLabel(locale, backendStatus.runtime)}</dd>
              <dt>{t("ui.property.endpoints", "Endpoints")}</dt>
              <dd>{backendStatus.endpointCount}</dd>
              <dt>{t("ui.property.plugins", "Plugins")}</dt>
              <dd>{backendStatus.pluginCount}</dd>
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

          <Panel title={t("ui.panel.viewport", "Viewport 3D")} accent={t("ui.panel.scene_host", "Hote de scene")}>
            <div className="viewport-card">
              <div className="viewport-wireframe" />
              <div className="viewport-caption">{t("ui.viewport.caption", "Shell React/Tauri")}</div>
            </div>
          </Panel>
        </section>

        <aside className="workspace-right">
          <Panel title={t("ui.panel.output", "Sortie")} accent={t("ui.panel.live", "Actif")}>
            <pre className="output-box">
{JSON.stringify(backendStatus, null, 2)}
            </pre>
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
