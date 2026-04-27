"""
Machine Learning Module - Decision Scoring Models
=================================================
ML models for intelligent decision approval.

Components:
- model_registry: MLflow-compatible model registry
- training: Model training pipeline (future)

Usage:
    from ml import ModelRegistry

    registry = ModelRegistry()
    model = registry.load_model("decision-scorer")
    score = model.predict(decision_dict)
"""

from .model_registry import (
    ModelRegistry,
    DecisionScorerModel,
    ModelMetadata
)

__all__ = [
    'ModelRegistry',
    'DecisionScorerModel',
    'ModelMetadata'
]

__version__ = '1.0.0'
