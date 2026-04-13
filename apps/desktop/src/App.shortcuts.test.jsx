import { describe, test } from "vitest";

import { assert, renderApp, screen, waitFor } from "./App.test-helpers.jsx";

describe("App keyboard shortcuts", () => {
  test("keyboard shortcuts execute visible shell commands", async () => {
    const { user } = await renderApp();

    await user.keyboard("{Control>}{Shift>}N{/Shift}{/Control}");
    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "project.create",
      );
      assert.ok(screen.getAllByText("FutureAero Session").length >= 1);
    });

    await user.keyboard("{F4}");
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
        "view.properties",
      );
    });

    await user.keyboard("{Alt>}{Enter}{/Alt}");
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
        "project.properties",
      );
    });
  });
});
