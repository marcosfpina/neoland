---
name: ux-phantom-security-dashboard
type: frontend-ux-specification
version: 1.0.0
---

# UX Specification for PHANTOM Security Dashboard

## Design Philosophy

**Purpose**: Real-time security operations interface for threat analysis and incident response.

**Target Audience**: 
- Security Engineers (like you!)
- SOC Analysts
- DevSecOps teams
- High-stress, time-critical decision making

**Aesthetic Direction**: **Industrial Brutalism + Cyberpunk Neon**

Think: Dark server rooms, neon-lit terminals, Matrix-meets-Minority-Report, surgical precision over decoration.

**The Unforgettable Element**: 
Animated 3D network topology that reacts to threats in real-time - nodes pulse with activity, threat vectors glow red with particle trails.

---

## Typography System

### Font Choices
```
Display: "Space Grotesk" (geometric, slightly futuristic)
Mono: "JetBrains Mono" (for code/data/metrics)
Body: "Inter Variable" (only if needed for long text)
```

### Scale & Hierarchy
```
Hero: 4rem / 700 weight
Section: 2.5rem / 600 weight  
Subsection: 1.5rem / 500 weight
Body: 1rem / 400 weight
Caption: 0.875rem / 400 weight
Code/Data: 0.875rem mono / 500 weight
```

### Rules
- Monospace dominates (60% of text should be mono)
- Never center-align code blocks
- Line height: 1.5 for body, 1.2 for headings, 1.6 for code

---

## Color System

### Base Palette
```css
--bg-primary: #0A0E27      /* Deep navy - main background */
--bg-secondary: #151B3D    /* Slightly lighter panels */
--bg-tertiary: #1E2749     /* Cards/modals */

--text-primary: #E0E6FF    /* Almost white, slight blue tint */
--text-secondary: #8B93B0  /* Muted for secondary info */
--text-tertiary: #5A6279   /* Disabled/placeholder */

--accent-cyan: #00F5FF     /* Primary CTA, active states */
--accent-magenta: #FF00E5  /* Warnings, special alerts */

--semantic-danger: #FF3366  /* Critical threats */
--semantic-warning: #FFAA00 /* Medium threats */
--semantic-success: #00FF88 /* Resolved/safe */
--semantic-info: #3366FF    /* Informational */
```

