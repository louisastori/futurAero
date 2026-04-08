import React from "react";
import { afterEach } from "vitest";
import assert from "node:assert/strict";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import App from "./App.jsx";
import { localizeMenuModel, translate } from "@futureaero/ui";
import { aerospaceReferenceScenes } from "@futureaero/viewport";

afterEach(() => {
  cleanup();
});

function createSnapshot({
  fixtureId = "pick-and-place-demo.faero",
  projectName = "Pick And Place Demo",
  projectId = "prj_test_001",
  openSpecDocuments = [
    {
      id: "ops_001",
      title: "Readable Layout Intent",
      kind: "design_intent",
      status: "active",
      linkedEntityCount: 1,
      linkedExternalCount: 1,
      tagCount: 2,
      excerpt: "Cellule lisible en clair sans binaire vendor.",
    },
  ],
} = {}) {
  return {
    status: {
      runtime: "test-runtime",
      fixtureId,
      projectName,
      entityCount: 1,
      endpointCount: 1,
      streamCount: 1,
      pluginCount: 1,
    },
    details: {
      projectId,
      formatVersion: "0.1.0",
      defaultFrame: "world",
      rootSceneId: null,
      activeConfigurationId: "cfg_default",
    },
    entities: [
      {
        id: "ent_001",
        entityType: "Part",
        name: "Bracket-001",
        revision: "rev_a",
        status: "active",
        detail: "120.0 x 80.0 x 10.0 mm | 259.2 g",
        data: {
          tags: ["seed", "parametric"],
          parameterSet: {
            widthMm: 120,
            heightMm: 80,
            depthMm: 10,
            quality: {
              toleranceMm: 0.4,
            },
            checkpoints: [120, 80, 10],
          },
        },
        partGeometry: {
          state: "well_constrained",
          widthMm: 120,
          heightMm: 80,
          depthMm: 10,
          pointCount: 4,
          perimeterMm: 400,
          areaMm2: 9600,
          volumeMm3: 96000,
          estimatedMassGrams: 259.2,
          materialName: "Aluminum 6061",
        },
      },
    ],
    endpoints: [
      {
        id: "ext_001",
        name: "Robot Controller",
        endpointType: "robot_controller",
        transportKind: "robot_controller",
        mode: "live",
        address: "robot.local",
        status: "ready",
      },
    ],
    streams: [
      {
        id: "str_001",
        name: "Telemetry",
        endpointId: "ext_001",
        streamType: "mqtt_topic",
        direction: "Inbound",
        status: "ready",
      },
    ],
    plugins: [
      {
        pluginId: "plg.desktop.runtime",
        version: "0.1.0",
        enabled: true,
        status: "installed",
      },
    ],
    openSpecDocuments,
    recentActivity: [
      {
        id: "cmd_seed_001",
        channel: "command",
        kind: "project.loaded",
        timestamp: "2026-04-06T12:00:00Z",
        targetId: fixtureId,
      },
    ],
  };
}

