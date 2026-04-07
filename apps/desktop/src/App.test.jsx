import React from "react";
import { afterEach, describe, test } from "vitest";
import assert from "node:assert/strict";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
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
  projectId = "prj_test_001"
} = {}) {
  return {
    status: {
      runtime: "test-runtime",
      fixtureId,
      projectName,
      entityCount: 1,
      endpointCount: 1,
      streamCount: 1,
      pluginCount: 1
    },
    details: {
      projectId,
      formatVersion: "0.1.0",
      defaultFrame: "world",
      rootSceneId: null,
      activeConfigurationId: "cfg_default"
    },
    entities: [
      {
        id: "ent_001",
        entityType: "Part",
        name: "Bracket-001",
        revision: "rev_a",
        status: "active"
      }
    ],
    endpoints: [
      {
        id: "ext_001",
        name: "Robot Controller",
        endpointType: "robot_controller",
        transportKind: "robot_controller",
        mode: "live",
        address: "robot.local",
        status: "ready"
      }
    ],
    streams: [
      {
        id: "str_001",
        name: "Telemetry",
        endpointId: "ext_001",
        streamType: "mqtt_topic",
        direction: "Inbound",
        status: "ready"
      }
    ],
    plugins: [
      {
        pluginId: "plg.desktop.runtime",
        version: "0.1.0",
        enabled: true,
        status: "installed"
      }
    ],
    recentActivity: [
      {
        id: "cmd_seed_001",
        channel: "command",
        kind: "project.loaded",
        timestamp: "2026-04-06T12:00:00Z",
        targetId: fixtureId
      }
    ]
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
    warning: null
  };
  const fixtures = [
    { id: "pick-and-place-demo.faero", projectName: "Pick And Place Demo" },
    { id: "empty-project.faero", projectName: "Empty Project" }
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
        targetId
      },
      ...snapshot.recentActivity
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
              projectId: "prj_empty_001"
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
          projectId: "prj_session_001"
        });
        snapshot.entities = [];
        snapshot.endpoints = [];
        snapshot.streams = [];
        snapshot.plugins = [];
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
              materialName: "Aluminum 6061"
            }
          }
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
            status: "ready"
          }
        ];
        snapshot.streams = [
          ...snapshot.streams,
          {
            id: `str_wifi_${index.toString().padStart(3, "0")}`,
            name: `Telemetry-${index.toString().padStart(3, "0")}`,
            endpointId: `ext_wifi_${index.toString().padStart(3, "0")}`,
            streamType: "mqtt_topic",
            direction: "Inbound",
            status: "ready"
          }
        ];
        snapshot.status.endpointCount = snapshot.endpoints.length;
        snapshot.status.streamCount = snapshot.streams.length;
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
              status: "installed"
            }
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
          message: `handled ${commandId}`
        }
      };
    },
    async regenerateLatestPart(payload) {
      const index = snapshot.entities.findLastIndex((entity) => entity.partGeometry);
      assert.notEqual(index, -1);

      const areaMm2 = payload.widthMm * payload.heightMm;
      const volumeMm3 = areaMm2 * payload.depthMm;
      const estimatedMassGrams = volumeMm3 * 0.0027;
      const updated = {
        ...snapshot.entities[index],
        detail: `${payload.widthMm.toFixed(1)} x ${payload.heightMm.toFixed(1)} x ${payload.depthMm.toFixed(1)} mm | ${estimatedMassGrams.toFixed(1)} g`,
        partGeometry: {
          ...snapshot.entities[index].partGeometry,
          widthMm: payload.widthMm,
          heightMm: payload.heightMm,
          depthMm: payload.depthMm,
          perimeterMm: 2 * (payload.widthMm + payload.heightMm),
          areaMm2,
          volumeMm3,
          estimatedMassGrams
        }
      };
      snapshot.entities = snapshot.entities.map((entity, entityIndex) =>
        entityIndex === index ? updated : entity
      );
      pushActivity("build.regenerate_part", updated.id);

      return {
        snapshot: clone(snapshot),
        result: {
          commandId: "build.regenerate_part",
          status: "applied",
          message: `regenerated ${updated.detail}`
        }
      };
    },
    async fetchAiRuntimeStatus() {
      return clone(runtime);
    },
    async sendAiChatMessage(message, locale, history, selectedModel, currentSnapshot) {
      lastSelectedModel = selectedModel;
      return {
        answer: `[${locale}] ${message} :: ${selectedModel ?? runtime.activeModel} :: ${currentSnapshot.status.projectName} :: ${history.length}`,
        runtime: {
          ...clone(runtime),
          activeModel: selectedModel ?? runtime.activeModel
        },
        references: [`project:${currentSnapshot.details.projectId}`],
        warnings: [],
        source: "mock-backend"
      };
    },
    getLastSelectedModel() {
      return lastSelectedModel;
    }
  };
}

