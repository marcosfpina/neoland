# Agentic Breakpoints And Live Steering Roadmap

**Last Updated**: 2026-05-17
**Current Status**: backend/TUI communication foundation is implemented; tool
interception and corrective replanning remain the next meaningful work.

---

## Goal

Agentic breakpoints make autonomous tool execution collaborative and safer. The
system pauses before sensitive tool calls, shows the operator what is about to
happen, accepts approve/reject/steer input, and feeds that decision back into
the running session.

---

## Current Architecture

1. **TUI (`src/tui/`)**: code-task workstation and live event consumer.
2. **Server (`src/server/mod.rs`)**: control plane, SSE stream, breakpoint resolution endpoint.
3. **Orchestrator (`src/agents/orchestrator.rs`)**: publishes agent events and waits for human decisions.
4. **MCP server/client (`src/mcp/`)**: native tool registry and call surface.
5. **Python DSPy pipeline (`agents/neoland_agents/`)**: reasoning engine and checkpoint producer.

---

## Implementation Status

## Phase 1 - Domain Events, API, And TUI Wiring

**Status**: `done`

- [x] `AgentEvent::BreakpointHit`
- [x] `AgentEvent::BreakpointResolved`
- [x] SSE parser support for `breakpoint_resolved`
- [x] TUI transition out of `WaitingForBreakpoint`
- [x] `POST /v1/agents/session/:id/breakpoint/resolve`
- [x] visible error routing for failed steering/breakpoint calls
- [x] serialization tests for breakpoint events

## Phase 2 - Async Orchestrator Blocking Path

**Status**: `mostly_done`

- [x] pending breakpoint registry exists in orchestrator;
- [x] resolver sends decision through the waiting channel;
- [x] resolved event is published after the oneshot send;
- [x] server passes `tool_name` into breakpoint registration;
- [ ] add a timeout/recovery policy for abandoned breakpoint waits.

## Phase 3 - Tool Interception

**Status**: `next`

The central design question remains:

- Rust-owned MCP path: Rust intercepts `mcp.call()`, asks for approval, then calls or rejects the tool.
- Python-owned tool path: Python calls back into Rust before executing a tool, and Rust blocks that call until the operator decides.

Preferred direction for Neoland's control-plane architecture:

- keep policy and operator approval in Rust;
- let Python receive a clear tool-denied or steer-corrected result;
- keep the TUI as a code-task operator interface, not a general dashboard.

Work:

- [ ] choose the tool ownership path for the first release;
- [ ] intercept at least shell/file-modifying tools;
- [ ] publish `BreakpointHit` before execution;
- [ ] enforce approve/reject/steer result;
- [ ] return structured errors to Python and TUI.

## Phase 4 - Corrective Live Steering

**Status**: `planned`

- [ ] treat `steer` as a corrective instruction, not only a rejection;
- [ ] feed the instruction into the next planning step;
- [ ] preserve the decision in session/checkpoint state;
- [ ] show the correction in frontend session/ADR forensics.

---

## Release Cut

Breakpoints should block public release only for tool actions that can mutate
files, shell state, network state, or secrets. Rich corrective replanning can
ship after the first public pre-release if the first release does not expose
dangerous autonomous tools by default.
