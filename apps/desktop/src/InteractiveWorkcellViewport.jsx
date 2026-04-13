import { useMemo, useRef, useState } from "react";

const VIEWPORT_WIDTH = 1100;
const VIEWPORT_HEIGHT = 680;
const CAMERA_PRESETS = {
  iso: { yaw: -34, pitch: 24, zoom: 1, offsetX: 0, offsetY: 18 },
  front: { yaw: 0, pitch: 4, zoom: 1.08, offsetX: 0, offsetY: 16 },
  right: { yaw: -90, pitch: 6, zoom: 1.08, offsetX: 0, offsetY: 16 },
  top: { yaw: 0, pitch: 82, zoom: 1.22, offsetX: 0, offsetY: 18 },
};
const CAMERA_PRESET_ORDER = ["iso", "front", "right", "top"];
const ZOOM_MIN = 0.55;
const ZOOM_MAX = 2.6;
const ORBIT_SENSITIVITY = 0.32;
const PAN_SENSITIVITY = 1;
const UNIT_OPTIONS = {
  mm: { id: "mm", label: "mm", factor: 1, step: "1", precision: 0 },
  cm: { id: "cm", label: "cm", factor: 10, step: "0.1", precision: 1 },
  m: { id: "m", label: "m", factor: 1000, step: "0.01", precision: 2 },
  in: { id: "in", label: "in", factor: 25.4, step: "0.01", precision: 2 },
};
const SHAPE_OPTIONS = [
  {
    id: "block",
    label: "Bloc",
    description: "Volume simple pour une piece ou un colis.",
  },
  {
    id: "step",
    label: "Marche",
    description: "Forme a deux niveaux pour jouer avec les retraits.",
  },
  {
    id: "l_profile",
    label: "Profil L",
    description: "Section mecanique en equerre pour support ou chassis.",
  },
  {
    id: "portal",
    label: "Portique",
    description: "Deux montants et une traverse pour une cellule outillage.",
  },
];

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

function degToRad(value) {
  return (value * Math.PI) / 180;
}

function hexToRgb(hex) {
  const normalized = hex.replace("#", "");
  const expanded =
    normalized.length === 3
      ? normalized
          .split("")
          .map((entry) => `${entry}${entry}`)
          .join("")
      : normalized;

  return {
    r: Number.parseInt(expanded.slice(0, 2), 16),
    g: Number.parseInt(expanded.slice(2, 4), 16),
    b: Number.parseInt(expanded.slice(4, 6), 16),
  };
}

function shadeHex(hex, ratio) {
  const { r, g, b } = hexToRgb(hex);
  const nextChannel = (channel) =>
    clamp(
      Math.round(
        ratio >= 0
          ? channel + (255 - channel) * ratio
          : channel * (1 + ratio),
      ),
      0,
      255,
    );
  return `rgb(${nextChannel(r)}, ${nextChannel(g)}, ${nextChannel(b)})`;
}

function rotatePoint(point, yawDeg, pitchDeg) {
  const yaw = degToRad(yawDeg);
  const pitch = degToRad(pitchDeg);
  const cosYaw = Math.cos(yaw);
  const sinYaw = Math.sin(yaw);
  const cosPitch = Math.cos(pitch);
  const sinPitch = Math.sin(pitch);
  const yawX = point.x * cosYaw + point.z * sinYaw;
  const yawZ = -point.x * sinYaw + point.z * cosYaw;
  return {
    x: yawX,
    y: point.y * cosPitch - yawZ * sinPitch,
    z: point.y * sinPitch + yawZ * cosPitch,
  };
}

function projectPoint(point, camera) {
  const depth = 980 + point.z;
  const scale = (camera.zoom * 860) / depth;
  return {
    x: VIEWPORT_WIDTH / 2 + camera.offsetX + point.x * scale,
    y: VIEWPORT_HEIGHT / 2 + camera.offsetY - point.y * scale,
    z: point.z,
  };
}

function cuboidVertices(center, size) {
  const halfWidth = size.width / 2;
  const halfHeight = size.height / 2;
  const halfDepth = size.depth / 2;
  return {
    lbf: {
      x: center.x - halfWidth,
      y: center.y - halfHeight,
      z: center.z - halfDepth,
    },
    rbf: {
      x: center.x + halfWidth,
      y: center.y - halfHeight,
      z: center.z - halfDepth,
    },
    rtf: {
      x: center.x + halfWidth,
      y: center.y + halfHeight,
      z: center.z - halfDepth,
    },
    ltf: {
      x: center.x - halfWidth,
      y: center.y + halfHeight,
      z: center.z - halfDepth,
    },
    lbb: {
      x: center.x - halfWidth,
      y: center.y - halfHeight,
      z: center.z + halfDepth,
    },
    rbb: {
      x: center.x + halfWidth,
      y: center.y - halfHeight,
      z: center.z + halfDepth,
    },
    rtb: {
      x: center.x + halfWidth,
      y: center.y + halfHeight,
      z: center.z + halfDepth,
    },
    ltb: {
      x: center.x - halfWidth,
      y: center.y + halfHeight,
      z: center.z + halfDepth,
    },
  };
}

