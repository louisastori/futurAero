import { useEffect, useRef, useState } from "react";

import {
  buildEntityInspectorSchema,
  buildInspectorDraftFromSchema,
  calculateResizedDockWidths,
  coerceInspectorDraftValue,
  defaultWorkspaceDockWidths,
  defaultWorkspacePanels,
  defaultLocale,
  findMenuEntryByCommand,
  findMenuCommandByShortcut,
  findInspectorFieldByPath,
  flattenEditableInspectorFields,
  formatShortcutLabel,
  getWorkspaceColumnState,
  getVisibleSidebarWidth,
  localizeMenuModel,
  panelIdFromCommand,
  shouldHandleShortcutEvent,
  supportedLocales,
  toggleWorkspacePanel,
  visualStudioInspiredMenus,
  WORKSPACE_RESIZER_WIDTH,
  translate,
} from "@futureaero/ui";
import {
  aerospaceReferenceScenes,
  defaultAerospaceSceneId,
  getAerospaceScene,
} from "@futureaero/viewport";

const FALLBACK_FIXTURES = [
  { id: "pick-and-place-demo.faero", projectName: "Pick And Place Demo" },
  {
    id: "wireless-integration-demo.faero",
    projectName: "Wireless Integration Demo",
  },
  { id: "empty-project.faero", projectName: "Empty Project" },
];

const FALLBACK_STATUS = {
  runtime: "web-preview",
  fixtureId: "pick-and-place-demo.faero",
  projectName: "Shell Preview",
  entityCount: 0,
  endpointCount: 0,
  streamCount: 0,
  pluginCount: 0,
};

const FALLBACK_SNAPSHOT = {
  status: FALLBACK_STATUS,
  details: {
    projectId: "prj_preview",
    formatVersion: "0.1.0",
    defaultFrame: "world",
    rootSceneId: null,
    activeConfigurationId: "cfg_default",
  },
  entities: [],
  endpoints: [],
  streams: [],
  plugins: [],
  openSpecDocuments: [
    {
      id: "ops_preview_layout",
      title: "Preview Layout Intent",
      kind: "design_intent",
      status: "active",
      linkedEntityCount: 1,
      linkedExternalCount: 1,
      tagCount: 2,
      excerpt: "Cellule lisible en clair sans dependre d un binaire vendor.",
    },
  ],
  recentActivity: [],
};

const FALLBACK_AI_STATUS = {
  available: false,
  provider: "ollama",
  endpoint: "http://127.0.0.1:11434",
  mode: "web-preview",
  localOnly: true,
  activeProfile: "balanced",
  availableProfiles: ["balanced", "max", "furnace"],
  activeModel: null,
  availableModels: [],
  gemma3Models: [],
  warning: "Local AI runtime unavailable in web preview.",
};

const COMMAND_TOAST_TIMEOUT_MS = 2400;
const NATIVE_MENU_EVENT_NAME = "futureaero:menu-command";

function getGemma3Models(runtime) {
  if (Array.isArray(runtime?.gemma3Models) && runtime.gemma3Models.length > 0) {
    return [...new Set(runtime.gemma3Models)];
  }

  return [
    ...new Set(
      (runtime?.availableModels ?? []).filter((model) =>
        model.startsWith("gemma3:"),
      ),
    ),
  ];
}

function defaultGemma3Model(runtime) {
  const gemma3Models = getGemma3Models(runtime);
  if (gemma3Models.includes("gemma3:27b")) {
    return "gemma3:27b";
  }

  if (runtime?.activeModel && gemma3Models.includes(runtime.activeModel)) {
    return runtime.activeModel;
  }

  return gemma3Models[0] ?? "";
}

function getAvailableAiProfiles(runtime) {
  if (
    Array.isArray(runtime?.availableProfiles) &&
    runtime.availableProfiles.length > 0
  ) {
    return [...new Set(runtime.availableProfiles)];
  }

  return ["balanced", "max", "furnace"];
}

function MenuBar({ menus, activeMenuId, onSelect }) {
  return (
    <nav className="menu-bar" aria-label="Application menu">
      {menus.map((menu) => (
        <button
          key={menu.id}
          data-menu-id={menu.id}
          className={
            menu.id === activeMenuId ? "menu-button active" : "menu-button"
          }
          type="button"
          onClick={() => onSelect(menu.id)}
        >
          {menu.label}
        </button>
      ))}
    </nav>
  );
}

