import test from "node:test";
import assert from "node:assert/strict";

import {
  aerospaceReferenceScenes,
  defaultAerospaceSceneId,
  getAerospaceScene,
  getAerospaceSceneIds
} from "./aerospace-scenes.mjs";

test("aerospace viewport exposes five reproducible reference scenes", () => {
  assert.equal(aerospaceReferenceScenes.length, 5);
  assert.deepEqual(getAerospaceSceneIds(), [
    "turbofan_exploded",
    "airframe_transparent",
    "wireframe_maintenance",
    "stress_map",
    "aero_heatmap"
  ]);
});

test("default scene resolves to the first aerospace preset", () => {
  assert.equal(defaultAerospaceSceneId, "turbofan_exploded");
  assert.equal(getAerospaceScene().id, "turbofan_exploded");
});

test("scene lookup falls back safely when an unknown id is requested", () => {
  const fallback = getAerospaceScene("unknown-scene");
  assert.equal(fallback.id, defaultAerospaceSceneId);
  assert.equal(fallback.category, "engine");
});

test("every scene carries localized analysis and legend references", () => {
  for (const scene of aerospaceReferenceScenes) {
    assert.equal(scene.analysisKeys.length, 3);
    assert.equal(scene.legendKeys.length, 3);
    assert.ok(scene.titleKey.startsWith("ui.viewport.scene."));
    assert.ok(scene.summaryKey.startsWith("ui.viewport.scene."));
  }
});
