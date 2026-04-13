# Proposal: Robot Cell Signals And Control MVP

## Summary

Implement `ST-501` by formalizing the existing robot-cell control scaffold into a shared MVP contract for typed signals, explicit controller transitions, and deterministic blocked-sequence detection.

## Motivation

`ST-402` delivered a structured robot cell with editable targets and a minimal pick-and-place sequence, and the desktop shell already seeds sample `Signal` and `Controller` entities plus a simple simulation stub. However, that control graph still lives as loosely coupled shell data: `faero-robotics` does not validate it, the fallback duplicates it, and there is no dedicated OpenSpec capability describing what "signals and minimal control" means before the full simulation runner lands.

## What Changes

- introduce a shared robot-cell control model covering typed signals, controller states, explicit transition conditions, and lightweight blocked-sequence validation
- align robot-cell creation and property updates so the Tauri backend persists and recomputes control summaries from explicit support entities
- make the minimal control contract reusable by `faero-sim` and the web fallback instead of keeping shell-only ad hoc structures
- expose a readable control summary in robot-cell snapshots, including signal counts, transition counts, and blocked state information

## Out of Scope

- asynchronous simulation job orchestration from `ST-502`
- detailed PLC or robot-controller protocol integration
- a dedicated visual editor for controller graphs

## Capabilities

### New Capabilities

- `robot-cell-signals-control`: typed robot-cell signals, explicit controller transitions, and deterministic blocked-sequence detection for the MVP cell workflow

### Modified Capabilities

- None.

## Impact

- `crates/faero-robotics`
- `crates/faero-sim`
- `apps/desktop/src-tauri`
- `apps/desktop/src`
