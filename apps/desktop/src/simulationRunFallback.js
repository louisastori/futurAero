function roundTwoDecimals(value) {
  return Math.round(value * 100) / 100;
}

function formatSimulationStatus({
  collisionCount,
  blockedSequenceDetected,
  maxTrackingErrorMm,
}) {
  if (collisionCount > 0) {
    return "collided";
  }
  if (blockedSequenceDetected || maxTrackingErrorMm > 2.5) {
    return "warning";
  }
  return "completed";
}

function buildTimelineSamples(stepCount, cycleTimeMs, maxTrackingErrorMm) {
  const sampledStepCount = Math.min(stepCount, 12);
  return Array.from({ length: sampledStepCount }, (_, index) => {
    const stepIndex = index;
    const progress =
      sampledStepCount <= 1 ? 0 : stepIndex / (sampledStepCount - 1);
    return {
      stepIndex,
      timestampMs:
        stepCount <= 1
          ? cycleTimeMs
          : Math.round((cycleTimeMs * (stepIndex + 1)) / stepCount),
      trackingErrorMm: roundTwoDecimals(
        maxTrackingErrorMm * (0.45 + progress * 0.55),
      ),
      speedScale: roundTwoDecimals(Math.min(0.82 + progress * 0.18, 1.0)),
    };
  });
}

function buildProgressSamples(
  scenarioName,
  stepCount,
  timelineSamples,
  signalSamples,
  controllerStateSamples,
  blockedSequenceDetected,
) {
  return [
    {
      phase: "queued",
      progress: 0,
      message: `job queued for ${scenarioName}`,
    },
    {
      phase: "running",
      progress: 0.35,
      message: `${stepCount} steps scheduled`,
    },
    {
      phase: "trace_persisted",
      progress: 0.78,
      message: `${timelineSamples.length} timeline | ${signalSamples.length} signal changes | ${controllerStateSamples.length} controller states`,
    },
    {
      phase: "completed",
      progress: 1,
      message: blockedSequenceDetected
        ? "run completed with blocked sequence"
        : "run completed successfully",
    },
  ];
}

function computeCollisionCount({ seed, stepCount, safetyZoneCount, endpointCount }) {
  if (safetyZoneCount === 0) {
    return (seed + stepCount) % 3;
  }
  if (endpointCount > 2) {
    return (seed + endpointCount) % 2;
  }
  return 0;
}

function stepTimestampMs(stepIndex, stepCount, cycleTimeMs) {
  if (stepCount <= 1) {
    return cycleTimeMs;
  }
  return Math.round((cycleTimeMs * (stepIndex + 1)) / stepCount);
}

function contactLocationLabel(pair) {
  return `${pair.id} | ${pair.leftEntityId} x ${pair.rightEntityId}`;
}

function contactPhase(stepIndex, stepCount) {
  if (stepCount <= 1 || stepIndex + 1 >= stepCount) {
    return "completed";
  }
  return "running";
}

function collisionEventId(contact) {
  return `collision-${contact.stepIndex}-${contact.pairId}`;
}

function controllerEventId(sample) {
  return `controller-${sample.stepIndex}-${sample.stateId}`;
}

function timelineEventId(sample) {
  return `timeline-${sample.stepIndex}`;
}

function pushUnique(items, value) {
  if (!value || items.includes(value)) {
    return;
  }
  items.push(value);
}

function buildContacts({
  contactPairs,
  collisionCount,
  stepCount,
  cycleTimeMs,
  seed,
  controllerStateSamples,
}) {
  if (collisionCount <= 0) {
    return [];
  }

  const pairs =
    Array.isArray(contactPairs) && contactPairs.length > 0
      ? contactPairs
      : [
          {
            id: "pair_default",
            leftEntityId: "ent_tool_001",
            rightEntityId: "ent_fixture_001",
            baseClearanceMm: 0.8,
          },
        ];

  return Array.from({ length: collisionCount }, (_, index) => {
    const pair = pairs[index % pairs.length];
    const stepIndex = Math.min(
      Math.floor(((index + 1) * stepCount) / (collisionCount + 1)),
      Math.max(stepCount - 1, 0),
    );
    const timestampMs = stepTimestampMs(stepIndex, stepCount, cycleTimeMs);
    const state =
      [...controllerStateSamples]
        .reverse()
        .find((sample) => sample.stepIndex <= stepIndex) ?? null;

    return {
      stepIndex,
      timestampMs,
      pairId: pair.id,
      leftEntityId: pair.leftEntityId,
      rightEntityId: pair.rightEntityId,
      locationLabel: contactLocationLabel(pair),
      phase: contactPhase(stepIndex, stepCount),
      stateId: state?.stateId ?? null,
      overlapMm: roundTwoDecimals(
        Number(pair.baseClearanceMm ?? 0.8) + (seed % 7) * 0.11 + index * 0.07,
      ),
      severity: "collision",
    };
  });
}