function collectSolidVertices(solid) {
  return Object.values(cuboidVertices(solid.center, solid.size));
}

function buildCuboidFaces(solid) {
  const points = cuboidVertices(solid.center, solid.size);
  return [
    {
      id: `${solid.id}-top`,
      color: shadeHex(solid.color, 0.22),
      stroke: shadeHex(solid.color, -0.18),
      normal: { x: 0, y: 1, z: 0 },
      points: [points.ltf, points.rtf, points.rtb, points.ltb],
      opacity: solid.opacity ?? 1,
    },
    {
      id: `${solid.id}-front`,
      color: shadeHex(solid.color, 0.02),
      stroke: shadeHex(solid.color, -0.14),
      normal: { x: 0, y: 0, z: -1 },
      points: [points.lbf, points.rbf, points.rtf, points.ltf],
      opacity: solid.opacity ?? 1,
    },
    {
      id: `${solid.id}-right`,
      color: shadeHex(solid.color, -0.14),
      stroke: shadeHex(solid.color, -0.22),
      normal: { x: 1, y: 0, z: 0 },
      points: [points.rbf, points.rbb, points.rtb, points.rtf],
      opacity: solid.opacity ?? 1,
    },
    {
      id: `${solid.id}-left`,
      color: shadeHex(solid.color, -0.08),
      stroke: shadeHex(solid.color, -0.2),
      normal: { x: -1, y: 0, z: 0 },
      points: [points.lbb, points.lbf, points.ltf, points.ltb],
      opacity: solid.opacity ?? 1,
    },
    {
      id: `${solid.id}-back`,
      color: shadeHex(solid.color, -0.2),
      stroke: shadeHex(solid.color, -0.26),
      normal: { x: 0, y: 0, z: 1 },
      points: [points.rbb, points.lbb, points.ltb, points.rtb],
      opacity: solid.opacity ?? 1,
    },
  ];
}

function buildFloorGrid() {
  const segments = [];
  for (let x = -420; x <= 420; x += 60) {
    segments.push({
      id: `grid-x-${x}`,
      start: { x, y: 0, z: -320 },
      end: { x, y: 0, z: 320 },
      color: "rgba(173, 208, 255, 0.16)",
      width: 1,
    });
  }
  for (let z = -300; z <= 300; z += 60) {
    segments.push({
      id: `grid-z-${z}`,
      start: { x: -440, y: 0, z },
      end: { x: 440, y: 0, z },
      color: "rgba(173, 208, 255, 0.16)",
      width: 1,
    });
  }
  return segments;
}

function buildSafetyCage() {
  const corners = [
    { x: -380, z: -260 },
    { x: 380, z: -260 },
    { x: 380, z: 260 },
    { x: -380, z: 260 },
  ];
  const segments = [];
  corners.forEach((corner, index) => {
    const nextCorner = corners[(index + 1) % corners.length];
    segments.push({
      id: `cage-bottom-${index}`,
      start: { x: corner.x, y: 0, z: corner.z },
      end: { x: nextCorner.x, y: 0, z: nextCorner.z },
      color: "rgba(111, 172, 255, 0.32)",
      width: 2,
    });
    segments.push({
      id: `cage-top-${index}`,
      start: { x: corner.x, y: 150, z: corner.z },
      end: { x: nextCorner.x, y: 150, z: nextCorner.z },
      color: "rgba(111, 172, 255, 0.28)",
      width: 2,
    });
    segments.push({
      id: `cage-post-${index}`,
      start: { x: corner.x, y: 0, z: corner.z },
      end: { x: corner.x, y: 150, z: corner.z },
      color: "rgba(111, 172, 255, 0.3)",
      width: 2,
    });
  });
  return segments;
}

function buildTransferPath() {
  return [
    { x: -220, y: 82, z: -128 },
    { x: -90, y: 138, z: -92 },
    { x: 12, y: 194, z: -16 },
    { x: 128, y: 176, z: 46 },
    { x: 244, y: 122, z: 74 },
    { x: 330, y: 90, z: 80 },
  ].map((point, index) => ({ point, id: `transfer-${index}` }));
}

