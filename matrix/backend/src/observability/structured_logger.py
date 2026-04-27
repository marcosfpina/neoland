#!/usr/bin/env python3
"""
Structured Logger - ELK/Loki Compatible
=======================================
Provides structured JSON logging for ELK Stack / Grafana Loki.

Usage:
    from observability.structured_logger import get_logger, log_decision

    logger = get_logger("ai-ranking")
    logger.info("Decision processed", extra={
        "agent_id": "claude",
        "score": 0.95
    })

    # Or use helper
    log_decision(
        agent_id="claude",
        decision_type="code_review",
        score=0.95,
        approved=True,
        stf_compliant=True,
        context={"file": "main.py"}
    )
"""

import logging
import json
import sys
from datetime import datetime
from typing import Any, Dict, Optional
from pathlib import Path


# =============================================================================
# STRUCTURED FORMATTER
# =============================================================================

class StructuredFormatter(logging.Formatter):
    """
    JSON formatter for structured logging.

    Outputs logs in JSON format compatible with ELK/Loki.
    """

    def format(self, record: logging.LogRecord) -> str:
        """Format log record as JSON"""
        # Base log entry
        log_entry = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "module": record.module,
            "function": record.funcName,
            "line": record.lineno,
        }

        # Add exception info if present
        if record.exc_info:
            log_entry["exception"] = self.formatException(record.exc_info)

        # Add extra fields (custom data)
        if hasattr(record, 'extra_fields'):
            log_entry.update(record.extra_fields)

        # Add any other attributes not in the base record
        for key, value in record.__dict__.items():
            if key not in [
                'name', 'msg', 'args', 'created', 'filename', 'funcName',
                'levelname', 'lineno', 'module', 'msecs', 'message',
                'pathname', 'process', 'processName', 'relativeCreated',
                'thread', 'threadName', 'exc_info', 'exc_text', 'stack_info',
                'extra_fields'
            ] and not key.startswith('_'):
                try:
                    # Only add JSON-serializable values
                    json.dumps(value)
                    log_entry[key] = value
                except (TypeError, ValueError):
                    log_entry[key] = str(value)

        return json.dumps(log_entry)


# =============================================================================
# LOGGER FACTORY
# =============================================================================

_loggers = {}


def get_logger(
    name: str,
    level: int = logging.INFO,
    log_file: Optional[Path] = None,
    stdout: bool = True
) -> logging.Logger:
    """
    Get or create a structured logger.

    Args:
        name: Logger name
        level: Logging level (default INFO)
        log_file: Optional file path for file logging
        stdout: Whether to log to stdout (default True)

    Returns:
        Configured logger instance
    """
    if name in _loggers:
        return _loggers[name]

    logger = logging.getLogger(name)
    logger.setLevel(level)
    logger.propagate = False  # Prevent duplicate logs

    formatter = StructuredFormatter()

    # Console handler (stdout)
    if stdout:
        console_handler = logging.StreamHandler(sys.stdout)
        console_handler.setFormatter(formatter)
        logger.addHandler(console_handler)

    # File handler (for local debugging)
    if log_file:
        log_file = Path(log_file)
        log_file.parent.mkdir(parents=True, exist_ok=True)
        file_handler = logging.FileHandler(log_file)
        file_handler.setFormatter(formatter)
        logger.addHandler(file_handler)

    _loggers[name] = logger
    return logger


# =============================================================================
# DECISION LOGGING HELPER
# =============================================================================

decision_logger = get_logger("ai-ranking.decisions")