function buildRunReport({
  status,
  blockedSequenceDetected,
  blockedStateId,
  collisionCount,
  cycleTimeMs,
  maxTrackingErrorMm,
  timelineSamples,
  signalSamples,
  controllerStateSamples,
  contacts,
}) {
  if (status === "collided") {
    const primaryContact = contacts[0] ?? null;
    const criticalEventIds = [];
    pushUnique(
      criticalEventIds,
      primaryContact ? collisionEventId(primaryContact) : null,
    );
    const relatedControllerSample = primaryContact
      ? [...controllerStateSamples]
          .reverse()
          .find((sample) => sample.stepIndex <= primaryContact.stepIndex) ?? null
      : null;
    pushUnique(
      criticalEventIds,
      relatedControllerSample ? controllerEventId(relatedControllerSample) : null,
    );

    return {
      status,
      headline: `Collision critique sur ${primaryContact?.locationLabel ?? "zone inconnue"}`,
      findings: [
        `${collisionCount} collision(s) detectee(s) sur ${primaryContact?.locationLabel ?? "zone inconnue"}.`,
        `Instant critique a t=${primaryContact?.timestampMs ?? 0} ms | overlap ${primaryContact?.overlapMm ?? 0} mm | phase ${primaryContact?.phase ?? "running"}.`,
        `Cycle estime ${cycleTimeMs} ms | tracking max ${maxTrackingErrorMm} mm.`,
      ],
      criticalEventIds,
      recommendedActions: [
        `Inspecter la paire ${primaryContact?.pairId ?? "pair_default"} autour de ${primaryContact?.locationLabel ?? "zone inconnue"}.`,
        "Rejouer un run apres ajustement de trajectoire ou de clearance.",
        ...(primaryContact?.stateId
          ? [
              `Verifier la logique de transition associee a l etat ${primaryContact.stateId}.`,
            ]
          : []),
      ],
    };
  }

  if (status === "warning" || blockedSequenceDetected) {
    const blockedSample =
      [...controllerStateSamples]
        .reverse()
        .find((sample) => sample.reason === "sequence_blocked") ??
      controllerStateSamples.at(-1) ??
      null;
    const stateId = blockedStateId ?? blockedSample?.stateId ?? "state";
    const criticalEventIds = [];
    pushUnique(
      criticalEventIds,
      blockedSample ? controllerEventId(blockedSample) : null,
    );
    pushUnique(
      criticalEventIds,
      timelineSamples.at(-1) ? timelineEventId(timelineSamples.at(-1)) : null,
    );

    return {
      status: "warning",
      headline: `Sequence bloquee sur l etat ${stateId}`,
      findings: [
        `Le run termine sans collision mais reste bloque sur ${stateId}.`,
        `Cycle estime ${cycleTimeMs} ms | tracking max ${maxTrackingErrorMm} mm.`,
        `${timelineSamples.length} echantillon(s) timeline et ${controllerStateSamples.length} etat(s) controle persistent le run.`,
      ],
      criticalEventIds,
      recommendedActions: [
        `Verifier la transition de controle qui doit sortir de l etat ${stateId}.`,
        "Rejouer un run apres correction du signal ou de la condition bloquante.",
      ],
    };
  }

  return {
    status: "completed",
    headline: `Run nominal termine sans collision en ${cycleTimeMs} ms`,
    findings: [
      "Aucune collision ni sequence bloquee n est detectee sur ce run.",
      `${timelineSamples.length} echantillon(s) timeline, ${signalSamples.length} changement(s) signal et ${controllerStateSamples.length} etat(s) controle restent inspectables.`,
      `Etat final de controle: ${controllerStateSamples.at(-1)?.stateId ?? "completed"}.`,
    ],
    criticalEventIds: timelineSamples.at(-1)
      ? [timelineEventId(timelineSamples.at(-1))]
      : [],
    recommendedActions: [
      "Conserver ce run comme reference white-box du cycle nominal.",
      "Relancer la simulation apres chaque modification de trajectoire, signal ou safety.",
    ],
  };
}

