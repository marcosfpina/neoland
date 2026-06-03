# PHANTOM UX → Agent Pipeline
## Workflow Completo: Design System → Terminal Agent → Production Code

### 1. Setup Inicial (Uma vez)

```bash
# Clone o repo com suas UX specs
git clone git@github.com:pina/phantom-ux-specs.git ~/.config/phantom/ux-specs

# Enter no flake environment
cd phantom-ux && nix develop

# Ou adiciona ao seu sistema NixOS
sudo nixos-rebuild switch --flake .#phantom-ux
```

### 2. Criar Nova UX Spec

```bash
# Cria spec interativamente
phantom-ux create threat-analysis-dashboard

# Template YAML será criado:
# ~/.config/phantom/ux-specs/threat-analysis-dashboard.yml
```

Estrutura da spec (YAML):
```yaml
name: threat-analysis-dashboard
version: 1.0.0

philosophy:
  tone: "cyberpunk brutalism"
  purpose: "Real-time threat correlation and analysis"
  memorable_element: "3D attack surface visualization with live particle effects"
  target_audience: "Security Operations Center analysts"

typography:
  display_font: "Orbitron"      # Futuristic, geometric
  mono_font: "JetBrains Mono"   # Code/data
  body_font: "Inter Variable"   # Fallback
  scale_ratio: 1.25             # Major third
  weights: [400, 500, 700]

color_system:
  mode: "dark"
  base:
    bg_primary: "#0A0E27"
    bg_secondary: "#151B3D"
    bg_tertiary: "#1E2749"
  text:
    primary: "#E0E6FF"
    secondary: "#8B93B0"
    tertiary: "#5A6279"
  accents:
    primary: "#00F5FF"      # Cyan
    secondary: "#FF00E5"    # Magenta
  semantic:
    critical: "#FF3366"
    high: "#FFAA00"
    medium: "#3366FF"
    low: "#00FF88"

layout_rules:
  grid:
    columns: 12
    gutter: "24px"
    breakpoints:
      mobile: "480px"
      tablet: "768px"
      desktop: "1024px"
      wide: "1440px"
  asymmetry: true
  direction: "left-heavy"  # Main content left, sidebar right
  overlap: true            # Panels can overlap with z-index

animation:
  page_load:
    strategy: "staggered_reveal"
    base_delay: 0.3
    stagger: 0.1
  interactions:
    hover_duration: 0.2
    hover_effect: "glow + scale"
    active_duration: 0.15
  data_updates:
    number_transition: "countup"
    graph_easing: "elastic"
  
constraints:
  framework: "React 18+"
  styling: "Tailwind CSS v3+"
  animation_lib: "Framer Motion"
  canvas: "Three.js" # para 3D graph
  state: "Zustand"
  charts: "Recharts"
  performance:
    bundle_js: "< 150kb"
    bundle_css: "< 50kb"
    fcp: "< 1.5s"
    tti: "< 3s"
  accessibility: "WCAG AA"
  browsers: ["Chrome 120+", "Firefox 120+", "Safari 17+"]

anti_patterns:
  - "purple gradients on white"
  - "Inter/Roboto fonts"
  - "centered hero sections"
  - "generic card grids"
  - "excessive drop shadows"
```

### 3. Usar com Agente Terminal

#### 3.1. Via MCP Server (Recomendado para PHANTOM)

```bash
# Start MCP server (auto-start com PHANTOM_MCP_AUTO=1)
ux-spec-mcp-server &

# Em outro terminal, agent conecta via MCP
# Claude Desktop config (~/.config/Claude/claude_desktop_config.json):
{
  "mcpServers": {
    "ux-specs": {
      "command": "ux-spec-mcp-server",
      "env": {
        "UX_SPECS_DIR": "/home/pina/.config/phantom/ux-specs"
      }
    }
  }
}

# Agora Claude pode:
# - list_ux_specs() → mostra specs disponíveis
# - get_ux_spec(spec_name, format) → carrega spec
# - generate_component_prompt(spec, component_type) → gera prompt otimizado
```

