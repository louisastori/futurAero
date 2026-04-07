import { useEffect, useRef, useState } from "react";

import {
  calculateResizedDockWidths,
  defaultWorkspaceDockWidths,
  defaultWorkspacePanels,
  defaultLocale,
  findMenuCommandByShortcut,
  getWorkspaceColumnState,
  getVisibleSidebarWidth,
  localizeMenuModel,
  panelIdFromCommand,
  shouldHandleShortcutEvent,
  supportedLocales,
  toggleWorkspacePanel,
  WORKSPACE_RESIZER_WIDTH,
  translate
} from "@futureaero/ui";
import {
  aerospaceReferenceScenes,
  defaultAerospaceSceneId,
  getAerospaceScene
} from "@futureaero/viewport";

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
  gemma3Models: [],
  warning: "Local AI runtime unavailable in web preview."
};

function getGemma3Models(runtime) {
  if (Array.isArray(runtime?.gemma3Models) && runtime.gemma3Models.length > 0) {
    return [...new Set(runtime.gemma3Models)];
  }

  return [...new Set((runtime?.availableModels ?? []).filter((model) => model.startsWith("gemma3:")))];
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

function MenuBar({ menus, activeMenuId, onSelect }) {
  return (
    <nav className="menu-bar" aria-label="Application menu">
      {menus.map((menu) => (
        <button
          key={menu.id}
          data-menu-id={menu.id}
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

function Panel({ panelId, title, children, accent, collapsed = false, onToggle, toggleLabel }) {
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
          <span className="viewport-legend-swatch" style={{ background: item.color }} />
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
        <path d="M0 -12 C18 -36 36 -72 40 -120 C24 -102 12 -70 -4 -28 Z" fill="rgba(179,197,255,0.72)" stroke="rgba(240,246,255,0.38)" strokeWidth="1" />
      </g>
    );
  });

  const compressorStages = Array.from({ length: 8 }, (_, index) => (
    <g key={`stage-${index}`} transform={`translate(${322 + index * 58} 140)`}>
      <ellipse cx="0" cy="54" rx="34" ry="98" fill="none" stroke="rgba(210,160,255,0.62)" strokeWidth="1.5" />
      <ellipse cx="0" cy="54" rx="20" ry="66" fill="none" stroke="rgba(135,255,205,0.45)" strokeWidth="1.25" />
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
        <circle cx="126" cy="186" r="122" fill="none" stroke="#7bc6ff" strokeWidth="1.4" />
        <circle cx="126" cy="186" r="82" fill="none" stroke="#7bc6ff" strokeWidth="1" />
      </g>
      <g>{fanBlades}</g>
      <ellipse cx="140" cy="186" rx="54" ry="54" fill="#171d2d" stroke="rgba(255,255,255,0.18)" strokeWidth="4" />
      <ellipse cx="140" cy="186" rx="20" ry="20" fill="#d7e3ff" fillOpacity="0.86" />
      <path d="M160 130 L270 130 L308 186 L270 242 L160 242 Q128 214 128 186 Q128 158 160 130 Z" fill="url(#turbofan-shell)" stroke="rgba(255,255,255,0.22)" strokeWidth="2" />
      <rect x="190" y="170" width="592" height="32" rx="16" fill="url(#turbofan-core)" />
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
      <path d="M84 196 C144 136 232 108 394 114 L704 126 C776 130 834 160 884 184 C838 204 794 230 742 242 L504 258 C324 270 174 252 88 212 Z" fill="rgba(180,203,255,0.08)" stroke="rgba(220,230,255,0.42)" strokeWidth="2.4" />
      <path d="M82 196 L44 202 L86 170 Z" fill="rgba(255,255,255,0.18)" />
      <path d="M430 128 L586 54 L602 130" fill="rgba(124,199,255,0.08)" stroke="rgba(190,232,255,0.42)" strokeWidth="2" />
      <path d="M458 248 L684 324 L602 250" fill="rgba(124,199,255,0.06)" stroke="rgba(190,232,255,0.42)" strokeWidth="2" />
      <path d="M350 250 L168 340 L292 252" fill="rgba(255,140,200,0.05)" stroke="rgba(255,196,92,0.45)" strokeWidth="2" />
      <g>{frames}</g>
      <g>{ribs}</g>
      <g opacity="0.88">
        <rect x="300" y="178" width="88" height="14" rx="7" fill="#ff8ec7" />
        <rect x="432" y="172" width="122" height="16" rx="8" fill="#7cd6ff" />
        <rect x="586" y="166" width="92" height="18" rx="9" fill="#ffe783" />
        <circle cx="618" cy="188" r="42" fill="none" stroke="rgba(255,255,255,0.48)" strokeWidth="2.2" />
        <circle cx="676" cy="192" r="44" fill="none" stroke="rgba(255,255,255,0.48)" strokeWidth="2.2" />
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
      <path d="M108 220 C160 168 260 132 424 126 L712 136 L854 176 L724 214 L416 244 C270 258 164 254 108 220 Z" fill="none" stroke="rgba(228,240,255,0.58)" strokeWidth="2" />
      <path d="M394 134 L550 70 L572 138" fill="none" stroke="rgba(160,200,255,0.52)" strokeWidth="1.8" />
      <path d="M418 244 L668 340 L572 246" fill="none" stroke="rgba(160,200,255,0.52)" strokeWidth="1.8" />
      <path d="M312 246 L146 338 L256 248" fill="none" stroke="rgba(160,200,255,0.52)" strokeWidth="1.8" />
      <circle cx="690" cy="196" r="54" fill="none" stroke="rgba(244,255,141,0.5)" strokeWidth="1.6" />
      <circle cx="754" cy="192" r="58" fill="none" stroke="rgba(244,255,141,0.5)" strokeWidth="1.6" />
      <g>{grid}</g>
      <g opacity="0.82">
        <path d="M74 124 L116 124 L96 88 Z" fill="#f1ff85" />
        <path d="M88 136 L88 248" stroke="#f1ff85" strokeWidth="2.5" />
        <circle cx="126" cy="300" r="62" fill="none" stroke="rgba(212,166,255,0.65)" strokeWidth="2" strokeDasharray="6 6" />
        <path d="M146 300 L280 242" stroke="#d4a6ff" strokeWidth="2.2" strokeDasharray="6 6" />
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
      <path d="M178 334 L252 126 C270 84 314 72 346 98 L412 154 L474 104 C510 74 562 88 578 130 L664 332" fill="none" stroke="rgba(255,255,255,0.14)" strokeWidth="42" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M178 334 L252 126 C270 84 314 72 346 98 L412 154 L474 104 C510 74 562 88 578 130 L664 332" fill="none" stroke="url(#stressGradient)" strokeWidth="28" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="412" cy="154" r="22" fill="#ff5570" fillOpacity="0.9" />
      <circle cx="474" cy="104" r="18" fill="#ffd766" fillOpacity="0.9" />
      <circle cx="252" cy="126" r="16" fill="#63e4ff" fillOpacity="0.9" />
      <rect x="764" y="72" width="22" height="216" rx="11" fill="url(#stressGradient)" />
      <text x="804" y="84" fill="#d9e9ff" fontSize="16">High</text>
      <text x="804" y="286" fill="#d9e9ff" fontSize="16">Low</text>
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
      <path d="M136 240 C264 180 420 154 662 162 C716 164 774 186 820 210 C758 214 704 226 658 242 C438 316 284 324 126 286 Z" fill="rgba(255,255,255,0.05)" stroke="rgba(240,244,255,0.24)" strokeWidth="2" />
      <path d="M156 246 C274 198 418 178 646 186 C694 188 738 198 782 214 C736 218 694 230 654 246 C436 302 298 306 156 276 Z" fill="url(#aeroGradient)" fillOpacity="0.82" stroke="rgba(255,255,255,0.1)" strokeWidth="2" />
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
    color: [scene.palette.accent, scene.palette.secondary, scene.palette.tertiary][index]
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
      <div className="viewport-scene-tabs" role="tablist" aria-label={t("ui.viewport.reference_toolbar", "Scenes de reference")}>
        {aerospaceReferenceScenes.map((entry) => (
          <button
            key={entry.id}
            type="button"
            data-scene-id={entry.id}
            role="tab"
            aria-selected={entry.id === scene.id}
            className={entry.id === scene.id ? "viewport-scene-tab active" : "viewport-scene-tab"}
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
            {t("ui.viewport.reference_inspiration", "Reproduction originale inspiree des references fournies")}
          </div>
        </div>

        <div className="viewport-inspector">
          <div className="viewport-inspector-block">
            <div className="subsection-label">{t("ui.viewport.reference_caption", "Lecture de la scene")}</div>
            <strong className="viewport-scene-title">{t(scene.titleKey, scene.id)}</strong>
            <div className="muted">{t(scene.summaryKey, scene.summaryKey)}</div>
          </div>

          <div className="viewport-inspector-block">
            <div className="subsection-label">{t("ui.viewport.reference_analysis", "Analyse extraite des images")}</div>
            <ul className="viewport-analysis-list">
              {scene.analysisKeys.map((key) => (
                <li key={key}>{t(key, key)}</li>
              ))}
            </ul>
          </div>

          <div className="viewport-inspector-block">
            <div className="subsection-label">{t("ui.viewport.reference_legend", "Legende")}</div>
            <ViewportLegend items={legend} />
          </div>

          <div className="viewport-inspector-block">
            <div className="subsection-label">{t("ui.locale.label", "Langue")}</div>
            <div className="muted">{locale.toUpperCase()}</div>
          </div>
        </div>
      </div>
    </div>
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

async function requestWindowClose() {
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

  const currentIndex = fixtures.findIndex((fixture) => fixture.id === currentFixtureId);
  if (currentIndex === -1) {
    return fixtures[0].id;
  }

  return fixtures[(currentIndex + 1) % fixtures.length].id;
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
        ...appendFallbackActivity(currentSnapshot, "system", "entity.create.part", `ent_part_${String(index).padStart(3, "0")}`),
        status: {
          ...currentSnapshot.status,
          entityCount: currentSnapshot.entities.length + 1
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
            partGeometry: {
              state: "well_constrained",
              widthMm,
              heightMm,
              depthMm,
              pointCount: 4,
              perimeterMm: (2 * (widthMm + heightMm)),
              areaMm2,
              volumeMm3,
              estimatedMassGrams,
              materialName: "Aluminum 6061"
            }
          }
        ]
      },
      result: {
        commandId,
        status: "applied",
        message: "piece parametrique regeneree dans l apercu web"
      }
    };
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

async function sendAiChatMessage(message, locale, history, selectedModel, snapshot) {
  const response = await invokeBackend("ai_chat_send_message", {
    message,
    locale,
    history,
    selectedModel
  });
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

const defaultDesktopBackend = {
  fetchWorkspaceBootstrap,
  loadWorkspaceFixture,
  executeWorkspaceCommand,
  fetchAiRuntimeStatus,
  sendAiChatMessage
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
    maximumFractionDigits
  }).format(value);
}

function formatParametricPartSummary(locale, partGeometry) {
  return `${formatDecimal(locale, partGeometry.widthMm)} x ${formatDecimal(locale, partGeometry.heightMm)} x ${formatDecimal(locale, partGeometry.depthMm)} mm | ${formatDecimal(locale, partGeometry.estimatedMassGrams)} g`;
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

export default function App({ backend = defaultDesktopBackend }) {
  const [locale, setLocale] = useState(defaultLocale);
  const [projectSnapshot, setProjectSnapshot] = useState(FALLBACK_SNAPSHOT);
  const [fixtureProjects, setFixtureProjects] = useState(FALLBACK_FIXTURES);
  const [selectedFixtureId, setSelectedFixtureId] = useState(FALLBACK_STATUS.fixtureId);
  const [selectedViewportSceneId, setSelectedViewportSceneId] = useState(defaultAerospaceSceneId);
  const [activeMenuId, setActiveMenuId] = useState("file");
  const [fixtureLoading, setFixtureLoading] = useState(false);
  const [executingCommandId, setExecutingCommandId] = useState(null);
  const [commandResult, setCommandResult] = useState(null);
  const [aiRuntime, setAiRuntime] = useState(FALLBACK_AI_STATUS);
  const [aiMessages, setAiMessages] = useState([]);
  const [aiDraft, setAiDraft] = useState("");
  const [aiBusy, setAiBusy] = useState(false);
  const [selectedAiModel, setSelectedAiModel] = useState("");
  const [panelState, setPanelState] = useState(defaultWorkspacePanels);
  const [dockWidths, setDockWidths] = useState(defaultWorkspaceDockWidths);
  const [dragSide, setDragSide] = useState(null);
  const aiInputRef = useRef(null);
  const workspaceRef = useRef(null);
  const dragStateRef = useRef(null);

  const menus = localizeMenuModel(locale);
  const menu = menus.find((entry) => entry.id === activeMenuId) ?? menus[0];
  const currentStatus = projectSnapshot.status;
  const parametricParts = projectSnapshot.entities.filter((entity) => entity.partGeometry);
  const fixtureOptions =
    selectedFixtureId && !fixtureProjects.some((fixture) => fixture.id === selectedFixtureId)
      ? [{ id: selectedFixtureId, projectName: currentStatus.projectName }, ...fixtureProjects]
      : fixtureProjects;
  const t = (key, fallback = key) => translate(locale, key, fallback);
  const { leftExpanded, rightExpanded } = getWorkspaceColumnState(panelState);
  const gemma3Models = getGemma3Models(aiRuntime);
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
    setSelectedAiModel((previous) => {
      if (previous && gemma3Models.includes(previous)) {
        return previous;
      }

      return defaultGemma3Model(aiRuntime);
    });
  }, [aiRuntime.activeModel, gemma3Models.join("|")]);

  useEffect(() => {
    let mounted = true;

    async function bootstrapWorkspace() {
      const [bootstrap, runtime] = await Promise.all([
        backend.fetchWorkspaceBootstrap(),
        backend.fetchAiRuntimeStatus()
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

  useEffect(() => {
    async function handleWindowKeydown(event) {
      if (!shouldHandleShortcutEvent(event)) {
        return;
      }

      const shortcutMatch = findMenuCommandByShortcut(menus, event);
      if (!shortcutMatch) {
        return;
      }

      event.preventDefault();
      setActiveMenuId(shortcutMatch.menuId);
      await handleCommandExecute(shortcutMatch.commandId);
    }

    window.addEventListener("keydown", handleWindowKeydown);
    return () => {
      window.removeEventListener("keydown", handleWindowKeydown);
    };
  }, [aiBusy, fixtureProjects, locale, menus, panelState, projectSnapshot, selectedFixtureId, selectedAiModel]);

  async function loadFixtureById(nextFixtureId) {
    if (!nextFixtureId) {
      return null;
    }

    setSelectedFixtureId(nextFixtureId);
    setFixtureLoading(true);

    try {
      const [snapshot, runtime] = await Promise.all([
        backend.loadWorkspaceFixture(nextFixtureId),
        backend.fetchAiRuntimeStatus()
      ]);
      setProjectSnapshot(snapshot);
      setCommandResult(null);
      setAiRuntime(runtime);
      setAiMessages([]);
      setAiDraft("");
      return snapshot;
    } finally {
      setFixtureLoading(false);
    }
  }

  async function handleFixtureChange(event) {
    await loadFixtureById(event.target.value);
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

    if (commandId === "project.open") {
      const snapshot = await loadFixtureById(selectedFixtureId);
      if (snapshot) {
        setCommandResult({
          commandId,
          status: "applied",
          message: t(
            "ui.command.project_opened",
            "Projet charge depuis la fixture selectionnee."
          )
        });
      }
      return;
    }

    if (commandId === "project.open_recent") {
      const nextFixtureId = getNextFixtureId(fixtureProjects, selectedFixtureId);
      const snapshot = await loadFixtureById(nextFixtureId);
      if (snapshot) {
        setCommandResult({
          commandId,
          status: "applied",
          message: t(
            "ui.command.recent_opened",
            "Fixture suivante ouverte depuis la liste recente."
          )
        });
      }
      return;
    }

    if (["project.properties", "app.settings", "app.options"].includes(commandId)) {
      setPanelState((previous) => ({
        ...previous,
        properties: true
      }));
      setActiveMenuId("view");
      setCommandResult({
        commandId,
        status: "layout",
        message: t(
          "ui.command.properties_opened",
          "Panneau Proprietes ouvert."
        )
      });
      return;
    }

    if (commandId === "app.exit") {
      const closed = await requestWindowClose();
      setCommandResult({
        commandId,
        status: closed ? "applied" : "notice",
        message: closed
          ? t("ui.command.exit_requested", "Fermeture de l application demandee.")
          : t(
              "ui.command.exit_unavailable",
              "Fermeture indisponible dans cet apercu. Lance le shell Tauri pour fermer la fenetre."
            )
      });
      return;
    }

    setExecutingCommandId(commandId);

    try {
      const response = await backend.executeWorkspaceCommand(commandId, projectSnapshot);
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
      const response = await backend.sendAiChatMessage(
        trimmedMessage,
        locale,
        history,
        selectedAiModel || null,
        projectSnapshot
      );
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
                disabled={fixtureOptions.length === 0 || fixtureLoading}
              >
                {fixtureOptions.length > 0 ? (
                  fixtureOptions.map((fixture) => (
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
                : fixtureLabel(fixtureOptions, selectedFixtureId)}
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
                <div className="tree-section-title">{t("ui.workspace.entities_section", "Entites")}</div>
                <ul className="tree-sublist">
                  {projectSnapshot.entities.length > 0 ? (
                    projectSnapshot.entities.map((entity) => (
                      <li key={entity.id} className="tree-row">
                        <div className="tree-row-main">
                          <span>{entity.name}</span>
                          {entity.detail ? (
                            <div className="tree-detail">{entity.detail}</div>
                          ) : null}
                        </div>
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
              <dt>{t("ui.property.language", "Langue")}</dt>
              <dd>{supportedLocales.find((entry) => entry.id === locale)?.label ?? locale}</dd>
              <dt>{t("ui.property.fixture", "Fixture")}</dt>
              <dd>{fixtureLabel(fixtureOptions, selectedFixtureId)}</dd>
            </dl>

            <div className="property-section">
              <div className="subsection-label">
                {t("ui.property.parametric_parts", "Pieces parametriques")}
              </div>
              {parametricParts.length > 0 ? (
                <div className="property-card-list">
                  {[...parametricParts].reverse().slice(0, 3).map((entity) => (
                    <article
                      key={entity.id}
                      className="result-card property-card"
                      data-parametric-part-summary={entity.id}
                    >
                      <strong>{entity.name}</strong>
                      <div className="command-id">
                        {formatParametricPartSummary(locale, entity.partGeometry)}
                      </div>
                      <div className="muted">
                        {entity.partGeometry.materialName} | {entity.partGeometry.state}
                      </div>
                      <div className="property-inline-metrics">
                        <span>
                          {t("ui.property.area", "Aire")} {formatDecimal(locale, entity.partGeometry.areaMm2)} mm²
                        </span>
                        <span>
                          {t("ui.property.volume", "Volume")} {formatDecimal(locale, entity.partGeometry.volumeMm3)} mm³
                        </span>
                        <span data-parametric-part-mass={entity.id}>
                          {t("ui.property.mass", "Masse")} {formatDecimal(locale, entity.partGeometry.estimatedMassGrams)} g
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              ) : (
                <p className="muted">
                  {t(
                    "ui.property.no_parametric_parts",
                    "Aucune piece parametrique regeneree dans cette session."
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
                      <span className="shortcut">{item.shortcut ?? ""}</span>
                      <button
                        className="run-button"
                        type="button"
                        data-command-id={item.command}
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
            panelId="viewport"
            title={t("ui.panel.viewport", "Viewport 3D")}
            accent={projectSnapshot.details.rootSceneId ?? t("ui.panel.scene_host", "Hote de scene")}
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

        <aside className={rightExpanded ? "workspace-right" : "workspace-right workspace-column-collapsed"}>
          <Panel
            panelId="aiAssistant"
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
                      {t("ui.ai.model_unavailable", "Aucune variante gemma3 detectee localement.")}
                    </option>
                  )}
                </select>
                <div className="muted">
                  {gemma3Models.length > 0
                    ? t(
                        "ui.ai.model_hint",
                        "Le modele selectionne sera utilise au prochain message."
                      )
                    : t("ui.ai.model_unavailable", "Aucune variante gemma3 detectee localement.")}
                </div>
              </label>

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
                    "Pose une question sur le projet courant, la simulation, l integration ou la safety..."
                  )}
                  rows={4}
                />
                <div className="assistant-form-footer">
                  <span className="muted">{t("ui.ai.local_only", "Mode local uniquement")}</span>
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
              <div className="subsection-label">{t("ui.command.last_result", "Dernier resultat")}</div>
              {commandResult ? (
                <div className="result-card">
                  <strong data-last-command-id={commandResult.commandId}>
                    {commandResult.commandId}
                  </strong>
                  <div className="command-id" data-last-command-status={commandResult.status}>
                    {commandResult.status}
                  </div>
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
            panelId="problems"
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
