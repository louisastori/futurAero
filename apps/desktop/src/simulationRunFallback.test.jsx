import assert from "node:assert/strict";
import { describe, test } from "vitest";

import { buildFallbackRobotCellBundle } from "./robotCellFallback.js";
import {
  buildFallbackSimulationRunEntity,
  simulationRunReportFromPersistedData,
  simulationRunSummaryFromPersistedData,
} from "./simulationRunFallback.js";

describe("simulationRunFallback parity", () => {
  test("fallback simulation run exposes the persisted runner contract", () => {
    const entities = buildFallbackRobotCellBundle(1);
    const run = buildFallbackSimulationRunEntity({
      entities,
      runIndex: entities.length + 1,
      endpointCount: 1,
    });

    assert.equal(run.entityType, "SimulationRun");
    assert.equal(run.data.scenario.seed, 308);
    assert.equal(run.data.scenario.engineVersion, "faero-sim@0.2.0");
    assert.equal(run.data.scenario.stepCount, 12);
    assert.equal(run.data.summary.status, "completed");
    assert.equal(run.data.metrics.collisionCount, 0);
    assert.equal(run.data.report.status, "completed");
    assert.ok(run.data.report.headline.includes("Run nominal"));
    assert.equal(run.data.job.status, "completed");
    assert.equal(run.data.job.phase, "completed");
    assert.equal(run.data.job.progress, 1);
    assert.deepEqual(
      run.data.job.progressSamples.map((sample) => sample.phase),
      ["queued", "running", "trace_persisted", "completed"],
    );
    assert.equal(
      run.data.summary.timelineSampleCount,
      run.data.timelineSamples.length,
    );
    assert.equal(
      run.data.summary.signalSampleCount,
      run.data.signalSamples.length,
    );
    assert.equal(
      run.data.summary.controllerStateSampleCount,
      run.data.controllerStateSamples.length,
    );
    assert.deepEqual(
      run.simulationRunSummary,
      simulationRunSummaryFromPersistedData(run.data),
    );
    assert.deepEqual(
      run.data.report,
      simulationRunReportFromPersistedData(run.data),
    );
  });

  test("fallback simulation run can expose localized collisions and a readable report", () => {
    const entities = buildFallbackRobotCellBundle(1);
    const run = buildFallbackSimulationRunEntity({
      entities,
      runIndex: entities.length + 1,
      endpointCount: 3,
    });

    assert.equal(run.data.metrics.collisionCount, 1);
    assert.equal(run.data.summary.status, "collided");
    assert.equal(run.data.report.status, "collided");
    assert.equal(run.data.contacts.length, 1);
    assert.ok(run.data.contacts[0].locationLabel.includes("pair_"));
    assert.equal(run.data.contacts[0].phase, "running");
    assert.ok(run.data.report.criticalEventIds[0].startsWith("collision-"));
  });
});