#### 3.2. Via CLI Direto

```bash
# Gera prompt para agente
ux-prompt threat-analysis-dashboard ThreatCard > prompt.txt

# Envia pro Claude via API
claude_api < prompt.txt > ThreatCard.tsx

# Ou usa phantom-ux wrapper
phantom-ux gen threat-analysis-dashboard ThreatCard
# → Gera ./generated-component.tsx automaticamente
```

#### 3.3. Integração com Seu PHANTOM Framework

```typescript
// phantom-orchestrator.ts
import { PhantomAgent } from './phantom-core';
import { UxSpecClient } from './ux-spec-client';

const uxClient = new UxSpecClient({
  mcpServer: 'localhost:3000'
});

// Agent recebe spec via PHANTOM
const frontendAgent = new PhantomAgent({
  role: 'frontend-developer',
  capabilities: ['react', 'tailwind', 'framer-motion'],
  
  async onTask(task: FrontendTask) {
    // Carrega UX spec relevante
    const spec = await uxClient.getSpec(task.specName);
    
    // Injeta spec no contexto do agent
    this.context.uxSpec = spec;
    
    // Agent gera código seguindo spec
    const code = await this.generateComponent(task.component);
    
    // Valida contra spec
    const validation = await uxClient.validate(task.specName, code);
    
    return { code, validation };
  }
});

// Orchestrator delega tarefa
await frontendAgent.execute({
  specName: 'threat-analysis-dashboard',
  component: 'ThreatCard',
  requirements: 'Show real-time threat severity with pulse animation'
});
```

### 4. Validação Automática

```bash
# Valida código gerado contra spec
phantom-ux validate threat-analysis-dashboard ./ThreatCard.tsx

# Output:
# ✅ Font check passed (JetBrains Mono found)
# ✅ Color system check passed (#0A0E27 found)
# ⚠️  Animation duration mismatch (expected 0.2s, found 0.3s)
# ❌ Accessibility: Missing ARIA labels
```

### 5. CI/CD Integration

```yaml
# .github/workflows/ux-validation.yml
name: UX Spec Validation

on: [pull_request]

jobs:
  validate-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Nix
        uses: cachix/install-nix-action@v22
        
      - name: Enter PHANTOM UX environment
        run: nix develop
        
      - name: Validate all components
        run: |
          for component in src/components/*.tsx; do
            phantom-ux validate threat-analysis-dashboard "$component"
          done
          
      - name: Check bundle size
        run: |
          bun run build
          SIZE=$(stat -c%s dist/assets/*.js)
          MAX_SIZE=153600  # 150kb
          if [ $SIZE -gt $MAX_SIZE ]; then
            echo "Bundle too large: ${SIZE} > ${MAX_SIZE}"
            exit 1
          fi
```

### 6. Exemplo de Prompt Gerado

Quando você roda `phantom-ux gen threat-analysis-dashboard ThreatCard`, o agente recebe:

```markdown
# CREATE REACT COMPONENT: ThreatCard

## UX CONTEXT
AESTHETIC: Cyberpunk brutalism
PURPOSE: Display threat in real-time with severity indicators
TARGET: SOC analysts in high-stress environments

## DESIGN REQUIREMENTS

### Typography
- Headers: Orbitron, 1.5rem, 700 weight
- Data/Metrics: JetBrains Mono, 0.875rem, 500 weight
- Body: Inter Variable, 1rem, 400 weight

### Colors (Dark Mode)
Background: #1E2749 (tertiary)
Text: #E0E6FF (primary)
Border: rgba(0, 245, 255, 0.2) (accent cyan with alpha)
Severity indicator:
  - Critical: #FF3366 (left border 4px)
  - High: #FFAA00
  - Medium: #3366FF
  - Low: #00FF88

### Layout
- Card size: 320px width, auto height
- Padding: 1.5rem
- Border-radius: 8px
- Asymmetric: Content left-aligned, actions bottom-right

### Animation
- Hover: Glow effect (0 0 20px rgba(0,245,255,0.1)) + scale(1.02) - 200ms
- Severity pulse: Critical threats pulse at 1s interval
- Data updates: CountUp animation on metric changes

### Components Needed
1. Severity indicator (colored dot + left border)
2. Threat name (Orbitron, truncate if > 40 chars)
3. Source/Dest IPs (mono font, muted color)
4. Mini sparkline graph (last 10 data points)
5. Action button (Investigate → with accent cyan)

## TECHNICAL CONSTRAINTS
- React + TypeScript
- Tailwind CSS only (no separate CSS files)
- Framer Motion for animations
- Props: { threat: Threat, onInvestigate: () => void }
- WCAG AA compliant (4.5:1 contrast minimum)
- Mobile responsive (stack vertically < 768px)

## ANTI-PATTERNS TO AVOID
❌ Generic card with drop shadow
❌ Centered content
❌ Default Tailwind colors (use custom from spec)
❌ No animation
❌ Poor contrast

## EXPECTED OUTPUT
Production-ready React component with:
- Full TypeScript types
- Tailwind classes (no inline styles)
- Framer Motion animations
- Responsive design
- Accessibility features (ARIA labels, keyboard nav)

Generate the complete component now.
```

### 7. Advanced: Multi-Agent Workflow

```typescript
// phantom-ux-multi-agent.ts
import { PhantomOrchestrator } from './phantom-core';

const orchestrator = new PhantomOrchestrator({
  agents: {
    uxDesigner: {
      role: 'ux-designer',
      task: 'Create UX spec from requirements',
      output: 'ux-spec.yml'
    },
    
    frontendDev: {
      role: 'frontend-developer',
      task: 'Generate components from UX spec',
      input: 'ux-spec.yml',
      output: 'components/*.tsx'
    },
    
    qaEngineer: {
      role: 'qa-engineer',
      task: 'Validate components against spec',
      input: ['ux-spec.yml', 'components/*.tsx'],
      output: 'validation-report.md'
    }
  }
});

// Pipeline automático
await orchestrator.execute({
  userRequirement: "Create a threat analysis dashboard",
  workflow: [
    'uxDesigner',      // Gera spec YAML
    'frontendDev',     // Implementa componentes
    'qaEngineer'       // Valida tudo
  ]
});
```

### 8. Benefícios desta Abordagem

✅ **Declarativo**: UX spec é código (YAML/Git)
✅ **Reproduzível**: Mesmo spec → mesmo output
✅ **Validável**: Automação de QA
✅ **Modular**: Reutiliza specs entre projetos
✅ **Type-safe**: TypeScript em toda stack
✅ **NixOS Native**: Ambiente reproduzível
✅ **MCP Compatible**: Integra com Claude Desktop
✅ **PHANTOM Ready**: Orquestra multi-agent

### 9. Pro Tips

**Spec Versionamento**:
```bash
# Specs no Git com semantic versioning
git tag ux/threat-dashboard/v1.0.0
git tag ux/threat-dashboard/v1.1.0  # Breaking changes
```

**Spec Inheritance**:
```yaml
# base-dark-theme.yml
extends: null
color_system: { ... }

# threat-dashboard.yml
extends: base-dark-theme.yml
philosophy: { ... }  # Override só o necessário
```

**Dynamic Specs**:
```typescript
// Gera specs programaticamente
const spec = await generateUxSpec({
  purpose: task.description,
  brandColors: company.colors,
  constraints: project.techStack
});
```

---

## Resultado Final

Com este setup, você tem:

1. **UX Specs declarativas** (YAML versionado no Git)
2. **MCP Server** que serve specs para agentes
3. **CLI tools** (`phantom-ux`) para workflow manual
4. **Multi-agent orchestration** via PHANTOM Framework
5. **Validação automática** (CI/CD)
6. **NixOS integration** (ambiente reproduzível)

Agentes de terminal recebem instruções UX **precisas, completas e validáveis** - não mais "make it look nice" vago!

🎯 **Prompts são specs executáveis.**
