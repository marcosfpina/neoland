"""pytest configuration for neoland-agents tests."""
import pytest


def pytest_configure(config):
    config.addinivalue_line("markers", "integration: requires LLM and DB")
    config.addinivalue_line("markers", "contract: schema validation only, no LLM")
