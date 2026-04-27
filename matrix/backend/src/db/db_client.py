#!/usr/bin/env python3
"""
Database Client - TimescaleDB Persistence
==========================================
Handles persistence of agent decisions to TimescaleDB.

Usage:
    from db import get_db_client

    db = get_db_client()
    db.store_decision(decision_data)
"""

import os
import logging
from typing import Dict, Any, Optional, List
from datetime import datetime
import psycopg2
from psycopg2.extras import Json
from contextlib import contextmanager

logger = logging.getLogger(__name__)


class DatabaseClient:
    """
    TimescaleDB client for storing agent decisions.
    """

    def __init__(self, connection_string: Optional[str] = None):
        """
        Initialize database client.

        Args:
            connection_string: PostgreSQL connection string
                              Defaults to DATABASE_URL env var
        """
        self.connection_string = connection_string or os.getenv(
            'DATABASE_URL',
            'postgresql://localhost:5432/ai_agent_hub'
        )
        self.enabled = self._check_connection()

    def _check_connection(self) -> bool:
        """
        Check if database is available.

        Returns:
            True if database is available, False otherwise
        """
        try:
            with psycopg2.connect(self.connection_string) as conn:
                with conn.cursor() as cur:
                    cur.execute('SELECT 1')
                    logger.info("✓ Database connection successful")
                    return True
        except Exception as e:
            logger.warning(f"✗ Database not available: {e}")
            logger.warning("Decisions will not be persisted to database")
            return False

    @contextmanager
    def get_connection(self):
        """
        Context manager for database connections.

        Yields:
            psycopg2 connection
        """
        conn = psycopg2.connect(self.connection_string)
        try:
            yield conn
            conn.commit()
        except Exception as e:
            conn.rollback()
            logger.error(f"Database error: {e}")
            raise
        finally:
            conn.close()

    def store_decision(
        self,
        agent_id: str,
        decision_type: str,
        score: float,
        approved: bool,
        stf_compliant: bool,
        stf_violations: List[str],
        context: Dict[str, Any],
        proposed_action: str,
        model_version: str,
        latency_ms: float,
        requires_human_review: bool,
        timestamp: Optional[datetime] = None
    ) -> Optional[str]:
        """
        Store decision in TimescaleDB.

        Args:
            agent_id: Agent identifier
            decision_type: Type of decision
            score: Decision score (0.0 - 1.0)
            approved: Whether decision was approved
            stf_compliant: Whether decision is STF compliant
            stf_violations: List of STF violations
            context: Decision context (JSON)
            proposed_action: Proposed action
            model_version: ML model version
            latency_ms: Processing latency in milliseconds
            requires_human_review: Whether human review is required
            timestamp: Decision timestamp (defaults to now)

        Returns:
            decision_id (UUID) if successful, None otherwise
        """
        if not self.enabled:
            logger.debug("Database not enabled, skipping persistence")
            return None

        timestamp = timestamp or datetime.utcnow()

        query = """
        INSERT INTO decisions (
            time,
            agent_id,
            decision_type,
            score,
            approved,
            stf_compliant,
            stf_violations,
            context,
            proposed_action,
            model_version,
            latency_ms,
            requires_human_review
        ) VALUES (
            %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s
        )
        RETURNING decision_id;
        """

        try:
            with self.get_connection() as conn:
                with conn.cursor() as cur:
                    cur.execute(query, (
                        timestamp,
                        agent_id,
                        decision_type,
                        score,
                        approved,
                        stf_compliant,
                        stf_violations,
                        Json(context),
                        proposed_action,
                        model_version,
                        latency_ms,
                        requires_human_review
                    ))

                    decision_id = cur.fetchone()[0]
                    logger.debug(f"Stored decision {decision_id} for agent {agent_id}")
                    return str(decision_id)

        except Exception as e:
            logger.error(f"Failed to store decision: {e}")
            return None

    def get_recent_decisions(
        self,
        agent_id: Optional[str] = None,
        limit: int = 10
    ) -> List[Dict[str, Any]]:
        """
        Get recent decisions from database.

        Args:
            agent_id: Filter by agent ID (optional)
            limit: Maximum number of decisions to return

        Returns:
            List of decision dictionaries
        """
        if not self.enabled:
            return []

        query = """
        SELECT
            decision_id,
            time,
            agent_id,
            decision_type,
            score,
            approved,
            stf_compliant,
            stf_violations,
            context,
            proposed_action,
            model_version,
            latency_ms,
            requires_human_review
        FROM decisions
        WHERE ($1::text IS NULL OR agent_id = $1)
        ORDER BY time DESC
        LIMIT $2;
        """

        try:
            with self.get_connection() as conn:
                with conn.cursor() as cur:
                    cur.execute(query, (agent_id, limit))
                    rows = cur.fetchall()

                    decisions = []
                    for row in rows:
                        decisions.append({
                            'decision_id': str(row[0]),
                            'timestamp': row[1].isoformat(),
                            'agent_id': row[2],
                            'decision_type': row[3],
                            'score': row[4],
                            'approved': row[5],
                            'stf_compliant': row[6],
                            'stf_violations': row[7],
                            'context': row[8],
                            'proposed_action': row[9],
                            'model_version': row[10],
                            'latency_ms': row[11],
                            'requires_human_review': row[12]
                        })

                    return decisions

        except Exception as e:
            logger.error(f"Failed to get recent decisions: {e}")
            return []

    def get_agent_stats(
        self,
        agent_id: str,
        hours: int = 24
    ) -> Dict[str, Any]:
        """
        Get statistics for an agent over time period.

        Args:
            agent_id: Agent identifier
            hours: Time period in hours

        Returns:
            Statistics dictionary
        """
        if not self.enabled:
            return {}

        query = """
        SELECT
            COUNT(*) as total_decisions,
            SUM(CASE WHEN approved THEN 1 ELSE 0 END) as approved_count,
            AVG(CASE WHEN approved THEN 1.0 ELSE 0.0 END) as approval_rate,
            SUM(CASE WHEN stf_compliant THEN 1 ELSE 0 END) as stf_compliant_count,
            AVG(CASE WHEN stf_compliant THEN 1.0 ELSE 0.0 END) as stf_compliance_rate,
            AVG(score) as avg_score,
            AVG(latency_ms) as avg_latency,
            PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms) as p95_latency
        FROM decisions
        WHERE agent_id = %s
          AND time > NOW() - INTERVAL '%s hours';
        """

        try:
            with self.get_connection() as conn:
                with conn.cursor() as cur:
                    cur.execute(query, (agent_id, hours))
                    row = cur.fetchone()

                    if row:
                        return {
                            'agent_id': agent_id,
                            'time_period_hours': hours,
                            'total_decisions': row[0] or 0,
                            'approved_count': row[1] or 0,
                            'approval_rate': float(row[2] or 0.0),
                            'stf_compliant_count': row[3] or 0,
                            'stf_compliance_rate': float(row[4] or 0.0),
                            'avg_score': float(row[5] or 0.0),
                            'avg_latency_ms': float(row[6] or 0.0),
                            'p95_latency_ms': float(row[7] or 0.0)
                        }

                    return {}

        except Exception as e:
            logger.error(f"Failed to get agent stats: {e}")
            return {}


# =============================================================================
# SINGLETON INSTANCE
# =============================================================================

_db_client: Optional[DatabaseClient] = None


def get_db_client() -> DatabaseClient:
    """
    Get singleton database client instance.

    Returns:
        DatabaseClient instance
    """
    global _db_client

    if _db_client is None:
        _db_client = DatabaseClient()

    return _db_client


# =============================================================================
# EXAMPLE USAGE
# =============================================================================

if __name__ == '__main__':
    # Initialize client
    db = get_db_client()

    if db.enabled:
        # Store test decision
        decision_id = db.store_decision(
            agent_id='test-agent',
            decision_type='code_generation',
            score=0.95,
            approved=True,
            stf_compliant=True,
            stf_violations=[],
            context={'test': True},
            proposed_action='Generate test',
            model_version='v1',
            latency_ms=42.5,
            requires_human_review=False
        )

        print(f"Stored decision: {decision_id}")

        # Get recent decisions
        recent = db.get_recent_decisions(agent_id='test-agent', limit=5)
        print(f"Recent decisions: {len(recent)}")

        # Get stats
        stats = db.get_agent_stats('test-agent', hours=24)
        print(f"Stats: {stats}")
    else:
        print("Database not available")
