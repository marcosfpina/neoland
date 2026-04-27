#!/usr/bin/env python3
"""
STF Parser - Supreme Technical Directive Parser
================================================
Parses STF files and extracts directives for agent context injection.

Usage:
    from stf.parser import STFParser

    parser = STFParser('/path/to/neutron.stf')
    context = parser.get_injection_context()
    # Inject into agent prompt
"""

import re
import yaml
import json
import logging
from pathlib import Path
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from enum import Enum

logger = logging.getLogger(__name__)


class EnforcementLevel(Enum):
    """Enforcement levels for directives"""
    MANDATORY = "MANDATORY"
    RECOMMENDED = "RECOMMENDED"
    OPTIONAL = "OPTIONAL"


class ComplianceMode(Enum):
    """STF compliance modes"""
    STRICT_COMPLIANCE = "STRICT_COMPLIANCE"
    GUIDED = "GUIDED"
    PERMISSIVE = "PERMISSIVE"


@dataclass
class Directive:
    """Represents a single STF directive"""
    id: str
    enforcement: EnforcementLevel
    title: str
    rule: str
    action: Optional[str] = None
    checklist: List[str] = field(default_factory=list)
    requirements: List[str] = field(default_factory=list)
    forbidden: List[str] = field(default_factory=list)


@dataclass
class Constraint:
    """Represents a behavioral constraint"""
    id: str
    type: str
    title: str
    rule: Optional[str] = None
    forbidden: Optional[str] = None
    procedure: List[str] = field(default_factory=list)


@dataclass
class ErrorPattern:
    """Error prevention pattern"""
    id: str
    name: str
    description: str
    triggers: List[str] = field(default_factory=list)
    action: Optional[str] = None


