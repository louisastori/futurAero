import test from "node:test";
import assert from "node:assert/strict";

import { hasTranslation, supportedLocales } from "./i18n.mjs";
import {
  getAllMenuCommands,
  getTopLevelMenuLabels,
  localizeMenuModel,
  visualStudioInspiredMenus
} from "./menu-model.mjs";

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
