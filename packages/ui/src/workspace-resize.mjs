export const WORKSPACE_COLLAPSED_WIDTH = 84;
export const WORKSPACE_RESIZER_WIDTH = 10;
export const WORKSPACE_CENTER_MIN_WIDTH = 480;

export const workspaceDockWidthLimits = {
  left: { min: 220, max: 460, default: 290 },
  right: { min: 260, max: 560, default: 320 }
};

export const defaultWorkspaceDockWidths = {
  left: workspaceDockWidthLimits.left.default,
  right: workspaceDockWidthLimits.right.default
};

export function getVisibleSidebarWidth(width, expanded) {
  return expanded ? width : WORKSPACE_COLLAPSED_WIDTH;
}

export function calculateResizedDockWidths({
  side,
  startWidths,
  deltaX,
  layoutWidth,
  leftExpanded,
  rightExpanded
}) {
  if (side !== "left" && side !== "right") {
    return startWidths;
  }

  if (side === "left") {
    return {
      ...startWidths,
      left: clampSidebarWidth({
        side,
        proposedWidth: startWidths.left + deltaX,
        layoutWidth,
        oppositeWidth: startWidths.right,
        oppositeExpanded: rightExpanded
      })
    };
  }

  return {
    ...startWidths,
    right: clampSidebarWidth({
      side,
      proposedWidth: startWidths.right - deltaX,
      layoutWidth,
      oppositeWidth: startWidths.left,
      oppositeExpanded: leftExpanded
    })
  };
}

function clampSidebarWidth({
  side,
  proposedWidth,
  layoutWidth,
  oppositeWidth,
  oppositeExpanded
}) {
  const limits = workspaceDockWidthLimits[side];
  if (!limits) {
    return proposedWidth;
  }

  const maxFromLayout = getMaxSidebarWidthFromLayout({
    side,
    layoutWidth,
    oppositeWidth,
    oppositeExpanded
  });
  return clamp(proposedWidth, limits.min, Math.min(limits.max, maxFromLayout));
}

function getMaxSidebarWidthFromLayout({
  side,
  layoutWidth,
  oppositeWidth,
  oppositeExpanded
}) {
  if (!Number.isFinite(layoutWidth) || layoutWidth <= 0) {
    return workspaceDockWidthLimits[side].max;
  }

  const visibleOppositeWidth = getVisibleSidebarWidth(oppositeWidth, oppositeExpanded);
  const reservedWidth =
    visibleOppositeWidth +
    WORKSPACE_CENTER_MIN_WIDTH +
    WORKSPACE_RESIZER_WIDTH * 2;

  return Math.max(workspaceDockWidthLimits[side].min, layoutWidth - reservedWidth);
}

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}
