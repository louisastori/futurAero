import test from "node:test";
import assert from "node:assert/strict";

import { hasTranslation, supportedLocales } from "./i18n.mjs";
import {
  getAllMenuCommands,
  getTopLevelMenuLabels,
  localizeMenuModel,
  visualStudioInspiredMenus
} from "./menu-model.mjs";

const requiredShellTranslationKeys = [
  "ui.brand.subtitle",
  "ui.panel.project_explorer",
  "ui.panel.properties",
  "ui.panel.command_surface",
  "ui.panel.viewport",
  "ui.panel.output",
  "ui.panel.problems",
  "ui.property.project",
  "ui.property.project_id",
  "ui.property.runtime",
  "ui.property.default_frame",
  "ui.property.root_scene",
  "ui.property.endpoints",
  "ui.property.streams",
  "ui.property.plugins",
  "ui.property.language",
  "ui.property.fixture",
  "ui.workspace.entities_section",
  "ui.workspace.endpoints_section",
  "ui.workspace.streams_section",
  "ui.workspace.plugins_section",
  "ui.workspace.empty_section",
  "ui.workspace.enabled",
  "ui.workspace.disabled",
  "ui.output.recent_activity",
  "ui.output.no_activity",
  "ui.output.raw_status",
  "ui.output.problems",
  "ui.activity.command",
  "ui.activity.event",
  "ui.locale.label",
  "ui.fixture.label",
  "ui.fixture.empty",
  "ui.fixture.loading",
  "ui.problems.none_blocking",
  "ui.status.web_preview",
  "ui.status.tauri"
];

test("top level menu order stays close to Visual Studio with French as default locale", () => {
  assert.deepEqual(getTopLevelMenuLabels(), [
    "Fichier",
    "Edition",
    "Affichage",
    "Git",
    "Projet",
    "Generation",
    "Debogage",
    "Test",
    "Analyse",
    "Outils",
    "Fenetre",
    "Aide"
  ]);
});

test("secondary locales remain available for English and Spanish shells", () => {
  assert.deepEqual(getTopLevelMenuLabels("en"), [
    "File",
    "Edit",
    "View",
    "Git",
    "Project",
    "Build",
    "Debug",
    "Test",
    "Analyze",
    "Tools",
    "Window",
    "Help"
  ]);
  assert.deepEqual(getTopLevelMenuLabels("es"), [
    "Archivo",
    "Editar",
    "Ver",
    "Git",
    "Proyecto",
    "Compilar",
    "Depurar",
    "Pruebas",
    "Analizar",
    "Herramientas",
    "Ventana",
    "Ayuda"
  ]);
});

test("all actionable menu items expose a command id", () => {
  for (const menu of visualStudioInspiredMenus) {
    for (const item of menu.items) {
      if (item.type === "separator") {
        continue;
      }

      assert.equal(typeof item.command, "string");
      assert.ok(item.command.length > 0);
    }
  }
});

test("menu commands stay unique to avoid ambiguous routing", () => {
  const commands = getAllMenuCommands();
  const uniqueCommands = new Set(commands);

  assert.equal(commands.length, uniqueCommands.size);
});

test("every menu translation key is available in every supported locale", () => {
  const sourceMenus = localizeMenuModel("en");

  for (const { id: locale } of supportedLocales) {
    for (const menu of sourceMenus) {
      assert.equal(hasTranslation(locale, menu.translationKey), true, `${locale} missing ${menu.translationKey}`);

      for (const item of menu.items) {
        if (item.type === "separator") {
          continue;
        }

        assert.equal(
          hasTranslation(locale, item.translationKey),
          true,
          `${locale} missing ${item.translationKey}`
        );
      }
    }
  }
});

test("desktop shell translation keys stay available in all supported locales", () => {
  for (const { id: locale } of supportedLocales) {
    for (const translationKey of requiredShellTranslationKeys) {
      assert.equal(
        hasTranslation(locale, translationKey),
        true,
        `${locale} missing ${translationKey}`
      );
    }
  }
});
