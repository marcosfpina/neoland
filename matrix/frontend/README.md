# Neoland Operational Interface

The official frontend orchestrator for the Neoland ecosystem. A technical, high-performance console designed for agent orchestration, pipeline observation, and session forensics.

## 🚀 Overview

Neoland UI repositioned from the Matrix base to serve as the primary workbench for the Neoland control plane. It provides real-time visibility into agentic workflows and architecture decisions.

## ✨ Core Pillars

### ⚡ Pipeline Intelligence
- **Live Progression**: Watch agent execution steps in real-time.
- **Confidence & Risk**: Monitor real-time sentiment and risk analysis from the Phantom scanner.
- **Escalation Management**: Handle violet-state escalations requiring human intervention.

### 🔍 Session Forensics
- **State Inspection**: Deep dive into individual session variables and history.
- **Timeline Reconstruction**: Rebuild the sequence of agent decisions.
- **Mmap IPC Status**: Monitor low-level synchronization between Rust and Python agents.

### 📜 ADR Vault
- **Architecture Decisions**: Render ADRs as readable, versioned ledger entries.
- **Checkpoint Verification**: Verify signed architecture checkpoints via the `adr-ledger`.
- **Relationship Graph**: Visualize how decisions enable or supersede each other.

### 🏥 Service Health
- **Ecosystem Status**: Health checks for Rust Server, NATS, and DSPy Pipelines.
- **Prometheus Metrics**: Visualize counters and histograms for agent efficiency.
- **Dependency Map**: Real-time graph of service interconnections.

## 🛠️ Technology Stack

- **Framework**: Next.js 15, React 19, TypeScript
- **Styling**: Tailwind CSS, shadcn/ui components (Neoland semantic palette)
- **State Management**: React Query (TanStack) for backend synchronization
- **Icons**: Lucide React

## 🚀 Getting Started

### Prerequisites

- Nix with flakes enabled

### Installation & Execution

1. **Enter Development Shell**
   ```bash
   nix develop
   ```

2. **Start Backend Services**
   Ensure the Neoland Rust control plane is running (default REST port 3001).

3. **Start Frontend**
   ```bash
   cd matrix/frontend
   npm run dev
   ```

4. **Access Dashboard**
   Navigate to [http://localhost:3006](http://localhost:3006)

## 🎨 Semantic Visual Language

Neoland uses strict semantic coloring for operational clarity:
- 🟢 **Approve/Healthy**: Operations proceeding as planned.
- 🔴 **Reject/Failure**: System failure or rejected decision.
- 🟠 **Defer/Warning**: Degraded state or non-critical warning.
- 🟣 **Escalate/Human**: Decision requires human-in-the-loop intervention.

---
**Built for the Neoland Control Plane**