function buildPrimitiveAssembly(primitive) {
  const width = primitive.widthMm;
  const height = primitive.heightMm;
  const depth = primitive.depthMm;
  const anchor = { x: 340, y: 60, z: 82 };
  const solids = [];

  if (primitive.shapeId === "step") {
    const lowerHeight = Math.max(24, height * 0.46);
    const upperHeight = Math.max(18, height - lowerHeight);
    solids.push({
      id: "play-primitive-lower",
      center: { x: anchor.x, y: anchor.y + lowerHeight / 2, z: anchor.z },
      size: { width, height: lowerHeight, depth },
      color: "#ffb860",
    });
    solids.push({
      id: "play-primitive-upper",
      center: {
        x: anchor.x + width * 0.12,
        y: anchor.y + lowerHeight + upperHeight / 2,
        z: anchor.z - depth * 0.08,
      },
      size: {
        width: Math.max(26, width * 0.58),
        height: upperHeight,
        depth: Math.max(24, depth * 0.56),
      },
      color: "#ffd08d",
    });
  } else if (primitive.shapeId === "l_profile") {
    const legWidth = Math.max(20, width * 0.28);
    const footHeight = Math.max(16, height * 0.22);
    solids.push({
      id: "play-primitive-leg",
      center: {
        x: anchor.x - width / 2 + legWidth / 2,
        y: anchor.y + height / 2,
        z: anchor.z,
      },
      size: { width: legWidth, height, depth },
      color: "#ffb860",
    });
    solids.push({
      id: "play-primitive-foot",
      center: { x: anchor.x, y: anchor.y + footHeight / 2, z: anchor.z },
      size: { width, height: footHeight, depth },
      color: "#ffd08d",
    });
  } else if (primitive.shapeId === "portal") {
    const legWidth = Math.max(18, width * 0.18);
    const beamHeight = Math.max(18, height * 0.18);
    const legHeight = Math.max(28, height - beamHeight);
    solids.push({
      id: "play-primitive-left-leg",
      center: {
        x: anchor.x - width / 2 + legWidth / 2,
        y: anchor.y + legHeight / 2,
        z: anchor.z,
      },
      size: { width: legWidth, height: legHeight, depth },
      color: "#ffb860",
    });
    solids.push({
      id: "play-primitive-right-leg",
      center: {
        x: anchor.x + width / 2 - legWidth / 2,
        y: anchor.y + legHeight / 2,
        z: anchor.z,
      },
      size: { width: legWidth, height: legHeight, depth },
      color: "#ffb860",
    });
    solids.push({
      id: "play-primitive-beam",
      center: {
        x: anchor.x,
        y: anchor.y + legHeight + beamHeight / 2,
        z: anchor.z,
      },
      size: { width, height: beamHeight, depth },
      color: "#ffd08d",
    });
  } else {
    solids.push({
      id: "play-primitive-block",
      center: { x: anchor.x, y: anchor.y + height / 2, z: anchor.z },
      size: { width, height, depth },
      color: "#ffb860",
    });
  }

  return {
    solids,
    vertices: solids.flatMap((solid) => collectSolidVertices(solid)),
  };
}

function buildWorkcellGeometry(primitive) {
  const primitiveAssembly = buildPrimitiveAssembly(primitive);
  return {
    solids: [
      {
        id: "floor",
        center: { x: 0, y: -12, z: 0 },
        size: { width: 900, height: 24, depth: 680 },
        color: "#233548",
      },
      {
        id: "machine-base",
        center: { x: 0, y: 36, z: -12 },
        size: { width: 220, height: 72, depth: 180 },
        color: "#5d7fe0",
      },
      {
        id: "machine-head",
        center: { x: 0, y: 116, z: -8 },
        size: { width: 96, height: 88, depth: 96 },
        color: "#7f9eff",
      },
      {
        id: "feeder-station",
        center: { x: -255, y: 28, z: -126 },
        size: { width: 170, height: 56, depth: 140 },
        color: "#61cbb9",
      },
      {
        id: "feeder-tray",
        center: { x: -260, y: 64, z: -126 },
        size: { width: 102, height: 44, depth: 92 },
        color: "#d9b564",
      },
      {
        id: "inspection-post",
        center: { x: -118, y: 68, z: 138 },
        size: { width: 72, height: 136, depth: 44 },
        color: "#839ec6",
      },
      {
        id: "robot-base",
        center: { x: 56, y: 28, z: 150 },
        size: { width: 92, height: 56, depth: 92 },
        color: "#404d61",
      },
      {
        id: "robot-arm-1",
        center: { x: 136, y: 96, z: 116 },
        size: { width: 160, height: 22, depth: 26 },
        color: "#d8e2f4",
      },
      {
        id: "robot-arm-2",
        center: { x: 208, y: 154, z: 58 },
        size: { width: 28, height: 116, depth: 28 },
        color: "#c3d0e5",
      },
      {
        id: "robot-gripper",
        center: { x: 226, y: 214, z: 12 },
        size: { width: 108, height: 18, depth: 18 },
        color: "#eef3ff",
      },
      {
        id: "conveyor",
        center: { x: 290, y: 26, z: 82 },
        size: { width: 292, height: 52, depth: 116 },
        color: "#995ff2",
      },
      {
        id: "conveyor-top",
        center: { x: 290, y: 60, z: 82 },
        size: { width: 274, height: 10, depth: 94 },
        color: "#c38cff",
      },
      {
        id: "buffer",
        center: { x: 286, y: 28, z: -178 },
        size: { width: 126, height: 56, depth: 120 },
        color: "#8bc8e8",
      },
      ...primitiveAssembly.solids,
    ],
    primitiveVertices: primitiveAssembly.vertices,
    grid: buildFloorGrid(),
    cage: buildSafetyCage(),
    transferPath: buildTransferPath(),
  };
}

