import assert from "node:assert/strict";
import test from "node:test";

import {
  buildEntityInspectorSchema,
  buildInspectorDraftFromSchema,
  coerceInspectorDraftValue,
  findInspectorFieldByPath,
  flattenEditableInspectorFields,
} from "./property-inspector.mjs";

function samplePartEntity() {
  return {
    id: "ent_part_001",
    entityType: "Part",
    name: "Bracket-001",
    revision: "rev_0001",
    status: "active",
    data: {
      tags: ["part", "parametric"],
      parameterSet: {
        widthMm: 120,
        heightMm: 80,
        dimensions: {
          chamferMm: 2.5,
        },
        checkpoints: [1, 2, 3],
      },
      summary: {
        state: "well_constrained",
      },
    },
  };
}

test("buildEntityInspectorSchema exposes editable identity and parameter fields", () => {
  const schema = buildEntityInspectorSchema(samplePartEntity());
  const editablePaths = flattenEditableInspectorFields(schema).map(
    (field) => field.path,
  );

  assert.deepEqual(editablePaths, [
    "name",
    "tags",
    "parameterSet.widthMm",
    "parameterSet.heightMm",
    "parameterSet.dimensions.chamferMm",
    "parameterSet.checkpoints",
  ]);
});

test("buildInspectorDraftFromSchema serializes array fields as JSON and tags as CSV", () => {
  const schema = buildEntityInspectorSchema(samplePartEntity());
  const draft = buildInspectorDraftFromSchema(schema);

  assert.equal(draft.name, "Bracket-001");
  assert.equal(draft.tags, "part, parametric");
  assert.equal(draft["parameterSet.widthMm"], "120");
  assert.equal(draft["parameterSet.checkpoints"], "[\n  1,\n  2,\n  3\n]");
});

test("coerceInspectorDraftValue validates numbers and JSON arrays", () => {
  const schema = buildEntityInspectorSchema(samplePartEntity());
  const widthField = findInspectorFieldByPath(schema, "parameterSet.widthMm");
  const checkpointsField = findInspectorFieldByPath(
    schema,
    "parameterSet.checkpoints",
  );

  assert.deepEqual(coerceInspectorDraftValue("150", widthField), {
    ok: true,
    value: 150,
  });
  assert.equal(coerceInspectorDraftValue("0", widthField).ok, false);
  assert.deepEqual(
    coerceInspectorDraftValue("[1,2,3,4]", checkpointsField),
    {
      ok: true,
      value: [1, 2, 3, 4],
    },
  );
  assert.equal(
    coerceInspectorDraftValue('{"bad":true}', checkpointsField).ok,
    false,
  );
});

test("signal schema exposes enum and typed current value editing", () => {
  const schema = buildEntityInspectorSchema({
    id: "ent_sig_001",
    entityType: "Signal",
    name: "CycleStart",
    revision: "rev_seed",
    status: "active",
    data: {
      tags: ["control"],
      kind: "boolean",
      currentValue: false,
      parameterSet: {
        unit: "bool",
      },
    },
  });

  assert.equal(findInspectorFieldByPath(schema, "kind")?.fieldType, "enum");
  assert.deepEqual(findInspectorFieldByPath(schema, "kind")?.options, [
    "boolean",
    "scalar",
    "text",
  ]);
  assert.equal(
    findInspectorFieldByPath(schema, "currentValue")?.fieldType,
    "boolean",
  );
});
