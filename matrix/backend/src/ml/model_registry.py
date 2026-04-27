#!/usr/bin/env python3
"""
MLflow Model Registry - Decision Scoring Models
===============================================
Manages ML models for decision scoring using MLflow.

Models:
- Decision Scorer: Predicts approval probability
- STF Compliance Predictor: Predicts STF violations
- Risk Assessor: Assesses decision risk

Usage:
    from ml.model_registry import ModelRegistry

    registry = ModelRegistry()
    model = registry.load_model("decision-scorer-v1")
    score = model.predict(features)

STF Compliance: Enables data-driven decision approval
"""

import logging
import pickle
import json
from pathlib import Path
from typing import Dict, Any, Optional, List, Tuple
from dataclasses import dataclass
from datetime import datetime
import numpy as np

logger = logging.getLogger(__name__)


@dataclass
class ModelMetadata:
    """Model metadata"""
    model_name: str
    version: str
    accuracy: float
    created_at: str
    features: List[str]
    target: str


class DecisionScorerModel:
    """
    Decision scoring model (simplified ML model).

    In production, this would be a trained sklearn/XGBoost/neural network model.
    For now, we use a heuristic-based model with feature engineering.
    """

    def __init__(self, model_params: Optional[Dict] = None):
        """
        Initialize model.

        Args:
            model_params: Model parameters
        """
        self.params = model_params or self._get_default_params()
        self.version = "1.1.0-pm-intelligence"
        self.feature_names = [
            'decision_type_critical',
            'context_entropy',
            'agent_recent_approval_rate',
            'stf_compliant',
            'has_documentation',
            'code_complexity',
            'risk_score',
            'pm_alignment_score'
        ]
        self.pm_vision = self._load_pm_vision()

    def _load_pm_vision(self) -> Dict[str, Any]:
        """Load PM Strategic Vision configuration"""
        try:
            vision_path = Path(__file__).parent / 'pm_vision.json'
            if vision_path.exists():
                with open(vision_path, 'r') as f:
                    return json.load(f)
        except Exception as e:
            logging.error(f"Failed to load PM vision: {e}")
        
        # Default fallback vision
        return {
            "projects": {},
            "agent_roles": {},
            "strategic_vision": {"focus_areas": {}}
        }

    def _get_default_params(self) -> Dict:
        """Get default model parameters"""
        return {
            'critical_decision_penalty': 0.15,
            'entropy_weight': 0.25,
            'stf_weight': 0.40,
            'documentation_bonus': 0.10,
            'approval_history_weight': 0.20,
            'complexity_penalty_threshold': 100,
            'pm_reward_weight': 0.20
        }

    def extract_features(self, decision: Dict[str, Any]) -> Dict[str, float]:
        """
        Extract features from decision request.

        Args:
            decision: Decision dictionary

        Returns:
            Feature dictionary
        """
        features = {}

        # Critical decision types
        critical_types = ['deploy', 'delete_db', 'modify_acl', 'system_config']
        features['decision_type_critical'] = 1.0 if decision.get('decision_type') in critical_types else 0.0

        # Context entropy
        context = decision.get('context', {})
        features['context_entropy'] = context.get('entropy', 0.5)

        # Agent history (would come from DB in production)
        features['agent_recent_approval_rate'] = 0.85  # Placeholder

        # STF compliance
        features['stf_compliant'] = 1.0 if decision.get('stf_compliant', True) else 0.0

        # Documentation presence
        features['has_documentation'] = 1.0 if self._has_documentation(context) else 0.0

        # Code complexity (from context)
        features['code_complexity'] = context.get('complexity_score', 50) / 100.0

        # Risk score (computed)
        features['risk_score'] = self._compute_risk_score(decision)

        # PM Alignment (New)
        features['pm_alignment_score'] = self._compute_pm_reward(decision)

        return features

    def _compute_pm_reward(self, decision: Dict) -> float:
        """Compute reward based on PM strategic alignment"""
        reward = 0.0
        
        # 1. Project Priority
        project_id = decision.get('project_id')
        if project_id:
            projects = self.pm_vision.get('projects', {})
            # Normalized project weight (1.0 baseline)
            proj_data = projects.get(project_id, {'weight': 1.0})
            weight_bonus = (proj_data.get('weight', 1.0) - 1.0) * 0.5
            reward += weight_bonus

        # 2. Agent Role Affinity
        agent_id = decision.get('agent_id', '')
        # Simplified: map agent_id to role (e.g., CodeAnalyzer-1 -> CodeAnalyzer)
        agent_role = next((role for role in self.pm_vision.get('agent_roles', {}) if role in agent_id), None)
        
        if agent_role:
            role_config = self.pm_vision['agent_roles'][agent_role]
            task_type = decision.get('decision_type', '')
            
            # Direct task match
            if any(t in task_type.lower() for t in role_config.get('preferred_tasks', [])):
                reward += role_config.get('affinity_bonus', 0.0)
                
        # 3. Strategic Focus
        focus_areas = self.pm_vision.get('strategic_vision', {}).get('focus_areas', {})
        context = decision.get('context', {})
        
        for area, multiplier in focus_areas.items():
            # Check if context keywords match focus area
            context_str = str(context).lower()
            if area in context_str:
                reward += (multiplier - 1.0) * 0.2

        return min(reward, 0.5)  # Cap reward at 0.5

    def _has_documentation(self, context: Dict) -> bool:
        """Check if decision has documentation"""
        doc_indicators = ['readme', 'docs', 'adr', 'comment', 'docstring']
        return any(
            indicator in str(context.get('files', [])).lower()
            for indicator in doc_indicators
        )

    def _compute_risk_score(self, decision: Dict) -> float:
        """Compute risk score based on decision characteristics"""
        risk = 0.0

        # File-based risks
        context = decision.get('context', {})
        modified_files = context.get('modified_files', [])

        if any('config' in f.lower() for f in modified_files):
            risk += 0.2
        if any('schema' in f.lower() or 'migration' in f.lower() for f in modified_files):
            risk += 0.3
        if any('auth' in f.lower() or 'security' in f.lower() for f in modified_files):
            risk += 0.4

        # Decision type risks
        risky_types = ['deploy', 'delete', 'modify_acl']
        if decision.get('decision_type') in risky_types:
            risk += 0.3

        return min(risk, 1.0)

    def predict(self, decision: Dict[str, Any]) -> float:
        """
        Predict approval score for a decision.

        Args:
            decision: Decision dictionary with context

        Returns:
            Approval score (0.0 - 1.0)
        """
        features = self.extract_features(decision)

        # Base score
        score = 0.85 # Slightly lower base to allow PM rewards to shine

        # Apply penalties and bonuses
        if features['decision_type_critical']:
            score -= self.params['critical_decision_penalty']

        if features['context_entropy'] > 0.8:
            score -= self.params['entropy_weight']

        if not features['stf_compliant']:
            score -= self.params['stf_weight']
        else:
            score += 0.05  # Small bonus for STF compliance

        if features['has_documentation']:
            score += self.params['documentation_bonus']

        # Approval history influence
        approval_rate = features['agent_recent_approval_rate']
        if approval_rate < 0.7:
            score -= 0.1
        elif approval_rate > 0.9:
            score += 0.05

        # Risk penalty
        risk = features['risk_score']
        score -= risk * 0.15

        # Code complexity
        if features['code_complexity'] > 0.8:
            score -= 0.1
            
        # PM Intelligence Reward
        pm_reward = features['pm_alignment_score']
        score += pm_reward

        return max(0.0, min(1.0, score))

    def explain(self, decision: Dict[str, Any]) -> Dict[str, Any]:
        """
        Explain model prediction.

        Args:
            decision: Decision dictionary

        Returns:
            Explanation with feature contributions
        """
        features = self.extract_features(decision)
        score = self.predict(decision)

        contributions = []

        if features['decision_type_critical']:
            contributions.append({
                'feature': 'Critical Decision Type',
                'impact': -self.params['critical_decision_penalty'],
                'value': features['decision_type_critical']
            })

        if features['context_entropy'] > 0.8:
            contributions.append({
                'feature': 'High Entropy',
                'impact': -self.params['entropy_weight'],
                'value': features['context_entropy']
            })

        if not features['stf_compliant']:
            contributions.append({
                'feature': 'STF Non-Compliant',
                'impact': -self.params['stf_weight'],
                'value': 0.0
            })
        
        # PM Rewards
        if features['pm_alignment_score'] > 0:
            contributions.append({
                'feature': 'PM Strategy Bonus',
                'impact': +features['pm_alignment_score'],
                'value': features['pm_alignment_score']
            })

        if features['has_documentation']:
            contributions.append({
                'feature': 'Has Documentation',
                'impact': +self.params['documentation_bonus'],
                'value': 1.0
            })

        return {
            'final_score': score,
            'features': features,
            'contributions': contributions,
            'model_version': self.version
        }