function segmentDepth(segment, camera) {
  const start = rotatePoint(segment.start, camera.yaw, camera.pitch);
  const end = rotatePoint(segment.end, camera.yaw, camera.pitch);
  return (start.z + end.z) / 2;
}

function renderScene(geometry, camera) {
  const faces = geometry.solids
    .flatMap((solid) => buildCuboidFaces(solid))
    .map((face) => {
      const rotatedPoints = face.points.map((point) =>
        rotatePoint(point, camera.yaw, camera.pitch),
      );
      const rotatedNormal = rotatePoint(face.normal, camera.yaw, camera.pitch);
      if (rotatedNormal.z >= 0) {
        return null;
      }

      const projectedPoints = rotatedPoints.map((point) =>
        projectPoint(point, camera),
      );

      return {
        ...face,
        depth:
          rotatedPoints.reduce((sum, point) => sum + point.z, 0) /
          rotatedPoints.length,
        projectedPath: projectedPoints
          .map((point) => `${point.x},${point.y}`)
          .join(" "),
      };
    })
    .filter(Boolean)
    .sort((left, right) => right.depth - left.depth);

  const projectSegment = (segment) => {
    const rotatedStart = rotatePoint(segment.start, camera.yaw, camera.pitch);
    const rotatedEnd = rotatePoint(segment.end, camera.yaw, camera.pitch);
    return {
      ...segment,
      depth: segmentDepth(segment, camera),
      start: projectPoint(rotatedStart, camera),
      end: projectPoint(rotatedEnd, camera),
    };
  };

  const gridSegments = geometry.grid
    .map(projectSegment)
    .sort((left, right) => right.depth - left.depth);
  const cageSegments = geometry.cage
    .map(projectSegment)
    .sort((left, right) => right.depth - left.depth);

  const transferPath = geometry.transferPath.map(({ id, point }) => {
    const rotatedPoint = rotatePoint(point, camera.yaw, camera.pitch);
    return {
      id,
      point: projectPoint(rotatedPoint, camera),
      depth: rotatedPoint.z,
    };
  });

  let primitiveBounds = null;
  if (geometry.primitiveVertices.length > 0) {
    const projectedPrimitive = geometry.primitiveVertices.map((point) =>
      projectPoint(rotatePoint(point, camera.yaw, camera.pitch), camera),
    );
    const xs = projectedPrimitive.map((point) => point.x);
    const ys = projectedPrimitive.map((point) => point.y);
    primitiveBounds = {
      minX: Math.min(...xs),
      maxX: Math.max(...xs),
      minY: Math.min(...ys),
      maxY: Math.max(...ys),
    };
  }

  const axisOriginWorld = { x: -360, y: 0, z: 250 };
  const axisOrigin = projectPoint(
    rotatePoint(axisOriginWorld, camera.yaw, camera.pitch),
    camera,
  );
  const axisVectors = [
    {
      id: "x",
      label: "X",
      color: "rgba(255, 128, 128, 0.95)",
      world: {
        x: axisOriginWorld.x + 120,
        y: axisOriginWorld.y,
        z: axisOriginWorld.z,
      },
    },
    {
      id: "y",
      label: "Y",
      color: "rgba(142, 255, 196, 0.95)",
      world: {
        x: axisOriginWorld.x,
        y: axisOriginWorld.y + 120,
        z: axisOriginWorld.z,
      },
    },
    {
      id: "z",
      label: "Z",
      color: "rgba(131, 196, 255, 0.95)",
      world: {
        x: axisOriginWorld.x,
        y: axisOriginWorld.y,
        z: axisOriginWorld.z + 120,
      },
    },
  ].map((axis) => ({
    ...axis,
    end: projectPoint(rotatePoint(axis.world, camera.yaw, camera.pitch), camera),
  }));

  return {
    faces,
    gridSegments,
    cageSegments,
    transferPath,
    primitiveBounds,
    axisOrigin,
    axisVectors,
  };
}

function formatCameraState(camera) {
  return [
    `yaw ${camera.yaw.toFixed(1)}deg`,
    `pitch ${camera.pitch.toFixed(1)}deg`,
    `zoom ${camera.zoom.toFixed(2)}x`,
    `pan ${camera.offsetX.toFixed(0)},${camera.offsetY.toFixed(0)}`,
  ].join(" | ");
}

