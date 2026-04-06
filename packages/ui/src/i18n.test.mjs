import test from "node:test";
import assert from "node:assert/strict";

import { aerospaceReferenceScenes } from "../../viewport/src/aerospace-scenes.mjs";
import { hasTranslation, supportedLocales } from "./i18n.mjs";

const staticViewportKeys = [
  "ui.viewport.reference_toolbar",
  "ui.viewport.reference_caption",
  "ui.viewport.reference_inspiration",
  "ui.viewport.reference_analysis",
  "ui.viewport.reference_legend",
  "ui.viewport.caption",
  "ui.viewport.legend.primary_flow",
  "ui.viewport.legend.rotor_stage",
  "ui.viewport.legend.core_shaft",
  "ui.viewport.legend.skin_shell",
  "ui.viewport.legend.internal_systems",
  "ui.viewport.legend.primary_structure",
  "ui.viewport.legend.reference_axis",
  "ui.viewport.legend.access_volume",
  "ui.viewport.legend.routing_bundle",
  "ui.viewport.legend.stress_low",
  "ui.viewport.legend.stress_mid",
  "ui.viewport.legend.stress_high",
  "ui.viewport.legend.flow_attached",
  "ui.viewport.legend.flow_transition",
  "ui.viewport.legend.flow_detached"
];

const dynamicSceneKeys = aerospaceReferenceScenes.flatMap((scene) => [
  scene.titleKey,
  scene.summaryKey,
  ...scene.analysisKeys
]);

test("viewport aerospace translations exist in all supported locales", () => {
  const keys = [...staticViewportKeys, ...dynamicSceneKeys];

  for (const locale of supportedLocales.map((entry) => entry.id)) {
    for (const key of keys) {
      assert.equal(
        hasTranslation(locale, key),
        true,
        `missing translation for ${locale}:${key}`
      );
    }
  }
});
