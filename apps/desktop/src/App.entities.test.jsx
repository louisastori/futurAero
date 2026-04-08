import { describe, test } from "vitest";

import {
  assert,
  fireEvent,
  renderApp,
  screen,
  waitFor,
} from "./App.test-helpers.jsx";

describe("App entity and simulation flows", () => {
  test("creating a part surfaces parametric geometry metrics in properties", async () => {
    const { user } = await renderApp();

    await user.click(screen.getByRole("button", { name: "Insertion" }));
    await user.click(
      document.querySelector('[data-command-id="entity.create.part"]'),
    );

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

    await user.click(screen.getByRole("button", { name: "Insertion" }));
    await user.click(
      document.querySelector('[data-command-id="entity.create.part"]'),
    );

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

    await user.click(screen.getByRole("button", { name: "Insertion" }));
    await user.click(
      document.querySelector('[data-command-id="entity.create.part"]'),
    );

    await waitFor(() => {
      assert.ok(document.querySelector('[data-entity-select="ent_part_002"]'));
    });

    await user.click(
      document.querySelector('[data-entity-select="ent_part_002"]'),
    );

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

    await user.click(screen.getByRole("button", { name: "Simulation" }));
    await user.click(
      document.querySelector('[data-command-id="simulation.run.start"]'),
    );

    await waitFor(() => {
      assert.ok(
        document.querySelector('[data-simulation-timeline-focus]'),
      );
      assert.ok(
        Array.from(document.querySelectorAll("[data-simulation-event]")).length >=
          3,
      );
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

  test("creating a robot cell surfaces path and timing metrics in properties", async () => {
    const { user } = await renderApp();

    await user.click(screen.getByRole("button", { name: "Insertion" }));
    await user.click(
      document.querySelector('[data-command-id="entity.create.robot_cell"]'),
    );

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
    });
  });

  test("signal inspector edits enum kind and typed current value", async () => {
    const { user } = await renderApp();

    await user.click(screen.getByRole("button", { name: "Insertion" }));
    await user.click(
      document.querySelector('[data-command-id="entity.create.robot_cell"]'),
    );

    await waitFor(() => {
      assert.ok(document.querySelector('[data-entity-select^="ent_sig_"]'));
    });

    await user.click(document.querySelector('[data-entity-select^="ent_sig_"]'));

    const kindSelect = screen.getByLabelText("kind");
    const currentValueInput = screen.getByLabelText("currentValue");
    const checkpointsInput = screen.getByLabelText("checkpoints");

    await user.selectOptions(kindSelect, "text");
    fireEvent.change(currentValueInput, { target: { value: "ready" } });
    fireEvent.change(checkpointsInput, { target: { value: '["ready", "done"]' } });
    await user.click(
      document.querySelector('[data-entity-save^="ent_sig_"]'),
    );

    await waitFor(() => {
      assert.equal(screen.getByLabelText("kind").value, "text");
      assert.equal(screen.getByLabelText("currentValue").value, "ready");
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "entity.properties.update",
      );
    });
  });

  test("starting a simulation surfaces a completed run summary in properties", async () => {
    const { user } = await renderApp();

    await user.click(screen.getByRole("button", { name: "Simulation" }));
    await user.click(
      document.querySelector('[data-command-id="simulation.run.start"]'),
    );

    await waitFor(() => {
      assert.ok(screen.getByText("Runs de simulation"));
      assert.ok(
        document.querySelector('[data-simulation-run-summary="ent_run_003"]'),
      );
      assert.ok(
        document
          .querySelector('[data-simulation-run-collisions="ent_run_003"]')
          ?.textContent?.includes("0"),
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

    await user.click(screen.getByRole("button", { name: "Simulation" }));
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

    await user.click(screen.getByRole("button", { name: "Simulation" }));
    await user.click(
      document.querySelector('[data-command-id="analyze.safety"]'),
    );

    await waitFor(() => {
      assert.ok(screen.getByText("Rapports safety"));
      assert.ok(
        document.querySelector('[data-safety-report-summary="ent_safe_003"]'),
      );
      assert.ok(
        document
          .querySelector('[data-safety-report-blocks="ent_safe_003"]')
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

    await user.click(screen.getByRole("button", { name: "Aide" }));
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