class ModelRegistry:
    """
    MLflow-compatible model registry.

    Manages versioned models for decision scoring.
    """

    def __init__(
        self,
        registry_path: Optional[Path] = None,
        mlflow_tracking_uri: Optional[str] = None
    ):
        """
        Initialize model registry.

        Args:
            registry_path: Local path for model storage
            mlflow_tracking_uri: MLflow tracking server URI
        """
        self.registry_path = Path(registry_path or '/tmp/ai-agent-models')
        self.registry_path.mkdir(parents=True, exist_ok=True)

        self.mlflow_uri = mlflow_tracking_uri
        self._models = {}
        self._load_registry()

    def _load_registry(self):
        """Load model registry index"""
        index_file = self.registry_path / 'registry_index.json'

        if index_file.exists():
            try:
                with open(index_file, 'r') as f:
                    self._models = json.load(f)
                logger.info(f"Loaded {len(self._models)} models from registry")
            except Exception as e:
                logger.error(f"Failed to load registry: {e}")
        else:
            # Initialize with default model
            self._register_default_model()

    def _register_default_model(self):
        """Register default decision scorer model"""
        model_name = "decision-scorer"
        version = "v1"

        model = DecisionScorerModel()
        model_path = self.registry_path / f"{model_name}-{version}.pkl"

        # Save model
        with open(model_path, 'wb') as f:
            pickle.dump(model, f)

        # Add to registry
        self._models[model_name] = {
            'name': model_name,
            'version': version,
            'path': str(model_path),
            'created_at': datetime.utcnow().isoformat(),
            'accuracy': 0.85,  # Placeholder
            'features': model.feature_names,
            'target': 'approval_probability'
        }

        self._save_registry()
        logger.info(f"Registered default model: {model_name}-{version}")

    def _save_registry(self):
        """Save registry index"""
        index_file = self.registry_path / 'registry_index.json'

        with open(index_file, 'w') as f:
            json.dump(self._models, f, indent=2)

    def load_model(self, model_name: str = "decision-scorer") -> DecisionScorerModel:
        """
        Load a model from registry.

        Args:
            model_name: Model name

        Returns:
            Loaded model instance
        """
        if model_name not in self._models:
            raise ValueError(f"Model not found in registry: {model_name}")

        model_info = self._models[model_name]
        model_path = Path(model_info['path'])

        if not model_path.exists():
            logger.warning(f"Model file not found, creating new: {model_path}")
            return DecisionScorerModel()

        with open(model_path, 'rb') as f:
            model = pickle.load(f)

        logger.info(f"Loaded model: {model_name} {model_info['version']}")
        return model

    def get_model_info(self, model_name: str) -> Dict[str, Any]:
        """Get model metadata"""
        if model_name not in self._models:
            raise ValueError(f"Model not found: {model_name}")

        return self._models[model_name]

    def list_models(self) -> List[Dict[str, Any]]:
        """List all registered models"""
        return list(self._models.values())


# =============================================================================
# EXAMPLE USAGE
# =============================================================================

if __name__ == '__main__':
    # Initialize registry
    registry = ModelRegistry()

    # Load model
    model = registry.load_model("decision-scorer")

    # Test decision
    test_decision = {
        'decision_type': 'code_generation',
        'agent_id': 'claude-code',
        'stf_compliant': True,
        'context': {
            'files': ['src/main.py', 'README.md'],
            'complexity_score': 45,
            'entropy': 0.3,
            'modified_files': ['src/main.py']
        }
    }

    # Predict
    score = model.predict(test_decision)
    print(f"Predicted score: {score:.3f}")

    # Explain
    explanation = model.explain(test_decision)
    print(f"\nExplanation:")
    print(json.dumps(explanation, indent=2))

    # List models
    models = registry.list_models()
    print(f"\nRegistered models: {len(models)}")
    for m in models:
        print(f"  - {m['name']} {m['version']} (accuracy: {m['accuracy']:.2%})")