function formatUnitValue(valueMm, unitId) {
  const unit = UNIT_OPTIONS[unitId] ?? UNIT_OPTIONS.mm;
  return (valueMm / unit.factor).toFixed(unit.precision);
}

function parseUnitValue(rawValue, unitId) {
  const numericValue = Number.parseFloat(String(rawValue).replace(",", "."));
  if (!Number.isFinite(numericValue)) {
    return Number.NaN;
  }

  const unit = UNIT_OPTIONS[unitId] ?? UNIT_OPTIONS.mm;
  return numericValue * unit.factor;
}

function formatMeasurement(valueMm, unitId) {
  const unit = UNIT_OPTIONS[unitId] ?? UNIT_OPTIONS.mm;
  return `${formatUnitValue(valueMm, unitId)} ${unit.label}`;
}

function formatVolume(widthMm, heightMm, depthMm, unitId) {
  const unit = UNIT_OPTIONS[unitId] ?? UNIT_OPTIONS.mm;
  const cubicValue =
    (widthMm * heightMm * depthMm) / (unit.factor * unit.factor * unit.factor);
  const precision = unit.id === "mm" ? 0 : 2;
  return `${cubicValue.toFixed(precision)} ${unit.label}^3`;
}

function shapeOptionLabel(shapeId) {
  return (
    SHAPE_OPTIONS.find((option) => option.id === shapeId)?.label ?? "Bloc"
  );
}

