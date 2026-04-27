/**
 * Backend API Client - AI Assistant Backend
 * ==========================================
 * Centralized client for communicating with Python FastAPI backend.
 *
 * Features:
 * - Retry logic with exponential backoff
 * - Circuit breaker pattern
 * - Fallback to simulated data
 * - Request timeout handling
 * - Error recovery
 *
 * Backend: apps/backend (Python/FastAPI)
 * Endpoints: /rank, /metrics, /health, /stf/context
 */

import {
  DecisionRequest,
  RankingResponse,
  HealthStatus,
  STFContext,
  PrometheusMetrics,
  BackendClientConfig,
  RequestOptions,
  SimulatedRanking,
  isRankingResponse,
  isHealthStatus,
} from './types';

// =============================================================================
// CLIENT CLASS
// =============================================================================

export class BackendClient {
  private config: BackendClientConfig;
  private circuitBreakerOpen: boolean = false;
  private circuitBreakerTimeout: NodeJS.Timeout | null = null;
  private requestCount: number = 0;
  private errorCount: number = 0;

  constructor(config?: Partial<BackendClientConfig>) {
    this.config = {
      baseURL: process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8000',
      timeout: 5000, // 5 seconds
      retries: 3,
      retryDelay: 1000, // 1 second
      enableFallback: true,
      ...config,
    };
  }

  // ===========================================================================
  // CORE REQUEST METHOD
  // ===========================================================================

