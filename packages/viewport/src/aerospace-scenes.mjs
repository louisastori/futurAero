export const aerospaceReferenceScenes = [
  {
    id: "turbofan_exploded",
    category: "engine",
    titleKey: "ui.viewport.scene.turbofan.title",
    summaryKey: "ui.viewport.scene.turbofan.summary",
    analysisKeys: [
      "ui.viewport.scene.turbofan.analysis_1",
      "ui.viewport.scene.turbofan.analysis_2",
      "ui.viewport.scene.turbofan.analysis_3"
    ],
    legendKeys: [
      "ui.viewport.legend.primary_flow",
      "ui.viewport.legend.rotor_stage",
      "ui.viewport.legend.core_shaft"
    ],
    palette: {
      background: "#0f1622",
      accent: "#88c8ff",
      secondary: "#d9a3ff",
      tertiary: "#83ffc6"
    }
  },
  {
    id: "airframe_transparent",
    category: "airframe",
    titleKey: "ui.viewport.scene.airframe.title",
    summaryKey: "ui.viewport.scene.airframe.summary",
    analysisKeys: [
      "ui.viewport.scene.airframe.analysis_1",
      "ui.viewport.scene.airframe.analysis_2",
      "ui.viewport.scene.airframe.analysis_3"
    ],
    legendKeys: [
      "ui.viewport.legend.skin_shell",
      "ui.viewport.legend.internal_systems",
      "ui.viewport.legend.primary_structure"
    ],
    palette: {
      background: "#10153a",
      accent: "#ffe783",
      secondary: "#7cc7ff",
      tertiary: "#ff8cc8"
    }
  },
  {
    id: "wireframe_maintenance",
    category: "maintenance",
    titleKey: "ui.viewport.scene.wireframe.title",
    summaryKey: "ui.viewport.scene.wireframe.summary",
    analysisKeys: [
      "ui.viewport.scene.wireframe.analysis_1",
      "ui.viewport.scene.wireframe.analysis_2",
      "ui.viewport.scene.wireframe.analysis_3"
    ],
    legendKeys: [
      "ui.viewport.legend.reference_axis",
      "ui.viewport.legend.access_volume",
      "ui.viewport.legend.routing_bundle"
    ],
    palette: {
      background: "#111749",
      accent: "#f4ff8d",
      secondary: "#9cc4ff",
      tertiary: "#d4a6ff"
    }
  },
  {
    id: "stress_map",
    category: "simulation",
    titleKey: "ui.viewport.scene.stress.title",
    summaryKey: "ui.viewport.scene.stress.summary",
    analysisKeys: [
      "ui.viewport.scene.stress.analysis_1",
      "ui.viewport.scene.stress.analysis_2",
      "ui.viewport.scene.stress.analysis_3"
    ],
    legendKeys: [
      "ui.viewport.legend.stress_low",
      "ui.viewport.legend.stress_mid",
      "ui.viewport.legend.stress_high"
    ],
    palette: {
      background: "#18203a",
      accent: "#7fe7ff",
      secondary: "#ffcf7f",
      tertiary: "#ff6f82"
    }
  },
  {
    id: "aero_heatmap",
    category: "aerodynamics",
    titleKey: "ui.viewport.scene.aero.title",
    summaryKey: "ui.viewport.scene.aero.summary",
    analysisKeys: [
      "ui.viewport.scene.aero.analysis_1",
      "ui.viewport.scene.aero.analysis_2",
      "ui.viewport.scene.aero.analysis_3"
    ],
    legendKeys: [
      "ui.viewport.legend.flow_attached",
      "ui.viewport.legend.flow_transition",
      "ui.viewport.legend.flow_detached"
    ],
    palette: {
      background: "#272a3f",
      accent: "#9af98f",
      secondary: "#ffe166",
      tertiary: "#ff4dc4"
    }
  }
];

export const defaultAerospaceSceneId = aerospaceReferenceScenes[0].id;

export function getAerospaceScene(sceneId = defaultAerospaceSceneId) {
  return (
    aerospaceReferenceScenes.find((scene) => scene.id === sceneId) ??
    aerospaceReferenceScenes[0]
  );
}

export function getAerospaceSceneIds() {
  return aerospaceReferenceScenes.map((scene) => scene.id);
}