def log_decision(
    agent_id: str,
    decision_type: str,
    score: float,
    approved: bool,
    stf_compliant: bool,
    stf_violations: Optional[list] = None,
    latency_ms: Optional[float] = None,
    context: Optional[Dict[str, Any]] = None,
    metadata: Optional[Dict[str, Any]] = None
):
    """
    Log a decision in structured format.

    Args:
        agent_id: Agent identifier
        decision_type: Type of decision
        score: Decision score
        approved: Whether approved
        stf_compliant: STF compliance status
        stf_violations: List of violations
        latency_ms: Processing latency
        context: Decision context
        metadata: Additional metadata
    """
    log_data = {
        "event_type": "decision",
        "agent_id": agent_id,
        "decision_type": decision_type,
        "score": round(score, 4),
        "approved": approved,
        "stf_compliant": stf_compliant,
        "stf_violations": stf_violations or [],
        "violation_count": len(stf_violations) if stf_violations else 0,
        "latency_ms": round(latency_ms, 2) if latency_ms else None,
        "context": context or {},
        "metadata": metadata or {}
    }

    # Log at appropriate level
    if not stf_compliant:
        decision_logger.warning(
            f"Decision rejected (STF violations): {agent_id}",
            extra={"extra_fields": log_data}
        )
    elif not approved:
        decision_logger.warning(
            f"Decision requires review: {agent_id}",
            extra={"extra_fields": log_data}
        )
    else:
        decision_logger.info(
            f"Decision approved: {agent_id}",
            extra={"extra_fields": log_data}
        )


# =============================================================================
# SYSTEM EVENT LOGGING
# =============================================================================

system_logger = get_logger("ai-ranking.system")


def log_system_event(
    event_type: str,
    status: str,
    details: Optional[Dict[str, Any]] = None
):
    """
    Log a system event.

    Args:
        event_type: Type of event (startup, shutdown, error, etc.)
        status: Event status (success, failure, warning)
        details: Additional details
    """
    log_data = {
        "event_type": event_type,
        "status": status,
        "details": details or {}
    }

    if status == "failure":
        system_logger.error(
            f"System event failed: {event_type}",
            extra={"extra_fields": log_data}
        )
    elif status == "warning":
        system_logger.warning(
            f"System event warning: {event_type}",
            extra={"extra_fields": log_data}
        )
    else:
        system_logger.info(
            f"System event: {event_type}",
            extra={"extra_fields": log_data}
        )


# =============================================================================
# ELASTICSEARCH INTEGRATION (Future)
# =============================================================================

def setup_elasticsearch_handler(
    logger: logging.Logger,
    es_host: str = "localhost:9200",
    index_prefix: str = "ai-agent-hub"
):
    """
    Setup Elasticsearch handler for direct log shipping.

    Note: Requires elasticsearch-py and CMRESHandler
    This is a stub for future implementation.

    Args:
        logger: Logger to attach handler to
        es_host: Elasticsearch host
        index_prefix: Index name prefix
    """
    try:
        # Import would be here
        # from cmreslogging.handlers import CMRESHandler

        # Create handler
        # es_handler = CMRESHandler(
        #     hosts=[{'host': es_host.split(':')[0], 'port': int(es_host.split(':')[1])}],
        #     auth_type=CMRESHandler.AuthType.NO_AUTH,
        #     es_index_name=index_prefix
        # )

        # logger.addHandler(es_handler)

        system_logger.info(f"Elasticsearch handler configured: {es_host}")
    except ImportError:
        system_logger.warning("Elasticsearch handler not available (cmreslogging not installed)")
    except Exception as e:
        system_logger.error(f"Failed to setup Elasticsearch handler: {e}")


# =============================================================================
# EXAMPLE USAGE
# =============================================================================

if __name__ == '__main__':
    # Example: Log a decision
    log_decision(
        agent_id="claude-code",
        decision_type="code_generation",
        score=0.95,
        approved=True,
        stf_compliant=True,
        latency_ms=42.5,
        context={
            "file": "/home/user/main.py",
            "language": "python"
        },
        metadata={
            "user_id": "kernelcore",
            "session_id": "abc123"
        }
    )

    # Example: Log STF violation
    log_decision(
        agent_id="gemini-flash",
        decision_type="package_install",
        score=0.45,
        approved=False,
        stf_compliant=False,
        stf_violations=[
            "Nix-First violation: pip install detected",
            "Forbidden pattern: apt-get"
        ],
        latency_ms=120.3
    )

    # Example: System event
    log_system_event(
        event_type="startup",
        status="success",
        details={
            "version": "1.0.0",
            "protocol": "NEUTRON",
            "port": 8090
        }
    )
