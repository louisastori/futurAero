import { describe, test } from "vitest";

import {
  assert,
  createMockBackend,
  fireEvent,
  renderApp,
  screen,
  waitFor,
} from "./App.test-helpers.jsx";

async function selectMenu(user, value) {
  await user.selectOptions(screen.getByLabelText("Menu"), value);
}

async function focusEntityInspector(user, entityId) {
  await selectMenu(user, "explorer");
  await waitFor(() => {
    assert.ok(document.querySelector(`[data-entity-select="${entityId}"]`));
  });
  await user.click(document.querySelector(`[data-entity-select="${entityId}"]`));
  await selectMenu(user, "properties");
}

describe("App entity and simulation flows", () => {
  test("workspace defaults to the white box while explorer and command overlays stay hidden", async () => {
    await renderApp();

    await waitFor(() => {
      assert.equal(
        document
          .querySelector('[data-main-screen-mode]')
          ?.getAttribute("data-main-screen-mode"),
        "whitebox",
      );
      assert.ok(document.querySelector('[data-panel-id="viewport"]'));
      assert.ok(document.querySelector('[data-panel-id="simulationTimeline"]'));
      assert.equal(
        document
          .querySelector('[data-overlay-kind="inspector"]')
          ?.getAttribute("aria-hidden"),
        "true",
      );
      assert.equal(
        document
          .querySelector('[data-overlay-kind="commands"]')
          ?.getAttribute("aria-hidden"),
        "true",
      );
    });
  });

  test("dropdown opens explorer, properties and simulation overlays then returns to white box", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "explorer");
    await waitFor(() => {
      assert.equal(
        document
          .querySelector('[data-overlay-kind="inspector"]')
          ?.getAttribute("aria-hidden"),
        "false",
      );
      assert.ok(screen.getAllByText("Explorateur de projet").length >= 1);
      assert.ok(document.querySelector("[data-entity-select]"));
    });

    await selectMenu(user, "properties");
    await waitFor(() => {
      assert.equal(
        document
          .querySelector('[data-overlay-kind="inspector"]')
          ?.getAttribute("aria-hidden"),
        "false",
      );
      assert.ok(document.querySelector('[data-openspec-summary="ops_001"]'));
    });

    await selectMenu(user, "commands:simulation");
    await waitFor(() => {
      assert.equal(
        document
          .querySelector('[data-overlay-kind="commands"]')
          ?.getAttribute("aria-hidden"),
        "false",
      );
      assert.ok(document.querySelector('[data-command-id="simulation.run.start"]'));
    });

    await selectMenu(user, "whitebox");
    await waitFor(() => {
      assert.equal(
        document
          .querySelector('[data-main-screen-mode]')
          ?.getAttribute("data-main-screen-mode"),
        "whitebox",
      );
      assert.equal(
        document
          .querySelector('[data-overlay-kind="inspector"]')
          ?.getAttribute("aria-hidden"),
        "true",
      );
      assert.equal(
        document
          .querySelector('[data-overlay-kind="commands"]')
          ?.getAttribute("aria-hidden"),
        "true",
      );
    });
  });

  test("creating a part surfaces parametric geometry metrics in properties", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:insert");
    await user.click(
      document.querySelector('[data-command-id="entity.create.part"]'),
    );
    await selectMenu(user, "properties");

    await waitFor(() => {
      assert.ok(screen.getByText("Pieces parametriques"));
      assert.ok(
        document.querySelector('[data-parametric-part-summary="ent_part_002"]'),
      );
      assert.ok(
        document
          .querySelector('[data-parametric-part-mass="ent_part_002"]')
          ?.textContent?.includes("367"),
      );
    });
  });

  test("parametric editor regenerates the latest part with new dimensions", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:insert");
    await user.click(
      document.querySelector('[data-command-id="entity.create.part"]'),
    );
    await selectMenu(user, "properties");

    const widthInput = screen.getByLabelText("Largeur");
    const heightInput = screen.getByLabelText("Hauteur");
    const depthInput = screen.getByLabelText("Profondeur");

    fireEvent.change(widthInput, { target: { value: "200" } });
    fireEvent.change(heightInput, { target: { value: "90" } });
    fireEvent.change(depthInput, { target: { value: "20" } });
    await user.click(
      document.querySelector('[data-parametric-regenerate="true"]'),
    );

    await waitFor(() => {
      assert.ok(
        document.querySelector('[data-parametric-part-summary="ent_part_002"]'),
      );
      assert.ok(
        document
          .querySelector('[data-parametric-part-mass="ent_part_002"]')
          ?.textContent?.includes("972"),
      );
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "build.regenerate_part",
      );
    });
  });

  test("generic inspector edits the selected entity name, nested parameters and list values", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:insert");
    await user.click(
      document.querySelector('[data-command-id="entity.create.part"]'),
    );

    await focusEntityInspector(user, "ent_part_002");

    const nameInput = screen.getByLabelText("Nom");
    const tagsInput = screen.getByLabelText("Tags");
    const widthInput = screen.getByLabelText("widthMm");
    const heightInput = screen.getByLabelText("heightMm");
    const depthInput = screen.getByLabelText("depthMm");
    const toleranceInput = screen.getByLabelText("toleranceMm");
    const checkpointsInput = screen.getByLabelText("checkpoints");

    fireEvent.change(nameInput, { target: { value: "Part-Edited-002" } });
    fireEvent.change(tagsInput, { target: { value: "edited, qa" } });
    fireEvent.change(widthInput, { target: { value: "210" } });
    fireEvent.change(heightInput, { target: { value: "95" } });
    fireEvent.change(depthInput, { target: { value: "18" } });
    fireEvent.change(toleranceInput, { target: { value: "0.25" } });
    fireEvent.change(checkpointsInput, { target: { value: "[210, 95, 18]" } });
    await user.click(
      document.querySelector('[data-entity-save="ent_part_002"]'),
    );

    await waitFor(() => {
      assert.equal(
        document.querySelector('[data-entity-inspector="ent_part_002"] strong')
          ?.textContent,
        "Part-Edited-002",
      );
      assert.equal(screen.getByLabelText("toleranceMm").value, "0.25");
      assert.equal(screen.getByLabelText("checkpoints").value, "[\n  210,\n  95,\n  18\n]");
      assert.ok(
        document.querySelector('[data-parametric-part-summary="ent_part_002"]'),
      );
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "entity.properties.update",
      );
      assert.ok(screen.getAllByText("Part-Edited-002").length >= 2);
    });
  });

  test("simulation timeline panel exposes signal and controller events from persisted artifacts", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:simulation");
    await user.click(
      document.querySelector('[data-command-id="simulation.run.start"]'),
    );

    await waitFor(() => {
      assert.ok(
        document.querySelector('[data-simulation-timeline-focus]'),
      );
      assert.ok(
        document.querySelector('[data-simulation-run-job-panel^="ent_run_"]'),
      );
      assert.ok(
        Array.from(document.querySelectorAll("[data-simulation-event]")).length >=
          3,
      );
      assert.ok(screen.getAllByText(/queued/).length >= 1);
      assert.ok(screen.getAllByText(/trace_persisted/).length >= 1);
    });

    await user.click(document.querySelector('[data-simulation-step="true"]'));

    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "simulation.timeline.step",
      );
    });
  });

  test("simulation timeline surfaces the run report and focuses the reported critical event", async () => {
    const backend = createMockBackend();
    await backend.executeWorkspaceCommand("entity.create.external_endpoint");
    await backend.executeWorkspaceCommand("entity.create.external_endpoint");
    await backend.executeWorkspaceCommand("simulation.run.start");

    const { user } = await renderApp({ backend });

    await selectMenu(user, "commands:simulation");

    await waitFor(() => {
      const report = document.querySelector(
        '[data-simulation-run-report^="ent_run_"]',
      );
      assert.ok(report);
      assert.ok(report.textContent?.includes("Collision critique"));
      assert.equal(
        document
          .querySelector("[data-simulation-timeline-focus]")
          ?.getAttribute("data-simulation-timeline-focus")
          ?.startsWith("collision-"),
        true,
      );
    });

    await user.click(document.querySelector('[data-simulation-step="true"]'));
    await user.click(
      document.querySelector('[data-simulation-jump-critical="true"]'),
    );

    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "simulation.timeline.jump_critical",
      );
      assert.equal(
        document
          .querySelector("[data-simulation-timeline-focus]")
          ?.getAttribute("data-simulation-timeline-focus")
          ?.startsWith("collision-"),
        true,
      );
    });
  });

  test("creating a robot cell surfaces structure and timing metrics in properties", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:insert");
    await user.click(
      document.querySelector('[data-command-id="entity.create.robot_cell"]'),
    );
    await selectMenu(user, "properties");

    await waitFor(() => {
      assert.ok(screen.getByText("Cellules robotiques"));
      assert.ok(
        document.querySelector('[data-robot-cell-summary="ent_cell_002"]'),
      );
      assert.ok(
        document
          .querySelector('[data-robot-cell-targets="ent_cell_002"]')
          ?.textContent?.includes("3"),
      );
      assert.ok(
        document
          .querySelector('[data-robot-cell-equipment="ent_cell_002"]')
          ?.textContent?.includes("3"),
      );
      assert.ok(
        document
          .querySelector('[data-robot-cell-scene="ent_cell_002"]')
          ?.textContent?.includes("ent_asm_cell_002"),
      );
      assert.ok(
        document
          .querySelector('[data-robot-cell-preview="ent_cell_002"]')
          ?.textContent?.includes("pick -> transfer -> place"),
      );
    });

    await selectMenu(user, "explorer");

    await waitFor(() => {
      assert.ok(document.querySelector('[data-entity-select="ent_robot_002"]'));
      assert.ok(
        document.querySelector('[data-entity-select="ent_conveyor_002"]'),
      );
      assert.ok(document.querySelector('[data-entity-select="ent_seq_002"]'));
      assert.ok(
        document.querySelector('[data-entity-select="ent_target_002_pick"]'),
      );
    });
  });

  test("editing a robot target order updates the ordered target preview", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:insert");
    await user.click(
      document.querySelector('[data-command-id="entity.create.robot_cell"]'),
    );
    await focusEntityInspector(user, "ent_target_002_transfer");

    fireEvent.change(screen.getByLabelText("orderIndex"), {
      target: { value: "4" },
    });
    await user.click(
      document.querySelector('[data-entity-save="ent_target_002_transfer"]'),
    );

    await waitFor(() => {
      assert.equal(screen.getByLabelText("orderIndex").value, "4");
      assert.ok(
        document
          .querySelector('[data-robot-cell-preview="ent_cell_002"]')
          ?.textContent?.includes("pick -> place -> transfer"),
      );
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "entity.properties.update",
      );
    });
  });

  test("signal inspector edits boolean, scalar and text values and refreshes blocked state", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:insert");
    await user.click(
      document.querySelector('[data-command-id="entity.create.robot_cell"]'),
    );
    await selectMenu(user, "properties");

    await waitFor(() => {
      assert.ok(
        document
          .querySelector('[data-robot-cell-blocked="ent_cell_002"]')
          ?.textContent?.includes("idle"),
      );
    });

    await focusEntityInspector(user, "ent_sig_002_cycle_start");
    await user.click(screen.getByLabelText("currentValue"));
    await user.click(
      document.querySelector('[data-entity-save="ent_sig_002_cycle_start"]'),
    );

    await waitFor(() => {
      assert.equal(screen.getByLabelText("currentValue").checked, true);
      assert.ok(
        document
          .querySelector('[data-robot-cell-blocked="ent_cell_002"]')
          ?.textContent?.includes("place"),
      );
    });

    await focusEntityInspector(user, "ent_sig_002_progress_gate");
    fireEvent.change(screen.getByLabelText("currentValue"), {
      target: { value: "1" },
    });
    await user.click(
      document.querySelector('[data-entity-save="ent_sig_002_progress_gate"]'),
    );

    await waitFor(() => {
      assert.equal(screen.getByLabelText("currentValue").value, "1");
      assert.ok(
        document
          .querySelector('[data-robot-cell-blocked="ent_cell_002"]')
          ?.textContent?.includes("control clear"),
      );
    });

    await focusEntityInspector(user, "ent_sig_002_operator_mode");
    assert.equal(screen.getByLabelText("kind").value, "text");
    fireEvent.change(screen.getByLabelText("currentValue"), {
      target: { value: "manual" },
    });
    await user.click(
      document.querySelector('[data-entity-save="ent_sig_002_operator_mode"]'),
    );

    await waitFor(() => {
      assert.equal(screen.getByLabelText("kind").value, "text");
      assert.equal(screen.getByLabelText("currentValue").value, "manual");
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "entity.properties.update",
      );
    });
  });

  test("controller inspector exposes explicit transitions and robot cell blocked state", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:insert");
    await user.click(
      document.querySelector('[data-command-id="entity.create.robot_cell"]'),
    );
    await selectMenu(user, "properties");

    await waitFor(() => {
      assert.ok(
        document
          .querySelector('[data-robot-cell-blocked="ent_cell_002"]')
          ?.textContent?.includes("idle"),
      );
      assert.ok(
        document
          .querySelector('[data-robot-cell-transitions="ent_cell_002"]')
          ?.textContent?.includes("3"),
      );
    });

    await focusEntityInspector(user, "ent_ctrl_002");

    await waitFor(() => {
      assert.ok(screen.getByText(/tr_start_cycle/));
      assert.ok(screen.getByText(/tr_reach_place/));
      assert.ok(screen.getByText(/tr_finish_cycle/));
    });
  });

  test("starting a simulation surfaces a completed run summary in properties", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:simulation");
    await user.click(
      document.querySelector('[data-command-id="simulation.run.start"]'),
    );
    await selectMenu(user, "properties");

    await waitFor(() => {
      assert.ok(screen.getByText("Runs de simulation"));
      const runSummary = document.querySelector(
        '[data-simulation-run-summary^="ent_run_"]',
      );
      assert.ok(runSummary);
      const runId = runSummary?.getAttribute("data-simulation-run-summary");
      assert.ok(
        document
          .querySelector(`[data-simulation-run-collisions="${runId}"]`)
          ?.textContent?.includes("0"),
      );
      assert.ok(
        document
          .querySelector(`[data-simulation-run-job="${runId}"]`)
          ?.textContent?.includes("completed"),
      );
      assert.ok(
        document
          .querySelector(`[data-simulation-run-job="${runId}"]`)
          ?.textContent?.includes("100"),
      );
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "simulation.run.start",
      );
    });
  });

  test("simulation run relaunches an automatic OpenSpec prompt in the local AI panel", async () => {
    const { user } = await renderApp();

    const autoPromptToggle = document.querySelector(
      '[data-ai-auto-prompts="true"]',
    );
    assert.equal(autoPromptToggle?.checked, true);

    await selectMenu(user, "commands:simulation");
    await user.click(
      document.querySelector('[data-command-id="simulation.run.start"]'),
    );

    await waitFor(() => {
      assert.ok(screen.getAllByText(/Mode summarize/).length >= 2);
      assert.ok(screen.getByText("auto-openspec"));
      assert.ok(document.querySelector('[data-ai-structured="true"]'));
    });
  });

  test("running safety analysis surfaces a safety report in properties", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "commands:simulation");
    await user.click(
      document.querySelector('[data-command-id="analyze.safety"]'),
    );
    await selectMenu(user, "properties");

    await waitFor(() => {
      assert.ok(screen.getByText("Rapports safety"));
      const reportSummary = document.querySelector(
        '[data-safety-report-summary^="ent_safe_"]',
      );
      assert.ok(reportSummary);
      const reportId = reportSummary?.getAttribute("data-safety-report-summary");
      assert.ok(
        document
          .querySelector(`[data-safety-report-blocks="${reportId}"]`)
          ?.textContent?.includes("0"),
      );
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "analyze.safety",
      );
    });
  });

  test("openspec documents are visible and the help command can be executed", async () => {
    const { user } = await renderApp();

    await selectMenu(user, "properties");

    await waitFor(() => {
      const openSpecCard = document.querySelector(
        '[data-openspec-summary="ops_001"]',
      );
      assert.ok(openSpecCard);
      assert.equal(
        openSpecCard?.querySelector("strong")?.textContent,
        "Readable Layout Intent",
      );
    });

    await selectMenu(user, "commands:help");
    await user.click(
      document.querySelector('[data-command-id="help.openspec"]'),
    );

    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "help.openspec",
      );
      assert.equal(
        document
          .querySelector("[data-last-command-id]")
          ?.getAttribute("data-last-command-id"),
        "help.openspec",
      );
    });
  });
});