### Usage Rules
- Background MUST be dark (#0A0E27 base)
- Accent colors used SPARINGLY (< 10% of UI)
- Semantic colors ONLY for their purpose
- Gradients ONLY on accent elements, never backgrounds
- Glow effects on interactive elements using box-shadow

---

## Layout & Spatial Composition

### Grid System
```
Base: 12-column grid with 24px gutters
Breakpoints: 
  - mobile: 480px
  - tablet: 768px  
  - desktop: 1024px
  - wide: 1440px
```

### Asymmetry & Flow
- LEFT heavy (nav + main content) vs RIGHT sidebar (alerts/stats)
- DIAGONAL data flows (top-left → bottom-right threat progression)
- OVERLAPPING panels with z-index depth
- GENEROUS negative space between critical elements

### Component Spacing
```
Section margins: 4rem vertical
Card padding: 2rem
Element spacing: 1.5rem default
Micro-spacing: 0.5rem for related items
```

---

## Motion & Animation

### Page Load Sequence
```javascript
1. Background fade-in (0.3s)
2. Logo + header (0.5s, slide from top)
3. Main content (0.7s, staggered reveal - each item +100ms delay)
4. Sidebar stats (1s, slide from right)
5. Network graph (1.2s, nodes pop in with elastic easing)
```

### Micro-interactions
```
Hover states: 
  - Buttons: subtle glow + scale(1.02) - 200ms
  - Cards: lift with shadow increase - 300ms
  - Links: cyan underline slide-in - 150ms

Active threats:
  - Pulse animation (1s infinite)
  - Red glow intensity oscillation
  - Particle trail on threat path (canvas animation)

Data updates:
  - Number transitions with CountUp.js
  - Graph nodes: gentle bounce on value change
  - Alert badges: attention-grabbing shake
```

### Scroll Behavior
- Parallax on background elements (0.5x scroll speed)
- Sticky nav with backdrop blur
- Scroll-triggered fade-ins for sections

---

## Component Patterns

### Threat Card
```
Structure:
┌─────────────────────────────────┐
│ [SEVERITY_DOT] Threat Name      │ ← Mono font, 1rem
│                                 │
│ Source IP: xxx.xxx.xxx.xxx      │ ← Mono, muted
│ Destination: service_name       │
│                                 │
│ [GRAPH_MINI] Activity timeline  │ ← Sparkline
│                                 │
│ [ACTION_BUTTON] Investigate →   │ ← Accent cyan
└─────────────────────────────────┘

Style:
- Background: var(--bg-tertiary)
- Border: 1px solid rgba(0, 245, 255, 0.2)
- Border-left: 4px solid SEVERITY_COLOR
- Glow on hover: 0 0 20px rgba(0, 245, 255, 0.1)
```

### Network Graph
```
Canvas-based force-directed graph:
- Nodes: Circles, size based on connection count
- Active nodes: Pulsing glow (requestAnimationFrame)
- Threat paths: Red particles moving along edges
- User interaction: Drag nodes, zoom/pan
- Performance: Max 100 nodes visible, LOD for distant nodes
```

### Metrics Panel
```
[ICON] Metric Name
─────────────────
█████████░░░░░░  [85%]  ← Progress bar with gradient
Current: 1,234 ← CountUp animation
Change: +12% ↑  ← Green if positive, red if negative

Style:
- Large monospace numbers (2rem)
- Subtle background gradient based on status
- Icon: Lucide React, accent color
```

---

## Technical Implementation

### Stack Requirements
```
Framework: React 18+ with Hooks
Styling: Tailwind CSS v3+ (utility-first)
Animation: Framer Motion for React components
Canvas: Three.js for 3D graph (if needed) OR D3.js for 2D
State: Zustand or Jotai (lightweight)
Charts: Recharts (customized heavily)
```

### Performance Budgets
```
Initial JS: < 150kb gzipped
Initial CSS: < 50kb
Fonts: < 100kb (woff2 subset)
Images: WebP, lazy-loaded
First Contentful Paint: < 1.5s
Time to Interactive: < 3s
```

### Accessibility
```
WCAG AA compliance:
- Contrast ratios: 4.5:1 minimum (text/background)
- Keyboard navigation: Tab order logical
- Screen reader: ARIA labels on graphs/charts
- Focus indicators: Visible outline (2px cyan)
- No color-only status (icons + text always)
```

---

## Anti-Patterns to AVOID

### Design
❌ Purple gradients on white backgrounds (overused AI aesthetic)
❌ Centered hero sections with generic CTAs
❌ Generic card grids (Pinterest layout)
❌ Overuse of drop shadows
❌ Excessive animations (distraction)

### Typography  
❌ Inter/Roboto/Arial (boring defaults)
❌ Too many font families (> 3)
❌ Poor hierarchy (everything same size)

### Color
❌ Low-contrast color schemes
❌ Rainbow gradients everywhere
❌ Using accent colors for large areas

### Code
❌ Inline styles (use Tailwind classes)
❌ !important overrides
❌ Massive component files (> 300 lines)
❌ No component reusability

---

## Validation Checklist

Before considering the UI complete, verify:

- [ ] Aesthetic is DISTINCTIVE (not generic AI slop)
- [ ] Typography uses specified fonts + scale
- [ ] Color system matches specification exactly
- [ ] Animations feel polished, not janky
- [ ] Performance budgets met
- [ ] Accessibility standards met (WCAG AA)
- [ ] Mobile responsive (all breakpoints tested)
- [ ] Dark theme consistent throughout
- [ ] Network graph is the MEMORABLE element
- [ ] Code is production-ready (no TODOs, proper error handling)

---

## Example Prompt for Agent

```
Create a React + Tailwind dashboard component following this UX spec:

AESTHETIC: Industrial brutalism + cyberpunk neon
- Dark navy background (#0A0E27)
- Cyan accents (#00F5FF) used sparingly
- JetBrains Mono for all data/metrics
- Space Grotesk for headers

LAYOUT: Asymmetric with LEFT main content + RIGHT sidebar
- 12-column grid, 24px gutters
- Generous vertical spacing (4rem between sections)
- Overlapping panels with depth

UNFORGETTABLE: Animated network graph
- Canvas-based force-directed layout
- Nodes pulse on activity
- Red particle trails for threat paths

COMPONENTS NEEDED:
1. Threat Cards (with severity indicators, sparklines)
2. Network Topology Graph (interactive, real-time)
3. Metrics Panel (CountUp animations, progress bars)
4. Alert Feed (auto-scroll, priority sorting)

MOTION:
- Staggered reveal on page load (100ms delays)
- Hover: subtle glow + scale(1.02)
- Data updates: smooth number transitions

CONSTRAINTS:
- React + Tailwind only (no separate CSS files)
- Performance: < 150kb JS bundle
- WCAG AA compliant
- Works in Chrome/Firefox/Safari

Generate production-ready code with exceptional attention to aesthetic details.
```