function Panel({
  panelId,
  title,
  children,
  accent,
  collapsed = false,
  onToggle,
  toggleLabel,
}) {
  return (
    <section
      className={collapsed ? "panel panel-collapsed" : "panel"}
      data-panel-id={panelId}
    >
      <header className="panel-header">
        <div className="panel-header-main">
          <span className="panel-title">{title}</span>
          {accent ? <span className="panel-accent">{accent}</span> : null}
        </div>
        {onToggle ? (
          <button
            className="panel-toggle"
            type="button"
            data-panel-toggle={panelId}
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

function ViewportLegend({ items }) {
  return (
    <ul className="viewport-legend-list">
      {items.map((item) => (
        <li key={item.label} className="viewport-legend-item">
          <span
            className="viewport-legend-swatch"
            style={{ background: item.color }}
          />
          <span>{item.label}</span>
        </li>
      ))}
    </ul>
  );
}

function TurbofanReferenceArtwork() {
  const fanBlades = Array.from({ length: 16 }, (_, index) => {
    const angle = index * 22.5;
    return (
      <g key={`fan-${angle}`} transform={`translate(138 186) rotate(${angle})`}>
        <path
          d="M0 -12 C18 -36 36 -72 40 -120 C24 -102 12 -70 -4 -28 Z"
          fill="rgba(179,197,255,0.72)"
          stroke="rgba(240,246,255,0.38)"
          strokeWidth="1"
        />
      </g>
    );
  });

  const compressorStages = Array.from({ length: 8 }, (_, index) => (
    <g key={`stage-${index}`} transform={`translate(${322 + index * 58} 140)`}>
      <ellipse
        cx="0"
        cy="54"
        rx="34"
        ry="98"
        fill="none"
        stroke="rgba(210,160,255,0.62)"
        strokeWidth="1.5"
      />
      <ellipse
        cx="0"
        cy="54"
        rx="20"
        ry="66"
        fill="none"
        stroke="rgba(135,255,205,0.45)"
        strokeWidth="1.25"
      />
      <line x1="-34" y1="54" x2="34" y2="54" stroke="rgba(255,255,255,0.18)" />
    </g>
  ));

  return (
    <svg viewBox="0 0 960 420" className="viewport-svg" aria-hidden="true">
      <defs>
        <linearGradient id="turbofan-core" x1="0%" x2="100%">
          <stop offset="0%" stopColor="#8ad1ff" stopOpacity="0.92" />
          <stop offset="100%" stopColor="#5fa9ff" stopOpacity="0.25" />
        </linearGradient>
        <linearGradient id="turbofan-shell" x1="0%" x2="100%">
          <stop offset="0%" stopColor="#dfe7ff" stopOpacity="0.55" />
          <stop offset="100%" stopColor="#ffffff" stopOpacity="0.08" />
        </linearGradient>
      </defs>
      <rect width="960" height="420" rx="24" fill="#0f1622" />
      <g opacity="0.18">
        <circle
          cx="126"
          cy="186"
          r="122"
          fill="none"
          stroke="#7bc6ff"
          strokeWidth="1.4"
        />
        <circle
          cx="126"
          cy="186"
          r="82"
          fill="none"
          stroke="#7bc6ff"
          strokeWidth="1"
        />
      </g>
      <g>{fanBlades}</g>
      <ellipse
        cx="140"
        cy="186"
        rx="54"
        ry="54"
        fill="#171d2d"
        stroke="rgba(255,255,255,0.18)"
        strokeWidth="4"
      />
      <ellipse
        cx="140"
        cy="186"
        rx="20"
        ry="20"
        fill="#d7e3ff"
        fillOpacity="0.86"
      />
      <path
        d="M160 130 L270 130 L308 186 L270 242 L160 242 Q128 214 128 186 Q128 158 160 130 Z"
        fill="url(#turbofan-shell)"
        stroke="rgba(255,255,255,0.22)"
        strokeWidth="2"
      />
      <rect
        x="190"
        y="170"
        width="592"
        height="32"
        rx="16"
        fill="url(#turbofan-core)"
      />
      <g>{compressorStages}</g>
      <g opacity="0.6">
        <path d="M248 130 L248 242" stroke="#7ae7c7" strokeDasharray="5 6" />
        <path d="M296 110 L296 262" stroke="#7ae7c7" strokeDasharray="5 6" />
        <path d="M784 146 L900 112" stroke="#ffe37a" strokeWidth="2.5" />
        <path d="M784 226 L900 258" stroke="#ff92ca" strokeWidth="2.5" />
      </g>
    </svg>
  );
}

function AirframeTransparentArtwork() {
  const frames = Array.from({ length: 13 }, (_, index) => (
    <ellipse
      key={`frame-${index}`}
      cx={150 + index * 42}
      cy="188"
      rx={index > 9 ? 22 : 28}
      ry={index > 9 ? 64 : 92}
      fill="none"
      stroke="rgba(236,243,255,0.4)"
      strokeWidth="1.2"
    />
  ));

  const ribs = Array.from({ length: 9 }, (_, index) => (
    <line
      key={`rib-${index}`}
      x1={250 + index * 46}
      y1="240"
      x2={168 + index * 50}
      y2="328"
      stroke="rgba(126,199,255,0.48)"
      strokeWidth="1.4"
    />
  ));

  return (
    <svg viewBox="0 0 960 420" className="viewport-svg" aria-hidden="true">
      <rect width="960" height="420" rx="24" fill="#10153a" />
      <path
        d="M84 196 C144 136 232 108 394 114 L704 126 C776 130 834 160 884 184 C838 204 794 230 742 242 L504 258 C324 270 174 252 88 212 Z"
        fill="rgba(180,203,255,0.08)"
        stroke="rgba(220,230,255,0.42)"
        strokeWidth="2.4"
      />
      <path d="M82 196 L44 202 L86 170 Z" fill="rgba(255,255,255,0.18)" />
      <path
        d="M430 128 L586 54 L602 130"
        fill="rgba(124,199,255,0.08)"
        stroke="rgba(190,232,255,0.42)"
        strokeWidth="2"
      />
      <path
        d="M458 248 L684 324 L602 250"
        fill="rgba(124,199,255,0.06)"
        stroke="rgba(190,232,255,0.42)"
        strokeWidth="2"
      />
      <path
        d="M350 250 L168 340 L292 252"
        fill="rgba(255,140,200,0.05)"
        stroke="rgba(255,196,92,0.45)"
        strokeWidth="2"
      />
      <g>{frames}</g>
      <g>{ribs}</g>
      <g opacity="0.88">
        <rect x="300" y="178" width="88" height="14" rx="7" fill="#ff8ec7" />
        <rect x="432" y="172" width="122" height="16" rx="8" fill="#7cd6ff" />
        <rect x="586" y="166" width="92" height="18" rx="9" fill="#ffe783" />
        <circle
          cx="618"
          cy="188"
          r="42"
          fill="none"
          stroke="rgba(255,255,255,0.48)"
          strokeWidth="2.2"
        />
        <circle
          cx="676"
          cy="192"
          r="44"
          fill="none"
          stroke="rgba(255,255,255,0.48)"
          strokeWidth="2.2"
        />
      </g>
    </svg>
  );
}

function WireframeMaintenanceArtwork() {
  const grid = Array.from({ length: 15 }, (_, index) => (
    <line
      key={`diag-${index}`}
      x1={90 + index * 48}
      y1="330"
      x2={160 + index * 38}
      y2="122"
      stroke="rgba(192,214,255,0.28)"
      strokeWidth="1"
    />
  ));

  return (
    <svg viewBox="0 0 960 420" className="viewport-svg" aria-hidden="true">
      <rect width="960" height="420" rx="24" fill="#111749" />
      <path
        d="M108 220 C160 168 260 132 424 126 L712 136 L854 176 L724 214 L416 244 C270 258 164 254 108 220 Z"
        fill="none"
        stroke="rgba(228,240,255,0.58)"
        strokeWidth="2"
      />
      <path
        d="M394 134 L550 70 L572 138"
        fill="none"
        stroke="rgba(160,200,255,0.52)"
        strokeWidth="1.8"
      />
      <path
        d="M418 244 L668 340 L572 246"
        fill="none"
        stroke="rgba(160,200,255,0.52)"
        strokeWidth="1.8"
      />
      <path
        d="M312 246 L146 338 L256 248"
        fill="none"
        stroke="rgba(160,200,255,0.52)"
        strokeWidth="1.8"
      />
      <circle
        cx="690"
        cy="196"
        r="54"
        fill="none"
        stroke="rgba(244,255,141,0.5)"
        strokeWidth="1.6"
      />
      <circle
        cx="754"
        cy="192"
        r="58"
        fill="none"
        stroke="rgba(244,255,141,0.5)"
        strokeWidth="1.6"
      />
      <g>{grid}</g>
      <g opacity="0.82">
        <path d="M74 124 L116 124 L96 88 Z" fill="#f1ff85" />
        <path d="M88 136 L88 248" stroke="#f1ff85" strokeWidth="2.5" />
        <circle
          cx="126"
          cy="300"
          r="62"
          fill="none"
          stroke="rgba(212,166,255,0.65)"
          strokeWidth="2"
          strokeDasharray="6 6"
        />
        <path
          d="M146 300 L280 242"
          stroke="#d4a6ff"
          strokeWidth="2.2"
          strokeDasharray="6 6"
        />
      </g>
    </svg>
  );
}

function StressMapArtwork() {
  return (
    <svg viewBox="0 0 960 420" className="viewport-svg" aria-hidden="true">
      <defs>
        <linearGradient id="stressGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#1e7fff" />
          <stop offset="35%" stopColor="#49d9ff" />
          <stop offset="62%" stopColor="#ffe066" />
          <stop offset="100%" stopColor="#ff506a" />
        </linearGradient>
      </defs>
      <rect width="960" height="420" rx="24" fill="#18203a" />
      <path
        d="M178 334 L252 126 C270 84 314 72 346 98 L412 154 L474 104 C510 74 562 88 578 130 L664 332"
        fill="none"
        stroke="rgba(255,255,255,0.14)"
        strokeWidth="42"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M178 334 L252 126 C270 84 314 72 346 98 L412 154 L474 104 C510 74 562 88 578 130 L664 332"
        fill="none"
        stroke="url(#stressGradient)"
        strokeWidth="28"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="412" cy="154" r="22" fill="#ff5570" fillOpacity="0.9" />
      <circle cx="474" cy="104" r="18" fill="#ffd766" fillOpacity="0.9" />
      <circle cx="252" cy="126" r="16" fill="#63e4ff" fillOpacity="0.9" />
      <rect
        x="764"
        y="72"
        width="22"
        height="216"
        rx="11"
        fill="url(#stressGradient)"
      />
      <text x="804" y="84" fill="#d9e9ff" fontSize="16">
        High
      </text>
      <text x="804" y="286" fill="#d9e9ff" fontSize="16">
        Low
      </text>
    </svg>
  );
}

function AeroHeatmapArtwork() {
  const streamlines = Array.from({ length: 11 }, (_, index) => (
    <path
      key={`flow-${index}`}
      d={`M${94 + index * 14} ${144 + index * 10} C${220 + index * 8} ${96 + index * 6}, ${412 + index * 10} ${118 + index * 5}, ${776 + index * 12} ${142 + index * 7}`}
      fill="none"
      stroke="rgba(160,255,152,0.34)"
      strokeWidth="1.4"
    />
  ));

  return (
    <svg viewBox="0 0 960 420" className="viewport-svg" aria-hidden="true">
      <defs>
        <linearGradient id="aeroGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#72ff9b" />
          <stop offset="46%" stopColor="#ffe56b" />
          <stop offset="100%" stopColor="#ff32c5" />
        </linearGradient>
      </defs>
      <rect width="960" height="420" rx="24" fill="#272a3f" />
      <path
        d="M136 240 C264 180 420 154 662 162 C716 164 774 186 820 210 C758 214 704 226 658 242 C438 316 284 324 126 286 Z"
        fill="rgba(255,255,255,0.05)"
        stroke="rgba(240,244,255,0.24)"
        strokeWidth="2"
      />
      <path
        d="M156 246 C274 198 418 178 646 186 C694 188 738 198 782 214 C736 218 694 230 654 246 C436 302 298 306 156 276 Z"
        fill="url(#aeroGradient)"
        fillOpacity="0.82"
        stroke="rgba(255,255,255,0.1)"
        strokeWidth="2"
      />
      <g>{streamlines}</g>
      <g opacity="0.84">
        <path d="M436 176 L494 126" stroke="#ffb6ef" strokeWidth="2.5" />
        <circle cx="494" cy="126" r="7" fill="#ffb6ef" />
        <path d="M700 238 L788 292" stroke="#ffe56b" strokeWidth="2.5" />
        <circle cx="788" cy="292" r="7" fill="#ffe56b" />
      </g>
    </svg>
  );
}

function AerospaceViewport({ locale, t, selectedSceneId, onSelectScene }) {
  const scene = getAerospaceScene(selectedSceneId);
  const legend = scene.legendKeys.map((key, index) => ({
    label: t(key, key),
    color: [
      scene.palette.accent,
      scene.palette.secondary,
      scene.palette.tertiary,
    ][index],
  }));

  let artwork = <TurbofanReferenceArtwork />;
  if (scene.id === "airframe_transparent") {
    artwork = <AirframeTransparentArtwork />;
  } else if (scene.id === "wireframe_maintenance") {
    artwork = <WireframeMaintenanceArtwork />;
  } else if (scene.id === "stress_map") {
    artwork = <StressMapArtwork />;
  } else if (scene.id === "aero_heatmap") {
    artwork = <AeroHeatmapArtwork />;
  }

  return (
    <div className="viewport-reference-layout">
      <div
        className="viewport-scene-tabs"
        role="tablist"
        aria-label={t("ui.viewport.reference_toolbar", "Scenes de reference")}
      >
        {aerospaceReferenceScenes.map((entry) => (
          <button
            key={entry.id}
            type="button"
            data-scene-id={entry.id}
            role="tab"
            aria-selected={entry.id === scene.id}
            className={
              entry.id === scene.id
                ? "viewport-scene-tab active"
                : "viewport-scene-tab"
            }
            onClick={() => onSelectScene(entry.id)}
          >
            {t(entry.titleKey, entry.id)}
          </button>
        ))}
      </div>

      <div className="viewport-showcase">
        <div className="viewport-canvas-shell">
          {artwork}
          <div className="viewport-caption">
            {t(
              "ui.viewport.reference_inspiration",
              "Reproduction originale inspiree des references fournies",
            )}
          </div>
        </div>

        <div className="viewport-inspector">
          <div className="viewport-inspector-block">
            <div className="subsection-label">
              {t("ui.viewport.reference_caption", "Lecture de la scene")}
            </div>
            <strong className="viewport-scene-title">
              {t(scene.titleKey, scene.id)}
            </strong>
            <div className="muted">{t(scene.summaryKey, scene.summaryKey)}</div>
          </div>

          <div className="viewport-inspector-block">
            <div className="subsection-label">
              {t(
                "ui.viewport.reference_analysis",
                "Analyse extraite des images",
              )}
            </div>
            <ul className="viewport-analysis-list">
              {scene.analysisKeys.map((key) => (
                <li key={key}>{t(key, key)}</li>
              ))}
            </ul>
          </div>

          <div className="viewport-inspector-block">
            <div className="subsection-label">
              {t("ui.viewport.reference_legend", "Legende")}
            </div>
            <ViewportLegend items={legend} />
          </div>

          <div className="viewport-inspector-block">
            <div className="subsection-label">
              {t("ui.locale.label", "Langue")}
            </div>
            <div className="muted">{locale.toUpperCase()}</div>
          </div>
        </div>
      </div>
    </div>
  );
}

async function invokeBackend(command, payload) {
  if (typeof window === "undefined" || typeof window.__TAURI_INTERNALS__ !== "object") {
    return null;
  }

  try {
    const tauriCore = await import("@tauri-apps/api/core");
    return await tauriCore.invoke(command, payload);
  } catch {
    return null;
  }
}

async function requestWindowClose() {
  if (typeof window === "undefined" || typeof window.__TAURI_INTERNALS__ !== "object") {
    return false;
  }

  try {
    const tauriWindow = await import("@tauri-apps/api/window");
    await tauriWindow.getCurrentWindow().close();
    return true;
  } catch {
    return false;
  }
}

function getNextFixtureId(fixtures, currentFixtureId) {
  if (fixtures.length === 0) {
    return "";
  }

  const currentIndex = fixtures.findIndex(
    (fixture) => fixture.id === currentFixtureId,
  );
  if (currentIndex === -1) {
    return fixtures[0].id;
  }

  return fixtures[(currentIndex + 1) % fixtures.length].id;
}

function buildFallbackSnapshot(projectId = FALLBACK_STATUS.fixtureId) {
  const fixture =
    FALLBACK_FIXTURES.find((entry) => entry.id === projectId) ??
    FALLBACK_FIXTURES[0];
  return {
    ...FALLBACK_SNAPSHOT,
    status: {
      ...FALLBACK_STATUS,
      fixtureId: fixture.id,
      projectName: fixture.projectName,
    },
    details: {
      ...FALLBACK_SNAPSHOT.details,
      projectId: `preview:${fixture.id}`,
    },
  };
}

function appendFallbackActivity(snapshot, channel, kind, targetId) {
  const entry = {
    id: `web_${snapshot.recentActivity.length + 1}`,
    channel,
    kind,
    timestamp: "2026-04-06T12:59:59Z",
    targetId,
  };

  return {
    ...snapshot,
    recentActivity: [entry, ...snapshot.recentActivity].slice(0, 12),
  };
}

async function fetchWorkspaceBootstrap() {
  return (
    (await invokeBackend("workspace_bootstrap")) ?? {
      fixtures: FALLBACK_FIXTURES,
      snapshot: buildFallbackSnapshot(),
    }
  );
}

async function loadWorkspaceFixture(projectId) {
  return (
    (await invokeBackend("workspace_load_fixture", { projectId })) ??
    buildFallbackSnapshot(projectId)
  );
}

async function executeWorkspaceCommand(commandId, currentSnapshot) {
  const response = await invokeBackend("workspace_execute_command", {
    commandId,
  });
  if (response) {
    return response;
  }

  if (commandId === "entity.create.part") {
    const index = currentSnapshot.entities.length + 1;
    const widthMm = 120 + index * 12;
    const heightMm = 80 + index * 6;
    const depthMm = 10 + index * 2;
    const areaMm2 = widthMm * heightMm;
    const volumeMm3 = areaMm2 * depthMm;
    const estimatedMassGrams = volumeMm3 * 0.0027;

    return {
      snapshot: {
        ...appendFallbackActivity(
          currentSnapshot,
          "system",
          "entity.create.part",
          `ent_part_${String(index).padStart(3, "0")}`,
        ),
        status: {
          ...currentSnapshot.status,
          entityCount: currentSnapshot.entities.length + 1,
        },
        entities: [
          ...currentSnapshot.entities,
          {
            id: `ent_part_${String(index).padStart(3, "0")}`,
            entityType: "Part",
            name: `Part-${String(index).padStart(3, "0")}`,
            revision: "rev_seed",
            status: "active",
            detail: `${widthMm.toFixed(1)} x ${heightMm.toFixed(1)} x ${depthMm.toFixed(1)} mm | ${estimatedMassGrams.toFixed(1)} g`,
            data: {
              tags: ["part", "parametric"],
              parameterSet: {
                widthMm,
                heightMm,
                depthMm,
              },
            },
            partGeometry: {
              state: "well_constrained",
              widthMm,
              heightMm,
              depthMm,
              pointCount: 4,
              perimeterMm: 2 * (widthMm + heightMm),
              areaMm2,
              volumeMm3,
              estimatedMassGrams,
              materialName: "Aluminum 6061",
            },
          },
        ],
      },
      result: {
        commandId,
        status: "applied",
        message: "piece parametrique regeneree dans l apercu web",
      },
    };
  }

  if (commandId === "entity.create.assembly") {
    const index = currentSnapshot.entities.length + 1;
    return {
      snapshot: {
        ...appendFallbackActivity(
          currentSnapshot,
          "system",
          "entity.create.assembly",
          `ent_asm_${String(index).padStart(3, "0")}`,
        ),
        status: {
          ...currentSnapshot.status,
          entityCount: currentSnapshot.entities.length + 1,
        },
        entities: [
          ...currentSnapshot.entities,
          {
            id: `ent_asm_${String(index).padStart(3, "0")}`,
            entityType: "Assembly",
            name: `Assembly-${String(index).padStart(3, "0")}`,
            revision: "rev_seed",
            status: "active",
            detail: "solved | 2 occ | 1 mates | 0 ddl",
            data: {
              tags: ["assembly"],
              parameterSet: {
                occurrenceCount: 2,
                mateCount: 1,
              },
            },
            assemblySummary: {
              status: "solved",
              occurrenceCount: 2,
              mateCount: 1,
              degreesOfFreedomEstimate: 0,
              warningCount: 0,
            },
          },
        ],
      },
      result: {
        commandId,
        status: "applied",
        message: "assemblage ajoute dans l apercu web",
      },
    };
  }

  if (commandId === "entity.create.robot_cell") {
    const index = currentSnapshot.entities.length + 1;
    return {
      snapshot: {
        ...appendFallbackActivity(
          currentSnapshot,
          "system",
          "entity.create.robot_cell",
          `ent_cell_${String(index).padStart(3, "0")}`,
        ),
        status: {
          ...currentSnapshot.status,
          entityCount: currentSnapshot.entities.length + 1,
        },
        entities: [
          ...currentSnapshot.entities,
          {
            id: `ent_cell_${String(index).padStart(3, "0")}`,
            entityType: "RobotCell",
            name: `RobotCell-${String(index).padStart(3, "0")}`,
            revision: "rev_seed",
            status: "active",
            detail: "3 pts | 4 sig | 3491 ms",
            data: {
              tags: ["robotics", "simulation", "mvp"],
              parameterSet: {
                tcpPayloadKg: 8,
                estimatedCycleTimeMs: 3491,
              },
            },
            robotCellSummary: {
              targetCount: 3,
              pathLengthMm: 896,
              maxSegmentMm: 470,
              estimatedCycleTimeMs: 3491,
              safetyZoneCount: 2,
              signalCount: 4,
              controllerTransitionCount: 3,
              warningCount: 0,
            },
          },
        ],
      },
      result: {
        commandId,
        status: "applied",
        message: "cellule robotique ajoutee dans l apercu web",
      },
    };
  }

  if (commandId === "simulation.run.start") {
    const robotCells = currentSnapshot.entities.filter(
      (entity) => entity.robotCellSummary,
    );
    const nextEntities = [...currentSnapshot.entities];
    if (robotCells.length === 0) {
      nextEntities.push({
        id: "ent_cell_001",
        entityType: "RobotCell",
        name: "RobotCell-001",
        revision: "rev_seed",
        status: "active",
        detail: "3 pts | 4 sig | 3491 ms",
        data: {
          tags: ["robotics", "simulation", "mvp"],
          parameterSet: {
            tcpPayloadKg: 8,
            estimatedCycleTimeMs: 3491,
          },
        },
        robotCellSummary: {
          targetCount: 3,
          pathLengthMm: 896,
          maxSegmentMm: 470,
          estimatedCycleTimeMs: 3491,
          safetyZoneCount: 2,
          signalCount: 4,
          controllerTransitionCount: 3,
          warningCount: 0,
        },
      });
    }

    const runIndex = nextEntities.length + 1;
    return {
      snapshot: {
        ...appendFallbackActivity(
          currentSnapshot,
          "system",
          "simulation.run.completed",
          `ent_run_${String(runIndex).padStart(3, "0")}`,
        ),
        status: {
          ...currentSnapshot.status,
          entityCount: nextEntities.length + 1,
        },
        entities: [
          ...nextEntities,
          {
            id: `ent_run_${String(runIndex).padStart(3, "0")}`,
            entityType: "SimulationRun",
            name: `SimulationRun-${String(runIndex).padStart(3, "0")}`,
            revision: "rev_seed",
            status: "active",
            detail: "completed | 3497 ms | 0 coll | 0 contact",
            data: {
              tags: ["simulation", "artifact", "mvp"],
              parameterSet: {
                seed: 308,
                stepCount: 12,
              },
            },
            simulationRunSummary: {
              status: "completed",
              collisionCount: 0,
              cycleTimeMs: 3497,
              maxTrackingErrorMm: 0.27,
              energyEstimateJ: 74.82,
              blockedSequenceDetected: false,
              contactCount: 0,
              signalSampleCount: 4,
              controllerStateSampleCount: 3,
              timelineSampleCount: 12,
            },
          },
        ],
      },
      result: {
        commandId,
        status: "applied",
        message: "run de simulation termine dans l apercu web",
      },
    };
  }

  if (commandId === "analyze.safety") {
    const nextEntities = [...currentSnapshot.entities];
    if (!nextEntities.some((entity) => entity.robotCellSummary)) {
      nextEntities.push({
        id: "ent_cell_001",
        entityType: "RobotCell",
        name: "RobotCell-001",
        revision: "rev_seed",
        status: "active",
        detail: "3 pts | 4 sig | 3491 ms",
        data: {
          tags: ["robotics", "simulation", "mvp"],
          parameterSet: {
            tcpPayloadKg: 8,
            estimatedCycleTimeMs: 3491,
          },
        },
        robotCellSummary: {
          targetCount: 3,
          pathLengthMm: 896,
          maxSegmentMm: 470,
          estimatedCycleTimeMs: 3491,
          safetyZoneCount: 2,
          signalCount: 4,
          controllerTransitionCount: 3,
          warningCount: 0,
        },
      });
    }

    const reportIndex = nextEntities.length + 1;
    return {
      snapshot: {
        ...appendFallbackActivity(
          currentSnapshot,
          "system",
          "analysis.safety.generated",
          `ent_safe_${String(reportIndex).padStart(3, "0")}`,
        ),
        status: {
          ...currentSnapshot.status,
          entityCount: nextEntities.length + 1,
        },
        entities: [
          ...nextEntities,
          {
            id: `ent_safe_${String(reportIndex).padStart(3, "0")}`,
            entityType: "SafetyReport",
            name: `SafetyReport-${String(reportIndex).padStart(3, "0")}`,
            revision: "rev_seed",
            status: "active",
            detail: "warning | 1 active | 0 block",
            data: {
              tags: ["safety", "analysis"],
              parameterSet: {
                attemptedAction: "robot.move",
              },
            },
            safetyReportSummary: {
              status: "warning",
              inhibited: false,
              activeZoneCount: 1,
              blockingInterlockCount: 0,
              advisoryZoneCount: 1,
            },
          },
        ],
      },
      result: {
        commandId,
        status: "applied",
        message: "rapport safety genere dans l apercu web",
      },
    };
  }

  return {
    snapshot: appendFallbackActivity(
      currentSnapshot,
      "system",
      "command.simulated",
      commandId,
    ),
    result: {
      commandId,
      status: "simulated",
      message: "commande simulee dans l apercu web",
    },
  };
}

async function regenerateLatestPart(payload, currentSnapshot) {
  const response = await invokeBackend("workspace_regenerate_latest_part", {
    payload,
  });
  if (response) {
    return response;
  }

  const latestPart = latestParametricPartFromSnapshot(currentSnapshot);
  if (!latestPart?.partGeometry) {
    return {
      snapshot: appendFallbackActivity(
        currentSnapshot,
        "system",
        "build.regenerate_part",
        null,
      ),
      result: {
        commandId: "build.regenerate_part",
        status: "notice",
        message: "aucune piece parametrique a regenerer dans l apercu web",
      },
    };
  }

  const areaMm2 = payload.widthMm * payload.heightMm;
  const volumeMm3 = areaMm2 * payload.depthMm;
  const estimatedMassGrams = volumeMm3 * 0.0027;
  const updatedEntity = {
    ...latestPart,
    detail: `${payload.widthMm.toFixed(1)} x ${payload.heightMm.toFixed(1)} x ${payload.depthMm.toFixed(1)} mm | ${estimatedMassGrams.toFixed(1)} g`,
    data: {
      ...(latestPart.data ?? {}),
      parameterSet: {
        widthMm: payload.widthMm,
        heightMm: payload.heightMm,
        depthMm: payload.depthMm,
      },
    },
    partGeometry: {
      ...latestPart.partGeometry,
      widthMm: payload.widthMm,
      heightMm: payload.heightMm,
      depthMm: payload.depthMm,
      perimeterMm: 2 * (payload.widthMm + payload.heightMm),
      areaMm2,
      volumeMm3,
      estimatedMassGrams,
    },
  };

  return {
    snapshot: {
      ...appendFallbackActivity(
        currentSnapshot,
        "system",
        "build.regenerate_part",
        latestPart.id,
      ),
      entities: currentSnapshot.entities.map((entity) =>
        entity.id === latestPart.id ? updatedEntity : entity,
      ),
    },
    result: {
      commandId: "build.regenerate_part",
      status: "applied",
      message: `piece regeneree dans l apercu web: ${updatedEntity.detail}`,
    },
  };
}

async function fetchAiRuntimeStatus(selectedProfile = null) {
  return (
    (await invokeBackend("ai_runtime_status", { selectedProfile })) ??
    FALLBACK_AI_STATUS
  );
}

function buildFallbackAiReferences(snapshot) {
  return [
    `project:${snapshot.details.projectId}`,
    ...snapshot.entities.slice(0, 3).map((entity) => `entity:${entity.id}`),
    ...snapshot.endpoints
      .slice(0, 2)
      .map((endpoint) => `endpoint:${endpoint.id}`),
    ...snapshot.streams.slice(0, 2).map((stream) => `stream:${stream.id}`),
    ...snapshot.plugins
      .slice(0, 1)
      .map((plugin) => `plugin:${plugin.pluginId}`),
    ...(snapshot.openSpecDocuments ?? [])
      .slice(0, 2)
      .map((document) => `openspec:${document.id}`),
  ].slice(0, 8);
}

function buildFallbackAiAnswer(locale, snapshot, message) {
  const summary = `${snapshot.status.projectName} | ${snapshot.status.entityCount} entites | ${snapshot.status.endpointCount} endpoints | ${snapshot.status.streamCount} flux | ${(snapshot.openSpecDocuments ?? []).length} docs OpenSpec`;

  if (locale === "en") {
    return `The local AI panel is running in web preview fallback mode. Current project: ${summary}. Your question was: "${message}". Start the Tauri shell with Ollama available on http://127.0.0.1:11434 to get a true local model-backed discussion.`;
  }

  if (locale === "es") {
    return `El panel de IA local esta en modo fallback de vista web. Proyecto actual: ${summary}. Tu pregunta fue: "${message}". Inicia el shell Tauri con Ollama disponible en http://127.0.0.1:11434 para obtener una conversacion local real con modelo.`;
  }

  return `Le panneau IA locale tourne en mode fallback d apercu web. Projet courant: ${summary}. Ta question etait: "${message}". Lance le shell Tauri avec Ollama disponible sur http://127.0.0.1:11434 pour obtenir une vraie discussion locale avec modele.`;
}

function buildFallbackStructuredExplain(snapshot, message) {
  const latestSimulationRun = latestSimulationRunFromSnapshot(snapshot);
  const latestSafetyReport = latestSafetyReportFromSnapshot(snapshot);
  const contextRefs = [
    { entityId: null, role: "source", path: "metadata.projectId" },
    ...(latestSimulationRun
      ? [
          {
            entityId: latestSimulationRun.id,
            role: "source",
            path: "simulationRunSummary.collisionCount",
          },
        ]
      : []),
    ...(latestSafetyReport
      ? [
          {
            entityId: latestSafetyReport.id,
            role: "source",
            path: "safetyReportSummary.blockingInterlockCount",
          },
        ]
      : []),
  ].slice(0, 4);

  return {
    summary: latestSimulationRun
      ? `Le dernier run ${latestSimulationRun.name} reste la source principale pour expliquer la demande.`
      : `Le projet ${snapshot.status.projectName} ne contient pas encore de run exploitable pour expliquer "${message}".`,
    contextRefs,
    confidence: latestSimulationRun || latestSafetyReport ? 0.76 : 0.52,
    riskLevel:
      latestSimulationRun?.simulationRunSummary?.collisionCount > 0 ||
      latestSafetyReport?.safetyReportSummary?.blockingInterlockCount > 0
        ? "high"
        : latestSimulationRun?.simulationRunSummary?.blockedSequenceDetected
          ? "medium"
          : "low",
    limitations:
      latestSimulationRun || latestSafetyReport
        ? [
            "Le fallback web n utilise pas le modele local, seulement le snapshot charge.",
          ]
        : [
            "Aucun artefact de simulation ou de safety n est encore disponible dans cet apercu.",
          ],
    proposedCommands: [],
    explanation: latestSimulationRun
      ? [
          `Le run retenu expose ${latestSimulationRun.simulationRunSummary?.collisionCount ?? 0} collision(s) et ${latestSimulationRun.simulationRunSummary?.timelineSampleCount ?? 0} echantillon(s) de timeline.`,
          `La demande etait: "${message}".`,
        ]
      : [`La demande etait: "${message}".`],
  };
}

async function sendAiChatMessage(
  message,
  locale,
  history,
  selectedModel,
  selectedProfile,
  snapshot,
) {
  const response = await invokeBackend("ai_chat_send_message", {
    message,
    locale,
    history,
    selectedModel,
    selectedProfile,
  });
  if (response) {
    return response;
  }

  return {
    answer: buildFallbackAiAnswer(locale, snapshot, message),
    runtime: FALLBACK_AI_STATUS,
    references: buildFallbackAiReferences(snapshot),
    structured: buildFallbackStructuredExplain(snapshot, message),
    suggestionId: null,
    warnings: [FALLBACK_AI_STATUS.warning],
    source: "web-preview",
  };
}

function setNestedDraftValue(target, path, value) {
  const segments = path.split(".").filter(Boolean);
  if (segments.length === 0) {
    return target;
  }

  let current = target;
  for (const segment of segments.slice(0, -1)) {
    if (
      typeof current[segment] !== "object" ||
      current[segment] === null ||
      Array.isArray(current[segment])
    ) {
      current[segment] = {};
    }
    current = current[segment];
  }
  current[segments[segments.length - 1]] = value;
  return target;
}

function valueAtPath(source, path) {
  return path
    .split(".")
    .filter(Boolean)
    .reduce(
      (current, segment) =>
        current && typeof current === "object" ? current[segment] : undefined,
      source,
    );
}

async function updateEntityProperties(payload, currentSnapshot) {
  const response = await invokeBackend("workspace_update_entity_properties", {
    payload,
  });
  if (response) {
    return response;
  }

  const index = currentSnapshot.entities.findIndex(
    (entity) => entity.id === payload.entityId,
  );
  if (index === -1) {
    return {
      snapshot: currentSnapshot,
      result: {
        commandId: "entity.properties.update",
        status: "rejected",
        message: "entite introuvable dans l apercu web",
      },
    };
  }

  const currentEntity = currentSnapshot.entities[index];
  let nextEntity = structuredClone(currentEntity);
  const changes = payload.changes ?? {};
  if (typeof changes.name === "string" && changes.name.trim().length > 0) {
    nextEntity.name = changes.name.trim();
  }
  if (!nextEntity.data || typeof nextEntity.data !== "object") {
    nextEntity.data = {};
  }
  if (Array.isArray(changes.tags)) {
    nextEntity.data.tags = changes.tags;
  }
  for (const [path, value] of Object.entries(changes)) {
    if (path === "name" || path === "tags") {
      continue;
    }
    setNestedDraftValue(nextEntity.data, path, value);
  }

  if (nextEntity.partGeometry) {
    const widthMm = Number(
      valueAtPath(nextEntity.data, "parameterSet.widthMm") ??
        nextEntity.partGeometry.widthMm,
    );
    const heightMm = Number(
      valueAtPath(nextEntity.data, "parameterSet.heightMm") ??
        nextEntity.partGeometry.heightMm,
    );
    const depthMm = Number(
      valueAtPath(nextEntity.data, "parameterSet.depthMm") ??
        nextEntity.partGeometry.depthMm,
    );
    const areaMm2 = widthMm * heightMm;
    const volumeMm3 = areaMm2 * depthMm;
    const estimatedMassGrams = volumeMm3 * 0.0027;
    nextEntity = {
      ...nextEntity,
      detail: `${widthMm.toFixed(1)} x ${heightMm.toFixed(1)} x ${depthMm.toFixed(1)} mm | ${estimatedMassGrams.toFixed(1)} g`,
      partGeometry: {
        ...nextEntity.partGeometry,
        widthMm,
        heightMm,
        depthMm,
        perimeterMm: 2 * (widthMm + heightMm),
        areaMm2,
        volumeMm3,
        estimatedMassGrams,
      },
    };
  }

  if (nextEntity.entityType === "Signal") {
    nextEntity = {
      ...nextEntity,
      detail: `${nextEntity.data.signalId ?? "signal"} | ${String(nextEntity.data.currentValue ?? "false")}`,
    };
  }

  return {
    snapshot: {
      ...appendFallbackActivity(
        currentSnapshot,
        "system",
        "entity.properties.updated",
        payload.entityId,
      ),
      entities: currentSnapshot.entities.map((entity, entityIndex) =>
        entityIndex === index ? nextEntity : entity,
      ),
    },
    result: {
      commandId: "entity.properties.update",
      status: "applied",
      message: `proprietes mises a jour pour ${payload.entityId}`,
    },
  };
}

async function applyAiSuggestion(suggestionId, currentSnapshot) {
  const response = await invokeBackend("workspace_apply_ai_suggestion", {
    suggestionId,
  });
  if (response) {
    return response;
  }

  return {
    snapshot: appendFallbackActivity(
      currentSnapshot,
      "system",
      "ai.suggestion.applied",
      suggestionId,
    ),
    result: {
      commandId: "ai.suggestion.apply",
      status: "notice",
      message: `suggestion ${suggestionId} marquee comme appliquee dans l apercu web`,
    },
  };
}

async function rejectAiSuggestion(suggestionId, currentSnapshot) {
  const response = await invokeBackend("workspace_reject_ai_suggestion", {
    suggestionId,
  });
  if (response) {
    return response;
  }

  return {
    snapshot: appendFallbackActivity(
      currentSnapshot,
      "system",
      "ai.suggestion.rejected",
      suggestionId,
    ),
    result: {
      commandId: "ai.suggestion.reject",
      status: "notice",
      message: `suggestion ${suggestionId} rejetee dans l apercu web`,
    },
  };
}

const defaultDesktopBackend = {
  fetchWorkspaceBootstrap,
  loadWorkspaceFixture,
  executeWorkspaceCommand,
  regenerateLatestPart,
  updateEntityProperties,
  applyAiSuggestion,
  rejectAiSuggestion,
  fetchAiRuntimeStatus,
  sendAiChatMessage,
};

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

function formatDecimal(locale, value, maximumFractionDigits = 1) {
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits,
  }).format(value);
}

function formatParametricPartSummary(locale, partGeometry) {
  return `${formatDecimal(locale, partGeometry.widthMm)} x ${formatDecimal(locale, partGeometry.heightMm)} x ${formatDecimal(locale, partGeometry.depthMm)} mm | ${formatDecimal(locale, partGeometry.estimatedMassGrams)} g`;
}

function latestParametricPartFromSnapshot(snapshot) {
  const parametricParts = snapshot.entities.filter(
    (entity) => entity.partGeometry,
  );
  return parametricParts[parametricParts.length - 1] ?? null;
}

function formatRobotCellSummary(locale, robotCellSummary) {
  return `${robotCellSummary.targetCount} pts | ${formatDecimal(locale, robotCellSummary.pathLengthMm, 0)} mm | ${robotCellSummary.estimatedCycleTimeMs} ms`;
}

function latestRobotCellFromSnapshot(snapshot) {
  const robotCells = snapshot.entities.filter(
    (entity) => entity.robotCellSummary,
  );
  return robotCells[robotCells.length - 1] ?? null;
}

function formatSimulationRunSummary(locale, simulationRunSummary) {
  return `${simulationRunSummary.status} | ${simulationRunSummary.cycleTimeMs} ms | ${simulationRunSummary.collisionCount} coll`;
}

function latestSimulationRunFromSnapshot(snapshot) {
  const simulationRuns = snapshot.entities.filter(
    (entity) => entity.simulationRunSummary,
  );
  return simulationRuns[simulationRuns.length - 1] ?? null;
}

function formatSafetyReportSummary(locale, safetyReportSummary) {
  return `${safetyReportSummary.status} | ${safetyReportSummary.activeZoneCount} active | ${safetyReportSummary.blockingInterlockCount} block`;
}

function latestSafetyReportFromSnapshot(snapshot) {
  const safetyReports = snapshot.entities.filter(
    (entity) => entity.safetyReportSummary,
  );
  return safetyReports[safetyReports.length - 1] ?? null;
}

function formatInspectorValue(value) {
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }

  if (typeof value === "number") {
    return String(value);
  }

  if (typeof value === "string") {
    return value;
  }

  if (value === null || value === undefined) {
    return "null";
  }

  return JSON.stringify(value, null, 2);
}

function buildInspectorDraft(entity) {
  return buildInspectorDraftFromSchema(buildEntityInspectorSchema(entity));
}

function timelineSeverityRank(kind) {
  if (kind === "collision") {
    return 0;
  }
  if (kind === "signal") {
    return 1;
  }
  if (kind === "controller") {
    return 2;
  }
  return 3;
}

function buildSimulationTimelineEvents(runEntity) {
  if (!runEntity?.data) {
    return [];
  }

  const timelineSamples = Array.isArray(runEntity.data.timelineSamples)
    ? runEntity.data.timelineSamples
    : [];
  const signalSamples = Array.isArray(runEntity.data.signalSamples)
    ? runEntity.data.signalSamples
    : [];
  const controllerStateSamples = Array.isArray(runEntity.data.controllerStateSamples)
    ? runEntity.data.controllerStateSamples
    : [];
  const contacts = Array.isArray(runEntity.data.contacts)
    ? runEntity.data.contacts
    : [];

  const events = [
    ...contacts.map((contact) => ({
      id: `collision-${contact.stepIndex}-${contact.pairId}`,
      kind: "collision",
      title: `${contact.leftEntityId} x ${contact.rightEntityId}`,
      summary: `${contact.overlapMm} mm | ${contact.severity}`,
      timestampMs: contact.timestampMs,
      stepIndex: contact.stepIndex,
      payload: contact,
    })),
    ...signalSamples
      .filter((sample) => sample.stepIndex > 0)
      .map((sample) => ({
        id: `signal-${sample.stepIndex}-${sample.signalId}-${sample.reason}`,
        kind: "signal",
        title: sample.signalId,
        summary: `${formatInspectorValue(sample.value)} | ${sample.reason}`,
        timestampMs: sample.timestampMs,
        stepIndex: sample.stepIndex,
        payload: sample,
      })),
    ...controllerStateSamples
      .filter((sample) => sample.stepIndex > 0)
      .map((sample) => ({
        id: `controller-${sample.stepIndex}-${sample.stateId}`,
        kind: "controller",
        title: sample.stateName,
        summary: `${sample.stateId} | ${sample.reason}`,
        timestampMs: sample.timestampMs,
        stepIndex: sample.stepIndex,
        payload: sample,
      })),
    ...timelineSamples.map((sample) => ({
      id: `timeline-${sample.stepIndex}`,
      kind: "timeline",
      title: `step ${sample.stepIndex}`,
      summary: `${sample.trackingErrorMm} mm | speed ${sample.speedScale}`,
      timestampMs: sample.timestampMs,
      stepIndex: sample.stepIndex,
      payload: sample,
    })),
  ];

  return events.sort((left, right) => {
    if (left.timestampMs !== right.timestampMs) {
      return left.timestampMs - right.timestampMs;
    }
    if (left.stepIndex !== right.stepIndex) {
      return left.stepIndex - right.stepIndex;
    }
    return timelineSeverityRank(left.kind) - timelineSeverityRank(right.kind);
  });
}

function criticalTimelineEvent(events) {
  return (
    events.find((event) => event.kind === "collision") ??
    events.find((event) => event.kind === "signal") ??
    events[0] ??
    null
  );
}

function summarizeAiProposedCommand(command) {
  if (!command) {
    return "";
  }

  const target = command.targetId ? ` | ${command.targetId}` : "";
  const payload =
    command.payload && Object.keys(command.payload).length > 0
      ? ` | ${JSON.stringify(command.payload)}`
      : "";
  return `${command.kind}${target}${payload}`;
}

function formatStructuredConfidence(value) {
  return `${Math.round(Number(value ?? 0) * 100)}%`;
}

function latestOpenSpecDocumentSummary(snapshot) {
  const documents = snapshot?.openSpecDocuments ?? [];
  return documents[documents.length - 1] ?? documents[0] ?? null;
}

function buildOpenSpecAutoPrompt(commandId, snapshot) {
  const openSpec = latestOpenSpecDocumentSummary(snapshot);
  const openSpecHint = openSpec
    ? ` en suivant le document OpenSpec "${openSpec.title}" (${openSpec.kind})`
    : " en suivant OpenSpec";

  if (commandId === "simulation.run.start") {
    const run = latestSimulationRunFromSnapshot(snapshot);
    if (!run?.simulationRunSummary) {
      return null;
    }

    if (run.simulationRunSummary.collisionCount > 0) {
      return `Mode explain${openSpecHint}: explique pourquoi ${run.name} detecte ${run.simulationRunSummary.collisionCount} collision(s). Cite la timeline, les contacts et les signaux du run courant.`;
    }

    if (run.simulationRunSummary.blockedSequenceDetected) {
      return `Mode explain${openSpecHint}: explique pourquoi ${run.name} reste bloque. Cite l etat controle, les signaux et les limites du run courant.`;
    }

    return `Mode summarize${openSpecHint}: resume ${run.name}, les signaux clefs du controle et le prochain jalon technique concret.`;
  }

  if (commandId === "analyze.safety") {
    const report = latestSafetyReportFromSnapshot(snapshot);
    if (!report?.safetyReportSummary) {
      return null;
    }

    if (report.safetyReportSummary.inhibited) {
      return `Mode explain${openSpecHint}: explique pourquoi ${report.name} bloque l action. Cite les zones actives et les interlocks.`;
    }

    return `Mode explain${openSpecHint}: explique pourquoi ${report.name} autorise encore l action sous surveillance. Cite les zones et les interlocks.`;
  }

  if (commandId === "entity.create.robot_cell") {
    const cell = latestRobotCellFromSnapshot(snapshot);
    if (!cell?.robotCellSummary) {
      return null;
    }

    return `Mode document${openSpecHint}: resume la nouvelle cellule ${cell.name}, ses signaux, son controle minimal et le prochain jalon OpenSpec a traiter.`;
  }

  if (commandId === "perception.run.start") {
    const run = [...(snapshot.entities ?? [])]
      .filter((entity) => entity.perceptionRunSummary)
      .at(-1);
    if (!run?.perceptionRunSummary) {
      return null;
    }

    return `Mode explain${openSpecHint}: explique ${run.name}, les ecarts observes, les obstacles inconnus et la prochaine action commissioning recommandee.`;
  }

  if (commandId === "commissioning.session.start") {
    const session = [...(snapshot.entities ?? [])]
      .filter((entity) => entity.commissioningSessionSummary)
      .at(-1);
    if (!session?.commissioningSessionSummary) {
      return null;
    }

    return `Mode summarize${openSpecHint}: resume ${session.name}, les captures terrain attachees, les ajustements ouverts et la prochaine verification as-built.`;
  }

  if (commandId === "commissioning.compare.as_built") {
    const comparison = [...(snapshot.entities ?? [])]
      .filter((entity) => entity.asBuiltComparisonSummary)
      .at(-1);
    if (!comparison?.asBuiltComparisonSummary) {
      return null;
    }

    return `Mode explain${openSpecHint}: explique les ecarts as-built de ${comparison.name}, les tolerances depassees et les corrections prioritaire a appliquer.`;
  }

  if (commandId === "optimization.run.start") {
    const study = [...(snapshot.entities ?? [])]
      .filter((entity) => entity.optimizationStudySummary)
      .at(-1);
    if (!study?.optimizationStudySummary) {
      return null;
    }

    return `Mode explain${openSpecHint}: explique le classement de ${study.name}, le meilleur candidat et les contraintes qui limitent l optimisation.`;
  }

  if (commandId === "integration.replay.degraded") {
    return `Mode explain${openSpecHint}: analyse le replay degrade de connectivite, les effets de jitter/pertes et le risque sur la cellule ou la telemetrie.`;
  }

  if (commandId === "help.openspec") {
    return `Mode summarize: a partir des documents OpenSpec visibles, quel est le prochain jalon technique concret et pourquoi ?`;
  }

  return null;
}

function parsePositiveDimension(value) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }

  return parsed;
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
    return `${translate(locale, "ui.ai.runtime_ready", "Pret")} | ${runtime.activeProfile ?? "balanced"} | ${runtime.activeModel ?? translate(locale, "ui.ai.no_model", "aucun modele")}`;
  }

  return translate(locale, "ui.ai.runtime_fallback", "Fallback local");
}