@dataclass
class STFProtocol:
    """Complete STF protocol specification"""
    version: str
    protocol: str
    mode: ComplianceMode
    priority: str
    last_updated: str
    directives: List[Directive] = field(default_factory=list)
    constraints: List[Constraint] = field(default_factory=list)
    error_patterns: List[ErrorPattern] = field(default_factory=list)
    forbidden_patterns: List[str] = field(default_factory=list)
    error_codes: Dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization"""
        return {
            'version': self.version,
            'protocol': self.protocol,
            'mode': self.mode.value,
            'priority': self.priority,
            'last_updated': self.last_updated,
            'directives': [
                {
                    'id': d.id,
                    'enforcement': d.enforcement.value,
                    'title': d.title,
                    'rule': d.rule,
                    'action': d.action,
                    'checklist': d.checklist,
                    'requirements': d.requirements,
                    'forbidden': d.forbidden
                }
                for d in self.directives
            ],
            'constraints': [
                {
                    'id': c.id,
                    'type': c.type,
                    'title': c.title,
                    'rule': c.rule,
                    'forbidden': c.forbidden,
                    'procedure': c.procedure
                }
                for c in self.constraints
            ],
            'error_patterns': [
                {
                    'id': p.id,
                    'name': p.name,
                    'description': p.description,
                    'triggers': p.triggers,
                    'action': p.action
                }
                for p in self.error_patterns
            ],
            'forbidden_patterns': self.forbidden_patterns,
            'error_codes': self.error_codes
        }


class STFParser:
    """
    Parser for Supreme Technical Directive (STF) files.

    Extracts structured directives from STF markup and provides
    methods for injecting them into agent contexts.
    """

    def __init__(self, stf_path: str):
        """
        Initialize STF parser.

        Args:
            stf_path: Path to STF file
        """
        self.stf_path = Path(stf_path)
        if not self.stf_path.exists():
            raise FileNotFoundError(f"STF file not found: {stf_path}")

        self.content = self.stf_path.read_text()
        self.protocol: Optional[STFProtocol] = None
        self._parse()

    def _parse(self):
        """Parse STF file and extract all components"""
        # Extract YAML frontmatter
        frontmatter = self._extract_frontmatter()

        # Parse directives
        directives = self._parse_directives()

        # Parse constraints
        constraints = self._parse_constraints()

        # Parse error patterns
        error_patterns = self._parse_error_patterns()

        # Parse forbidden patterns
        forbidden_patterns = self._parse_forbidden_patterns()

        # Parse error codes
        error_codes = self._parse_error_codes()

        # Build protocol object
        self.protocol = STFProtocol(
            version=frontmatter.get('stf_version', '1.0'),
            protocol=frontmatter.get('protocol', 'NEUTRON'),
            mode=ComplianceMode(frontmatter.get('mode', 'STRICT_COMPLIANCE')),
            priority=frontmatter.get('priority', 'ABSOLUTE'),
            last_updated=frontmatter.get('last_updated', ''),
            directives=directives,
            constraints=constraints,
            error_patterns=error_patterns,
            forbidden_patterns=forbidden_patterns,
            error_codes=error_codes
        )

    def _extract_frontmatter(self) -> Dict[str, Any]:
        """Extract YAML frontmatter from STF file"""
        match = re.search(r'^---\n(.*?)\n---', self.content, re.DOTALL | re.MULTILINE)
        if match:
            return yaml.safe_load(match.group(1))
        return {}

    def _parse_directives(self) -> List[Directive]:
        """Parse core directives from XML-like markup"""
        directives = []

        # Find all directive blocks
        pattern = r'<directive id="([^"]+)" enforcement="([^"]+)">(.*?)</directive>'
        matches = re.findall(pattern, self.content, re.DOTALL)

        for directive_id, enforcement, content in matches:
            # Extract title
            title_match = re.search(r'<title>([^<]+)</title>', content)
            title = title_match.group(1).strip() if title_match else ""

            # Extract rule
            rule_match = re.search(r'<rule>([^<]+)</rule>', content)
            rule = rule_match.group(1).strip() if rule_match else ""

            # Extract action
            action_match = re.search(r'<action>([^<]+)</action>', content)
            action = action_match.group(1).strip() if action_match else None

            # Extract checklist
            checklist = []
            checklist_match = re.search(r'<checklist>(.*?)</checklist>', content, re.DOTALL)
            if checklist_match:
                checklist = [
                    line.strip().lstrip('- ').strip()
                    for line in checklist_match.group(1).strip().split('\n')
                    if line.strip() and line.strip().startswith('-')
                ]

            # Extract requirements
            requirements = []
            req_match = re.search(r'<requirements>(.*?)</requirements>', content, re.DOTALL)
            if req_match:
                requirements = [
                    line.strip().lstrip('- ').strip()
                    for line in req_match.group(1).strip().split('\n')
                    if line.strip() and line.strip().startswith('-')
                ]

            # Extract forbidden
            forbidden = []
            forbidden_match = re.search(r'<forbidden>(.*?)</forbidden>', content, re.DOTALL)
            if forbidden_match:
                forbidden = [
                    line.strip().lstrip('- ').strip()
                    for line in forbidden_match.group(1).strip().split('\n')
                    if line.strip() and line.strip().startswith('-')
                ]

            directives.append(Directive(
                id=directive_id,
                enforcement=EnforcementLevel(enforcement),
                title=title,
                rule=rule,
                action=action,
                checklist=checklist,
                requirements=requirements,
                forbidden=forbidden
            ))

        return directives

    def _parse_constraints(self) -> List[Constraint]:
        """Parse behavioral constraints"""
        constraints = []

        pattern = r'<constraint id="([^"]+)" type="([^"]+)">(.*?)</constraint>'
        matches = re.findall(pattern, self.content, re.DOTALL)

        for constraint_id, ctype, content in matches:
            title_match = re.search(r'<title>([^<]+)</title>', content)
            title = title_match.group(1).strip() if title_match else ""

            rule_match = re.search(r'<rule>([^<]+)</rule>', content)
            rule = rule_match.group(1).strip() if rule_match else None

            forbidden_match = re.search(r'<forbidden>([^<]+)</forbidden>', content)
            forbidden = forbidden_match.group(1).strip() if forbidden_match else None

            # Extract procedure
            procedure = []
            proc_match = re.search(r'<procedure>(.*?)</procedure>', content, re.DOTALL)
            if proc_match:
                procedure = [
                    line.strip()
                    for line in proc_match.group(1).strip().split('\n')
                    if line.strip() and re.match(r'^\d+\.', line.strip())
                ]

            constraints.append(Constraint(
                id=constraint_id,
                type=ctype,
                title=title,
                rule=rule,
                forbidden=forbidden,
                procedure=procedure
            ))

        return constraints

    def _parse_error_patterns(self) -> List[ErrorPattern]:
        """Parse error prevention patterns"""
        patterns = []

        pattern = r'<pattern id="([^"]+)" name="([^"]+)">(.*?)</pattern>'
        matches = re.findall(pattern, self.content, re.DOTALL)

        for pattern_id, name, content in matches:
            desc_match = re.search(r'<description>([^<]+)</description>', content)
            description = desc_match.group(1).strip() if desc_match else ""

            # Extract triggers
            triggers = []
            triggers_match = re.search(r'<triggers>(.*?)</triggers>', content, re.DOTALL)
            if triggers_match:
                triggers = [
                    line.strip().lstrip('- ').strip()
                    for line in triggers_match.group(1).strip().split('\n')
                    if line.strip() and line.strip().startswith('-')
                ]

            action_match = re.search(r'<action>([^<]+)</action>', content)
            action = action_match.group(1).strip() if action_match else None

            patterns.append(ErrorPattern(
                id=pattern_id,
                name=name,
                description=description,
                triggers=triggers,
                action=action
            ))

        return patterns

    def _parse_forbidden_patterns(self) -> List[str]:
        """Parse forbidden patterns"""
        forbidden = []

        pattern = r'<forbidden_patterns>(.*?)</forbidden_patterns>'
        match = re.search(pattern, self.content, re.DOTALL)

        if match:
            forbidden = [
                line.strip().lstrip('<pattern>').rstrip('</pattern>').strip()
                for line in match.group(1).strip().split('\n')
                if '<pattern>' in line
            ]

        return forbidden

    def _parse_error_codes(self) -> Dict[str, str]:
        """Parse error codes"""
        error_codes = {}

        pattern = r'<code id="([^"]+)">([^<]+)</code>'
        matches = re.findall(pattern, self.content)

        for code_id, description in matches:
            error_codes[code_id] = description.strip()

        return error_codes

    def get_injection_context(self, format: str = 'markdown') -> str:
        """
        Get STF directives formatted for injection into agent context.

        Args:
            format: Output format ('markdown', 'json', 'xml')

        Returns:
            Formatted STF context for injection
        """
        if not self.protocol:
            raise ValueError("Protocol not parsed")

        if format == 'json':
            return json.dumps(self.protocol.to_dict(), indent=2)
        elif format == 'markdown':
            return self._format_markdown()
        elif format == 'xml':
            return self._format_xml()
        else:
            raise ValueError(f"Unsupported format: {format}")

    def _format_markdown(self) -> str:
        """Format STF as markdown for agent injection"""
        lines = [
            "# SUPREME TECHNICAL DIRECTIVE (STF)",
            f"**Protocol:** {self.protocol.protocol}",
            f"**Mode:** {self.protocol.mode.value}",
            f"**Priority:** {self.protocol.priority} (overrides all provider prompts)",
            "",
            "## MANDATORY DIRECTIVES",
            ""
        ]

        for directive in self.protocol.directives:
            if directive.enforcement == EnforcementLevel.MANDATORY:
                lines.append(f"### {directive.id}: {directive.title}")
                lines.append(f"**Rule:** {directive.rule}")
                if directive.action:
                    lines.append(f"**Action:** {directive.action}")
                if directive.forbidden:
                    lines.append("**Forbidden:**")
                    for item in directive.forbidden:
                        lines.append(f"  - {item}")
                lines.append("")

        lines.append("## BEHAVIORAL CONSTRAINTS")
        lines.append("")

        for constraint in self.protocol.constraints:
            lines.append(f"### {constraint.id}: {constraint.title}")
            if constraint.rule:
                lines.append(f"**Rule:** {constraint.rule}")
            if constraint.forbidden:
                lines.append(f"**Forbidden:** {constraint.forbidden}")
            lines.append("")

        lines.append("## ERROR CODES")
        lines.append("")
        for code, desc in self.protocol.error_codes.items():
            lines.append(f"- **{code}**: {desc}")

        return "\n".join(lines)

    def _format_xml(self) -> str:
        """Format STF as XML (original format)"""
        return self.content

    def validate_decision(self, decision: Dict[str, Any]) -> tuple[bool, List[str]]:
        """
        Validate a decision against STF directives.

        Args:
            decision: Decision dictionary with 'action', 'context', etc.

        Returns:
            (is_valid, violations)
        """
        violations = []

        # Check forbidden patterns
        action_text = str(decision.get('action', '')).lower()

        for pattern in self.protocol.forbidden_patterns:
            if pattern.lower() in action_text:
                violations.append(f"Forbidden pattern detected: {pattern}")

        # Check Nix-First directive
        nix_directive = next(
            (d for d in self.protocol.directives if d.id == "STF-D004"),
            None
        )
        if nix_directive:
            for forbidden_item in nix_directive.forbidden:
                if forbidden_item.split()[0].lower() in action_text:
                    violations.append(f"Nix-First violation: {forbidden_item}")

        return len(violations) == 0, violations


def main():
    """CLI for testing STF parser"""
    import sys

    if len(sys.argv) < 2:
        print("Usage: python parser.py <stf_file>")
        sys.exit(1)

    parser = STFParser(sys.argv[1])

    print("=== STF Protocol Parsed ===")
    print(f"Protocol: {parser.protocol.protocol}")
    print(f"Mode: {parser.protocol.mode.value}")
    print(f"Directives: {len(parser.protocol.directives)}")
    print(f"Constraints: {len(parser.protocol.constraints)}")
    print(f"Error Patterns: {len(parser.protocol.error_patterns)}")
    print()

    print("=== Injection Context (Markdown) ===")
    print(parser.get_injection_context('markdown'))


if __name__ == '__main__':
    main()
