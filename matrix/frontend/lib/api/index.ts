/**
 * API Module - Centralized Backend Communication
 * ===============================================
 * Export all API-related types and clients.
 */

export { BackendClient, getBackendClient } from './backend-client';
export type {
  DecisionRequest,
  RankingResponse,
  HealthStatus,
  STFContext,
  PrometheusMetrics,
  BackendClientConfig,
  RequestOptions,
  SimulatedRanking,
  WorkflowContext,
  CodeAnalysisRequest,
  CodeAnalysisResponse,
  APIError,
  ValidationError,
} from './types';

export {
  isRankingResponse,
  isAPIError,
  isHealthStatus,
} from './types';
