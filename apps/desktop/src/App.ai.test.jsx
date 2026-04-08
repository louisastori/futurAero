import { describe, test } from "vitest";

import { assert, renderApp, screen, waitFor } from "./App.test-helpers.jsx";

describe("App local AI flows", () => {
  test("gemma3 selector defaults to 27b and sends the chosen variant to the backend", async () => {
    const { user, backend } = await renderApp();

    const selector = screen.getByLabelText("Modele Gemma3");
    assert.equal(selector.value, "gemma3:27b");

    await user.selectOptions(selector, "gemma3:12b");
    assert.equal(selector.value, "gemma3:12b");

    const textarea = screen.getByPlaceholderText(
      "Pose une question sur le projet courant, la simulation, l integration ou la safety...",
    );
    await user.type(textarea, "Compare les variantes gemma3");
    await user.click(document.querySelector('[data-ai-send="true"]'));

    await waitFor(() => {
      assert.equal(backend.getLastSelectedModel(), "gemma3:12b");
      assert.ok(
        screen.getByText(/\[fr\] Compare les variantes gemma3 :: gemma3:12b/),
      );
    });
  });

  test("structured AI suggestions can be previewed, applied and rejected explicitly", async () => {
    const { user } = await renderApp();

    const textarea = screen.getByPlaceholderText(
      "Pose une question sur le projet courant, la simulation, l integration ou la safety...",
    );
    await user.type(textarea, "Propose un changement applicable");
    await user.click(document.querySelector('[data-ai-send="true"]'));

    await waitFor(() => {
      assert.ok(document.querySelector("[data-ai-suggestion-preview]"));
      assert.ok(
        document.querySelector(
          '[data-ai-proposed-command="entity.properties.update"]',
        ),
      );
      assert.ok(document.querySelector("[data-ai-apply-suggestion]"));
    });

    await user.click(document.querySelector("[data-ai-apply-suggestion]"));

    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "ai.suggestion.apply",
      );
      assert.ok(screen.getByText("Suggestion appliquee"));
    });

    await user.type(textarea, "Propose encore un changement");
    await user.click(document.querySelector('[data-ai-send="true"]'));

    await waitFor(() => {
      assert.equal(document.querySelectorAll("[data-ai-reject-suggestion]").length >= 1, true);
    });

    const rejectButtons = document.querySelectorAll("[data-ai-reject-suggestion]");
    await user.click(rejectButtons[rejectButtons.length - 1]);

    await waitFor(() => {
      assert.equal(
        document
          .querySelector("[data-command-feedback]")
          ?.getAttribute("data-command-feedback"),
        "ai.suggestion.reject",
      );
      assert.ok(screen.getByText("Suggestion rejetee"));
    });
  });
});
