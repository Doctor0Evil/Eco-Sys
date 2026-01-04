#include "OrchestratorKernel.hpp"
#include "TransitOptimizer.hpp"
#include "WaterOptimizer.hpp"
#include "GridCouplingOptimizer.hpp"
#include "EcoScoreEngine.hpp"

#include <chrono>
#include <iostream>

void OrchestratorKernel::runDayAheadPlanning(const ScenarioConfig& cfg) {
    TransitPlan transitPlan = transitOptimizer_.optimize(cfg.transitConfig);
    WaterPlan   waterPlan   = waterOptimizer_.optimize(cfg.waterConfig);

    CoupledPlan coupled = gridOptimizer_.couple(transitPlan, waterPlan, cfg.gridConfig);

    double ecoScore = ecoScoreEngine_.computeEcoScore(coupled);

    persistPlans(coupled, ecoScore);
    logResult("day-ahead", ecoScore);
}

void OrchestratorKernel::runIntraDayUpdate(const TelemetrySnapshot& snap) {
    UpdatedTransitState tState = adapters_.transit.updateState(snap.transitData);
    UpdatedWaterState   wState = adapters_.water.updateState(snap.waterData);
    UpdatedGridState    gState = adapters_.grid.updateState(snap.gridData);

    CoupledPlan updated = gridOptimizer_.reOptimize(tState, wState, gState);

    double ecoScore = ecoScoreEngine_.computeEcoScore(updated);

    dispatchCommands(updated);
    logResult("intra-day", ecoScore);
}