  private async request<T>(
    endpoint: string,
    options: RequestInit & RequestOptions = {}
  ): Promise<T> {
    const { timeout, retries, signal, ...fetchOptions } = options;

    const requestTimeout = timeout || this.config.timeout;
    const maxRetries = retries !== undefined ? retries : this.config.retries;

    // Circuit breaker check
    if (this.circuitBreakerOpen) {
      throw new Error('Circuit breaker is open - backend unavailable');
    }

    // Create abort controller
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), requestTimeout);

    let lastError: Error | null = null;

    // Retry loop
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        this.requestCount++;

        const response = await fetch(`${this.config.baseURL}${endpoint}`, {
          ...fetchOptions,
          signal: signal || controller.signal,
        });

        clearTimeout(timeoutId);

        if (!response.ok) {
          // Handle HTTP errors
          const errorText = await response.text();
          throw new Error(
            `HTTP ${response.status}: ${errorText || response.statusText}`
          );
        }

        // Success - reset error count
        this.errorCount = 0;

        // Parse response based on content type
        const contentType = response.headers.get('content-type');
        if (contentType?.includes('application/json')) {
          return await response.json();
        } else {
          return (await response.text()) as T;
        }
      } catch (error: any) {
        lastError = error;
        this.errorCount++;

        // Check if we should open circuit breaker
        if (this.errorCount >= 5) {
          this.openCircuitBreaker();
        }

        // Don't retry if aborted or on last attempt
        if (error.name === 'AbortError' || attempt === maxRetries) {
          break;
        }

        // Exponential backoff
        const delay = this.config.retryDelay * Math.pow(2, attempt);
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }

    clearTimeout(timeoutId);

    // All retries failed
    throw lastError || new Error('Request failed after retries');
  }

  // ===========================================================================
  // CIRCUIT BREAKER
  // ===========================================================================

  private openCircuitBreaker() {
    console.warn('Circuit breaker opened - backend marked as unavailable');
    this.circuitBreakerOpen = true;

    // Auto-reset after 30 seconds
    if (this.circuitBreakerTimeout) {
      clearTimeout(this.circuitBreakerTimeout);
    }

    this.circuitBreakerTimeout = setTimeout(() => {
      console.info('Circuit breaker reset - retrying backend connection');
      this.circuitBreakerOpen = false;
      this.errorCount = 0;
    }, 30000);
  }

  // ===========================================================================
  // API METHODS
  // ===========================================================================

  /**
   * Rank a decision using ML model + STF validation
   *
   * POST /rank
   */
  async rankDecision(
    request: DecisionRequest,
    options?: RequestOptions
  ): Promise<RankingResponse> {
    try {
      const response = await this.request<RankingResponse>('/rank', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
        ...options,
      });

      if (!isRankingResponse(response)) {
        throw new Error('Invalid response format from /rank endpoint');
      }

      return response;
    } catch (error) {
      console.error('Failed to rank decision:', error);

      if (this.config.enableFallback) {
        return this.simulateRanking(request);
      }

      throw error;
    }
  }

  /**
   * Get system health status
   *
   * GET /health
   */
  async getHealth(options?: RequestOptions): Promise<HealthStatus> {
    try {
      const response = await this.request<HealthStatus>('/health', {
        method: 'GET',
        ...options,
      });

      if (!isHealthStatus(response)) {
        throw new Error('Invalid response format from /health endpoint');
      }

      return response;
    } catch (error) {
      console.error('Failed to get health status:', error);

      return {
        status: 'down',
        mode: 'unknown',
        stf_protocol: 'not_loaded',
        ml_model: 'not_loaded',
      };
    }
  }

  /**
   * Get Prometheus metrics
   *
   * GET /metrics
   */
  async getMetrics(options?: RequestOptions): Promise<PrometheusMetrics> {
    try {
      const metricsText = await this.request<string>('/metrics', {
        method: 'GET',
        ...options,
      });

      return this.parsePrometheusMetrics(metricsText);
    } catch (error) {
      console.error('Failed to get metrics:', error);

      if (this.config.enableFallback) {
        return this.getSimulatedMetrics();
      }

      throw error;
    }
  }

  /**
   * Get STF context for agent injection
   *
   * GET /stf/context
   */
  async getSTFContext(
    format: 'markdown' | 'json' | 'xml' = 'markdown',
    options?: RequestOptions
  ): Promise<STFContext> {
    try {
      return await this.request<STFContext>(
        `/stf/context?format=${format}`,
        {
          method: 'GET',
          ...options,
        }
      );
    } catch (error) {
      console.error('Failed to get STF context:', error);
      throw error;
    }
  }

  // ===========================================================================
  // FALLBACK / SIMULATION
  // ===========================================================================

  /**
   * Simulate ranking when backend unavailable
   */
  private simulateRanking(request: DecisionRequest): RankingResponse {
    console.warn('Using simulated ranking - backend unavailable');

    // Simple heuristic simulation
    const isCritical = ['deploy', 'delete_db', 'modify_acl'].includes(
      request.decision_type
    );
    const highEntropy = request.context.entropy > 0.8;

    let score = 0.85;
    if (isCritical) score -= 0.15;
    if (highEntropy) score -= 0.2;

    const approved = score >= 0.9;

    return {
      agent_id: request.agent_id,
      score: Math.max(0, Math.min(1, score)),
      approved,
      requires_human_review: !approved,
      stf_compliant: true,
      stf_violations: [],
      timestamp: Date.now() / 1000,
    };
  }

  /**
   * Get simulated metrics
   */
  private getSimulatedMetrics(): PrometheusMetrics {
    return {
      decisions_total: 0,
      decisions_by_agent: {},
      decisions_by_type: {},
      approval_rate: 0,
      approval_rate_by_agent: {},
      stf_compliance_rate: 0,
      stf_violations_total: 0,
      violations_by_type: {},
      latency_p50: 0,
      latency_p95: 0,
      latency_p99: 0,
      avg_latency: 0,
      human_reviews_total: 0,
      human_reviews_by_reason: {},
      score_avg: 0,
      score_distribution: {},
    };
  }

  // ===========================================================================
  // PROMETHEUS PARSER
  // ===========================================================================

  /**
   * Parse Prometheus text format to JSON
   */
  private parsePrometheusMetrics(text: string): PrometheusMetrics {
    const metrics: PrometheusMetrics = {
      decisions_total: 0,
      decisions_by_agent: {},
      decisions_by_type: {},
      approval_rate: 0,
      approval_rate_by_agent: {},
      stf_compliance_rate: 0,
      stf_violations_total: 0,
      violations_by_type: {},
      latency_p50: 0,
      latency_p95: 0,
      latency_p99: 0,
      avg_latency: 0,
      human_reviews_total: 0,
      human_reviews_by_reason: {},
      score_avg: 0,
      score_distribution: {},
    };

    const lines = text.split('\n');

    for (const line of lines) {
      // Skip comments and empty lines
      if (line.startsWith('#') || !line.trim()) continue;

      // Parse metric line: metric_name{labels} value
      const match = line.match(/^([a-zA-Z_:][a-zA-Z0-9_:]*)\{([^}]*)\}\s+([\d.]+)/);
      if (!match) {
        // Try without labels
        const simpleMatch = line.match(/^([a-zA-Z_:][a-zA-Z0-9_:]*)\s+([\d.]+)/);
        if (simpleMatch) {
          const [, name, value] = simpleMatch;
          this.assignMetricValue(metrics, name, parseFloat(value));
        }
        continue;
      }

      const [, name, labels, value] = match;
      const labelPairs = this.parseLabels(labels);

      this.assignMetricValue(metrics, name, parseFloat(value), labelPairs);
    }

    return metrics;
  }

  /**
   * Parse Prometheus labels: key="value",key2="value2"
   */
  private parseLabels(labelString: string): Record<string, string> {
    const labels: Record<string, string> = {};
    const pairs = labelString.match(/(\w+)="([^"]*)"/g);

    if (pairs) {
      for (const pair of pairs) {
        const [key, value] = pair.split('=');
        labels[key] = value.replace(/"/g, '');
      }
    }

    return labels;
  }

  /**
   * Assign parsed metric value to metrics object
   */
  private assignMetricValue(
    metrics: PrometheusMetrics,
    name: string,
    value: number,
    labels?: Record<string, string>
  ) {
    // Map metric names to our structure
    if (name === 'ai_agent_decisions_total') {
      metrics.decisions_total += value;
      if (labels?.agent_id) {
        metrics.decisions_by_agent[labels.agent_id] = value;
      }
      if (labels?.decision_type) {
        metrics.decisions_by_type[labels.decision_type] = value;
      }
    } else if (name === 'ai_agent_approval_rate') {
      metrics.approval_rate = value;
      if (labels?.agent_id) {
        metrics.approval_rate_by_agent[labels.agent_id] = value;
      }
    } else if (name === 'ai_agent_stf_compliance_rate') {
      metrics.stf_compliance_rate = value;
    } else if (name === 'ai_agent_stf_violations_total') {
      metrics.stf_violations_total += value;
      if (labels?.violation_type) {
        metrics.violations_by_type[labels.violation_type] = value;
      }
    } else if (name === 'ai_agent_human_reviews_total') {
      metrics.human_reviews_total += value;
      if (labels?.reason) {
        metrics.human_reviews_by_reason[labels.reason] = value;
      }
    }
  }

  // ===========================================================================
  // UTILITY METHODS
  // ===========================================================================

  /**
   * Get client statistics
   */
  getStats() {
    return {
      requestCount: this.requestCount,
      errorCount: this.errorCount,
      circuitBreakerOpen: this.circuitBreakerOpen,
      errorRate:
        this.requestCount > 0 ? this.errorCount / this.requestCount : 0,
    };
  }

  /**
   * Reset circuit breaker manually
   */
  resetCircuitBreaker() {
    this.circuitBreakerOpen = false;
    this.errorCount = 0;
    if (this.circuitBreakerTimeout) {
      clearTimeout(this.circuitBreakerTimeout);
      this.circuitBreakerTimeout = null;
    }
  }

  /**
   * Check if backend is available
   */
  async isAvailable(): Promise<boolean> {
    try {
      const health = await this.getHealth({ timeout: 2000, retries: 0 });
      return health.status === 'ok';
    } catch {
      return false;
    }
  }
}

// =============================================================================
// SINGLETON INSTANCE
// =============================================================================

let backendClient: BackendClient | null = null;

/**
 * Get singleton backend client instance
 */
export function getBackendClient(): BackendClient {
  if (!backendClient) {
    backendClient = new BackendClient();
  }
  return backendClient;
}

// =============================================================================
// EXPORTS
// =============================================================================

export default BackendClient;