function assistantBadge(locale, runtime) {
  if (runtime.available) {
    return `${translate(locale, "ui.ai.badge", "IA locale")} | ${runtime.activeProfile ?? "balanced"} | ${runtime.activeModel ?? translate(locale, "ui.ai.no_model", "aucun modele")}`;
  }

  return `${translate(locale, "ui.ai.badge", "IA locale")} | ${translate(locale, "ui.ai.runtime_fallback", "Fallback local")}`;
}

export default function App({ backend = defaultDesktopBackend }) {
  const [locale, setLocale] = useState(defaultLocale);
  const [projectSnapshot, setProjectSnapshot] = useState(FALLBACK_SNAPSHOT);
  const [fixtureProjects, setFixtureProjects] = useState(FALLBACK_FIXTURES);
  const [selectedFixtureId, setSelectedFixtureId] = useState(
    FALLBACK_STATUS.fixtureId,
  );
  const [selectedViewportSceneId, setSelectedViewportSceneId] = useState(
    defaultAerospaceSceneId,
  );
  const [activeMenuId, setActiveMenuId] = useState("file");
  const [fixtureLoading, setFixtureLoading] = useState(false);
  const [executingCommandId, setExecutingCommandId] = useState(null);
  const [commandResult, setCommandResult] = useState(null);
  const [commandToast, setCommandToast] = useState(null);
  const [aiRuntime, setAiRuntime] = useState(FALLBACK_AI_STATUS);
  const [aiMessages, setAiMessages] = useState([]);
  const [aiDraft, setAiDraft] = useState("");
  const [aiBusy, setAiBusy] = useState(false);
  const [autoPromptEnabled, setAutoPromptEnabled] = useState(true);
  const [autoPromptQueue, setAutoPromptQueue] = useState([]);
  const [selectedAiModel, setSelectedAiModel] = useState("");
  const [selectedAiProfile, setSelectedAiProfile] = useState("balanced");
  const [panelState, setPanelState] = useState(defaultWorkspacePanels);
  const [dockWidths, setDockWidths] = useState(defaultWorkspaceDockWidths);
  const [dragSide, setDragSide] = useState(null);
  const [selectedEntityId, setSelectedEntityId] = useState(null);
  const [selectedSimulationRunId, setSelectedSimulationRunId] = useState(null);
  const [selectedTimelineEventId, setSelectedTimelineEventId] = useState(null);
  const [inspectorDraft, setInspectorDraft] = useState(
    buildInspectorDraft(null),
  );
  const [inspectorBusy, setInspectorBusy] = useState(false);
  const [inspectorError, setInspectorError] = useState("");
  const [partEditor, setPartEditor] = useState({
    widthMm: "",
    heightMm: "",
    depthMm: "",
  });
  const aiInputRef = useRef(null);
  const handleCommandExecuteRef = useRef(null);
  const workspaceRef = useRef(null);
  const dragStateRef = useRef(null);

  const menus = localizeMenuModel(locale);
  const menu = menus.find((entry) => entry.id === activeMenuId) ?? menus[0];
  const currentStatus = projectSnapshot.status;
  const openSpecDocuments = projectSnapshot.openSpecDocuments ?? [];
  const parametricParts = projectSnapshot.entities.filter(
    (entity) => entity.partGeometry,
  );
  const latestParametricPart =
    latestParametricPartFromSnapshot(projectSnapshot);
  const robotCells = projectSnapshot.entities.filter(
    (entity) => entity.robotCellSummary,
  );
  const latestRobotCell = latestRobotCellFromSnapshot(projectSnapshot);
  const simulationRuns = projectSnapshot.entities.filter(
    (entity) => entity.simulationRunSummary,
  );
  const latestSimulationRun = latestSimulationRunFromSnapshot(projectSnapshot);
  const safetyReports = projectSnapshot.entities.filter(
    (entity) => entity.safetyReportSummary,
  );
  const latestSafetyReport = latestSafetyReportFromSnapshot(projectSnapshot);
  const selectedSimulationRun =
    simulationRuns.find((entity) => entity.id === selectedSimulationRunId) ??
    latestSimulationRun ??
    null;
  const simulationTimelineEvents = buildSimulationTimelineEvents(
    selectedSimulationRun,
  );
  const focusedTimelineEvent =
    simulationTimelineEvents.find(
      (event) => event.id === selectedTimelineEventId,
    ) ??
    criticalTimelineEvent(simulationTimelineEvents);
  const selectedEntity =
    projectSnapshot.entities.find((entity) => entity.id === selectedEntityId) ??
    projectSnapshot.entities[projectSnapshot.entities.length - 1] ??
    null;
  const inspectorSchema = buildEntityInspectorSchema(selectedEntity);
  const fixtureOptions =
    selectedFixtureId &&
    !fixtureProjects.some((fixture) => fixture.id === selectedFixtureId)
      ? [
          { id: selectedFixtureId, projectName: currentStatus.projectName },
          ...fixtureProjects,
        ]
      : fixtureProjects;
  const t = (key, fallback = key) => translate(locale, key, fallback);
  const shortcutLabelForCommand = (commandId) =>
    formatShortcutLabel(
      findMenuEntryByCommand(commandId, menus)?.item?.shortcut,
    );
  const commandButtonTitle = (commandId, label) => {
    const shortcutLabel = shortcutLabelForCommand(commandId);
    return shortcutLabel ? `${label} (${shortcutLabel})` : label;
  };
  const { leftExpanded, rightExpanded } = getWorkspaceColumnState(panelState);
  const gemma3Models = getGemma3Models(aiRuntime);
  const availableAiProfiles = getAvailableAiProfiles(aiRuntime);
  const workspaceStyle = {
    "--workspace-left-column": `${getVisibleSidebarWidth(dockWidths.left, leftExpanded)}px`,
    "--workspace-right-column": `${getVisibleSidebarWidth(dockWidths.right, rightExpanded)}px`,
    "--workspace-left-resizer": leftExpanded
      ? `${WORKSPACE_RESIZER_WIDTH}px`
      : "0px",
    "--workspace-right-resizer": rightExpanded
      ? `${WORKSPACE_RESIZER_WIDTH}px`
      : "0px",
  };
  const starterPrompts = [
    t("ui.ai.prompt.summary", "Resume le projet courant"),
    t(
      "ui.ai.prompt.integration",
      "Quels endpoints et flux sont relies a ce projet ?",
    ),
    t(
      "ui.ai.prompt.next_step",
      "Quel est le prochain jalon technique concret ?",
    ),
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
      startWidths: dockWidths,
    };
    setDragSide(side);
  }

  function resetDockWidth(side) {
    setDockWidths((previous) => ({
      ...previous,
      [side]: defaultWorkspaceDockWidths[side],
    }));
  }

  function handlePartEditorChange(field, value) {
    setPartEditor((previous) => ({
      ...previous,
      [field]: value,
    }));
  }

  function handleEntitySelect(entityId) {
    setSelectedEntityId(entityId);
    setInspectorError("");
  }

  function handleInspectorChange(field, value) {
    setInspectorDraft((previous) => ({
      ...previous,
      [field]: value,
    }));
  }

  function setAiSuggestionStatus(suggestionId, nextStatus) {
    if (!suggestionId) {
      return;
    }

    setAiMessages((previous) =>
      previous.map((entry) =>
        entry.suggestionId === suggestionId
          ? { ...entry, suggestionStatus: nextStatus }
          : entry,
      ),
    );
  }

  function focusTimelineEvent(runId, eventId) {
    if (runId) {
      setSelectedSimulationRunId(runId);
    }
    if (eventId) {
      setSelectedTimelineEventId(eventId);
    }
    setPanelState((previous) => ({
      ...previous,
      simulationTimeline: true,
    }));
    setActiveMenuId("simulation");
  }

  function enqueueOpenSpecAutoPrompt(commandId, snapshot) {
    if (!autoPromptEnabled) {
      return;
    }

    const prompt = buildOpenSpecAutoPrompt(commandId, snapshot);
    if (!prompt) {
      return;
    }

    setAutoPromptQueue((previous) => [
      ...previous,
      {
        id: `${commandId}-${snapshot.status.fixtureId}-${previous.length + 1}`,
        commandId,
        message: prompt,
        snapshot,
      },
    ]);
  }

  async function handleInspectorSubmit() {
    if (!selectedEntity) {
      return;
    }

    const changes = {};
    for (const field of flattenEditableInspectorFields(inspectorSchema)) {
      const effectiveField =
        selectedEntity.entityType === "Signal" && field.path === "currentValue"
          ? {
              ...field,
              fieldType:
                inspectorDraft.kind === "text"
                  ? "string"
                  : inspectorDraft.kind === "scalar"
                    ? "number"
                    : "boolean",
            }
          : field;
      const result = coerceInspectorDraftValue(
        inspectorDraft[field.path],
        effectiveField,
      );
      if (!result.ok) {
        setInspectorError(
          t(
            "ui.property.invalid_parameter",
            `Le parametre ${field.label} contient une valeur invalide.`,
          ),
        );
        if (result.error) {
          setInspectorError(result.error);
        }
        return;
      }
      changes[field.path] = result.value;
    }

    setInspectorBusy(true);
    setInspectorError("");
    try {
      const response = await backend.updateEntityProperties(
        {
          entityId: selectedEntity.id,
          changes,
        },
        projectSnapshot,
      );
      setProjectSnapshot(response.snapshot);
      applyCommandFeedback(response.result);
      setSelectedEntityId(selectedEntity.id);
    } catch {
      setInspectorError(
        t(
          "ui.property.update_failed",
          "La mise a jour des proprietes a echoue sans crasher l interface.",
        ),
      );
    } finally {
      setInspectorBusy(false);
    }
  }

  async function handleAiSuggestionApply(suggestionId) {
    if (!suggestionId || aiBusy) {
      return;
    }

    setAiBusy(true);
    try {
      const response = await backend.applyAiSuggestion(
        suggestionId,
        projectSnapshot,
      );
      setProjectSnapshot(response.snapshot);
      applyCommandFeedback(response.result);
      setAiSuggestionStatus(suggestionId, "applied");
    } finally {
      setAiBusy(false);
    }
  }

  async function handleAiSuggestionReject(suggestionId) {
    if (!suggestionId || aiBusy) {
      return;
    }

    setAiBusy(true);
    try {
      const response = await backend.rejectAiSuggestion(
        suggestionId,
        projectSnapshot,
      );
      setProjectSnapshot(response.snapshot);
      applyCommandFeedback(response.result);
      setAiSuggestionStatus(suggestionId, "rejected");
    } finally {
      setAiBusy(false);
    }
  }

  function handleTimelineStep() {
    if (simulationTimelineEvents.length === 0) {
      applyCommandFeedback({
        commandId: "simulation.timeline.step",
        status: "notice",
        message: t(
          "ui.timeline.no_events",
          "Aucun evenement de timeline disponible pour avancer d un pas.",
        ),
      });
      return;
    }

    const currentIndex = simulationTimelineEvents.findIndex(
      (event) => event.id === focusedTimelineEvent?.id,
    );
    const nextEvent =
      simulationTimelineEvents[
        currentIndex >= 0
          ? (currentIndex + 1) % simulationTimelineEvents.length
          : 0
      ];
    focusTimelineEvent(selectedSimulationRun?.id, nextEvent.id);
    applyCommandFeedback({
      commandId: "simulation.timeline.step",
      status: "applied",
      message: `${nextEvent.kind} | step ${nextEvent.stepIndex} | t=${nextEvent.timestampMs} ms`,
    });
  }

  function applyCommandFeedback(result) {
    setCommandResult(result);
    setCommandToast({
      ...result,
      toastId: Date.now(),
    });
  }

  function focusAiInput() {
    showAiAssistant("ai.focus_input", true);
  }

  function showAiAssistant(commandId = "ai.show_panel", focusInput = false) {
    setActiveMenuId("ai");
    setPanelState((previous) => ({
      ...previous,
      aiAssistant: true,
    }));
    if (focusInput) {
      window.setTimeout(() => aiInputRef.current?.focus(), 0);
    }
    applyCommandFeedback({
      commandId,
      status: "layout",
      message: focusInput
        ? t("ui.ai.focused", "Panneau Assistant IA local actif.")
        : t("ui.panel.expanded_status", "Panneau rouvert."),
    });
  }

  async function handleLatestPartRegenerate() {
    if (!latestParametricPart?.partGeometry) {
      applyCommandFeedback({
        commandId: "build.regenerate_part",
        status: "notice",
        message: t(
          "ui.command.regenerate_part_unavailable",
          "Aucune piece parametrique disponible pour la regeneration.",
        ),
      });
      return;
    }

    const widthMm = parsePositiveDimension(partEditor.widthMm);
    const heightMm = parsePositiveDimension(partEditor.heightMm);
    const depthMm = parsePositiveDimension(partEditor.depthMm);
    if (!widthMm || !heightMm || !depthMm) {
      applyCommandFeedback({
        commandId: "build.regenerate_part",
        status: "rejected",
        message: t(
          "ui.command.invalid_part_dimensions",
          "Les dimensions de piece doivent etre strictement positives.",
        ),
      });
      return;
    }

    setExecutingCommandId("build.regenerate_part");
    try {
      const response = await backend.regenerateLatestPart(
        {
          widthMm,
          heightMm,
          depthMm,
        },
        projectSnapshot,
      );
      setProjectSnapshot(response.snapshot);
      applyCommandFeedback(response.result);
      enqueueOpenSpecAutoPrompt("build.regenerate_part", response.snapshot);
    } finally {
      setExecutingCommandId(null);
    }
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
          rightExpanded,
        }),
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
    setSelectedAiModel((previous) => {
      if (previous && gemma3Models.includes(previous)) {
        return previous;
      }

      return defaultGemma3Model(aiRuntime);
    });
  }, [aiRuntime.activeModel, gemma3Models.join("|")]);

  useEffect(() => {
    setSelectedAiProfile((previous) => {
      if (previous && availableAiProfiles.includes(previous)) {
        return previous;
      }

      return aiRuntime.activeProfile ?? availableAiProfiles[0] ?? "balanced";
    });
  }, [aiRuntime.activeProfile, availableAiProfiles.join("|")]);

  useEffect(() => {
    const entityIds = projectSnapshot.entities.map((entity) => entity.id);
    if (entityIds.includes(selectedEntityId)) {
      return;
    }

    setSelectedEntityId(entityIds[entityIds.length - 1] ?? null);
  }, [
    projectSnapshot.entities.map((entity) => entity.id).join("|"),
    selectedEntityId,
  ]);

  useEffect(() => {
    const runIds = simulationRuns.map((entity) => entity.id);
    if (runIds.includes(selectedSimulationRunId)) {
      return;
    }

    setSelectedSimulationRunId(runIds[runIds.length - 1] ?? null);
  }, [
    simulationRuns.map((entity) => entity.id).join("|"),
    selectedSimulationRunId,
  ]);

  useEffect(() => {
    if (simulationTimelineEvents.length === 0) {
      setSelectedTimelineEventId(null);
      return;
    }

    const stillExists = simulationTimelineEvents.some(
      (event) => event.id === selectedTimelineEventId,
    );
    if (stillExists) {
      return;
    }

    setSelectedTimelineEventId(
      criticalTimelineEvent(simulationTimelineEvents)?.id ?? null,
    );
  }, [
    selectedSimulationRun?.id,
    selectedTimelineEventId,
    simulationTimelineEvents.map((event) => event.id).join("|"),
  ]);

  useEffect(() => {
    setInspectorDraft(buildInspectorDraft(selectedEntity));
    setInspectorError("");
  }, [selectedEntity?.id, selectedEntity?.revision]);

  useEffect(() => {
    if (!latestParametricPart?.partGeometry) {
      setPartEditor({
        widthMm: "",
        heightMm: "",
        depthMm: "",
      });
      return;
    }

    setPartEditor({
      widthMm: String(latestParametricPart.partGeometry.widthMm),
      heightMm: String(latestParametricPart.partGeometry.heightMm),
      depthMm: String(latestParametricPart.partGeometry.depthMm),
    });
  }, [
    latestParametricPart?.id,
    latestParametricPart?.partGeometry?.widthMm,
    latestParametricPart?.partGeometry?.heightMm,
    latestParametricPart?.partGeometry?.depthMm,
  ]);

  useEffect(() => {
    let mounted = true;

    async function bootstrapWorkspace() {
      const [bootstrap, runtime] = await Promise.all([
        backend.fetchWorkspaceBootstrap(),
        backend.fetchAiRuntimeStatus(selectedAiProfile),
      ]);
      if (!mounted) {
        return;
      }

      setFixtureProjects(
        bootstrap.fixtures.length > 0 ? bootstrap.fixtures : FALLBACK_FIXTURES,
      );
      setProjectSnapshot(bootstrap.snapshot);
      setSelectedFixtureId(
        bootstrap.snapshot.status.fixtureId ?? FALLBACK_STATUS.fixtureId,
      );
      setAiRuntime(runtime);
    }

    bootstrapWorkspace();

    return () => {
      mounted = false;
    };
  }, []);

  useEffect(() => {
    let mounted = true;

    async function refreshRuntimeProfile() {
      const runtime = await backend.fetchAiRuntimeStatus(selectedAiProfile);
      if (!mounted) {
        return;
      }

      setAiRuntime(runtime);
    }

    refreshRuntimeProfile();

    return () => {
      mounted = false;
    };
  }, [backend, selectedAiProfile]);

  useEffect(() => {
    if (!autoPromptEnabled || aiBusy || autoPromptQueue.length === 0) {
      return;
    }

    const [nextPrompt, ...remainingPrompts] = autoPromptQueue;
    setAutoPromptQueue(remainingPrompts);
    submitAiMessage(nextPrompt.message, {
      snapshotOverride: nextPrompt.snapshot,
      source: "auto-openspec",
    });
  }, [
    autoPromptEnabled,
    aiBusy,
    autoPromptQueue,
    locale,
    projectSnapshot,
    selectedAiModel,
    selectedAiProfile,
  ]);

  useEffect(() => {
    function handleWindowKeydown(event) {
      if (!shouldHandleShortcutEvent(event)) {
        return;
      }

      const shortcutMatch = findMenuCommandByShortcut(
        visualStudioInspiredMenus,
        event,
      );
      if (!shortcutMatch) {
        return;
      }

      event.preventDefault();
      setActiveMenuId(shortcutMatch.menuId);
      void handleCommandExecuteRef.current?.(shortcutMatch.commandId);
    }

    window.addEventListener("keydown", handleWindowKeydown);
    return () => {
      window.removeEventListener("keydown", handleWindowKeydown);
    };
  }, []);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.__TAURI_INTERNALS__ !== "object") {
      return undefined;
    }

    let disposed = false;
    let unlisten;

    async function registerNativeMenuListener() {
      try {
        const tauriEvent = await import("@tauri-apps/api/event");
        if (disposed) {
          return;
        }

        unlisten = await tauriEvent.listen(
          NATIVE_MENU_EVENT_NAME,
          async (event) => {
            const commandId = event.payload?.commandId;
            if (typeof commandId !== "string" || commandId.length === 0) {
              return;
            }

            const entry = findMenuEntryByCommand(
              commandId,
              visualStudioInspiredMenus,
            );
            if (entry) {
              setActiveMenuId(entry.menuId);
            }

            await handleCommandExecuteRef.current?.(commandId);
          },
        );
      } catch {
        unlisten = null;
      }
    }

    registerNativeMenuListener();

    return () => {
      disposed = true;
      if (typeof unlisten === "function") {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    if (!commandToast) {
      return undefined;
    }

    const timeoutId = window.setTimeout(() => {
      setCommandToast((previous) =>
        previous?.toastId === commandToast.toastId ? null : previous,
      );
    }, COMMAND_TOAST_TIMEOUT_MS);

    return () => {
      window.clearTimeout(timeoutId);
    };
  }, [commandToast]);

  async function loadFixtureById(nextFixtureId) {
    if (!nextFixtureId) {
      return null;
    }

    setSelectedFixtureId(nextFixtureId);
    setFixtureLoading(true);

    try {
      const [snapshot, runtime] = await Promise.all([
        backend.loadWorkspaceFixture(nextFixtureId),
        backend.fetchAiRuntimeStatus(selectedAiProfile),
      ]);
      setProjectSnapshot(snapshot);
      setCommandResult(null);
      setCommandToast(null);
      setAiRuntime(runtime);
      setAiMessages([]);
      setAiDraft("");
      setAutoPromptQueue([]);
      return snapshot;
    } finally {
      setFixtureLoading(false);
    }
  }

  async function handleFixtureChange(event) {
    await loadFixtureById(event.target.value);
  }

  async function handleCommandExecute(commandId) {
    if (commandId === "ai.focus_input") {
      focusAiInput();
      return;
    }

    if (commandId === "ai.show_panel") {
      showAiAssistant(commandId);
      return;
    }

    if (commandId === "simulation.timeline.step") {
      handleTimelineStep();
      return;
    }

    const panelId = panelIdFromCommand(commandId);
    if (panelId) {
      const willBeVisible = !panelState[panelId];
      setPanelState((previous) => toggleWorkspacePanel(previous, panelId));
      applyCommandFeedback({
        commandId,
        status: "layout",
        message: panelState[panelId]
          ? t("ui.panel.collapsed_status", "Panneau replie.")
          : t("ui.panel.expanded_status", "Panneau rouvert."),
      });

      if (panelId === "aiAssistant" && willBeVisible) {
        window.setTimeout(() => aiInputRef.current?.focus(), 0);
      }
      return;
    }

    if (commandId === "project.open") {
      const snapshot = await loadFixtureById(selectedFixtureId);
      if (snapshot) {
        applyCommandFeedback({
          commandId,
          status: "applied",
          message: t(
            "ui.command.project_opened",
            "Projet charge depuis la fixture selectionnee.",
          ),
        });
      }
      return;
    }

    if (commandId === "project.open_recent") {
      const nextFixtureId = getNextFixtureId(
        fixtureProjects,
        selectedFixtureId,
      );
      const snapshot = await loadFixtureById(nextFixtureId);
      if (snapshot) {
        applyCommandFeedback({
          commandId,
          status: "applied",
          message: t(
            "ui.command.recent_opened",
            "Fixture suivante ouverte depuis la liste recente.",
          ),
        });
      }
      return;
    }

    if (
      ["project.properties", "app.settings", "app.options"].includes(commandId)
    ) {
      setPanelState((previous) => ({
        ...previous,
        properties: true,
      }));
      setActiveMenuId("insert");
      applyCommandFeedback({
        commandId,
        status: "layout",
        message: t(
          "ui.command.properties_opened",
          "Panneau Proprietes ouvert.",
        ),
      });
      return;
    }

    if (commandId === "app.exit") {
      const closed = await requestWindowClose();
      applyCommandFeedback({
        commandId,
        status: closed ? "applied" : "notice",
        message: closed
          ? t(
              "ui.command.exit_requested",
              "Fermeture de l application demandee.",
            )
          : t(
              "ui.command.exit_unavailable",
              "Fermeture indisponible dans cet apercu. Lance le shell Tauri pour fermer la fenetre.",
            ),
      });
      return;
    }

    if (commandId === "build.regenerate_part") {
      await handleLatestPartRegenerate();
      return;
    }

    setExecutingCommandId(commandId);

    try {
      const response = await backend.executeWorkspaceCommand(
        commandId,
        projectSnapshot,
      );
      setProjectSnapshot(response.snapshot);
      setSelectedFixtureId(response.snapshot.status.fixtureId);
      applyCommandFeedback(response.result);
      enqueueOpenSpecAutoPrompt(commandId, response.snapshot);

      if (commandId === "project.create") {
        setAiMessages([]);
        setAiDraft("");
        setAutoPromptQueue([]);
      }
    } finally {
      setExecutingCommandId(null);
    }
  }

  handleCommandExecuteRef.current = handleCommandExecute;

  async function submitAiMessage(message, options = {}) {
    const trimmedMessage = message.trim();
    if (!trimmedMessage || aiBusy) {
      return;
    }

    const activeSnapshot = options.snapshotOverride ?? projectSnapshot;
    const history = aiMessages.map((entry) => ({
      role: entry.role,
      content: entry.content,
    }));
    const userEntry = {
      role: "user",
      content: trimmedMessage,
      references: [],
      structured: null,
      suggestionId: null,
      suggestionStatus: null,
      warnings: [],
      source: options.source ?? "user",
    };

    setAiMessages((previous) => [...previous, userEntry]);
    setAiDraft("");
    setAiBusy(true);

    try {
      const response = await backend.sendAiChatMessage(
        trimmedMessage,
        locale,
        history,
        selectedAiModel || null,
        selectedAiProfile || null,
        activeSnapshot,
      );
      setAiRuntime(response.runtime);
      setAiMessages((previous) => [
        ...previous,
        {
          role: "assistant",
          content: response.answer,
          references: response.references ?? [],
          structured: response.structured ?? null,
          suggestionId: response.suggestionId ?? null,
          suggestionStatus:
            response.suggestionId &&
            (response.structured?.proposedCommands?.length ?? 0) > 0
              ? "pending"
              : null,
          warnings: response.warnings ?? [],
          source: response.source,
        },
      ]);
    } catch {
      setAiMessages((previous) => [
        ...previous,
        {
          role: "assistant",
          content: t(
            "ui.ai.error",
            "Le runtime IA local a renvoye une erreur.",
          ),
          references: [],
          structured: null,
          suggestionId: null,
          suggestionStatus: null,
          warnings: [],
          source: "error",
        },
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

  function renderInspectorField(field, depth = 0) {
    if (!field) {
      return null;
    }

    if (field.children?.length > 0) {
      return (
        <div
          key={field.path}
          className="property-field-group"
          data-entity-field-group={field.path}
          style={{ "--property-depth": depth }}
        >
          <div className="subsection-label">{field.label}</div>
          <div className="property-field-children">
            {field.children.map((child) => renderInspectorField(child, depth + 1))}
          </div>
        </div>
      );
    }

    const fieldLabel =
      field.path === "name"
        ? t("ui.property.entity_name", "Nom")
        : field.path === "tags"
          ? t("ui.property.entity_tags", "Tags")
          : field.label;
    const effectiveFieldType =
      selectedEntity?.entityType === "Signal" && field.path === "currentValue"
        ? inspectorDraft.kind === "text"
          ? "string"
          : inspectorDraft.kind === "scalar"
            ? "number"
            : "boolean"
        : field.fieldType;
    const inputId = `${selectedEntity?.id ?? "entity"}-${field.path}`;
    const draftValue =
      inspectorDraft[field.path] ??
      (field.fieldType === "boolean" ? false : formatInspectorValue(field.value));

    let control = (
      <div className="muted" data-entity-field-readonly={field.path}>
        {formatInspectorValue(field.value)}
      </div>
    );

    if (field.editable) {
      if (effectiveFieldType === "boolean") {
        control = (
          <input
            id={inputId}
            type="checkbox"
            aria-label={fieldLabel}
            data-entity-field-input={field.path}
            data-entity-name-input={field.path === "name" ? selectedEntity?.id : undefined}
            data-entity-tags-input={field.path === "tags" ? selectedEntity?.id : undefined}
            checked={Boolean(draftValue)}
            onChange={(event) =>
              handleInspectorChange(field.path, event.target.checked)
            }
          />
        );
      } else if (effectiveFieldType === "enum") {
        control = (
          <select
            id={inputId}
            className="shell-select"
            aria-label={fieldLabel}
            data-entity-field-input={field.path}
            value={draftValue ?? ""}
            onChange={(event) =>
              handleInspectorChange(field.path, event.target.value)
            }
          >
            {field.options.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        );
      } else if (effectiveFieldType === "list") {
        control = (
          <textarea
            id={inputId}
            className="assistant-input property-array-input"
            aria-label={fieldLabel}
            data-entity-field-input={field.path}
            rows={4}
            value={draftValue ?? "[]"}
            onChange={(event) =>
              handleInspectorChange(field.path, event.target.value)
            }
          />
        );
      } else {
        control = (
          <input
            id={inputId}
            className="shell-select shell-input"
            type={effectiveFieldType === "number" ? "number" : "text"}
            min={field.minimum ?? undefined}
            step={effectiveFieldType === "number" ? "any" : undefined}
            aria-label={fieldLabel}
            data-entity-field-input={field.path}
            data-entity-name-input={field.path === "name" ? selectedEntity?.id : undefined}
            data-entity-tags-input={field.path === "tags" ? selectedEntity?.id : undefined}
            value={draftValue ?? ""}
            onChange={(event) =>
              handleInspectorChange(field.path, event.target.value)
            }
          />
        );
      }
    }

    return (
      <label
        key={field.path}
        className="control-group property-control property-field-row"
        data-entity-field-path={field.path}
        style={{ "--property-depth": depth }}
      >
        <span>
          {fieldLabel}
          {field.required ? " *" : ""}
        </span>
        {control}
        {field.editable && field.minimum !== null ? (
          <div className="muted">{`min ${field.minimum}`}</div>
        ) : null}
      </label>
    );
  }

  return (
    <div className="shell">
      <header className="shell-header">
        <div className="brand-block">
          <div className="brand-mark">FA</div>
          <div>
            <div className="brand-title">FutureAero</div>
            <div className="brand-subtitle">
              {t("ui.brand.subtitle", "Desktop shell")}
            </div>
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
                disabled={fixtureOptions.length === 0 || fixtureLoading}
              >
                {fixtureOptions.length > 0 ? (
                  fixtureOptions.map((fixture) => (
                    <option key={fixture.id} value={fixture.id}>
                      {fixture.projectName}
                    </option>
                  ))
                ) : (
                  <option value="">
                    {t("ui.fixture.empty", "Aucune fixture")}
                  </option>
                )}
              </select>
            </label>
          </div>

          <div className="status-pills">
            <span className="status-pill">
              {runtimeLabel(locale, currentStatus.runtime)}
            </span>
            <span className="status-pill">{currentStatus.projectName}</span>
            <span className="status-pill">
              {assistantBadge(locale, aiRuntime)}
            </span>
            <span className="status-pill">
              {fixtureLoading
                ? t("ui.fixture.loading", "Chargement...")
                : fixtureLabel(fixtureOptions, selectedFixtureId)}
            </span>
          </div>
        </div>
      </header>

      <MenuBar
        menus={menus}
        activeMenuId={activeMenuId}
        onSelect={setActiveMenuId}
      />

      <div className="context-bar">
        <div className="context-title">{menu.label}</div>
        <div className="context-meta">
          {menu.items.filter((item) => item.type !== "separator").length}{" "}
          commandes
        </div>
      </div>

      {commandToast ? (
        <div
          className="command-toast"
          data-command-toast={commandToast.commandId}
          data-command-toast-status={commandToast.status}
        >
          <strong>{commandToast.commandId}</strong>
          <span className="command-id">{commandToast.status}</span>
          <span>{commandToast.message}</span>
        </div>
      ) : null}

      <main className="workspace" style={workspaceStyle} ref={workspaceRef}>
        <aside
          className={
            leftExpanded
              ? "workspace-left"
              : "workspace-left workspace-column-collapsed"
          }
        >
          <Panel
            panelId="projectExplorer"
            title={t("ui.panel.project_explorer", "Explorateur de projet")}
            accent={`${currentStatus.entityCount} ${t("ui.workspace.entities", "entites")}`}
            collapsed={!panelState.projectExplorer}
            onToggle={() => togglePanel("projectExplorer")}
            toggleLabel={panelToggleLabel("projectExplorer")}
          >
            <ul className="tree-list">
              <li className="tree-root">{currentStatus.projectName}</li>

              <li className="tree-section">
                <div className="tree-section-title">
                  {t("ui.workspace.entities_section", "Entites")}
                </div>
                <ul className="tree-sublist">
                  {projectSnapshot.entities.length > 0 ? (
                    projectSnapshot.entities.map((entity) => (
                      <li key={entity.id} className="tree-row">
                        <button
                          className="tree-row-button"
                          type="button"
                          data-entity-select={entity.id}
                          data-entity-selected={
                            selectedEntity?.id === entity.id ? "true" : "false"
                          }
                          onClick={() => handleEntitySelect(entity.id)}
                        >
                          <div className="tree-row-main">
                            <span>{entity.name}</span>
                            {entity.detail ? (
                              <div className="tree-detail">{entity.detail}</div>
                            ) : null}
                          </div>
                          <span className="tree-meta">{entity.entityType}</span>
                        </button>
                      </li>
                    ))
                  ) : (
                    <li className="tree-empty">
                      {t("ui.workspace.empty_section", "Aucun element")}
                    </li>
                  )}
                </ul>
              </li>

              <li className="tree-section">
                <div className="tree-section-title">
                  {t("ui.workspace.endpoints_section", "Endpoints")}
                </div>
                <ul className="tree-sublist">
                  {projectSnapshot.endpoints.length > 0 ? (
                    projectSnapshot.endpoints.map((endpoint) => (
                      <li key={endpoint.id} className="tree-row">
                        <span>{endpoint.name}</span>
                        <span className="tree-meta">
                          {endpoint.endpointType}
                        </span>
                      </li>
                    ))
                  ) : (
                    <li className="tree-empty">
                      {t("ui.workspace.empty_section", "Aucun element")}
                    </li>
                  )}
                </ul>
              </li>

              <li className="tree-section">
                <div className="tree-section-title">
                  {t("ui.workspace.streams_section", "Flux")}
                </div>
                <ul className="tree-sublist">
                  {projectSnapshot.streams.length > 0 ? (
                    projectSnapshot.streams.map((stream) => (
                      <li key={stream.id} className="tree-row">
                        <span>{stream.name}</span>
                        <span className="tree-meta">{stream.direction}</span>
                      </li>
                    ))
                  ) : (
                    <li className="tree-empty">
                      {t("ui.workspace.empty_section", "Aucun element")}
                    </li>
                  )}
                </ul>
              </li>

              <li className="tree-section">
                <div className="tree-section-title">
                  {t("ui.workspace.plugins_section", "Plugins")}
                </div>
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
                    <li className="tree-empty">
                      {t("ui.workspace.empty_section", "Aucun element")}
                    </li>
                  )}
                </ul>
              </li>

              <li className="tree-section">
                <div className="tree-section-title">
                  {t("ui.workspace.openspec_section", "OpenSpec")}
                </div>
                <ul className="tree-sublist">
                  {openSpecDocuments.length > 0 ? (
                    openSpecDocuments.map((document) => (
                      <li key={document.id} className="tree-row">
                        <div className="tree-row-main">
                          <span>{document.title}</span>
                          <div className="tree-detail">{document.excerpt}</div>
                        </div>
                        <span className="tree-meta">{document.kind}</span>
                      </li>
                    ))
                  ) : (
                    <li className="tree-empty">
                      {t("ui.workspace.no_openspec", "Aucun document OpenSpec")}
                    </li>
                  )}
                </ul>
              </li>
            </ul>
          </Panel>

          <Panel
            panelId="properties"
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
              <dt>{t("ui.property.openspec", "OpenSpec")}</dt>
              <dd>{openSpecDocuments.length}</dd>
              <dt>{t("ui.property.language", "Langue")}</dt>
              <dd>
                {supportedLocales.find((entry) => entry.id === locale)?.label ??
                  locale}
              </dd>
              <dt>{t("ui.property.fixture", "Fixture")}</dt>
              <dd>{fixtureLabel(fixtureOptions, selectedFixtureId)}</dd>
            </dl>

            <div className="property-section">
              <div className="subsection-label">
                {t("ui.property.generic_inspector", "Inspecteur generique")}
              </div>
              {selectedEntity ? (
                <div
                  className="property-card-list"
                  data-entity-inspector={selectedEntity.id}
                >
                  <article className="result-card property-card">
                    <strong>{selectedEntity.name}</strong>
                    <div className="command-id">
                      {selectedEntity.entityType} | {selectedEntity.status}
                    </div>
                    <div className="muted">
                      {t("ui.property.revision", "Revision")}{" "}
                      {selectedEntity.revision}
                    </div>
                    {inspectorSchema.sections.length > 0 ? (
                      <div className="property-schema-sections">
                        {inspectorSchema.sections.map((section) => (
                          <div
                            key={section.id}
                            className="property-editor-card"
                            data-entity-section={section.id}
                          >
                            <div className="subsection-label">{section.label}</div>
                            <div className="property-editor-grid">
                              {section.fields.map((field) =>
                                renderInspectorField(field),
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <p className="muted">
                        {t(
                          "ui.property.no_generic_parameters",
                          "Aucun schema exploitable pour l entite selectionnee.",
                        )}
                      </p>
                    )}

                    {inspectorError ? (
                      <div className="muted">{inspectorError}</div>
                    ) : null}

                    <div className="property-editor-actions">
                      <button
                        className="run-button"
                        type="button"
                        data-entity-save={selectedEntity.id}
                        disabled={inspectorBusy}
                        onClick={handleInspectorSubmit}
                      >
                        {inspectorBusy
                          ? t("ui.command.running", "Execution...")
                          : t("ui.property.apply", "Appliquer")}
                      </button>
                    </div>
                  </article>
                </div>
              ) : (
                <p className="muted">
                  {t(
                    "ui.property.no_entity_selected",
                    "Selectionne une entite dans l explorateur pour afficher ses proprietes.",
                  )}
                </p>
              )}
            </div>

            <div className="property-section">
              <div className="subsection-label">
                {t("ui.property.openspec_documents", "Documents OpenSpec")}
              </div>
              {openSpecDocuments.length > 0 ? (
                <div className="property-card-list">
                  {openSpecDocuments.slice(0, 3).map((document) => (
                    <article
                      key={document.id}
                      className="result-card property-card"
                      data-openspec-summary={document.id}
                    >
                      <strong>{document.title}</strong>
                      <div className="command-id">
                        {document.kind} | {document.status}
                      </div>
                      <div className="muted">{document.excerpt}</div>
                      <div className="property-inline-metrics">
                        <span>
                          {t("ui.property.linked_entities", "Entites liees")}{" "}
                          {document.linkedEntityCount}
                        </span>
                        <span>
                          {t("ui.property.linked_endpoints", "Endpoints lies")}{" "}
                          {document.linkedExternalCount}
                        </span>
                        <span>
                          {t("ui.property.tags", "Tags")} {document.tagCount}
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              ) : (
                <p className="muted">
                  {t(
                    "ui.property.no_openspec_documents",
                    "Aucun document OpenSpec lisible dans cette session.",
                  )}
                </p>
              )}
            </div>

            <div className="property-section">
              <div className="subsection-label">
                {t("ui.property.parametric_parts", "Pieces parametriques")}
              </div>
              {parametricParts.length > 0 ? (
                <div className="property-card-list">
                  <div className="property-editor-card">
                    <div className="subsection-label">
                      {t(
                        "ui.property.parametric_editor",
                        "Edition parametrique",
                      )}
                    </div>
                    <div className="property-editor-grid">
                      <label className="control-group property-control">
                        <span>{t("ui.property.width", "Largeur")}</span>
                        <input
                          className="shell-select shell-input"
                          type="number"
                          min="0.1"
                          step="0.1"
                          value={partEditor.widthMm}
                          onChange={(event) =>
                            handlePartEditorChange(
                              "widthMm",
                              event.target.value,
                            )
                          }
                        />
                      </label>
                      <label className="control-group property-control">
                        <span>{t("ui.property.height", "Hauteur")}</span>
                        <input
                          className="shell-select shell-input"
                          type="number"
                          min="0.1"
                          step="0.1"
                          value={partEditor.heightMm}
                          onChange={(event) =>
                            handlePartEditorChange(
                              "heightMm",
                              event.target.value,
                            )
                          }
                        />
                      </label>
                      <label className="control-group property-control">
                        <span>{t("ui.property.depth", "Profondeur")}</span>
                        <input
                          className="shell-select shell-input"
                          type="number"
                          min="0.1"
                          step="0.1"
                          value={partEditor.depthMm}
                          onChange={(event) =>
                            handlePartEditorChange(
                              "depthMm",
                              event.target.value,
                            )
                          }
                        />
                      </label>
                    </div>
                    <div className="property-editor-actions">
                      <button
                        className="run-button"
                        type="button"
                        data-parametric-regenerate="true"
                        disabled={
                          executingCommandId === "build.regenerate_part"
                        }
                        onClick={handleLatestPartRegenerate}
                        title={commandButtonTitle(
                          "build.regenerate_part",
                          t("ui.command.regenerate_part", "Regenerer la piece"),
                        )}
                      >
                        {executingCommandId === "build.regenerate_part"
                          ? t("ui.command.running", "Execution...")
                          : t(
                              "ui.command.regenerate_part",
                              "Regenerer la piece",
                            )}
                      </button>
                    </div>
                  </div>
                  {[...parametricParts]
                    .reverse()
                    .slice(0, 3)
                    .map((entity) => (
                      <article
                        key={entity.id}
                        className="result-card property-card"
                        data-parametric-part-summary={entity.id}
                      >
                        <strong>{entity.name}</strong>
                        <div className="command-id">
                          {formatParametricPartSummary(
                            locale,
                            entity.partGeometry,
                          )}
                        </div>
                        <div className="muted">
                          {entity.partGeometry.materialName} |{" "}
                          {entity.partGeometry.state}
                        </div>
                        <div className="property-inline-metrics">
                          <span>
                            {t("ui.property.area", "Aire")}{" "}
                            {formatDecimal(locale, entity.partGeometry.areaMm2)}{" "}
                            mm²
                          </span>
                          <span>
                            {t("ui.property.volume", "Volume")}{" "}
                            {formatDecimal(
                              locale,
                              entity.partGeometry.volumeMm3,
                            )}{" "}
                            mm³
                          </span>
                          <span data-parametric-part-mass={entity.id}>
                            {t("ui.property.mass", "Masse")}{" "}
                            {formatDecimal(
                              locale,
                              entity.partGeometry.estimatedMassGrams,
                            )}{" "}
                            g
                          </span>
                        </div>
                      </article>
                    ))}
                </div>
              ) : (
                <p className="muted">
                  {t(
                    "ui.property.no_parametric_parts",
                    "Aucune piece parametrique regeneree dans cette session.",
                  )}
                </p>
              )}
            </div>

            <div className="property-section">
              <div className="subsection-label">
                {t("ui.property.robot_cells", "Cellules robotiques")}
              </div>
              {robotCells.length > 0 ? (
                <div className="property-card-list">
                  {[...robotCells]
                    .reverse()
                    .slice(0, 2)
                    .map((entity) => (
                      <article
                        key={entity.id}
                        className="result-card property-card"
                        data-robot-cell-summary={entity.id}
                      >
                        <strong>{entity.name}</strong>
                        <div className="command-id">
                          {formatRobotCellSummary(
                            locale,
                            entity.robotCellSummary,
                          )}
                        </div>
                        <div className="muted">
                          {entity.robotCellSummary.safetyZoneCount}{" "}
                          {t("ui.property.safety_zones", "zones safety")} |{" "}
                          {entity.robotCellSummary.warningCount}{" "}
                          {t("ui.property.warnings", "warning(s)")}
                        </div>
                        <div className="property-inline-metrics">
                          <span>
                            {t("ui.property.path_length", "Trajet")}{" "}
                            {formatDecimal(
                              locale,
                              entity.robotCellSummary.pathLengthMm,
                              0,
                            )}{" "}
                            mm
                          </span>
                          <span>
                            {t("ui.property.max_segment", "Segment max")}{" "}
                            {formatDecimal(
                              locale,
                              entity.robotCellSummary.maxSegmentMm,
                              0,
                            )}{" "}
                            mm
                          </span>
                          <span data-robot-cell-targets={entity.id}>
                            {t("ui.property.targets", "Cibles")}{" "}
                            {entity.robotCellSummary.targetCount}
                          </span>
                        </div>
                      </article>
                    ))}
                </div>
              ) : (
                <p className="muted">
                  {t(
                    "ui.property.no_robot_cells",
                    "Aucune cellule robotique construite dans cette session.",
                  )}
                </p>
              )}
            </div>

            <div className="property-section">
              <div className="subsection-label">
                {t("ui.property.simulation_runs", "Runs de simulation")}
              </div>
              {simulationRuns.length > 0 ? (
                <div className="property-card-list">
                  {[...simulationRuns]
                    .reverse()
                    .slice(0, 2)
                    .map((entity) => (
                      <article
                        key={entity.id}
                        className="result-card property-card"
                        data-simulation-run-summary={entity.id}
                      >
                        <strong>{entity.name}</strong>
                        <div className="command-id">
                          {formatSimulationRunSummary(
                            locale,
                            entity.simulationRunSummary,
                          )}
                        </div>
                        <div className="muted">
                          {t(
                            "ui.property.timeline_samples",
                            "Echantillons timeline",
                          )}{" "}
                          {entity.simulationRunSummary.timelineSampleCount}
                        </div>
                        <div className="property-inline-metrics">
                          <span>
                            {t("ui.property.tracking_error", "Erreur max")}{" "}
                            {formatDecimal(
                              locale,
                              entity.simulationRunSummary.maxTrackingErrorMm,
                              2,
                            )}{" "}
                            mm
                          </span>
                          <span>
                            {t("ui.property.energy", "Energie")}{" "}
                            {formatDecimal(
                              locale,
                              entity.simulationRunSummary.energyEstimateJ,
                              2,
                            )}{" "}
                            J
                          </span>
                          <span data-simulation-run-collisions={entity.id}>
                            {t("ui.property.collisions", "Collisions")}{" "}
                            {entity.simulationRunSummary.collisionCount}
                          </span>
                        </div>
                      </article>
                    ))}
                </div>
              ) : (
                <p className="muted">
                  {t(
                    "ui.property.no_simulation_runs",
                    "Aucun run de simulation execute dans cette session.",
                  )}
                </p>
              )}
            </div>

            <div className="property-section">
              <div className="subsection-label">
                {t("ui.property.safety_reports", "Rapports safety")}
              </div>
              {safetyReports.length > 0 ? (
                <div className="property-card-list">
                  {[...safetyReports]
                    .reverse()
                    .slice(0, 2)
                    .map((entity) => (
                      <article
                        key={entity.id}
                        className="result-card property-card"
                        data-safety-report-summary={entity.id}
                      >
                        <strong>{entity.name}</strong>
                        <div className="command-id">
                          {formatSafetyReportSummary(
                            locale,
                            entity.safetyReportSummary,
                          )}
                        </div>
                        <div className="muted">
                          {entity.safetyReportSummary.inhibited
                            ? t("ui.property.safety_blocked", "Action bloquee")
                            : t(
                                "ui.property.safety_not_blocked",
                                "Action autorisee sous surveillance",
                              )}
                        </div>
                        <div className="property-inline-metrics">
                          <span>
                            {t("ui.property.active_zones", "Zones actives")}{" "}
                            {entity.safetyReportSummary.activeZoneCount}
                          </span>
                          <span>
                            {t("ui.property.advisories", "Advisories")}{" "}
                            {entity.safetyReportSummary.advisoryZoneCount}
                          </span>
                          <span data-safety-report-blocks={entity.id}>
                            {t("ui.property.interlocks", "Interlocks")}{" "}
                            {entity.safetyReportSummary.blockingInterlockCount}
                          </span>
                        </div>
                      </article>
                    ))}
                </div>
              ) : (
                <p className="muted">
                  {t(
                    "ui.property.no_safety_reports",
                    "Aucun rapport safety genere dans cette session.",
                  )}
                </p>
              )}
            </div>
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
            panelId="commandSurface"
            title={t("ui.panel.command_surface", "Surface de commandes")}
            accent={menu.label}
            collapsed={!panelState.commandSurface}
            onToggle={() => togglePanel("commandSurface")}
            toggleLabel={panelToggleLabel("commandSurface")}
          >
            {commandResult ? (
              <div
                className="command-feedback"
                data-command-feedback={commandResult.commandId}
                data-command-feedback-status={commandResult.status}
              >
                <strong>{commandResult.commandId}</strong>
                <div className="command-id">{commandResult.status}</div>
                <div className="muted">{commandResult.message}</div>
              </div>
            ) : null}

            <ul className="command-list">
              {menu.items.map((item, index) =>
                item.type === "separator" ? (
                  <li key={`${menu.id}-sep-${index}`} className="separator" />
                ) : (
                  <li
                    key={item.id}
                    className={
                      commandResult?.commandId === item.command
                        ? "command-row is-last-command"
                        : "command-row"
                    }
                  >
                    <div>
                      <strong>{item.label}</strong>
                      <div className="command-id">{item.command}</div>
                    </div>
                    <div className="command-actions">
                      <span className="shortcut">
                        {formatShortcutLabel(item.shortcut)}
                      </span>
                      <button
                        className="run-button"
                        type="button"
                        data-command-id={item.command}
                        disabled={executingCommandId !== null}
                        onClick={() => handleCommandExecute(item.command)}
                        title={commandButtonTitle(item.command, item.label)}
                      >
                        {executingCommandId === item.command
                          ? t("ui.command.running", "Execution...")
                          : t("ui.command.run", "Executer")}
                      </button>
                    </div>
                  </li>
                ),
              )}
            </ul>
          </Panel>

          <Panel
            panelId="viewport"
            title={t("ui.panel.viewport", "Viewport 3D")}
            accent={
              projectSnapshot.details.rootSceneId ??
              t("ui.panel.scene_host", "Hote de scene")
            }
            collapsed={!panelState.viewport}
            onToggle={() => togglePanel("viewport")}
            toggleLabel={panelToggleLabel("viewport")}
          >
            <AerospaceViewport
              locale={locale}
              t={t}
              selectedSceneId={selectedViewportSceneId}
              onSelectScene={setSelectedViewportSceneId}
            />
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

        <aside
          className={
            rightExpanded
              ? "workspace-right"
              : "workspace-right workspace-column-collapsed"
          }
        >
          <Panel
            panelId="simulationTimeline"
            title={t("ui.panel.simulation_timeline", "Timeline de simulation")}
            accent={
              selectedSimulationRun?.simulationRunSummary
                ? `${selectedSimulationRun.simulationRunSummary.status} | F10`
                : "F10"
            }
            collapsed={!panelState.simulationTimeline}
            onToggle={() => togglePanel("simulationTimeline")}
            toggleLabel={panelToggleLabel("simulationTimeline")}
          >
            <div className="stack-block">
              {simulationRuns.length > 0 ? (
                <>
                  <div className="assistant-message-tags">
                    {[...simulationRuns].reverse().slice(0, 4).map((run) => (
                      <button
                        key={run.id}
                        type="button"
                        className={
                          run.id === selectedSimulationRun?.id
                            ? "assistant-starter is-active"
                            : "assistant-starter"
                        }
                        data-simulation-run-select={run.id}
                        onClick={() => focusTimelineEvent(run.id, null)}
                      >
                        {run.name}
                      </button>
                    ))}
                  </div>

                  {focusedTimelineEvent ? (
                    <div
                      className="result-card"
                      data-simulation-timeline-focus={focusedTimelineEvent.id}
                    >
                      <strong>{focusedTimelineEvent.title}</strong>
                      <div className="command-id">
                        {focusedTimelineEvent.kind} | step{" "}
                        {focusedTimelineEvent.stepIndex} | t=
                        {focusedTimelineEvent.timestampMs} ms
                      </div>
                      <div className="muted">{focusedTimelineEvent.summary}</div>
                      <div className="property-editor-actions">
                        <button
                          className="run-button"
                          type="button"
                          data-simulation-jump-critical="true"
                          onClick={() => {
                            const criticalEvent = criticalTimelineEvent(
                              simulationTimelineEvents,
                            );
                            if (!criticalEvent) {
                              return;
                            }
                            focusTimelineEvent(
                              selectedSimulationRun?.id,
                              criticalEvent.id,
                            );
                            applyCommandFeedback({
                              commandId: "simulation.timeline.jump_critical",
                              status: "applied",
                              message: `${criticalEvent.kind} | step ${criticalEvent.stepIndex} | t=${criticalEvent.timestampMs} ms`,
                            });
                          }}
                        >
                          {t(
                            "ui.timeline.jump_critical",
                            "Aller a l instant critique",
                          )}
                        </button>
                        <button
                          className="run-button"
                          type="button"
                          data-simulation-step="true"
                          onClick={handleTimelineStep}
                        >
                          {t("ui.timeline.step", "Pas suivant")}
                        </button>
                      </div>
                    </div>
                  ) : null}

                  <ul className="command-list simulation-timeline-list">
                    {simulationTimelineEvents.map((event) => (
                      <li
                        key={event.id}
                        className={
                          event.id === focusedTimelineEvent?.id
                            ? "command-row is-last-command"
                            : "command-row"
                        }
                      >
                        <div>
                          <strong>{event.title}</strong>
                          <div className="command-id">
                            {event.kind} | step {event.stepIndex} | t=
                            {event.timestampMs} ms
                          </div>
                          <div className="muted">{event.summary}</div>
                        </div>
                        <div className="command-actions">
                          <button
                            className="run-button"
                            type="button"
                            data-simulation-event={event.id}
                            onClick={() =>
                              focusTimelineEvent(
                                selectedSimulationRun?.id,
                                event.id,
                              )
                            }
                          >
                            {t("ui.timeline.focus", "Focus")}
                          </button>
                        </div>
                      </li>
                    ))}
                  </ul>
                </>
              ) : (
                <p className="muted">
                  {t(
                    "ui.timeline.no_runs",
                    "Aucun run persiste a afficher dans la timeline.",
                  )}
                </p>
              )}
            </div>
          </Panel>

          <Panel
            panelId="aiAssistant"
            title={t("ui.panel.ai_assistant", "Assistant IA local")}
            accent={assistantAccent(locale, aiRuntime)}
            collapsed={!panelState.aiAssistant}
            onToggle={() => togglePanel("aiAssistant")}
            toggleLabel={panelToggleLabel("aiAssistant")}
          >
            <div className="stack-block">
              <div
                className={
                  aiRuntime.available
                    ? "assistant-runtime ready"
                    : "assistant-runtime fallback"
                }
              >
                <div className="assistant-runtime-row">
                  <strong>
                    {aiRuntime.available
                      ? t("ui.ai.runtime_ready", "Pret")
                      : t("ui.ai.runtime_fallback", "Fallback local")}
                  </strong>
                  <span className="command-id">
                    {aiRuntime.provider} | {aiRuntime.activeProfile ?? "balanced"} |{" "}
                    {aiRuntime.activeModel ??
                      t("ui.ai.no_model", "aucun modele")}
                  </span>
                </div>
                <div className="muted">
                  {t("ui.ai.endpoint", "Endpoint")} {aiRuntime.endpoint}
                </div>
                {aiRuntime.warning ? (
                  <div className="muted">{aiRuntime.warning}</div>
                ) : null}
              </div>

              <label className="control-group assistant-model-group">
                <span>{t("ui.ai.profile_label", "Profil IA")}</span>
                <select
                  className="shell-select"
                  data-ai-profile-select="true"
                  aria-label={t("ui.ai.profile_label", "Profil IA")}
                  value={selectedAiProfile}
                  onChange={(event) => setSelectedAiProfile(event.target.value)}
                  disabled={aiBusy}
                >
                  {availableAiProfiles.map((profile) => (
                    <option key={profile} value={profile}>
                      {profile}
                    </option>
                  ))}
                </select>
                <div className="muted">
                  {t(
                    "ui.ai.profile_hint",
                    "Le profil selectionne pilote la profondeur du runtime local au prochain message.",
                  )}
                </div>
              </label>

              <label className="control-group assistant-model-group">
                <span>{t("ui.ai.model_label", "Modele Gemma3")}</span>
                <select
                  className="shell-select"
                  data-ai-model-select="true"
                  aria-label={t("ui.ai.model_label", "Modele Gemma3")}
                  value={selectedAiModel}
                  onChange={(event) => setSelectedAiModel(event.target.value)}
                  disabled={gemma3Models.length === 0 || aiBusy}
                >
                  {gemma3Models.length > 0 ? (
                    gemma3Models.map((model) => (
                      <option key={model} value={model}>
                        {model}
                      </option>
                    ))
                  ) : (
                    <option value="">
                      {t(
                        "ui.ai.model_unavailable",
                        "Aucune variante gemma3 detectee localement.",
                      )}
                    </option>
                  )}
                </select>
                <div className="muted">
                  {gemma3Models.length > 0
                    ? t(
                        "ui.ai.model_hint",
                        "Le modele selectionne sera utilise au prochain message.",
                      )
                    : t(
                        "ui.ai.model_unavailable",
                        "Aucune variante gemma3 detectee localement.",
                      )}
                </div>
              </label>

              <label className="control-group assistant-model-group">
                <span>
                  {t("ui.ai.auto_prompt_label", "Relances auto OpenSpec")}
                </span>
                <input
                  type="checkbox"
                  data-ai-auto-prompts="true"
                  checked={autoPromptEnabled}
                  onChange={(event) =>
                    setAutoPromptEnabled(event.target.checked)
                  }
                />
                <div className="muted">
                  {autoPromptEnabled
                    ? t(
                        "ui.ai.auto_prompt_enabled",
                        `Actif apres simulation, safety ou revue OpenSpec. File: ${autoPromptQueue.length}.`,
                      )
                    : t(
                        "ui.ai.auto_prompt_disabled",
                        "Desactive. Aucun prompt automatique n est relance.",
                      )}
                </div>
              </label>

              <div className="assistant-thread">
                {aiMessages.length > 0 ? (
                  aiMessages.map((entry, index) => (
                    <article
                      key={`${entry.role}-${index}`}
                      className={
                        entry.role === "assistant"
                          ? "assistant-message assistant"
                          : "assistant-message user"
                      }
                    >
                      <div className="assistant-message-header">
                        <strong>
                          {assistantRoleLabel(locale, entry.role)}
                        </strong>
                        <span className="command-id">{entry.source}</span>
                      </div>
                      <div className="assistant-message-body">
                        {entry.content}
                      </div>
                      {entry.structured ? (
                        <div className="result-card" data-ai-structured="true">
                          <strong>{entry.structured.summary}</strong>
                          <div className="command-id">
                            {entry.structured.runtimeProfile ?? "balanced"} |{" "}
                            {entry.structured.riskLevel} |{" "}
                            {formatStructuredConfidence(
                              entry.structured.confidence,
                            )}
                          </div>
                          {entry.structured.explanation?.map((line) => (
                            <div key={line} className="muted">
                              {line}
                            </div>
                          ))}
                          {entry.structured.contextRefs?.length > 0 ? (
                            <div className="assistant-message-tags">
                              {entry.structured.contextRefs.map(
                                (reference, referenceIndex) => (
                                  <span
                                    key={`${reference.entityId ?? "project"}-${reference.path}-${referenceIndex}`}
                                    className="assistant-tag"
                                  >
                                    {reference.entityId ?? "project"} |{" "}
                                    {reference.path}
                                  </span>
                                ),
                              )}
                            </div>
                              ) : null}
                          {entry.structured.critiquePasses?.length > 0 ? (
                            <div className="property-card-list">
                              {entry.structured.critiquePasses.map((pass) => (
                                <div
                                  key={`${entry.suggestionId ?? "structured"}-${pass.stage}`}
                                  className="result-card"
                                  data-ai-critique-pass={pass.stage}
                                >
                                  <strong>{pass.stage}</strong>
                                  <div className="muted">{pass.summary}</div>
                                  {pass.issues?.length > 0 ? (
                                    <div className="assistant-warning-list">
                                      {pass.issues.map((issue) => (
                                        <div key={issue} className="muted">
                                          {issue}
                                        </div>
                                      ))}
                                    </div>
                                  ) : null}
                                </div>
                              ))}
                            </div>
                          ) : null}
                          {entry.structured.limitations?.length > 0 ? (
                            <div className="assistant-warning-list">
                              {entry.structured.limitations.map((limitation) => (
                                <div key={limitation} className="muted">
                                  {limitation}
                                </div>
                              ))}
                            </div>
                          ) : null}
                          {entry.structured.proposedCommands?.length > 0 ? (
                            <div
                              className="property-card-list"
                              data-ai-suggestion-preview={
                                entry.suggestionId ?? "preview-only"
                              }
                            >
                              {entry.structured.proposedCommands.map(
                                (command, commandIndex) => (
                                  <div
                                    key={`${entry.suggestionId ?? "preview"}-${command.kind}-${commandIndex}`}
                                    className="result-card"
                                    data-ai-proposed-command={command.kind}
                                  >
                                    <strong>{command.kind}</strong>
                                    <div className="muted">
                                      {summarizeAiProposedCommand(command)}
                                    </div>
                                  </div>
                                ),
                              )}
                              {entry.suggestionId ? (
                                <div className="property-editor-actions">
                                  <button
                                    className="run-button"
                                    type="button"
                                    data-ai-apply-suggestion={entry.suggestionId}
                                    disabled={
                                      aiBusy || entry.suggestionStatus === "applied"
                                    }
                                    onClick={() =>
                                      handleAiSuggestionApply(entry.suggestionId)
                                    }
                                  >
                                    {entry.suggestionStatus === "applied"
                                      ? t(
                                          "ui.ai.suggestion_applied",
                                          "Suggestion appliquee",
                                        )
                                      : t(
                                          "ui.ai.apply_suggestion",
                                          "Appliquer la suggestion",
                                        )}
                                  </button>
                                  <button
                                    className="run-button"
                                    type="button"
                                    data-ai-reject-suggestion={entry.suggestionId}
                                    disabled={
                                      aiBusy || entry.suggestionStatus === "rejected"
                                    }
                                    onClick={() =>
                                      handleAiSuggestionReject(entry.suggestionId)
                                    }
                                  >
                                    {entry.suggestionStatus === "rejected"
                                      ? t(
                                          "ui.ai.suggestion_rejected",
                                          "Suggestion rejetee",
                                        )
                                      : t(
                                          "ui.ai.reject_suggestion",
                                          "Rejeter",
                                        )}
                                  </button>
                                </div>
                              ) : null}
                            </div>
                          ) : null}
                        </div>
                      ) : null}
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
                        "Aucune discussion pour l instant. Le chat s appuie sur le projet charge et reste local.",
                      )}
                    </p>
                    <div className="assistant-starters">
                      {starterPrompts.map((prompt) => (
                        <button
                          key={prompt}
                          className="assistant-starter"
                          type="button"
                          data-ai-starter={prompt}
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
                    "Pose une question sur le projet courant, la simulation, l integration ou la safety...",
                  )}
                  rows={4}
                />
                <div className="assistant-form-footer">
                  <span className="muted">
                    {t("ui.ai.local_only", "Mode local uniquement")}
                  </span>
                  <button
                    className="run-button"
                    type="submit"
                    data-ai-send="true"
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
            panelId="output"
            title={t("ui.panel.output", "Sortie")}
            accent={t("ui.panel.live", "Actif")}
            collapsed={!panelState.output}
            onToggle={() => togglePanel("output")}
            toggleLabel={panelToggleLabel("output")}
          >
            <div className="stack-block">
              <div className="subsection-label">
                {t("ui.command.last_result", "Dernier resultat")}
              </div>
              {commandResult ? (
                <div className="result-card">
                  <strong data-last-command-id={commandResult.commandId}>
                    {commandResult.commandId}
                  </strong>
                  <div
                    className="command-id"
                    data-last-command-status={commandResult.status}
                  >
                    {commandResult.status}
                  </div>
                  <div className="muted">{commandResult.message}</div>
                </div>
              ) : (
                <p className="muted">
                  {t(
                    "ui.command.no_result",
                    "Aucune commande executee pendant cette session.",
                  )}
                </p>
              )}

              <div className="subsection-label">
                {t("ui.output.recent_activity", "Activite recente")}
              </div>
              {projectSnapshot.recentActivity.length > 0 ? (
                <ul className="command-list">
                  {projectSnapshot.recentActivity.map((entry) => (
                    <li key={entry.id} className="command-row">
                      <div>
                        <strong>{entry.kind}</strong>
                        <div className="command-id">
                          {activityChannelLabel(locale, entry.channel)} |{" "}
                          {entry.targetId ?? currentStatus.projectName}
                        </div>
                      </div>
                      <span className="shortcut">{entry.timestamp}</span>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="muted">
                  {t(
                    "ui.output.no_activity",
                    "Aucune activite commande/evenement.",
                  )}
                </p>
              )}

              <div className="subsection-label">
                {t("ui.output.raw_status", "Etat brut")}
              </div>
              <pre className="output-box">
                {JSON.stringify(currentStatus, null, 2)}
              </pre>
            </div>
          </Panel>

          <Panel
            panelId="problems"
            title={t("ui.panel.problems", "Problemes")}
            accent={t("ui.problems.none_blocking", "0 bloquant")}
            collapsed={!panelState.problems}
            onToggle={() => togglePanel("problems")}
            toggleLabel={panelToggleLabel("problems")}
          >
            <p className="muted">
              {t("ui.output.problems", "Les checks critiques remontent ici.")}
            </p>
          </Panel>
        </aside>
      </main>
    </div>
  );
}