function createMockBackend() {
  const runtime = {
    available: true,
    provider: "ollama",
    endpoint: "http://127.0.0.1:11434",
    mode: "test",
    localOnly: true,
    activeProfile: "balanced",
    availableProfiles: ["balanced", "max", "furnace"],
    activeModel: "gemma3:27b",
    availableModels: ["gemma3:27b", "gemma3:12b", "gemma3:4b", "phi3:mini"],
    gemma3Models: ["gemma3:27b", "gemma3:12b", "gemma3:4b"],
    warning: null,
  };
  const fixtures = [
    { id: "pick-and-place-demo.faero", projectName: "Pick And Place Demo" },
    { id: "empty-project.faero", projectName: "Empty Project" },
  ];
  let snapshot = createSnapshot();
  let activityCounter = 2;
  let lastSelectedModel = null;
  let lastSelectedProfile = null;
  let suggestionCounter = 1;

  function clone(value) {
    return structuredClone(value);
  }

  function pushActivity(kind, targetId) {
    snapshot.recentActivity = [
      {
        id: `act_${activityCounter++}`,
        channel: "system",
        kind,
        timestamp: `2026-04-06T12:00:${String(activityCounter).padStart(2, "0")}Z`,
        targetId,
      },
      ...snapshot.recentActivity,
    ].slice(0, 12);
  }

  function setNestedValue(target, path, value) {
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

  function buildTimelineArtifacts() {
    return {
      timelineSamples: [
        { stepIndex: 0, timestampMs: 0, trackingErrorMm: 0.11, speedScale: 0.82 },
        { stepIndex: 1, timestampMs: 320, trackingErrorMm: 0.14, speedScale: 0.86 },
        { stepIndex: 2, timestampMs: 710, trackingErrorMm: 0.19, speedScale: 0.93 },
      ],
      signalSamples: [
        {
          stepIndex: 0,
          timestampMs: 0,
          signalId: "sig_cycle_start",
          value: false,
          reason: "initial_value",
        },
        {
          stepIndex: 1,
          timestampMs: 320,
          signalId: "sig_cycle_start",
          value: true,
          reason: "simulation.run.start",
        },
      ],
      controllerStateSamples: [
        {
          stepIndex: 0,
          timestampMs: 0,
          stateId: "idle",
          stateName: "Idle",
          reason: "initial_state",
        },
        {
          stepIndex: 2,
          timestampMs: 710,
          stateId: "transfer",
          stateName: "Transfer",
          reason: "pick completed",
        },
      ],
      contacts: [],
    };
  }

  function applyEntityChanges(payload) {
    const entityIndex = snapshot.entities.findIndex(
      (entity) => entity.id === payload.entityId,
    );
    assert.notEqual(entityIndex, -1);

    let next = clone(snapshot.entities[entityIndex]);
    const changes = payload.changes ?? {};
    if (typeof changes.name === "string" && changes.name.trim().length > 0) {
      next.name = changes.name.trim();
    }
    if (!next.data || typeof next.data !== "object") {
      next.data = {};
    }
    if (Array.isArray(changes.tags)) {
      next.data.tags = changes.tags;
    }
    for (const [path, value] of Object.entries(changes)) {
      if (path === "name" || path === "tags") {
        continue;
      }
      setNestedValue(next.data, path, value);
    }

    if (next.partGeometry) {
      const widthMm = Number(
        valueAtPath(next.data, "parameterSet.widthMm") ?? next.partGeometry.widthMm,
      );
      const heightMm = Number(
        valueAtPath(next.data, "parameterSet.heightMm") ?? next.partGeometry.heightMm,
      );
      const depthMm = Number(
        valueAtPath(next.data, "parameterSet.depthMm") ?? next.partGeometry.depthMm,
      );
      const areaMm2 = widthMm * heightMm;
      const volumeMm3 = areaMm2 * depthMm;
      const estimatedMassGrams = volumeMm3 * 0.0027;
      next = {
        ...next,
        detail: `${widthMm.toFixed(1)} x ${heightMm.toFixed(1)} x ${depthMm.toFixed(1)} mm | ${estimatedMassGrams.toFixed(1)} g`,
        partGeometry: {
          ...next.partGeometry,
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

    if (next.entityType === "Signal") {
      next = {
        ...next,
        detail: `${next.data.signalId ?? "signal"} | ${String(next.data.currentValue ?? false)}`,
      };
    }

    next.revision = `rev_${String(activityCounter).padStart(4, "0")}`;

    snapshot.entities = snapshot.entities.map((entity, index) =>
      index === entityIndex ? next : entity,
    );
    pushActivity("entity.properties.updated", payload.entityId);

    return {
      snapshot: clone(snapshot),
      result: {
        commandId: "entity.properties.update",
        status: "applied",
        message: `updated ${payload.entityId}`,
      },
    };
  }

  return {
    async fetchWorkspaceBootstrap() {
      return { fixtures: clone(fixtures), snapshot: clone(snapshot) };
    },
    async loadWorkspaceFixture(projectId) {
      snapshot =
        projectId === "empty-project.faero"
          ? createSnapshot({
              fixtureId: "empty-project.faero",
              projectName: "Empty Project",
              projectId: "prj_empty_001",
              openSpecDocuments: [],
            })
          : createSnapshot();
      pushActivity("workspace.loaded", projectId);
      return clone(snapshot);
    },
    async executeWorkspaceCommand(commandId) {
      if (commandId === "project.create") {
        snapshot = createSnapshot({
          fixtureId: "session:untitled",
          projectName: "FutureAero Session",
          projectId: "prj_session_001",
        });
        snapshot.entities = [];
        snapshot.endpoints = [];
        snapshot.streams = [];
        snapshot.plugins = [];
        snapshot.openSpecDocuments = [];
        snapshot.status.entityCount = 0;
        snapshot.status.endpointCount = 0;
        snapshot.status.streamCount = 0;
        snapshot.status.pluginCount = 0;
      } else if (commandId === "entity.create.part") {
        const index = snapshot.entities.length + 1;
        snapshot.entities = [
          ...snapshot.entities,
          {
            id: `ent_part_${index.toString().padStart(3, "0")}`,
            entityType: "Part",
            name: `Part-${index.toString().padStart(3, "0")}`,
            revision: "rev_seed",
            status: "active",
            detail: "132.0 x 86.0 x 12.0 mm | 367.9 g",
            data: {
              tags: ["part", "parametric"],
              parameterSet: {
                widthMm: 132.0,
                heightMm: 86.0,
                depthMm: 12.0,
                quality: {
                  toleranceMm: 0.4,
                },
                checkpoints: [132, 86, 12],
              },
            },
            partGeometry: {
              state: "well_constrained",
              widthMm: 132.0,
              heightMm: 86.0,
              depthMm: 12.0,
              pointCount: 4,
              perimeterMm: 436.0,
              areaMm2: 11352.0,
              volumeMm3: 136224.0,
              estimatedMassGrams: 367.9,
              materialName: "Aluminum 6061",
            },
          },
        ];
        snapshot.status.entityCount = snapshot.entities.length;
      } else if (commandId === "entity.create.external_endpoint") {
        const index = snapshot.endpoints.length + 1;
        snapshot.endpoints = [
          ...snapshot.endpoints,
          {
            id: `ext_wifi_${index.toString().padStart(3, "0")}`,
            name: `Wireless Edge ${index.toString().padStart(3, "0")}`,
            endpointType: "wireless_edge",
            transportKind: "wifi",
            mode: "live",
            address: `wireless-edge-${index.toString().padStart(3, "0")}.local`,
            status: "ready",
          },
        ];
        snapshot.streams = [
          ...snapshot.streams,
          {
            id: `str_wifi_${index.toString().padStart(3, "0")}`,
            name: `Telemetry-${index.toString().padStart(3, "0")}`,
            endpointId: `ext_wifi_${index.toString().padStart(3, "0")}`,
            streamType: "mqtt_topic",
            direction: "Inbound",
            status: "ready",
          },
        ];
        snapshot.status.endpointCount = snapshot.endpoints.length;
        snapshot.status.streamCount = snapshot.streams.length;
      } else if (commandId === "entity.create.assembly") {
        const index = snapshot.entities.length + 1;
        snapshot.entities = [
          ...snapshot.entities,
          {
            id: `ent_asm_${index.toString().padStart(3, "0")}`,
            entityType: "Assembly",
            name: `Assembly-${index.toString().padStart(3, "0")}`,
            revision: "rev_seed",
            status: "active",
            detail: "solved | 2 occ | 1 mates | 0 ddl",
            data: {
              tags: ["assembly"],
              parameterSet: {
                occurrenceCount: 2,
                mateCount: 1,
              },
              occurrences: [
                {
                  id: "occ_001",
                  definitionEntityId: "ent_part_001",
                  transform: { xMm: 0, yMm: 0, zMm: 0, yawDeg: 0 },
                },
                {
                  id: "occ_002",
                  definitionEntityId: "ent_part_002",
                  transform: { xMm: 80, yMm: 0, zMm: 0, yawDeg: 0 },
                },
              ],
              mateConstraints: [
                {
                  id: "mate_001",
                  leftOccurrenceId: "occ_001",
                  rightOccurrenceId: "occ_002",
                  type: "coincident",
                },
              ],
              solveReport: {
                status: "solved",
                constrainedOccurrenceCount: 2,
                totalMateCount: 1,
                degreesOfFreedomEstimate: 0,
                solvedOccurrences: [
                  {
                    occurrenceId: "occ_001",
                    transform: { xMm: 0, yMm: 0, zMm: 0, yawDeg: 0 },
                  },
                  {
                    occurrenceId: "occ_002",
                    transform: { xMm: 0, yMm: 0, zMm: 0, yawDeg: 0 },
                  },
                ],
                warnings: [],
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
        ];
        snapshot.status.entityCount = snapshot.entities.length;
      } else if (commandId === "entity.create.robot_cell") {
        const index = snapshot.entities.length + 1;
        snapshot.entities = [
          ...snapshot.entities,
          {
            id: `ent_cell_${index.toString().padStart(3, "0")}`,
            entityType: "RobotCell",
            name: `RobotCell-${index.toString().padStart(3, "0")}`,
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
          {
            id: `ent_sig_${(index + 1).toString().padStart(3, "0")}`,
            entityType: "Signal",
            name: "Progress Gate",
            revision: "rev_seed",
            status: "active",
            detail: "sig_progress_gate | 0.62",
            data: {
              signalId: "sig_progress_gate",
              kind: "scalar",
              currentValue: 0.62,
              tags: ["control", "simulation"],
              parameterSet: {
                unit: "ratio",
                checkpoints: [0.25, 0.62, 1.0],
              },
            },
          },
        ];
        snapshot.status.entityCount = snapshot.entities.length;
      } else if (commandId === "simulation.run.start") {
        const hasRobotCell = snapshot.entities.some(
          (entity) => entity.robotCellSummary,
        );
        if (!hasRobotCell) {
          snapshot.entities = [
            ...snapshot.entities,
            {
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
            },
          ];
        }
        const index = snapshot.entities.length + 1;
        const artifacts = buildTimelineArtifacts();
        snapshot.entities = [
          ...snapshot.entities,
          {
            id: `ent_run_${index.toString().padStart(3, "0")}`,
            entityType: "SimulationRun",
            name: `SimulationRun-${index.toString().padStart(3, "0")}`,
            revision: "rev_seed",
            status: "active",
            detail: "completed | 3497 ms | 0 coll | 0 contact",
            data: {
              tags: ["simulation", "artifact", "mvp"],
              parameterSet: {
                seed: 308,
                stepCount: 12,
              },
              ...artifacts,
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
        ];
        snapshot.status.entityCount = snapshot.entities.length;
      } else if (commandId === "analyze.safety") {
        const hasRobotCell = snapshot.entities.some(
          (entity) => entity.robotCellSummary,
        );
        if (!hasRobotCell) {
          snapshot.entities = [
            ...snapshot.entities,
            {
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
            },
          ];
        }
        const index = snapshot.entities.length + 1;
        snapshot.entities = [
          ...snapshot.entities,
          {
            id: `ent_safe_${index.toString().padStart(3, "0")}`,
            entityType: "SafetyReport",
            name: `SafetyReport-${index.toString().padStart(3, "0")}`,
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
        ];
        snapshot.status.entityCount = snapshot.entities.length;
      } else if (commandId === "plugin.manage") {
        const existing = snapshot.plugins[0];
        if (existing) {
          existing.enabled = !existing.enabled;
        } else {
          snapshot.plugins = [
            {
              pluginId: "plg.desktop.runtime",
              version: "0.1.0",
              enabled: true,
              status: "installed",
            },
          ];
          snapshot.status.pluginCount = snapshot.plugins.length;
        }
      }

      pushActivity(commandId, snapshot.status.fixtureId);

      return {
        snapshot: clone(snapshot),
        result: {
          commandId,
          status: commandId.startsWith("view.") ? "layout" : "applied",
          message: `handled ${commandId}`,
        },
      };
    },
    async regenerateLatestPart(payload) {
      const index = snapshot.entities.findLastIndex(
        (entity) => entity.partGeometry,
      );
      assert.notEqual(index, -1);

      const areaMm2 = payload.widthMm * payload.heightMm;
      const volumeMm3 = areaMm2 * payload.depthMm;
      const estimatedMassGrams = volumeMm3 * 0.0027;
      const updated = {
        ...snapshot.entities[index],
        detail: `${payload.widthMm.toFixed(1)} x ${payload.heightMm.toFixed(1)} x ${payload.depthMm.toFixed(1)} mm | ${estimatedMassGrams.toFixed(1)} g`,
        data: {
          ...(snapshot.entities[index].data ?? {}),
          parameterSet: {
            widthMm: payload.widthMm,
            heightMm: payload.heightMm,
            depthMm: payload.depthMm,
          },
        },
        partGeometry: {
          ...snapshot.entities[index].partGeometry,
          widthMm: payload.widthMm,
          heightMm: payload.heightMm,
          depthMm: payload.depthMm,
          perimeterMm: 2 * (payload.widthMm + payload.heightMm),
          areaMm2,
          volumeMm3,
          estimatedMassGrams,
        },
      };
      snapshot.entities = snapshot.entities.map((entity, entityIndex) =>
        entityIndex === index ? updated : entity,
      );
      pushActivity("build.regenerate_part", updated.id);

      return {
        snapshot: clone(snapshot),
        result: {
          commandId: "build.regenerate_part",
          status: "applied",
          message: `regenerated ${updated.detail}`,
        },
      };
    },
    async updateEntityProperties(payload) {
      return applyEntityChanges(payload);
    },
    async fetchAiRuntimeStatus(selectedProfile = null) {
      return {
        ...clone(runtime),
        activeProfile: selectedProfile ?? runtime.activeProfile,
      };
    },
    async sendAiChatMessage(
      message,
      locale,
      history,
      selectedModel,
      selectedProfile,
      currentSnapshot,
    ) {
      lastSelectedModel = selectedModel;
      lastSelectedProfile = selectedProfile;
      const suggestionId = `ent_ai_suggestion_${String(suggestionCounter++).padStart(3, "0")}`;
      return {
        answer: `[${locale}] ${message} :: ${selectedProfile ?? runtime.activeProfile} :: ${selectedModel ?? runtime.activeModel} :: ${currentSnapshot.status.projectName} :: ${history.length}`,
        runtime: {
          ...clone(runtime),
          activeProfile: selectedProfile ?? runtime.activeProfile,
          activeModel: selectedModel ?? runtime.activeModel,
        },
        references: [`project:${currentSnapshot.details.projectId}`],
        structured: {
          summary: `Analyse structuree pour ${currentSnapshot.status.projectName}`,
          runtimeProfile: selectedProfile ?? runtime.activeProfile,
          contextRefs: [
            {
              entityId: currentSnapshot.entities[0]?.id ?? null,
              role: "source",
              path: "simulationRunSummary.collisionCount",
            },
          ],
          confidence: 0.82,
          riskLevel: "medium",
          limitations: ["Mock backend structure la reponse localement."],
          critiquePasses: [
            {
              stage: "critic",
              summary: "Le critic mock ne releve pas de contradiction majeure.",
              confidenceDelta: -0.03,
              issues: ["validation manuelle recommandee"],
              adjustments: ["review before apply"],
            },
          ],
          proposedCommands: [
            {
              kind: "entity.properties.update",
              targetId: currentSnapshot.entities[0]?.id ?? null,
              payload: {
                changes: {
                  "parameterSet.widthMm": 140,
                  "parameterSet.quality.toleranceMm": 0.35,
                  "parameterSet.checkpoints": [140, 90, 20],
                },
              },
            },
            {
              kind: "simulation.run.start",
              targetId: null,
              payload: {},
            },
          ],
          explanation: [`Historique recu: ${history.length}`],
        },
        suggestionId,
        warnings: [],
        source: "mock-backend",
      };
    },
    async applyAiSuggestion(suggestionId) {
      const command = {
        entityId: "ent_001",
        changes: {
          "parameterSet.widthMm": 140,
          "parameterSet.quality.toleranceMm": 0.35,
          "parameterSet.checkpoints": [140, 90, 20],
        },
      };
      applyEntityChanges(command);
      pushActivity("ai.suggestion.applied", suggestionId);
      return {
        snapshot: clone(snapshot),
        result: {
          commandId: "ai.suggestion.apply",
          status: "applied",
          message: `applied ${suggestionId}`,
        },
      };
    },
    async rejectAiSuggestion(suggestionId) {
      pushActivity("ai.suggestion.rejected", suggestionId);
      return {
        snapshot: clone(snapshot),
        result: {
          commandId: "ai.suggestion.reject",
          status: "applied",
          message: `rejected ${suggestionId}`,
        },
      };
    },
    getLastSelectedModel() {
      return lastSelectedModel;
    },
    getLastSelectedProfile() {
      return lastSelectedProfile;
    },
  };
}

async function renderApp() {
  const backend = createMockBackend();
  const user = userEvent.setup();
  render(<App backend={backend} />);

  await waitFor(() => {
    assert.equal(
      document.querySelector(".brand-title")?.textContent,
      "FutureAero",
    );
    assert.equal(
      document.querySelector(".context-title")?.textContent,
      "Fichier",
    );
  });

  return { backend, user };
}

export {
  aerospaceReferenceScenes,
  assert,
  cleanup,
  fireEvent,
  localizeMenuModel,
  renderApp,
  screen,
  translate,
  waitFor,
  within,
};