function simulationRunSummaryFromPersistedData(data) {
  const summary = data?.summary ?? {};
  const metrics = data?.metrics ?? {};
  const job = data?.job ?? {};
  const timelineSamples = Array.isArray(data?.timelineSamples)
    ? data.timelineSamples
    : [];
  const signalSamples = Array.isArray(data?.signalSamples) ? data.signalSamples : [];
  const controllerStateSamples = Array.isArray(data?.controllerStateSamples)
    ? data.controllerStateSamples
    : [];
  const contacts = Array.isArray(data?.contacts) ? data.contacts : [];

  return {
    status: summary.status ?? "unknown",
    collisionCount: Number(metrics.collisionCount ?? 0),
    cycleTimeMs: Number(metrics.cycleTimeMs ?? 0),
    maxTrackingErrorMm: Number(metrics.maxTrackingErrorMm ?? 0),
    energyEstimateJ: Number(metrics.energyEstimateJ ?? 0),
    blockedSequenceDetected: Boolean(summary.blockedSequenceDetected),
    blockedStateId: summary.blockedStateId ?? null,
    contactCount: Number(summary.contactCount ?? contacts.length),
    signalSampleCount: Number(summary.signalSampleCount ?? signalSamples.length),
    controllerStateSampleCount: Number(
      summary.controllerStateSampleCount ?? controllerStateSamples.length,
    ),
    timelineSampleCount: Number(summary.timelineSampleCount ?? timelineSamples.length),
    jobStatus: job.status ?? "unknown",
    jobPhase: job.phase ?? "unknown",
    jobProgress: Number(job.progress ?? 0),
  };
}

function simulationRunReportFromPersistedData(data) {
  const report = data?.report ?? {};

  return {
    status: report.status ?? data?.summary?.status ?? "unknown",
    headline: report.headline ?? "",
    findings: Array.isArray(report.findings) ? report.findings : [],
    criticalEventIds: Array.isArray(report.criticalEventIds)
      ? report.criticalEventIds
      : [],
    recommendedActions: Array.isArray(report.recommendedActions)
      ? report.recommendedActions
      : [],
  };
}

function simulationRunDetailFromSummary(summary) {
  return `${summary.status} | ${summary.jobPhase} ${Math.round(summary.jobProgress * 100)}% | ${summary.cycleTimeMs} ms | ${summary.collisionCount} coll | ${summary.contactCount} contact`;
}

