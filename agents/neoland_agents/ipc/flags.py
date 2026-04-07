"""
Python companion to ``src/agents/flags.rs`` — reads and writes the shared-memory
flag region used for zero-copy IPC between the Rust control plane and the DSPy
pipeline.

Layout (must stay in sync with ``SharedFlags`` in flags.rs):

  Offset  Size  Type   Field
  ──────  ────  ─────  ─────────────────────────────────────────
  0       1     u8     pipeline_active       (0 = false, 1 = true)
  1       1     u8     escalate_to_architect (0 = false, 1 = true)
  2       1     u8     abort_requested       (0 = false, 1 = true)
  3       1     —      pad
  4       4     f32le  junior_confidence     (IEEE 754, little-endian)
  8       1     u8     risk_level            (0=low 1=medium 2=high 3=critical)
  9       7     —      pad
  16      36    bytes  session_id            (UTF-8, null-padded)
  52      12    —      pad
  ──────  ────
  Total   64

Usage::

    with AgentFlags() as flags:
        flags.set_pipeline_active(True)
        flags.set_session_id(session_id)
        flags.set_junior_confidence(0.72)
        ...
        state = flags.read()
        print(state["risk_level"])      # "medium"
"""

from __future__ import annotations

import mmap
import os
import struct
from enum import IntEnum
from pathlib import Path
from typing import TypedDict

# ──────────────────────────────────────────────────────────────────────────────
# Layout constants (must match flags.rs)
# ──────────────────────────────────────────────────────────────────────────────

_SHM_PATH = os.getenv("NEOLAND_SHM_PATH", "/run/neoland/agent-flags.shm")

# struct format: native byte order, standard sizes
# B  = unsigned char (1 byte)
# x  = pad byte
# f  = float (4 bytes, little-endian via '=')
# 7x = 7 pad bytes
# 36s = 36-byte char array
# 12x = 12 pad bytes
_FMT = "=BBBxfB7x36s12x"
_SIZE = struct.calcsize(_FMT)
assert _SIZE == 64, f"Layout mismatch: expected 64 bytes, got {_SIZE}"

# Field byte offsets (for single-field writes without unpacking the whole struct)
_OFF_PIPELINE_ACTIVE = 0
_OFF_ESCALATE = 1
_OFF_ABORT = 2
_OFF_CONFIDENCE = 4  # 4-byte float
_OFF_RISK = 8
_OFF_SESSION_ID = 16  # 36 bytes


# ──────────────────────────────────────────────────────────────────────────────
# Types
# ──────────────────────────────────────────────────────────────────────────────

class RiskLevel(IntEnum):
    LOW = 0
    MEDIUM = 1
    HIGH = 2
    CRITICAL = 3


class FlagsSnapshot(TypedDict):
    pipeline_active: bool
    escalate_to_architect: bool
    abort_requested: bool
    junior_confidence: float
    risk_level: str
    session_id: str


# ──────────────────────────────────────────────────────────────────────────────
# AgentFlags
# ──────────────────────────────────────────────────────────────────────────────

class AgentFlags:
    """Shared-memory view of the Rust ``SharedFlags`` struct.

    The file must already exist and have the correct size (64 bytes) — the
    Rust control plane creates and initialises it on startup.

    Typical use in the pipeline::

        with AgentFlags() as flags:
            flags.set_pipeline_active(True)
            flags.set_session_id(run_id)
            result = run_pipeline(task)
            flags.set_junior_confidence(result.junior.confidence)
            flags.set_risk_level(RiskLevel.HIGH)
            flags.set_pipeline_active(False)
    """

    def __init__(self, path: str | Path = _SHM_PATH) -> None:
        self.path = Path(path)
        self._fd: "int | None" = None
        self._mm: "mmap.mmap | None" = None

    # ── context manager ──────────────────────────────────────────────────────

    def open(self) -> "AgentFlags":
        """Open the shared-memory file.  Returns ``self`` for chaining."""
        self._fd = os.open(str(self.path), os.O_RDWR)
        self._mm = mmap.mmap(self._fd, _SIZE)
        return self

    def close(self) -> None:
        if self._mm is not None:
            self._mm.close()
            self._mm = None
        if self._fd is not None:
            os.close(self._fd)
            self._fd = None

    def __enter__(self) -> "AgentFlags":
        return self.open()

    def __exit__(self, *_: object) -> None:
        self.close()

    # ── read ─────────────────────────────────────────────────────────────────

    def read(self) -> FlagsSnapshot:
        """Return a consistent snapshot of all flags."""
        mm = self._require_open()
        mm.seek(0)
        raw = mm.read(_SIZE)
        pa, eta, ar, conf, risk, sid_bytes = struct.unpack(_FMT, raw)
        sid = sid_bytes.rstrip(b"\x00").decode("utf-8", errors="replace")
        risk_name = RiskLevel(risk).name.lower() if risk in range(4) else "unknown"
        return FlagsSnapshot(
            pipeline_active=bool(pa),
            escalate_to_architect=bool(eta),
            abort_requested=bool(ar),
            junior_confidence=conf,
            risk_level=risk_name,
            session_id=sid,
        )

    def abort_requested(self) -> bool:
        """Fast path: check only the abort flag (one byte read)."""
        mm = self._require_open()
        return bool(mm[_OFF_ABORT])

    # ── write ─────────────────────────────────────────────────────────────────

    def set_pipeline_active(self, active: bool) -> None:
        mm = self._require_open()
        mm[_OFF_PIPELINE_ACTIVE : _OFF_PIPELINE_ACTIVE + 1] = bytes([int(active)])
        mm.flush()

    def set_escalate_to_architect(self, escalate: bool) -> None:
        mm = self._require_open()
        mm[_OFF_ESCALATE : _OFF_ESCALATE + 1] = bytes([int(escalate)])
        mm.flush()

    def set_junior_confidence(self, confidence: float) -> None:
        mm = self._require_open()
        mm[_OFF_CONFIDENCE : _OFF_CONFIDENCE + 4] = struct.pack("=f", float(confidence))
        mm.flush()

    def set_risk_level(self, level: RiskLevel | int) -> None:
        mm = self._require_open()
        mm[_OFF_RISK : _OFF_RISK + 1] = bytes([int(level) & 0xFF])
        mm.flush()

    def set_session_id(self, session_id: str) -> None:
        mm = self._require_open()
        encoded = session_id.encode("utf-8")[:36].ljust(36, b"\x00")
        mm[_OFF_SESSION_ID : _OFF_SESSION_ID + 36] = encoded
        mm.flush()

    # ── helpers ───────────────────────────────────────────────────────────────

    def _require_open(self) -> mmap.mmap:
        if self._mm is None:
            raise RuntimeError("AgentFlags is not open — use as a context manager")
        return self._mm
