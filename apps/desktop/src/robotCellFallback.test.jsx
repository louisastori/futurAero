import assert from "node:assert/strict";
import { describe, test } from "vitest";

import {
  buildFallbackRobotCellBundle,
  syncFallbackRobotCellControl,
} from "./robotCellFallback.js";

describe("robotCellFallback control parity", () => {
  test("fallback bundle exposes the backend control summary contract", () => {
    const entities = buildFallbackRobotCellBundle(1);
    const robotCell = entities.find((entity) => entity.entityType === "RobotCell");

    assert.ok(robotCell);
    assert.equal(robotCell.robotCellSummary.signalCount, 5);
    assert.equal(robotCell.robotCellSummary.controllerTransitionCount, 3);
    assert.equal(robotCell.robotCellSummary.blockedSequenceDetected, true);
    assert.equal(robotCell.robotCellSummary.blockedStateId, "idle");
    assert.equal(robotCell.data.control.states.length, 4);
    assert.equal(robotCell.data.control.transitions.length, 3);
  });

  test("fallback control summary recomputes from signal and controller entities", () => {
    let entities = buildFallbackRobotCellBundle(1);

    const cycleStart = entities.find((entity) => entity.id === "ent_sig_001_cycle_start");
    cycleStart.data.currentValue = true;
    entities = syncFallbackRobotCellControl(entities, cycleStart.id);

    let robotCell = entities.find((entity) => entity.id === "ent_cell_001");
    assert.equal(robotCell.robotCellSummary.blockedStateId, "place");

    const progressGate = entities.find(
      (entity) => entity.id === "ent_sig_001_progress_gate",
    );
    progressGate.data.currentValue = 1.0;
    entities = syncFallbackRobotCellControl(entities, progressGate.id);

    robotCell = entities.find((entity) => entity.id === "ent_cell_001");
    assert.equal(robotCell.robotCellSummary.blockedSequenceDetected, false);
    assert.equal(robotCell.robotCellSummary.blockedStateId, null);

    const controller = entities.find((entity) => entity.id === "ent_ctrl_001");
    controller.data.stateMachine.transitions =
      controller.data.stateMachine.transitions.slice(0, 2);
    entities = syncFallbackRobotCellControl(entities, controller.id);

    robotCell = entities.find((entity) => entity.id === "ent_cell_001");
    assert.equal(robotCell.robotCellSummary.controllerTransitionCount, 2);
    assert.equal(robotCell.data.control.transitions.length, 2);
  });
});