async function renderApp() {
  const backend = createMockBackend();
  const user = userEvent.setup();
  render(<App backend={backend} />);

  await waitFor(() => {
    assert.equal(document.querySelector(".brand-title")?.textContent, "FutureAero");
    assert.equal(document.querySelector(".context-title")?.textContent, "Fichier");
  });

  return { backend, user };
}

describe("App shell buttons", () => {
  test("top-level menu buttons all switch the active command surface", async () => {
    const { user } = await renderApp();
    const menus = localizeMenuModel("fr");

    for (const menu of menus) {
      const menuButton = screen.getByRole("button", { name: menu.label });
      await user.click(menuButton);
      assert.equal(document.querySelector(".context-title")?.textContent, menu.label);
      assert.ok(menuButton.className.includes("active"));
      assert.equal(
        document.querySelectorAll("[data-command-id]").length,
        menu.items.filter((item) => item.type !== "separator").length
      );
    }
  });

  test("panel toggle buttons collapse and reopen every workspace panel", async () => {
    const { user } = await renderApp();
    const panelExpectations = [
      { panelId: "projectExplorer", text: "Explorateur de projet" },
      { panelId: "properties", text: "Proprietes" },
      { panelId: "commandSurface", text: "Surface de commandes" },
      { panelId: "viewport", text: "Viewport 3D" },
      { panelId: "aiAssistant", text: "Assistant IA local" },
      { panelId: "output", text: "Sortie" },
      { panelId: "problems", text: "Problemes" }
    ];

    for (const entry of panelExpectations) {
      const panel = document.querySelector(`[data-panel-id="${entry.panelId}"]`);
      const toggle = document.querySelector(`[data-panel-toggle="${entry.panelId}"]`);

      assert.ok(panel);
      assert.ok(toggle);
      assert.ok(within(panel).getByText(entry.text));

      await user.click(toggle);
      await waitFor(() => {
        assert.equal(panel.querySelector(".panel-body"), null);
      });

      await user.click(toggle);
      await waitFor(() => {
        assert.notEqual(panel.querySelector(".panel-body"), null);
      });
    }
  });

  test(
    "all command surface run buttons execute at least one visible effect",
    async () => {
      const { user } = await renderApp();
      const menus = localizeMenuModel("fr");
      const panelCommands = new Map([
        ["view.project_explorer", "projectExplorer"],
        ["view.properties", "properties"],
      ["view.output", "output"],
      ["view.problems", "problems"],
      ["view.ai_assistant", "aiAssistant"],
      ["view.viewport_3d", "viewport"]
    ]);

    for (const menu of menus) {
      for (const item of menu.items.filter((entry) => entry.type !== "separator")) {
        await user.click(screen.getByRole("button", { name: menu.label }));
        const runButton = document.querySelector(`[data-command-id="${item.command}"]`);
        assert.ok(runButton, `missing run button for ${item.command}`);

        if (panelCommands.has(item.command)) {
            const panelId = panelCommands.get(item.command);
            const toggle = document.querySelector(`[data-panel-toggle="${panelId}"]`);
            const before = toggle.getAttribute("aria-expanded");
            await user.click(runButton);
            await waitFor(() => {
              assert.notEqual(toggle.getAttribute("aria-expanded"), before);
            });
            await user.click(runButton);
            await waitFor(() => {
              assert.equal(toggle.getAttribute("aria-expanded"), before);
            });
          } else {
            await user.click(runButton);
            await waitFor(() => {
              const lastCommand = document.querySelector("[data-last-command-id]");
              assert.equal(lastCommand?.getAttribute("data-last-command-id"), item.command);
            });
          }
        }
      }
    },
    30000
  );

  test("viewport scene tabs all switch the inspector title", async () => {
    const { user } = await renderApp();

    for (const scene of aerospaceReferenceScenes) {
      const expectedTitle = translate("fr", scene.titleKey, scene.id);
      await user.click(document.querySelector(`[data-scene-id="${scene.id}"]`));
      await waitFor(() => {
        const title = document.querySelector(".viewport-scene-title");
        assert.equal(title?.textContent, expectedTitle);
      });
    }
  });

  test("assistant starter buttons and custom send button both produce a local conversation", async () => {
    for (const starter of [
      "Resume le projet courant",
      "Quels endpoints et flux sont relies a ce projet ?",
      "Quel est le prochain jalon technique concret ?"
    ]) {
      const { user } = await renderApp();
      await user.click(screen.getByRole("button", { name: starter }));

      await waitFor(() => {
        assert.ok(screen.getByText(starter));
        assert.ok(screen.getByText(/mock-backend/));
      });

      cleanup();
    }

    const { user } = await renderApp();
    const textarea = screen.getByPlaceholderText(
      "Pose une question sur le projet courant, la simulation, l integration ou la safety..."
    );
    await user.type(textarea, "Montre moi le projet courant");
    await user.click(document.querySelector('[data-ai-send="true"]'));

    await waitFor(() => {
      assert.ok(screen.getByText("Montre moi le projet courant"));
      assert.ok(screen.getByText(/\[fr\] Montre moi le projet courant/));
    });
  });

  test("file execute buttons expose an immediate visible effect in the command surface", async () => {
    const { user } = await renderApp();

    const fixtureSelector = screen.getByLabelText("Projet de demonstration");
    await user.selectOptions(fixtureSelector, "empty-project.faero");

    await user.click(document.querySelector('[data-command-id="project.open"]'));
    await waitFor(() => {
      assert.equal(document.querySelector("[data-command-feedback]")?.getAttribute("data-command-feedback"), "project.open");
      assert.ok(screen.getAllByText("Empty Project").length >= 1);
    });

    const propertiesToggle = document.querySelector('[data-panel-toggle="properties"]');
    await user.click(propertiesToggle);
    await waitFor(() => {
      assert.equal(propertiesToggle.getAttribute("aria-expanded"), "false");
    });

    await user.click(document.querySelector('[data-command-id="app.settings"]'));
    await waitFor(() => {
      assert.equal(propertiesToggle.getAttribute("aria-expanded"), "true");
      assert.equal(
        document.querySelector("[data-command-feedback]")?.getAttribute("data-command-feedback"),
        "app.settings"
      );
    });
  });

  test("creating a part surfaces parametric geometry metrics in properties", async () => {
    const { user } = await renderApp();

    await user.click(screen.getByRole("button", { name: "Projet" }));
    await user.click(document.querySelector('[data-command-id="entity.create.part"]'));

    await waitFor(() => {
      assert.ok(screen.getByText("Pieces parametriques"));
      assert.ok(document.querySelector('[data-parametric-part-summary="ent_part_002"]'));
      assert.ok(document.querySelector('[data-parametric-part-mass="ent_part_002"]')?.textContent?.includes("367"));
    });
  });

  test("parametric editor regenerates the latest part with new dimensions", async () => {
    const { user } = await renderApp();

    await user.click(screen.getByRole("button", { name: "Projet" }));
    await user.click(document.querySelector('[data-command-id="entity.create.part"]'));

    const widthInput = screen.getByLabelText("Largeur");
    const heightInput = screen.getByLabelText("Hauteur");
    const depthInput = screen.getByLabelText("Profondeur");

    fireEvent.change(widthInput, { target: { value: "200" } });
    fireEvent.change(heightInput, { target: { value: "90" } });
    fireEvent.change(depthInput, { target: { value: "20" } });
    await user.click(document.querySelector('[data-parametric-regenerate="true"]'));

    await waitFor(() => {
      assert.ok(document.querySelector('[data-parametric-part-summary="ent_part_002"]'));
      assert.ok(document.querySelector('[data-parametric-part-mass="ent_part_002"]')?.textContent?.includes("972"));
      assert.equal(
        document.querySelector("[data-command-feedback]")?.getAttribute("data-command-feedback"),
        "build.regenerate_part"
      );
    });
  });

  test("gemma3 selector defaults to 27b and sends the chosen variant to the backend", async () => {
    const { user, backend } = await renderApp();

    const selector = screen.getByLabelText("Modele Gemma3");
    assert.equal(selector.value, "gemma3:27b");

    await user.selectOptions(selector, "gemma3:12b");
    assert.equal(selector.value, "gemma3:12b");

    const textarea = screen.getByPlaceholderText(
      "Pose une question sur le projet courant, la simulation, l integration ou la safety..."
    );
    await user.type(textarea, "Compare les variantes gemma3");
    await user.click(document.querySelector('[data-ai-send="true"]'));

    await waitFor(() => {
      assert.equal(backend.getLastSelectedModel(), "gemma3:12b");
      assert.ok(screen.getByText(/\[fr\] Compare les variantes gemma3 :: gemma3:12b/));
    });
  });

  test("keyboard shortcuts execute visible shell commands", async () => {
    const { user } = await renderApp();

    await user.keyboard("{Control>}{Shift>}N{/Shift}{/Control}");
    await waitFor(() => {
      assert.equal(
        document.querySelector("[data-command-feedback]")?.getAttribute("data-command-feedback"),
        "project.create"
      );
      assert.ok(screen.getAllByText("FutureAero Session").length >= 1);
    });

    const propertiesToggle = document.querySelector('[data-panel-toggle="properties"]');
    assert.equal(propertiesToggle?.getAttribute("aria-expanded"), "true");

    await user.keyboard("{F4}");
    await waitFor(() => {
      assert.equal(propertiesToggle?.getAttribute("aria-expanded"), "false");
      assert.equal(
        document.querySelector("[data-command-feedback]")?.getAttribute("data-command-feedback"),
        "view.properties"
      );
    });

    await user.keyboard("{Alt>}{Enter}{/Alt}");
    await waitFor(() => {
      assert.equal(propertiesToggle?.getAttribute("aria-expanded"), "true");
      assert.equal(document.querySelector(".context-title")?.textContent, "Affichage");
      assert.equal(
        document.querySelector("[data-command-feedback]")?.getAttribute("data-command-feedback"),
        "project.properties"
      );
    });
  });
});
