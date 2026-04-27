#!/usr/bin/env python3
"""
Feedback Collector - Decision Outcome Persistence
=================================================
Collects feedback on agent decisions and persists to TimescaleDB.

Workflow:
1. Decision is made and logged
2. User provides feedback (approve/reject/modify)
3. Feedback is collected and stored
4. ML models retrained periodically with feedback data

Usage:
    from feedback.collector import FeedbackCollector

    collector = FeedbackCollector()

    # Collect feedback
    collector.record_feedback(
        decision_id="abc-123",
        agent_id="claude",
        user_feedback="approved",
        outcome_score=1.0,
        comments="Good decision"
    )

STF Compliance: Implements feedback loop for continuous improvement
"""

import uuid
import logging
from datetime import datetime
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field
from enum import Enum
import json


logger = logging.getLogger(__name__)


class FeedbackType(Enum):
    """Type of feedback"""
    APPROVED = "approved"  # User approved the decision
    REJECTED = "rejected"  # User rejected the decision
    MODIFIED = "modified"  # User modified the decision
    REVERTED = "reverted"  # Decision was reverted/rolled back


class OutcomeScore(Enum):
    """Outcome scoring"""
    EXCELLENT = 1.0
    GOOD = 0.75
    ACCEPTABLE = 0.5
    POOR = 0.25
    FAILED = 0.0


