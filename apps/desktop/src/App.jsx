import { useEffect, useState } from "react";

import { visualStudioInspiredMenus } from "@futureaero/ui";

const FALLBACK_STATUS = {
  runtime: "web-preview",
  projectName: "Shell Preview",
  entityCount: 0,
  endpointCount: 0,
  pluginCount: 0
};

function MenuBar() {
  return (
    <nav className="menu-bar" aria-label="Application menu">
      {visualStudioInspiredMenus.map((menu) => (
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

async function fetchBackendStatus() {
  try {
    const tauriCore = await import("@tauri-apps/api/core");
    return await tauriCore.invoke("backend_status");
  } catch {
    return FALLBACK_STATUS;
  }
}

export default function App() {
  const [backendStatus, setBackendStatus] = useState(FALLBACK_STATUS);
  const [activeMenu, setActiveMenu] = useState("File");

  useEffect(() => {
    let mounted = true;

    fetchBackendStatus().then((status) => {
      if (mounted) {
        setBackendStatus(status);
      }
    });

    return () => {
      mounted = false;
    };
  }, []);

  const menu = visualStudioInspiredMenus.find((entry) => entry.label === activeMenu) ?? visualStudioInspiredMenus[0];

  return (
    <div className="shell">
      <header className="shell-header">
        <div className="brand-block">
          <div className="brand-mark">FA</div>
          <div>
            <div className="brand-title">FutureAero</div>
            <div className="brand-subtitle">Desktop shell inspired by Visual Studio</div>
          </div>
        </div>
        <div className="status-pills">
          <span className="status-pill">{backendStatus.runtime}</span>
          <span className="status-pill">{backendStatus.projectName}</span>
        </div>
      </header>

      <MenuBar />

      <div className="toolbar">
        {visualStudioInspiredMenus.map((entry) => (
          <button
            key={entry.id}
            className={entry.label === activeMenu ? "tool-button active" : "tool-button"}
            type="button"
            onClick={() => setActiveMenu(entry.label)}
          >
            {entry.label}
          </button>
        ))}
      </div>

      <main className="workspace">
        <aside className="workspace-left">
          <Panel title="Project Explorer" accent={`${backendStatus.entityCount} entities`}>
            <ul className="tree-list">
              <li>FutureAero Workspace</li>
              <li>Robot Cell</li>
              <li>Simulation</li>
              <li>Integration</li>
              <li>Plugins</li>
            </ul>
          </Panel>
          <Panel title="Properties" accent="F4">
            <dl className="property-grid">
              <dt>Runtime</dt>
              <dd>{backendStatus.runtime}</dd>
              <dt>Endpoints</dt>
              <dd>{backendStatus.endpointCount}</dd>
              <dt>Plugins</dt>
              <dd>{backendStatus.pluginCount}</dd>
            </dl>
          </Panel>
        </aside>

        <section className="workspace-center">
          <Panel title="Command Surface" accent={menu.label}>
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
          <Panel title="3D Viewport" accent="Scene host">
            <div className="viewport-card">
              <div className="viewport-wireframe" />
              <div className="viewport-caption">
                Shell React/Tauri branche au backend Rust minimal via `invoke("backend_status")`.
              </div>
            </div>
          </Panel>
        </section>

        <aside className="workspace-right">
          <Panel title="Output" accent="Live">
            <pre className="output-box">
{JSON.stringify(backendStatus, null, 2)}
            </pre>
          </Panel>
          <Panel title="Problems" accent="0 blocking">
            <p className="muted">
              Les checks critiques remontent ici: build, simulation, integration, coverage, plugins.
            </p>
          </Panel>
        </aside>
      </main>
    </div>
  );
}