export default function InteractiveWorkcellViewport({ sceneId }) {
  const [camera, setCamera] = useState({ ...CAMERA_PRESETS.iso });
  const [activePreset, setActivePreset] = useState("iso");
  const [primitive, setPrimitive] = useState({
    shapeId: "block",
    unitId: "mm",
    widthMm: 160,
    heightMm: 110,
    depthMm: 120,
  });
  const dragStateRef = useRef(null);

  const unitOption = UNIT_OPTIONS[primitive.unitId] ?? UNIT_OPTIONS.mm;
  const shapeMeta =
    SHAPE_OPTIONS.find((option) => option.id === primitive.shapeId) ??
    SHAPE_OPTIONS[0];
  const geometry = useMemo(() => buildWorkcellGeometry(primitive), [primitive]);
  const renderedScene = useMemo(
    () => renderScene(geometry, camera),
    [geometry, camera],
  );
  const cameraState = formatCameraState(camera);
  const measureSummary = [
    shapeOptionLabel(primitive.shapeId),
    `L ${formatMeasurement(primitive.widthMm, primitive.unitId)}`,
    `H ${formatMeasurement(primitive.heightMm, primitive.unitId)}`,
    `P ${formatMeasurement(primitive.depthMm, primitive.unitId)}`,
  ].join(" | ");

  function applyPreset(presetId) {
    setCamera({ ...CAMERA_PRESETS[presetId] });
    setActivePreset(presetId);
  }

  function updateCamera(recipe) {
    setCamera((currentCamera) => recipe(currentCamera));
    setActivePreset("custom");
  }

  function adjustZoom(delta) {
    updateCamera((currentCamera) => ({
      ...currentCamera,
      zoom: clamp(currentCamera.zoom + delta, ZOOM_MIN, ZOOM_MAX),
    }));
  }

  function updatePrimitiveDimension(dimensionKey, rawValue) {
    const nextValue = parseUnitValue(rawValue, primitive.unitId);
    if (!Number.isFinite(nextValue)) {
      return;
    }

    setPrimitive((currentPrimitive) => ({
      ...currentPrimitive,
      [dimensionKey]: clamp(nextValue, 25, 1200),
    }));
  }

  function handleWheel(event) {
    event.preventDefault();
    adjustZoom(event.deltaY < 0 ? 0.08 : -0.08);
  }

  function handlePointerDown(event) {
    if (event.button !== 0 && event.button !== 2) {
      return;
    }

    event.preventDefault();
    dragStateRef.current = {
      pointerId: event.pointerId,
      mode: event.button === 2 || event.shiftKey ? "pan" : "orbit",
      startX: event.clientX,
      startY: event.clientY,
      startCamera: camera,
    };
    event.currentTarget.setPointerCapture?.(event.pointerId);
  }

  function handlePointerMove(event) {
    const dragState = dragStateRef.current;
    if (!dragState || dragState.pointerId !== event.pointerId) {
      return;
    }

    const deltaX = event.clientX - dragState.startX;
    const deltaY = event.clientY - dragState.startY;
    if (dragState.mode === "pan") {
      updateCamera(() => ({
        ...dragState.startCamera,
        offsetX: dragState.startCamera.offsetX + deltaX * PAN_SENSITIVITY,
        offsetY: dragState.startCamera.offsetY + deltaY * PAN_SENSITIVITY,
      }));
      return;
    }

    updateCamera(() => ({
      ...dragState.startCamera,
      yaw: dragState.startCamera.yaw + deltaX * ORBIT_SENSITIVITY,
      pitch: clamp(
        dragState.startCamera.pitch - deltaY * ORBIT_SENSITIVITY,
        -88,
        88,
      ),
    }));
  }

  function endPointerInteraction(event) {
    if (
      dragStateRef.current &&
      (event == null || dragStateRef.current.pointerId === event.pointerId)
    ) {
      event?.currentTarget?.releasePointerCapture?.(event.pointerId);
      dragStateRef.current = null;
    }
  }

  function handleViewportKeyDown(event) {
    if (event.key === "Home" || event.key === "0") {
      event.preventDefault();
      applyPreset("iso");
      return;
    }

    if (event.key === "=" || event.key === "+") {
      event.preventDefault();
      adjustZoom(0.08);
      return;
    }

    if (event.key === "-" || event.key === "_") {
      event.preventDefault();
      adjustZoom(-0.08);
      return;
    }

    if (!event.key.startsWith("Arrow")) {
      return;
    }

    event.preventDefault();
    updateCamera((currentCamera) => ({
      ...currentCamera,
      yaw:
        currentCamera.yaw +
        (event.key === "ArrowLeft" ? -6 : event.key === "ArrowRight" ? 6 : 0),
      pitch: clamp(
        currentCamera.pitch +
          (event.key === "ArrowUp" ? 6 : event.key === "ArrowDown" ? -6 : 0),
        -88,
        88,
      ),
    }));
  }

  const dimensionLabelY = renderedScene.primitiveBounds
    ? clamp(renderedScene.primitiveBounds.maxY + 34, 40, VIEWPORT_HEIGHT - 22)
    : 0;
  const dimensionLabelX = renderedScene.primitiveBounds
    ? clamp(renderedScene.primitiveBounds.maxX + 30, 40, VIEWPORT_WIDTH - 40)
    : 0;
  const depthLabelY = renderedScene.primitiveBounds
    ? clamp(renderedScene.primitiveBounds.minY - 16, 28, VIEWPORT_HEIGHT - 36)
    : 0;

  return (
    <div
      className="viewport-workbench"
      data-viewport-active-preset={activePreset}
    >
      <div className="viewport-toolbar">
        <div className="viewport-action-group">
          {CAMERA_PRESET_ORDER.map((presetId) => (
            <button
              key={presetId}
              type="button"
              className={`viewport-action ${
                activePreset === presetId ? "is-active" : ""
              }`}
              data-viewport-preset={presetId}
              onClick={() => applyPreset(presetId)}
            >
              {presetId === "iso"
                ? "Isometrique"
                : presetId === "front"
                  ? "Face"
                  : presetId === "right"
                    ? "Droite"
                    : "Dessus"}
            </button>
          ))}
        </div>
        <div className="viewport-action-group">
          <button
            type="button"
            className="viewport-action"
            data-viewport-zoom="out"
            onClick={() => adjustZoom(-0.08)}
          >
            Zoom -
          </button>
          <button
            type="button"
            className="viewport-action"
            data-viewport-zoom="in"
            onClick={() => adjustZoom(0.08)}
          >
            Zoom +
          </button>
          <button
            type="button"
            className="viewport-action"
            data-viewport-reset="true"
            onClick={() => applyPreset("iso")}
          >
            Reset
          </button>
        </div>
      </div>

      <div className="viewport-stage-shell">
        <div className="viewport-stage-header">
          <div className="viewport-stage-title-block">
            <span className="panel-accent">Workcell parametrique</span>
            <strong className="viewport-workcell-title">{sceneId}</strong>
          </div>
          <div
            className="viewport-camera-state status-pill"
            data-viewport-camera-state={cameraState}
          >
            {cameraState}
          </div>
        </div>

        <div className="viewport-interactive-surface">
          <div className="viewport-hud viewport-hud-left">
            <div className="viewport-hud-card">
              <strong>Navigation</strong>
              <span>Glisser: orbite</span>
              <span>Shift ou clic droit: deplacement</span>
              <span>Molette: zoom</span>
            </div>
          </div>

          <div className="viewport-hud viewport-hud-right">
            <div className="viewport-hud-card">
              <strong data-viewport-shape-name={shapeMeta.label}>
                {shapeMeta.label}
              </strong>
              <span>{shapeMeta.description}</span>
              <span>{measureSummary}</span>
            </div>
          </div>

          <svg
            className="viewport-stage-svg"
            data-viewport-canvas="true"
            viewBox={`0 0 ${VIEWPORT_WIDTH} ${VIEWPORT_HEIGHT}`}
            onContextMenu={(event) => event.preventDefault()}
            onKeyDown={handleViewportKeyDown}
            onPointerCancel={endPointerInteraction}
            onPointerDown={handlePointerDown}
            onPointerLeave={endPointerInteraction}
            onPointerMove={handlePointerMove}
            onPointerUp={endPointerInteraction}
            onWheel={handleWheel}
            role="img"
            tabIndex={0}
            aria-label="Viewport 3D interactif"
          >
            <defs>
              <linearGradient
                id="viewport-floor-glow"
                x1="0%"
                x2="100%"
                y1="0%"
                y2="100%"
              >
                <stop offset="0%" stopColor="rgba(47, 103, 255, 0.14)" />
                <stop offset="100%" stopColor="rgba(10, 18, 30, 0.02)" />
              </linearGradient>
              <filter
                id="viewport-soft-shadow"
                x="-20%"
                y="-20%"
                width="140%"
                height="140%"
              >
                <feDropShadow
                  dx="0"
                  dy="18"
                  stdDeviation="14"
                  floodColor="rgba(0, 0, 0, 0.36)"
                />
              </filter>
            </defs>

            <rect
              x="0"
              y="0"
              width={VIEWPORT_WIDTH}
              height={VIEWPORT_HEIGHT}
              fill="url(#viewport-floor-glow)"
            />

            <ellipse
              cx="610"
              cy="566"
              rx="292"
              ry="72"
              fill="rgba(72, 133, 255, 0.08)"
            />

            <g opacity="0.9">
              {renderedScene.gridSegments.map((segment) => (
                <line
                  key={segment.id}
                  x1={segment.start.x}
                  y1={segment.start.y}
                  x2={segment.end.x}
                  y2={segment.end.y}
                  stroke={segment.color}
                  strokeWidth={segment.width}
                />
              ))}
            </g>

            <g opacity="0.62">
              {renderedScene.cageSegments.map((segment) => (
                <line
                  key={segment.id}
                  x1={segment.start.x}
                  y1={segment.start.y}
                  x2={segment.end.x}
                  y2={segment.end.y}
                  stroke={segment.color}
                  strokeWidth={segment.width}
                />
              ))}
            </g>

            <g filter="url(#viewport-soft-shadow)">
              {renderedScene.faces.map((face) => (
                <polygon
                  key={face.id}
                  fill={face.color}
                  fillOpacity={face.opacity}
                  points={face.projectedPath}
                  stroke={face.stroke}
                  strokeWidth="1.4"
                />
              ))}
            </g>

            <g>
              <polyline
                points={renderedScene.transferPath
                  .map(({ point }) => `${point.x},${point.y}`)
                  .join(" ")}
                fill="none"
                opacity="0.9"
                stroke="rgba(255, 178, 122, 0.72)"
                strokeDasharray="10 10"
                strokeWidth="3"
              />
              {renderedScene.transferPath.map(({ id, point }) => (
                <circle
                  key={id}
                  cx={point.x}
                  cy={point.y}
                  fill="rgba(255, 220, 160, 0.92)"
                  r="4.5"
                  stroke="rgba(20, 29, 45, 0.78)"
                  strokeWidth="1.5"
                />
              ))}
            </g>

            <g>
              {renderedScene.axisVectors.map((axis) => (
                <g key={axis.id}>
                  <line
                    x1={renderedScene.axisOrigin.x}
                    y1={renderedScene.axisOrigin.y}
                    x2={axis.end.x}
                    y2={axis.end.y}
                    stroke={axis.color}
                    strokeWidth="3"
                    strokeLinecap="round"
                  />
                  <text
                    x={axis.end.x + 8}
                    y={axis.end.y - 6}
                    fill={axis.color}
                    fontSize="14"
                    fontWeight="700"
                  >
                    {axis.label}
                  </text>
                </g>
              ))}
            </g>

            {renderedScene.primitiveBounds ? (
              <g className="viewport-dimension-overlay">
                <line
                  x1={renderedScene.primitiveBounds.minX}
                  y1={renderedScene.primitiveBounds.maxY}
                  x2={renderedScene.primitiveBounds.minX}
                  y2={dimensionLabelY - 10}
                />
                <line
                  x1={renderedScene.primitiveBounds.maxX}
                  y1={renderedScene.primitiveBounds.maxY}
                  x2={renderedScene.primitiveBounds.maxX}
                  y2={dimensionLabelY - 10}
                />
                <line
                  x1={renderedScene.primitiveBounds.minX}
                  y1={dimensionLabelY}
                  x2={renderedScene.primitiveBounds.maxX}
                  y2={dimensionLabelY}
                />
                <text
                  x={
                    (renderedScene.primitiveBounds.minX +
                      renderedScene.primitiveBounds.maxX) /
                    2
                  }
                  y={dimensionLabelY - 12}
                  textAnchor="middle"
                >
                  Largeur {formatMeasurement(primitive.widthMm, primitive.unitId)}
                </text>

                <line
                  x1={renderedScene.primitiveBounds.maxX}
                  y1={renderedScene.primitiveBounds.minY}
                  x2={dimensionLabelX - 8}
                  y2={renderedScene.primitiveBounds.minY}
                />
                <line
                  x1={renderedScene.primitiveBounds.maxX}
                  y1={renderedScene.primitiveBounds.maxY}
                  x2={dimensionLabelX - 8}
                  y2={renderedScene.primitiveBounds.maxY}
                />
                <line
                  x1={dimensionLabelX}
                  y1={renderedScene.primitiveBounds.minY}
                  x2={dimensionLabelX}
                  y2={renderedScene.primitiveBounds.maxY}
                />
                <text
                  x={dimensionLabelX + 8}
                  y={
                    (renderedScene.primitiveBounds.minY +
                      renderedScene.primitiveBounds.maxY) /
                    2
                  }
                >
                  Hauteur {formatMeasurement(primitive.heightMm, primitive.unitId)}
                </text>

                <text
                  x={renderedScene.primitiveBounds.maxX - 8}
                  y={depthLabelY}
                  textAnchor="end"
                >
                  Profondeur {formatMeasurement(primitive.depthMm, primitive.unitId)}
                </text>
              </g>
            ) : null}
          </svg>
        </div>
      </div>

      <div className="viewport-configurator-grid">
        <section className="viewport-config-card">
          <span className="panel-accent">Edition geometrique</span>
          <strong>{shapeMeta.label}</strong>
          <span className="viewport-shape-description">
            {shapeMeta.description}
          </span>

          <div className="viewport-control-grid viewport-control-grid-compact">
            <label className="control-group">
              <span>Forme</span>
              <select
                className="shell-select"
                data-viewport-shape-select="true"
                value={primitive.shapeId}
                onChange={(event) =>
                  setPrimitive((currentPrimitive) => ({
                    ...currentPrimitive,
                    shapeId: event.target.value,
                  }))
                }
              >
                {SHAPE_OPTIONS.map((option) => (
                  <option key={option.id} value={option.id}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>

            <label className="control-group">
              <span>Unite</span>
              <select
                className="shell-select"
                data-viewport-unit-select="true"
                value={primitive.unitId}
                onChange={(event) =>
                  setPrimitive((currentPrimitive) => ({
                    ...currentPrimitive,
                    unitId: event.target.value,
                  }))
                }
              >
                {Object.values(UNIT_OPTIONS).map((unit) => (
                  <option key={unit.id} value={unit.id}>
                    {unit.label}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <div className="viewport-control-grid">
            <label className="control-group">
              <span>Largeur forme</span>
              <input
                className="shell-select shell-input"
                data-viewport-dimension="width"
                min="0"
                step={unitOption.step}
                type="number"
                value={formatUnitValue(primitive.widthMm, primitive.unitId)}
                onChange={(event) =>
                  updatePrimitiveDimension("widthMm", event.target.value)
                }
              />
            </label>

            <label className="control-group">
              <span>Hauteur forme</span>
              <input
                className="shell-select shell-input"
                data-viewport-dimension="height"
                min="0"
                step={unitOption.step}
                type="number"
                value={formatUnitValue(primitive.heightMm, primitive.unitId)}
                onChange={(event) =>
                  updatePrimitiveDimension("heightMm", event.target.value)
                }
              />
            </label>

            <label className="control-group">
              <span>Profondeur forme</span>
              <input
                className="shell-select shell-input"
                data-viewport-dimension="depth"
                min="0"
                step={unitOption.step}
                type="number"
                value={formatUnitValue(primitive.depthMm, primitive.unitId)}
                onChange={(event) =>
                  updatePrimitiveDimension("depthMm", event.target.value)
                }
              />
            </label>
          </div>
        </section>

        <section className="viewport-config-card">
          <span className="panel-accent">Mesures actives</span>
          <strong>Resume dimensionnel</strong>
          <div
            className="viewport-measure-highlight"
            data-viewport-measure-summary={measureSummary}
          >
            {measureSummary}
          </div>
          <div className="viewport-measure-list">
            <div>
              <span>Largeur</span>
              <strong>{formatMeasurement(primitive.widthMm, primitive.unitId)}</strong>
            </div>
            <div>
              <span>Hauteur</span>
              <strong>{formatMeasurement(primitive.heightMm, primitive.unitId)}</strong>
            </div>
            <div>
              <span>Profondeur</span>
              <strong>{formatMeasurement(primitive.depthMm, primitive.unitId)}</strong>
            </div>
            <div>
              <span>Volume</span>
              <strong>
                {formatVolume(
                  primitive.widthMm,
                  primitive.heightMm,
                  primitive.depthMm,
                  primitive.unitId,
                )}
              </strong>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