function buildFallbackSimulationRunData({
  entities,
  runId,
  endpointCount = 1,
}) {
  const robotCell =
    [...(entities ?? [])].filter((entity) => entity.robotCellSummary).at(-1) ?? null;
  const scenarioName = robotCell?.name ?? "RobotCell-001";
  const control = robotCell?.data?.control ?? {};
  const signalIds = Array.isArray(control.signalIds) ? control.signalIds : [];
  const signalEntities = (entities ?? []).filter(
    (entity) => entity.entityType === "Signal" && signalIds.includes(entity.id),
  );
  const controllerId =
    Array.isArray(robotCell?.data?.controllerModelIds) &&
    robotCell.data.controllerModelIds.length > 0
      ? robotCell.data.controllerModelIds[0]
      : null;
  const targetCount = Number(robotCell?.robotCellSummary?.targetCount ?? 3);
  const stepCount = Math.max(targetCount, 3) * 4;
  const plannedCycleTimeMs = Number(
    robotCell?.robotCellSummary?.estimatedCycleTimeMs ?? 3420,
  );
  const pathLengthMm = Number(robotCell?.robotCellSummary?.pathLengthMm ?? 860);
  const safetyZoneCount = Number(robotCell?.robotCellSummary?.safetyZoneCount ?? 2);
  const seed = targetCount * 97 + endpointCount * 17;
  const cycleTimeMs = plannedCycleTimeMs + endpointCount * 6 + (seed % 11);
  const maxTrackingErrorMm = roundTwoDecimals(
    pathLengthMm / Math.max(stepCount, 1) / 400 + (seed % 17) * 0.04,
  );
  const energyEstimateJ = roundTwoDecimals(
    pathLengthMm * 0.035 + cycleTimeMs * 0.012 + endpointCount * 1.5,
  );
  const timelineSamples = buildTimelineSamples(
    stepCount,
    cycleTimeMs,
    maxTrackingErrorMm,
  );
  const signalSamples = [
    ...signalEntities.map((entity) => ({
      stepIndex: 0,
      timestampMs: 0,
      signalId: entity.data.signalId,
      value: entity.data.initialValue,
      reason: "initial_value",
    })),
    {
      stepIndex: 1,
      timestampMs: Math.round(cycleTimeMs / stepCount),
      signalId: "sig_cycle_start",
      value: true,
      reason: "simulation.run.start",
    },
    {
      stepIndex: 3,
      timestampMs: Math.round((cycleTimeMs * 3) / stepCount),
      signalId: "sig_progress_gate",
      value: 0.62,
      reason: "robot.transfer.reached",
    },
    {
      stepIndex: 7,
      timestampMs: Math.round((cycleTimeMs * 7) / stepCount),
      signalId: "sig_progress_gate",
      value: 1,
      reason: "robot.place.completed",
    },
  ];
  const controllerStateSamples = [
    {
      stepIndex: 0,
      timestampMs: 0,
      stateId: "idle",
      stateName: "Idle",
      reason: "initial_state",
    },
    {
      stepIndex: 2,
      timestampMs: Math.round((cycleTimeMs * 2) / stepCount),
      stateId: "transfer",
      stateName: "Transfer",
      reason: "pick completed",
    },
    {
      stepIndex: 7,
      timestampMs: Math.round((cycleTimeMs * 7) / stepCount),
      stateId: "done",
      stateName: "Done",
      reason: "finish_cycle",
    },
  ];
  const contactPairs = Array.isArray(control.contactPairs) ? control.contactPairs : [];
  const collisionCount = computeCollisionCount({
    seed,
    stepCount,
    safetyZoneCount,
    endpointCount,
  });
  const contacts = buildContacts({
    contactPairs,
    collisionCount,
    stepCount,
    cycleTimeMs,
    seed,
    controllerStateSamples,
  });
  const blockedSequenceDetected = false;
  const summaryStatus = formatSimulationStatus({
    collisionCount,
    blockedSequenceDetected,
    maxTrackingErrorMm,
  });
  const progressSamples = buildProgressSamples(
    scenarioName,
    stepCount,
    timelineSamples,
    signalSamples,
    controllerStateSamples,
    blockedSequenceDetected,
  );
  const report = buildRunReport({
    status: summaryStatus,
    blockedSequenceDetected,
    blockedStateId: null,
    collisionCount,
    cycleTimeMs,
    maxTrackingErrorMm,
    timelineSamples,
    signalSamples,
    controllerStateSamples,
    contacts,
  });

  return {
    tags: ["simulation", "artifact", "mvp"],
    robotCellId: robotCell?.id ?? null,
    scenario: {
      name: scenarioName,
      seed,
      engineVersion: "faero-sim@0.2.0",
      stepCount,
      plannedCycleTimeMs,
      pathLengthMm,
      endpointCount,
      safetyZoneCount,
      signalCount: signalEntities.length,
      scheduledSignalChangeCount: 3,
      contactPairCount: Array.isArray(control.contactPairs)
        ? control.contactPairs.length
        : 0,
      source: {
        robotCellId: robotCell?.id ?? null,
        controllerId,
        signalIds,
      },
    },
    summary: {
      status: summaryStatus,
      blockedSequenceDetected,
      blockedStateId: null,
      contactCount: contacts.length,
      signalSampleCount: signalSamples.length,
      controllerStateSampleCount: controllerStateSamples.length,
      timelineSampleCount: timelineSamples.length,
    },
    metrics: {
      collisionCount,
      cycleTimeMs,
      maxTrackingErrorMm,
      energyEstimateJ,
    },
    report,
    job: {
      jobId: `job_${runId.replace(/^ent_/, "")}`,
      status: "completed",
      phase: "completed",
      progress: 1,
      progressSamples,
      message: "simulation completed successfully",
    },
    timelineSamples,
    signalSamples,
    controllerStateSamples,
    contacts,
  };
}

function buildFallbackSimulationRunEntity({
  entities,
  runIndex,
  endpointCount = 1,
}) {
  const id = `ent_run_${String(runIndex).padStart(3, "0")}`;
  const name = `SimulationRun-${String(runIndex).padStart(3, "0")}`;
  const data = buildFallbackSimulationRunData({
    entities,
    runId: id,
    endpointCount,
  });
  const summary = simulationRunSummaryFromPersistedData(data);

  return {
    id,
    entityType: "SimulationRun",
    name,
    revision: "rev_seed",
    status: "active",
    detail: simulationRunDetailFromSummary(summary),
    data,
    simulationRunSummary: summary,
  };
}

export {
  buildFallbackSimulationRunData,
  buildFallbackSimulationRunEntity,
  simulationRunDetailFromSummary,
  simulationRunReportFromPersistedData,
  simulationRunSummaryFromPersistedData,
};
