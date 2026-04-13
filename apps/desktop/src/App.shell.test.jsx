import { describe, test } from "vitest";

import {
  assert,
  cleanup,
  fireEvent,
  localizeMenuModel,
  renderApp,
  screen,
  waitFor,
  within,
} from "./App.test-helpers.jsx";

describe("App shell chrome", () => {
  test("dropdown command entries switch the active command overlay", async () => {
    const { user } = await renderApp();
    const menus = localizeMenuModel("fr");
    const menuPicker = screen.getByLabelText("Menu");

    for (const menu of menus) {
      await user.selectOptions(menuPicker, `commands:${menu.id}`);
      await waitFor(() => {
        assert.equal(
          document.querySelector(".context-title")?.textContent,
          menu.label,
        );
        assert.equal(
          document
            .querySelector('[data-overlay-kind="commands"]')
            ?.getAttribute("aria-hidden"),
          "false",
        );
        assert.equal(
          document.querySelectorAll("[data-command-id]").length,
          menu.items.filter((item) => item.type !== "separator").length,
        );
      });
    }
  });

  test("panel toggle buttons collapse and reopen the docked right panels", async () => {
    const { user } = await renderApp();
    const panelExpectations = [
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
      ["view.project_explorer", "explorer"],
      ["view.properties", "properties"],
      ["view.output", "output"],
      ["view.problems", "problems"],
      ["view.ai_assistant", "aiAssistant"],
      ["view.viewport_3d", "whitebox"],
      ["view.simulation_timeline", "simulationTimeline"],
    ]);
    const menuPicker = screen.getByLabelText("Menu");

    for (const menu of menus) {
      for (const item of menu.items.filter(
        (entry) => entry.type !== "separator",
      )) {
        await user.selectOptions(menuPicker, `commands:${menu.id}`);
        await waitFor(() => {
          assert.equal(
            document.querySelector(".context-title")?.textContent,
            menu.label,
          );
        });
        const runButton = document.querySelector(
          `[data-command-id="${item.command}"]`,
        );
        assert.ok(runButton, `missing run button for ${item.command}`);

        if (panelCommands.has(item.command)) {
          const target = panelCommands.get(item.command);
          if (target === "explorer" || target === "properties" || target === "whitebox") {
            await user.click(runButton);
            await waitFor(() => {
              assert.equal(
                document
                  .querySelector("[data-main-screen-mode]")
                  ?.getAttribute("data-main-screen-mode"),
                target,
              );
            });
          } else {
            const toggle = document.querySelector(
              `[data-panel-toggle="${target}"]`,
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
          }
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

  test("viewport camera controls support presets, zoom and orbit", async () => {
    const { user } = await renderApp();
    const cameraState = document.querySelector("[data-viewport-camera-state]");
    const canvas = document.querySelector('[data-viewport-canvas="true"]');

    assert.ok(cameraState);
    assert.ok(canvas);
    const initialState = cameraState.getAttribute("data-viewport-camera-state");

    await user.click(document.querySelector('[data-viewport-preset="top"]'));
    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-viewport-active-preset]")
          ?.getAttribute("data-viewport-active-preset"),
        "top",
      );
    });

    const presetState = cameraState.getAttribute("data-viewport-camera-state");
    assert.notEqual(presetState, initialState);

    await user.click(document.querySelector('[data-viewport-zoom="in"]'));
    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-viewport-active-preset]")
          ?.getAttribute("data-viewport-active-preset"),
        "custom",
      );
    });

    const zoomedState = cameraState.getAttribute("data-viewport-camera-state");
    assert.notEqual(zoomedState, presetState);

    fireEvent.pointerDown(canvas, {
      pointerId: 1,
      clientX: 320,
      clientY: 280,
      button: 0,
    });
    fireEvent.pointerMove(canvas, {
      pointerId: 1,
      clientX: 410,
      clientY: 228,
      button: 0,
    });
    fireEvent.pointerUp(canvas, {
      pointerId: 1,
      clientX: 410,
      clientY: 228,
      button: 0,
    });

    await waitFor(() => {
      const orbitState = cameraState.getAttribute("data-viewport-camera-state");
      assert.notEqual(orbitState, zoomedState);
    });
  });

  test("viewport shape editor updates units and dimensional summary", async () => {
    const { user } = await renderApp();
    const unitSelect = document.querySelector('[data-viewport-unit-select="true"]');
    const shapeSelect = document.querySelector(
      '[data-viewport-shape-select="true"]',
    );
    const widthInput = document.querySelector(
      '[data-viewport-dimension="width"]',
    );

    assert.ok(unitSelect);
    assert.ok(shapeSelect);
    assert.ok(widthInput);

    await user.selectOptions(unitSelect, "in");
    await user.selectOptions(shapeSelect, "portal");
    fireEvent.change(widthInput, { target: { value: "18" } });

    await waitFor(() => {
      const shapeName = document
        .querySelector("[data-viewport-shape-name]")
        ?.getAttribute("data-viewport-shape-name");
      const measureSummary = document
        .querySelector("[data-viewport-measure-summary]")
        ?.getAttribute("data-viewport-measure-summary");

      assert.equal(shapeName, "Portique");
      assert.match(measureSummary ?? "", /18\.00 in/);
    });
  });

  test("file execute buttons expose an immediate visible effect in the command surface", async () => {
    const { user } = await renderApp();
    const menuPicker = screen.getByLabelText("Menu");

    const fixtureSelector = screen.getByLabelText("Projet de demonstration");
    await user.selectOptions(fixtureSelector, "empty-project.faero");
    await user.selectOptions(menuPicker, "commands:file");

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

    await user.click(
      document.querySelector('[data-command-id="app.settings"]'),
    );
    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-main-screen-mode]")
          ?.getAttribute("data-main-screen-mode"),
        "properties",
      );
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
