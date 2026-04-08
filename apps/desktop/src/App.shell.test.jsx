import { describe, test } from "vitest";

import {
  aerospaceReferenceScenes,
  assert,
  cleanup,
  localizeMenuModel,
  renderApp,
  screen,
  translate,
  waitFor,
  within,
} from "./App.test-helpers.jsx";

describe("App shell chrome", () => {
  test("top-level menu buttons all switch the active command surface", async () => {
    const { user } = await renderApp();
    const menus = localizeMenuModel("fr");

    for (const menu of menus) {
      const menuButton = screen.getByRole("button", { name: menu.label });
      await user.click(menuButton);
      assert.equal(
        document.querySelector(".context-title")?.textContent,
        menu.label,
      );
      assert.ok(menuButton.className.includes("active"));
      assert.equal(
        document.querySelectorAll("[data-command-id]").length,
        menu.items.filter((item) => item.type !== "separator").length,
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
      { panelId: "simulationTimeline", text: "Timeline de simulation" },
      { panelId: "aiAssistant", text: "Assistant IA local" },
      { panelId: "output", text: "Sortie" },
      { panelId: "problems", text: "Problemes" },
    ];

    for (const entry of panelExpectations) {
      const panel = document.querySelector(
        `[data-panel-id="${entry.panelId}"]`,
      );
      const toggle = document.querySelector(
        `[data-panel-toggle="${entry.panelId}"]`,
      );

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

  test("all command surface run buttons execute at least one visible effect", async () => {
    const { user } = await renderApp();
    const menus = localizeMenuModel("fr");
    const panelCommands = new Map([
      ["view.project_explorer", "projectExplorer"],
      ["view.properties", "properties"],
      ["view.output", "output"],
      ["view.problems", "problems"],
      ["view.ai_assistant", "aiAssistant"],
      ["view.viewport_3d", "viewport"],
      ["view.simulation_timeline", "simulationTimeline"],
    ]);

    for (const menu of menus) {
      for (const item of menu.items.filter(
        (entry) => entry.type !== "separator",
      )) {
        await user.click(screen.getByRole("button", { name: menu.label }));
        const runButton = document.querySelector(
          `[data-command-id="${item.command}"]`,
        );
        assert.ok(runButton, `missing run button for ${item.command}`);

        if (panelCommands.has(item.command)) {
          const panelId = panelCommands.get(item.command);
          const toggle = document.querySelector(
            `[data-panel-toggle="${panelId}"]`,
          );
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
            const lastCommand = document.querySelector(
              "[data-last-command-id]",
            );
            assert.equal(
              lastCommand?.getAttribute("data-last-command-id"),
              item.command,
            );
          });
        }
      }
    }
  }, 30000);

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

  test("file execute buttons expose an immediate visible effect in the command surface", async () => {
    const { user } = await renderApp();

    const fixtureSelector = screen.getByLabelText("Projet de demonstration");
    await user.selectOptions(fixtureSelector, "empty-project.faero");

    await user.click(
      document.querySelector('[data-command-id="project.open"]'),
    );
    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "project.open",
      );
      assert.ok(screen.getAllByText("Empty Project").length >= 1);
    });

    const propertiesToggle = document.querySelector(
      '[data-panel-toggle="properties"]',
    );
    await user.click(propertiesToggle);
    await waitFor(() => {
      assert.equal(propertiesToggle.getAttribute("aria-expanded"), "false");
    });

    await user.click(
      document.querySelector('[data-command-id="app.settings"]'),
    );
    await waitFor(() => {
      assert.equal(propertiesToggle.getAttribute("aria-expanded"), "true");
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "app.settings",
      );
    });
  });

  test("assistant starter buttons and custom send button both produce a local conversation", async () => {
    for (const starter of [
      "Resume le projet courant",
      "Quels endpoints et flux sont relies a ce projet ?",
      "Quel est le prochain jalon technique concret ?",
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
      "Pose une question sur le projet courant, la simulation, l integration ou la safety...",
    );
    await user.type(textarea, "Montre moi le projet courant");
    await user.click(document.querySelector('[data-ai-send="true"]'));

    await waitFor(() => {
      assert.ok(screen.getByText("Montre moi le projet courant"));
      assert.ok(screen.getByText(/\[fr\] Montre moi le projet courant/));
      assert.ok(document.querySelector('[data-ai-structured="true"]'));
    });
  });
});
