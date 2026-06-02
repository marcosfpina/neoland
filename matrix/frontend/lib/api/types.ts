/**
 * TypeScript Types - AI Assistant Backend API
 * ============================================
 * Type definitions aligned with Python backend (FastAPI).
 *
 * Backend source: apps/backend/src/ranking/main.py
 */

// =============================================================================
// REQUEST TYPES
// =============================================================================

/**
 * Decision request for /rank endpoint
 */
export interface DecisionRequest {
  agent_id: string;
  decision_type: string;
  context: Record<string, unknown>;
  proposed_action: string;
  model_version?: string;
}

/**
 * Decision context for workflow orchestration
 */
export interface WorkflowContext {
  workflow: string;
  agent: string;
  specialization: string;
  step?: number;
  total_steps?: number;
  [key: string]: unknown;
}

// =============================================================================
// RESPONSE TYPES
// =============================================================================

/**
 * Ranking response from /rank endpoint
 */
export interface RankingResponse {
  agent_id: string;
  score: number;
  approved: boolean;
  requires_human_review: boolean;
  stf_compliant: boolean;
  stf_violations: string[];
  timestamp: number;
}

/**
 * Health check response
 */
export interface HealthStatus {
  status: 'ok' | 'degraded' | 'down';
  mode: string;
  stf_protocol: 'loaded' | 'not_loaded';
  stf_version?: string;
  ml_model: 'loaded' | 'not_loaded';
  ml_version?: string;
}

/**
 * STF context response
 */
export interface STFContext {
  format: 'markdown' | 'json' | 'xml';
  context: string | object;
}

/**
 * Prometheus metrics (parsed from text format)
 */
export interface PrometheusMetrics {
  // Decision metrics
  decisions_total: number;
  decisions_by_agent: Record<string, number>;
  decisions_by_type: Record<string, number>;

  // Approval metrics
  approval_rate: number;
  approval_rate_by_agent: Record<string, number>;

  // STF compliance
  stf_compliance_rate: number;
  stf_violations_total: number;
  violations_by_type: Record<string, number>;

  // Latency metrics
  latency_p50: number;
  latency_p95: number;
  latency_p99: number;
  avg_latency: number;

  // Human review
  human_reviews_total: number;
  human_reviews_by_reason: Record<string, number>;

  // Score distribution
  score_avg: number;
  score_distribution: Record<string, number>;
}

// =============================================================================
// ERROR TYPES
// =============================================================================

/**
 * API error response
 */
export interface APIError {
  error: string;
  detail?: string;
  status_code: number;
  timestamp: string;
}

/**
 * Validation error (422)
 */
export interface ValidationError {
  detail: Array<{
    loc: string[];
    msg: string;
    type: string;
  }>;
}

// =============================================================================
// CLIENT CONFIG
// =============================================================================

/**
 * Backend client configuration
 */
export interface BackendClientConfig {
  baseURL: string;
  timeout: number;
  retries: number;
  retryDelay: number;
  enableFallback: boolean;
}

/**
 * Request options
 */
export interface RequestOptions {
  timeout?: number;
  retries?: number;
  signal?: AbortSignal;
}

// =============================================================================
// SIMULATED DATA (fallback)
// =============================================================================

/**
 * Simulated ranking response (used when backend unavailable)
 */
export interface SimulatedRanking {
  ranking: RankingResponse;
  simulated: true;
  reason: string;
}

// =============================================================================
// ANALYSIS TYPES (for future endpoints)
// =============================================================================

/**
 * Code analysis request
 */
export interface CodeAnalysisRequest {
  code: string;
  language: string;
  check_security?: boolean;
  check_performance?: boolean;
}

/**
 * Code analysis response
 */
export interface CodeAnalysisResponse {
  analysis: {
    complexity: number;
    lines_of_code: number;
    security_issues: string[];
    performance_issues: string[];
    suggestions: string[];
  };
  ranking: RankingResponse;
}

// =============================================================================
// TYPE GUARDS
// =============================================================================

/**
 * Type guard for RankingResponse
 */
export function isRankingResponse(obj: unknown): obj is RankingResponse {
  return (
    typeof obj === 'object' &&
    typeof obj.agent_id === 'string' &&
    typeof obj.score === 'number' &&
    typeof obj.approved === 'boolean' &&
    typeof obj.requires_human_review === 'boolean' &&
    typeof obj.stf_compliant === 'boolean' &&
    Array.isArray(obj.stf_violations) &&
    typeof obj.timestamp === 'number'
  );
}

/**
 * Type guard for APIError
 */
export function isAPIError(obj: unknown): obj is APIError {
  return (
    typeof obj === 'object' &&
    typeof obj.error === 'string' &&
    typeof obj.status_code === 'number'
  );
}

/**
 * Type guard for HealthStatus
 */
export function isHealthStatus(obj: unknown): obj is HealthStatus {
  return (
    typeof obj === 'object' &&
    typeof obj.status === 'string' &&
    typeof obj.mode === 'string' &&
    typeof obj.stf_protocol === 'string' &&
    typeof obj.ml_model === 'string'
  );
}
