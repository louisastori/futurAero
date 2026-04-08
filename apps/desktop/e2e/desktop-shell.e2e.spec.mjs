import { expect, test } from "@playwright/test";

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

function installMockBackend(page) {
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
    warning: null
  };

  const fixtures = [
    { id: "pick-and-place-demo.faero", projectName: "Pick And Place Demo" },
    { id: "empty-project.faero", projectName: "Empty Project" }
  ];

  return page.addInitScript(
    ({ runtime, fixtures }) => {
      let snapshot = {
        ...structuredClone({
          status: {
            runtime: "test-runtime",
            fixtureId: "pick-and-place-demo.faero",
            projectName: "Pick And Place Demo",
            entityCount: 1,
            endpointCount: 1,
            streamCount: 1,
            pluginCount: 1
          },
          details: {
            projectId: "prj_test_001",
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
              targetId: "pick-and-place-demo.faero"
            }
          ]
        })
      };
      let selectedModel = null;

      function clone(value) {
        return structuredClone(value);
      }

      function nextSnapshot(projectId) {
        if (projectId === "empty-project.faero") {
          return {
            ...clone(snapshot),
            status: {
              ...snapshot.status,
              fixtureId: "empty-project.faero",
              projectName: "Empty Project"
            },
            details: {
              ...snapshot.details,
              projectId: "prj_empty_001"
            },
            recentActivity: [
              {
                id: "act_001",
                channel: "system",
                kind: "workspace.loaded",
                timestamp: "2026-04-06T12:00:01Z",
                targetId: "empty-project.faero"
              }
            ]
          };
        }

        return {
          ...clone(snapshot),
          status: {
            ...snapshot.status,
            fixtureId: "pick-and-place-demo.faero",
            projectName: "Pick And Place Demo"
          },
          details: {
            ...snapshot.details,
            projectId: "prj_test_001"
          },
          recentActivity: [
            {
              id: "act_001",
              channel: "system",
              kind: "workspace.loaded",
              timestamp: "2026-04-06T12:00:01Z",
              targetId: "pick-and-place-demo.faero"
            }
          ]
        };
      }

      globalThis.__FUTUREAERO_BACKEND__ = {
        async fetchWorkspaceBootstrap() {
          return { fixtures: clone(fixtures), snapshot: clone(snapshot) };
        },
        async loadWorkspaceFixture(projectId) {
          snapshot = nextSnapshot(projectId);
          return clone(snapshot);
        },
        async executeWorkspaceCommand(commandId) {
          if (commandId === "project.create") {
            snapshot = {
              ...clone(snapshot),
              status: {
                ...snapshot.status,
                fixtureId: "session:untitled",
                projectName: "FutureAero Session",
                entityCount: 0,
                endpointCount: 0,
                streamCount: 0,
                pluginCount: 0
              },
              details: {
                ...snapshot.details,
                projectId: "prj_session_001"
              },
              entities: [],
              endpoints: [],
              streams: [],
              plugins: [],
              recentActivity: [
                {
                  id: "act_002",
                  channel: "system",
                  kind: "workspace.created",
                  timestamp: "2026-04-06T12:00:02Z",
                  targetId: "FutureAero Session"
                }
              ]
            };
          }

          return {
            snapshot: clone(snapshot),
            result: {
              commandId,
              status: "applied",
              message: `handled ${commandId}`
            }
          };
        },
        async fetchAiRuntimeStatus() {
          return clone(runtime);
        },
        async sendAiChatMessage(
          message,
          locale,
          history,
          model,
          selectedProfile,
          currentSnapshot
        ) {
          selectedModel = model;
          return {
            answer: `[${locale}] ${message} :: ${selectedProfile ?? runtime.activeProfile} :: ${model ?? runtime.activeModel} :: ${currentSnapshot.status.projectName} :: ${history.length}`,
            runtime: {
              ...clone(runtime),
              activeProfile: selectedProfile ?? runtime.activeProfile,
              activeModel: model ?? runtime.activeModel
            },
            references: [`project:${currentSnapshot.details.projectId}`],
            warnings: [],
            source: "playwright-mock"
          };
        },
        __getSelectedModel() {
          return selectedModel;
        }
      };
    },
    { runtime, fixtures }
  );
}

test.beforeEach(async ({ page }) => {
  await installMockBackend(page);
  await page.goto("/", { waitUntil: "domcontentloaded", timeout: 60_000 });
  await expect(page.getByText("FutureAero")).toBeVisible({ timeout: 60_000 });
});

test("execute buttons update visible command feedback in the shell", async ({ page }) => {
  await page.getByLabel("Projet de demonstration").selectOption("empty-project.faero");
  await page.locator("[data-command-id='project.open']").click();

  await expect(page.locator("[data-command-feedback='project.open']")).toBeVisible();
  await expect(page.locator(".status-pill").filter({ hasText: "Empty Project" }).first()).toBeVisible();

  await page.locator("[data-command-id='project.create']").click();

  await expect(page.locator("[data-command-feedback='project.create']")).toBeVisible();
  await expect(page.locator(".status-pill").filter({ hasText: "FutureAero Session" }).first()).toBeVisible();
});

test("settings command reopens properties and shows an immediate effect", async ({ page }) => {
  await page.locator("[data-panel-toggle='properties']").click();
  await expect(page.locator("[data-panel-id='properties'] .panel-body")).toHaveCount(0);

  await page.locator("[data-command-id='app.settings']").click();

  await expect(page.locator("[data-command-feedback='app.settings']")).toBeVisible();
  await expect(page.locator("[data-panel-id='properties'] .panel-body")).toHaveCount(1);
});

test("gemma3 selector drives the model used by the local chat flow", async ({ page }) => {
  await page.getByLabel("Modele Gemma3").selectOption("gemma3:12b");
  await page.getByPlaceholder(
    "Pose une question sur le projet courant, la simulation, l integration ou la safety..."
  ).fill("Compare les variantes gemma3");
  await page.locator("[data-ai-send='true']").click();

  await expect(
    page.getByText(/\[fr\] Compare les variantes gemma3 :: balanced :: gemma3:12b/)
  ).toBeVisible();
});
