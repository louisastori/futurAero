# Proposal: Robot Cell Target Sequence MVP

## Summary

Implement `ST-402` by turning MVP robot-cell targets into explicit versioned support entities, making simple target order editable, and exposing a readable ordered preview in the desktop shell.

## Motivation

`ST-401` delivered a structured `RobotCell`, but the sequence targets still live as embedded arrays inside `RobotCell` and `RobotSequence`. That keeps the MVP readable, but it does not yet satisfy the backlog requirement that target points are versioned, that sequence order is modifiable, and that the shell exposes a minimal target visualization.

## What Changes

- persist robot-cell targets as explicit `RobotTarget` support entities linked to the cell and the sequence
- promote ordered `targetIds` from synthetic values to real entity ids
- make target order and target pose/speed values editable through the existing property inspector flow
- recompute `RobotSequence` and `RobotCell` summaries when a target changes
- expose a minimal ordered target preview in the desktop shell and the web fallback

## Out of Scope

- full trajectory editing UI
- dedicated viewport overlays or 3D manipulators for targets
- multi-sequence robot cells or branching sequence logic

## Impact

- `crates/faero-robotics`
- `apps/desktop/src-tauri`
- `apps/desktop/src`
