
import pytest
from ml.model_registry import DecisionScorerModel

class TestRankingRewards:
    @pytest.fixture
    def model(self):
        return DecisionScorerModel()

    def test_project_priority_impact(self, model):
        """High priority project should score higher than low priority"""
        base_request = {
            'decision_type': 'refactor',
            'agent_id': 'CodeAnalyzer',
            'stf_compliant': True,
            'context': {'files': ['main.py']}
        }

        # 1. Critical Priority (mlx-coda: 1.5)
        high_req = base_request.copy()
        high_req['project_id'] = 'mlx-coda'
        high_score = model.predict(high_req)

        # 2. Low Priority (legacy-app: 0.5)
        low_req = base_request.copy()
        low_req['project_id'] = 'legacy-app'
        low_score = model.predict(low_req)

        print(f"\nAuthorization Scores - High: {high_score:.3f}, Low: {low_score:.3f}")
        assert high_score > low_score
        assert (high_score - low_score) >= 0.2  # Significant difference

    def test_agent_affinity_bonus(self, model):
        """Agent doing preferred task should get bonus"""
        # CodeAnalyzer prefers 'refactor'
        
        # Match
        match_req = {
            'decision_type': 'refactor_code',
            'agent_id': 'CodeAnalyzer',
            'stf_compliant': True,
            'context': {}
        }
        match_score = model.predict(match_req)

        # Mismatch (CodeAnalyzer doing 'deployment')
        mismatch_req = {
            'decision_type': 'deployment',
            'agent_id': 'CodeAnalyzer',
            'stf_compliant': True,
            'context': {}
        }
        mismatch_score = model.predict(mismatch_req)

        print(f"\nAffinity Scores - Match: {match_score:.3f}, Mismatch: {mismatch_score:.3f}")
        assert match_score > mismatch_score

    def test_strategic_focus_bonus(self, model):
        """Strategic keywords in context should boost score"""
        # Focus area: 'security' (1.2x)
        
        base_req = {
            'decision_type': 'review',
            'agent_id': 'GeneralAgent',
            'stf_compliant': True,
            'context': {'entropy': 0.5}
        }

        # With Security context
        sec_req = base_req.copy()
        sec_req['context'] = {'entropy': 0.5, 'description': 'fixing security vulnerability and auth'}
        sec_score = model.predict(sec_req)

        # Without Security context
        plain_req = base_req.copy()
        plain_req['context'] = {'entropy': 0.5, 'description': 'fixing indentation'}
        plain_score = model.predict(plain_req)

        print(f"\nFocus Scores - Security: {sec_score:.3f}, Plain: {plain_score:.3f}")
        assert sec_score > plain_score

    def test_full_explanation(self, model):
        """Explanation should include PM Strategy Bonus"""
        req = {
            'decision_type': 'audit',
            'agent_id': 'SecurityScanner', # Affinity bonus
            'project_id': 'mlx-coda',      # Priority bonus
            'stf_compliant': True,
            'context': {'description': 'security audit'} # Focus bonus
        }
        
        explanation = model.explain(req)
        
        found_pm_bonus = False
        for contrib in explanation['contributions']:
            if contrib['feature'] == 'PM Strategy Bonus':
                found_pm_bonus = True
                print(f"\nPM Bonus Found: {contrib['value']:.3f}")
                
        assert found_pm_bonus
