const SIGNAL_KIND_OPTIONS = ["boolean", "scalar", "text"];

function isObjectLike(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isEditablePath(entity, path) {
  if (!entity || !path) {
    return false;
  }

  if (path === "name" || path === "tags") {
    return true;
  }

  if (path.startsWith("parameterSet.")) {
    return true;
  }

  if (entity.entityType === "Signal") {
    return ["kind", "currentValue", "parameterSet.unit"].includes(path);
  }

  return false;
}

function signalCurrentValueFieldType(entity) {
  const kind = entity?.data?.kind;
  if (kind === "scalar") {
    return "number";
  }
  if (kind === "text") {
    return "string";
  }
  return "boolean";
}

function resolveFieldMetadata(entity, path, value) {
  if (path === "name") {
    return {
      fieldType: "string",
      editable: true,
      required: true,
    };
  }

  if (path === "tags") {
    return {
      fieldType: "tag-list",
      editable: true,
      required: false,
    };
  }

  if (entity?.entityType === "Signal" && path === "kind") {
    return {
      fieldType: "enum",
      editable: true,
      required: true,
      options: SIGNAL_KIND_OPTIONS,
    };
  }

  if (entity?.entityType === "Signal" && path === "currentValue") {
    return {
      fieldType: signalCurrentValueFieldType(entity),
      editable: true,
      required: true,
    };
  }

  if (Array.isArray(value)) {
    return {
      fieldType: "list",
      editable: isEditablePath(entity, path),
      required: false,
    };
  }

  if (isObjectLike(value)) {
    return {
      fieldType: "object",
      editable: false,
      required: false,
    };
  }

  if (typeof value === "boolean") {
    return {
      fieldType: "boolean",
      editable: isEditablePath(entity, path),
      required: false,
    };
  }

  if (typeof value === "number") {
    const numericMinimum =
      path.endsWith("widthMm") || path.endsWith("heightMm") || path.endsWith("depthMm")
        ? 0.000001
        : null;
    return {
      fieldType: "number",
      editable: isEditablePath(entity, path),
      required: false,
      minimum: numericMinimum,
    };
  }

  return {
    fieldType: "string",
    editable: isEditablePath(entity, path),
    required: false,
  };
}

function buildSchemaField(entity, path, label, value) {
  const metadata = resolveFieldMetadata(entity, path, value);

  if (metadata.fieldType === "object") {
    return {
      path,
      label,
      value,
      fieldType: metadata.fieldType,
      editable: metadata.editable,
      required: metadata.required,
      children: Object.entries(value).map(([key, childValue]) =>
        buildSchemaField(entity, `${path}.${key}`, key, childValue),
      ),
    };
  }

  return {
    path,
    label,
    value,
    fieldType: metadata.fieldType,
    editable: metadata.editable,
    required: metadata.required,
    minimum: metadata.minimum ?? null,
    options: metadata.options ?? [],
    children: [],
  };
}

function buildIdentitySection(entity) {
  const fields = [
    buildSchemaField(entity, "name", "name", entity.name ?? ""),
    {
      path: "status",
      label: "status",
      value: entity.status ?? "",
      fieldType: "string",
      editable: false,
      required: false,
      options: [],
      minimum: null,
      children: [],
    },
    {
      path: "revision",
      label: "revision",
      value: entity.revision ?? "",
      fieldType: "string",
      editable: false,
      required: false,
      options: [],
      minimum: null,
      children: [],
    },
    buildSchemaField(entity, "tags", "tags", entity?.data?.tags ?? []),
  ];

  return {
    id: "identity",
    label: "Identite",
    fields,
  };
}

function buildSignalSection(entity) {
  if (entity?.entityType !== "Signal") {
    return null;
  }

  const fields = [
    buildSchemaField(entity, "kind", "kind", entity?.data?.kind ?? "boolean"),
    buildSchemaField(
      entity,
      "currentValue",
      "currentValue",
      entity?.data?.currentValue ?? false,
    ),
  ];

  if (entity?.data?.parameterSet?.unit !== undefined) {
    fields.push(
      buildSchemaField(
        entity,
        "parameterSet.unit",
        "unit",
        entity.data.parameterSet.unit,
      ),
    );
  }

  return {
    id: "signal",
    label: "Signal",
    fields,
  };
}

function buildParameterSection(entity) {
  const parameterSet = entity?.data?.parameterSet;
  if (!isObjectLike(parameterSet)) {
    return null;
  }

  return {
    id: "parameters",
    label: "Parametres",
    fields: Object.entries(parameterSet).map(([key, value]) =>
      buildSchemaField(entity, `parameterSet.${key}`, key, value),
    ),
  };
}

function buildReadOnlyDataSection(entity) {
  const ignoredKeys = new Set(["tags", "parameterSet"]);
  if (entity?.entityType === "Signal") {
    ignoredKeys.add("kind");
    ignoredKeys.add("currentValue");
  }

  const entries = Object.entries(entity?.data ?? {}).filter(
    ([key]) => !ignoredKeys.has(key),
  );
  if (entries.length === 0) {
    return null;
  }

  return {
    id: "data",
    label: "Donnees",
    fields: entries.map(([key, value]) => buildSchemaField(entity, key, key, value)),
  };
}

export function buildEntityInspectorSchema(entity) {
  if (!entity) {
    return { sections: [] };
  }

  return {
    sections: [
      buildIdentitySection(entity),
      buildSignalSection(entity),
      buildParameterSection(entity),
      buildReadOnlyDataSection(entity),
    ].filter(Boolean),
  };
}

export function flattenEditableInspectorFields(schema) {
  const fields = [];

  function visit(field) {
    if (!field) {
      return;
    }

    if (field.children?.length > 0) {
      field.children.forEach(visit);
      return;
    }

    if (field.editable) {
      fields.push(field);
    }
  }

  for (const section of schema?.sections ?? []) {
    for (const field of section.fields ?? []) {
      visit(field);
    }
  }

  return fields;
}

function serializeDraftValue(field) {
  if (field.fieldType === "tag-list") {
    return Array.isArray(field.value) ? field.value.join(", ") : "";
  }

  if (field.fieldType === "list") {
    return JSON.stringify(field.value ?? [], null, 2);
  }

  if (field.fieldType === "number") {
    return field.value === null || field.value === undefined ? "" : String(field.value);
  }

  return field.value ?? "";
}

export function buildInspectorDraftFromSchema(schema) {
  return Object.fromEntries(
    flattenEditableInspectorFields(schema).map((field) => [
      field.path,
      serializeDraftValue(field),
    ]),
  );
}

export function findInspectorFieldByPath(schema, path) {
  let match = null;

  function visit(field) {
    if (match || !field) {
      return;
    }

    if (field.path === path) {
      match = field;
      return;
    }

    for (const child of field.children ?? []) {
      visit(child);
    }
  }

  for (const section of schema?.sections ?? []) {
    for (const field of section.fields ?? []) {
      visit(field);
    }
  }

  return match;
}

export function coerceInspectorDraftValue(rawValue, field) {
  if (!field) {
    return {
      ok: false,
      error: "unknown field",
    };
  }

  if (field.fieldType === "boolean") {
    return {
      ok: true,
      value: Boolean(rawValue),
    };
  }

  if (field.fieldType === "number") {
    const parsed = Number(rawValue);
    if (!Number.isFinite(parsed)) {
      return {
        ok: false,
        error: `${field.label} must be a valid number`,
      };
    }
    if (field.minimum !== null && parsed < field.minimum) {
      return {
        ok: false,
        error: `${field.label} must be >= ${field.minimum}`,
      };
    }
    return {
      ok: true,
      value: parsed,
    };
  }

  if (field.fieldType === "enum") {
    const normalized = String(rawValue ?? "").trim();
    if (!field.options.includes(normalized)) {
      return {
        ok: false,
        error: `${field.label} must match an allowed option`,
      };
    }
    return {
      ok: true,
      value: normalized,
    };
  }

  if (field.fieldType === "tag-list") {
    return {
      ok: true,
      value: String(rawValue ?? "")
        .split(",")
        .map((entry) => entry.trim())
        .filter(Boolean),
    };
  }

  if (field.fieldType === "list") {
    try {
      const parsed = JSON.parse(String(rawValue ?? "[]"));
      if (!Array.isArray(parsed)) {
        return {
          ok: false,
          error: `${field.label} must stay a JSON array`,
        };
      }
      return {
        ok: true,
        value: parsed,
      };
    } catch {
      return {
        ok: false,
        error: `${field.label} must stay valid JSON`,
      };
    }
  }

  const normalized = String(rawValue ?? "").trim();
  if (field.required && normalized.length === 0) {
    return {
      ok: false,
      error: `${field.label} must stay non-empty`,
    };
  }

  return {
    ok: true,
    value: normalized,
  };
}