@dataclass
class FeedbackRecord:
    """Feedback record for a decision"""
    feedback_id: str
    decision_id: str
    agent_id: str
    feedback_type: FeedbackType
    outcome_score: float
    user_id: Optional[str] = None
    comments: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)
    timestamp: str = field(default_factory=lambda: datetime.utcnow().isoformat())

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary"""
        return {
            'feedback_id': self.feedback_id,
            'decision_id': self.decision_id,
            'agent_id': self.agent_id,
            'feedback_type': self.feedback_type.value,
            'outcome_score': self.outcome_score,
            'user_id': self.user_id,
            'comments': self.comments,
            'metadata': self.metadata,
            'timestamp': self.timestamp
        }


class FeedbackCollector:
    """
    Collects and persists feedback on agent decisions.

    Supports multiple storage backends:
    - TimescaleDB (primary)
    - JSON file (fallback/testing)
    """

    def __init__(
        self,
        db_connection: Optional[Any] = None,
        fallback_file: Optional[str] = None
    ):
        """
        Initialize feedback collector.

        Args:
            db_connection: TimescaleDB connection (psycopg2)
            fallback_file: JSON file path for fallback storage
        """
        self.db_connection = db_connection
        self.fallback_file = fallback_file
        self._feedback_buffer = []

    def record_feedback(
        self,
        decision_id: str,
        agent_id: str,
        feedback_type: str,
        outcome_score: float,
        user_id: Optional[str] = None,
        comments: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Record feedback for a decision.

        Args:
            decision_id: Decision ID
            agent_id: Agent ID
            feedback_type: Type of feedback (approved/rejected/modified/reverted)
            outcome_score: Outcome score (0.0 - 1.0)
            user_id: User who provided feedback
            comments: Optional comments
            metadata: Additional metadata

        Returns:
            Feedback ID
        """
        feedback_id = str(uuid.uuid4())

        feedback = FeedbackRecord(
            feedback_id=feedback_id,
            decision_id=decision_id,
            agent_id=agent_id,
            feedback_type=FeedbackType(feedback_type),
            outcome_score=outcome_score,
            user_id=user_id,
            comments=comments,
            metadata=metadata or {}
        )

        # Try to persist to database
        if self.db_connection:
            try:
                self._persist_to_db(feedback)
                logger.info(f"Feedback recorded to DB: {feedback_id}")
            except Exception as e:
                logger.error(f"Failed to persist to DB: {e}")
                self._persist_to_fallback(feedback)
        else:
            self._persist_to_fallback(feedback)

        return feedback_id

    def _persist_to_db(self, feedback: FeedbackRecord):
        """Persist feedback to TimescaleDB"""
        if not self.db_connection:
            raise ValueError("Database connection not available")

        query = """
        INSERT INTO feedback (
            time,
            feedback_id,
            decision_id,
            agent_id,
            feedback_type,
            outcome_score,
            user_id,
            comments,
            metadata
        ) VALUES (
            NOW(),
            %s, %s, %s, %s, %s, %s, %s, %s
        )
        """

        with self.db_connection.cursor() as cursor:
            cursor.execute(query, (
                feedback.feedback_id,
                feedback.decision_id,
                feedback.agent_id,
                feedback.feedback_type.value,
                feedback.outcome_score,
                feedback.user_id,
                feedback.comments,
                json.dumps(feedback.metadata)
            ))
            self.db_connection.commit()

    def _persist_to_fallback(self, feedback: FeedbackRecord):
        """Persist feedback to JSON file (fallback)"""
        self._feedback_buffer.append(feedback.to_dict())

        if self.fallback_file:
            try:
                import os
                if os.path.exists(self.fallback_file):
                    with open(self.fallback_file, 'r') as f:
                        existing = json.load(f)
                else:
                    existing = []

                existing.append(feedback.to_dict())

                with open(self.fallback_file, 'w') as f:
                    json.dump(existing, f, indent=2)

                logger.info(f"Feedback recorded to file: {feedback.feedback_id}")
            except Exception as e:
                logger.error(f"Failed to persist to file: {e}")

    def get_feedback_for_decision(self, decision_id: str) -> List[FeedbackRecord]:
        """
        Get all feedback for a decision.

        Args:
            decision_id: Decision ID

        Returns:
            List of feedback records
        """
        if not self.db_connection:
            # Fallback: search in buffer/file
            return [
                FeedbackRecord(**f)
                for f in self._feedback_buffer
                if f['decision_id'] == decision_id
            ]

        query = """
        SELECT
            feedback_id,
            decision_id,
            agent_id,
            feedback_type,
            outcome_score,
            user_id,
            comments,
            metadata,
            time
        FROM feedback
        WHERE decision_id = %s
        ORDER BY time DESC
        """

        with self.db_connection.cursor() as cursor:
            cursor.execute(query, (decision_id,))
            rows = cursor.fetchall()

            return [
                FeedbackRecord(
                    feedback_id=row[0],
                    decision_id=row[1],
                    agent_id=row[2],
                    feedback_type=FeedbackType(row[3]),
                    outcome_score=row[4],
                    user_id=row[5],
                    comments=row[6],
                    metadata=row[7] if isinstance(row[7], dict) else json.loads(row[7]),
                    timestamp=row[8].isoformat() if row[8] else None
                )
                for row in rows
            ]

    def get_agent_feedback_summary(
        self,
        agent_id: str,
        hours: int = 24
    ) -> Dict[str, Any]:
        """
        Get feedback summary for an agent.

        Args:
            agent_id: Agent ID
            hours: Time window in hours

        Returns:
            Summary statistics
        """
        if not self.db_connection:
            # Fallback: calculate from buffer
            agent_feedback = [
                f for f in self._feedback_buffer
                if f['agent_id'] == agent_id
            ]

            if not agent_feedback:
                return {
                    "total_feedback": 0,
                    "avg_outcome_score": 0.0,
                    "feedback_distribution": {}
                }

            return {
                "total_feedback": len(agent_feedback),
                "avg_outcome_score": sum(f['outcome_score'] for f in agent_feedback) / len(agent_feedback),
                "feedback_distribution": self._calculate_distribution(agent_feedback)
            }

        query = """
        SELECT
            COUNT(*) as total_feedback,
            AVG(outcome_score) as avg_outcome_score,
            feedback_type,
            COUNT(*) as count
        FROM feedback
        WHERE agent_id = %s
          AND time > NOW() - INTERVAL '%s hours'
        GROUP BY feedback_type
        """

        with self.db_connection.cursor() as cursor:
            cursor.execute(query, (agent_id, hours))
            rows = cursor.fetchall()

            if not rows or rows[0][0] == 0:
                return {
                    "total_feedback": 0,
                    "avg_outcome_score": 0.0,
                    "feedback_distribution": {}
                }

            total = rows[0][0]
            avg_score = rows[0][1]
            distribution = {
                row[2]: row[3]
                for row in rows
            }

            return {
                "total_feedback": total,
                "avg_outcome_score": float(avg_score),
                "feedback_distribution": distribution
            }

    def _calculate_distribution(self, feedback_list: List[Dict]) -> Dict[str, int]:
        """Calculate feedback type distribution"""
        distribution = {}
        for feedback in feedback_list:
            ftype = feedback['feedback_type']
            distribution[ftype] = distribution.get(ftype, 0) + 1
        return distribution

    def export_training_data(
        self,
        agent_id: Optional[str] = None,
        min_outcome_score: float = 0.0,
        limit: int = 10000
    ) -> List[Dict[str, Any]]:
        """
        Export feedback data for ML model training.

        Args:
            agent_id: Filter by agent ID (None for all)
            min_outcome_score: Minimum outcome score
            limit: Maximum number of records

        Returns:
            Training data as list of dictionaries
        """
        if not self.db_connection:
            # Fallback: use buffer
            data = self._feedback_buffer
            if agent_id:
                data = [f for f in data if f['agent_id'] == agent_id]
            data = [f for f in data if f['outcome_score'] >= min_outcome_score]
            return data[:limit]

        query = """
        SELECT
            d.decision_id,
            d.agent_id,
            d.decision_type,
            d.score as original_score,
            d.approved as was_approved,
            d.stf_compliant,
            d.context,
            f.feedback_type,
            f.outcome_score,
            f.comments,
            f.metadata as feedback_metadata
        FROM feedback f
        JOIN decisions d ON f.decision_id = d.decision_id::text
        WHERE 1=1
        """

        params = []

        if agent_id:
            query += " AND d.agent_id = %s"
            params.append(agent_id)

        query += " AND f.outcome_score >= %s"
        params.append(min_outcome_score)

        query += " ORDER BY f.time DESC LIMIT %s"
        params.append(limit)

        with self.db_connection.cursor() as cursor:
            cursor.execute(query, tuple(params))
            rows = cursor.fetchall()

            return [
                {
                    'decision_id': row[0],
                    'agent_id': row[1],
                    'decision_type': row[2],
                    'original_score': row[3],
                    'was_approved': row[4],
                    'stf_compliant': row[5],
                    'context': row[6],
                    'feedback_type': row[7],
                    'outcome_score': row[8],
                    'comments': row[9],
                    'feedback_metadata': row[10]
                }
                for row in rows
            ]


# =============================================================================
# EXAMPLE USAGE
# =============================================================================

if __name__ == '__main__':
    # Example with fallback file
    collector = FeedbackCollector(fallback_file='.feedback_buffer.json')

    # Record feedback
    feedback_id = collector.record_feedback(
        decision_id="dec-123",
        agent_id="claude-code",
        feedback_type="approved",
        outcome_score=1.0,
        user_id="kernelcore",
        comments="Excellent code generation"
    )

    print(f"Feedback recorded: {feedback_id}")

    # Get summary
    summary = collector.get_agent_feedback_summary("claude-code")
    print(f"Summary: {summary}")

    # Export training data
    training_data = collector.export_training_data(agent_id="claude-code")
    print(f"Training data records: {len(training_data)}")
