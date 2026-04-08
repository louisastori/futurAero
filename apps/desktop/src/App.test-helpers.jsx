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
      const entityIndex = snapshot.entities.findIndex(
        (entity) => entity.id === payload.entityId,
      );
      assert.notEqual(entityIndex, -1);

      const current = snapshot.entities[entityIndex];
      let next = {
        ...current,
        name: payload.name,
        data: {
          ...(current.data ?? {}),
          tags: payload.tags,
          parameterSet: payload.parameters,
        },
      };

      if (next.partGeometry) {
        const widthMm = Number(payload.parameters.widthMm);
        const heightMm = Number(payload.parameters.heightMm);
        const depthMm = Number(payload.parameters.depthMm);
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
    },
    async fetchAiRuntimeStatus() {
      return clone(runtime);
    },
    async sendAiChatMessage(
      message,
      locale,
      history,
      selectedModel,
      currentSnapshot,
    ) {
      lastSelectedModel = selectedModel;
      return {
        answer: `[${locale}] ${message} :: ${selectedModel ?? runtime.activeModel} :: ${currentSnapshot.status.projectName} :: ${history.length}`,
        runtime: {
          ...clone(runtime),
          activeModel: selectedModel ?? runtime.activeModel,
        },
        references: [`project:${currentSnapshot.details.projectId}`],
        structured: {
          summary: `Analyse structuree pour ${currentSnapshot.status.projectName}`,
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
          proposedCommands: [],
          explanation: [`Historique recu: ${history.length}`],
        },
        warnings: [],
        source: "mock-backend",
      };
    },
    getLastSelectedModel() {
      return lastSelectedModel;
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
