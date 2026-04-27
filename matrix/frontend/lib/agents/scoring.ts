import { AgentIdentity, AgentMetrics, AgentAptitude } from './types';

/**
 * The Phi Score (Φ) Algorithm
 * 
 * Φ = (SuccessRate * ComplexityWeight) / (log(LatencyMs) * (EnergyCost + 1))
 * 
 * This formula penalizes high latency and high resource consumption 
 * while rewarding successful delivery of complex tasks.
 */
export function calculatePhiScore(
  successRate: number, // 0 to 1
  complexity: number,  // 1 to 10
  metrics: Partial<AgentMetrics>
): AgentAptitude {
  const { 
    avgResponseTime: latencyMs = 1000, 
    cpuUsagePercent = 10, 
    memoryUsageMb = 128, 
    gpuUsagePercent = 0 
  } = metrics;
  
  // Normalize resource consumption
  const energyFactor = (cpuUsagePercent * 0.4) + (gpuUsagePercent * 0.5) + (memoryUsageMb / 1024 * 0.1);
  
  // Professionalism: reliability and adherence to constraints
  const professionalism = Math.round(successRate * 100);
  
  // Code quality: simulated here - in real usage, this comes from lint/test results
  const codeQuality = Math.round(successRate * 95 + (10 - complexity));

  // Efficiency: inverse of resource usage over time
  const efficiency = Math.round(100 / (1 + (latencyMs / 1000) * (energyFactor / 10)));

  // Final Phi Score
  const logLatency = Math.log(Math.max(latencyMs, 2));
  const overallScore = (successRate * complexity * 100) / (logLatency * (energyFactor / 50 + 1));

  return {
    professionalism,
    codeQuality,
    efficiency,
    overallScore: Math.min(Math.round(overallScore), 100)
  };
}

export function updateAgentXP(agent: AgentIdentity, aptitude: AgentAptitude): number {
  const xpGain = Math.round(aptitude.overallScore * (agent.metrics.streak + 1) * 0.5);
  return xpGain;
}
