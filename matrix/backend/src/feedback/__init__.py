"""
Feedback Module - Decision Outcome Collection
=============================================
Collects and persists feedback on agent decisions for continuous improvement.

Usage:
    from feedback import FeedbackCollector

    collector = FeedbackCollector()
    collector.record_feedback(
        decision_id="abc-123",
        agent_id="claude",
        feedback_type="approved",
        outcome_score=1.0
    )
"""

from .collector import (
    FeedbackCollector,
    FeedbackRecord,
    FeedbackType,
    OutcomeScore
)

__all__ = [
    'FeedbackCollector',
    'FeedbackRecord',
    'FeedbackType',
    'OutcomeScore'
]

__version__ = '1.0.0'
