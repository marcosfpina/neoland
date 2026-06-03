// ux-spec-mcp-server.ts
// MCP Server que serve UX specifications para agentes
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import fs from 'fs/promises';
import path from 'path';
import yaml from 'yaml';

interface UxSpec {
  name: string;
  philosophy: {
    tone: string;
    purpose: string;
    memorable_element: string;
  };
  typography: Record<string, any>;
  color_system: Record<string, any>;
  layout_rules: Record<string, any>;
  animation: Record<string, any>;
  constraints: Record<string, any>;
}

const server = new Server(
  {
    name: 'ux-spec-server',
    version: '1.0.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// Diretório onde ficam as specs UX
const UX_SPECS_DIR = process.env.UX_SPECS_DIR || './ux-specs';

// Tool 1: Listar specs disponíveis
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'list_ux_specs',
        description: 'Lista todas as especificações UX disponíveis',
        inputSchema: {
          type: 'object',
          properties: {},
        },
      },
      {
        name: 'get_ux_spec',
        description: 'Retorna uma especificação UX completa para implementação',
        inputSchema: {
          type: 'object',
          properties: {
            spec_name: {
              type: 'string',
              description: 'Nome da spec UX (ex: phantom-security-dashboard)',
            },
            format: {
              type: 'string',
              enum: ['yaml', 'json', 'markdown', 'prompt'],
              description: 'Formato de saída',
              default: 'markdown',
            },
          },
          required: ['spec_name'],
        },
      },
      {
        name: 'generate_component_prompt',
        description: 'Gera prompt otimizado para agente criar componente seguindo UX spec',
        inputSchema: {
          type: 'object',
          properties: {
            spec_name: {
              type: 'string',
              description: 'Nome da spec UX',
            },
            component_type: {
              type: 'string',
              description: 'Tipo de componente (card, graph, panel, etc)',
            },
            requirements: {
              type: 'string',
              description: 'Requisitos específicos adicionais',
            },
          },
          required: ['spec_name', 'component_type'],
        },
      },
    ],
  };
});

// Handler para as tools
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name: toolName, arguments: args } = request.params;

  switch (toolName) {
    case 'list_ux_specs': {
      const files = await fs.readdir(UX_SPECS_DIR);
      const specs = files
        .filter(f => f.endsWith('.yml') || f.endsWith('.yaml'))
        .map(f => path.basename(f, path.extname(f)));
      
      return {
        content: [
          {
            type: 'text',
            text: JSON.stringify({ specs }, null, 2),
          },
        ],
      };
    }

    case 'get_ux_spec': {
      const { spec_name, format = 'markdown' } = args as {
        spec_name: string;
        format?: string;
      };

      const specPath = path.join(UX_SPECS_DIR, `${spec_name}.yml`);
      const specContent = await fs.readFile(specPath, 'utf-8');
      const spec: UxSpec = yaml.parse(specContent);

      let output: string;

      switch (format) {
        case 'yaml':
          output = specContent;
          break;
        case 'json':
          output = JSON.stringify(spec, null, 2);
          break;
        case 'prompt':
          output = generateAgentPrompt(spec);
          break;
        case 'markdown':
        default:
          output = generateMarkdownSpec(spec);
          break;
      }

      return {
        content: [
          {
            type: 'text',
            text: output,
          },
        ],
      };
    }

    case 'generate_component_prompt': {
      const { spec_name, component_type, requirements } = args as {
        spec_name: string;
        component_type: string;
        requirements?: string;
      };

      const specPath = path.join(UX_SPECS_DIR, `${spec_name}.yml`);
      const specContent = await fs.readFile(specPath, 'utf-8');
      const spec: UxSpec = yaml.parse(specContent);

      const prompt = generateComponentPrompt(spec, component_type, requirements);

      return {
        content: [
          {
            type: 'text',
            text: prompt,
          },
        ],
      };
    }

    default:
      throw new Error(`Unknown tool: ${toolName}`);
  }
});

// Gera prompt otimizado para agente
function generateAgentPrompt(spec: UxSpec): string {
  return `
# UX SPECIFICATION FOR AGENT

## Design Philosophy
TONE: ${spec.philosophy.tone}
PURPOSE: ${spec.philosophy.purpose}
UNFORGETTABLE ELEMENT: ${spec.philosophy.memorable_element}

## Typography
${Object.entries(spec.typography).map(([k, v]) => `${k}: ${v}`).join('\n')}

## Color System
${Object.entries(spec.color_system).map(([k, v]) => `${k}: ${v}`).join('\n')}

## Layout Rules
${Object.entries(spec.layout_rules).map(([k, v]) => `${k}: ${v}`).join('\n')}

## Animation
${Object.entries(spec.animation).map(([k, v]) => `${k}: ${v}`).join('\n')}

## Constraints
${Object.entries(spec.constraints).map(([k, v]) => `${k}: ${v}`).join('\n')}

---
IMPORTANT: Follow this specification EXACTLY. Do not use generic AI aesthetics.
Generate production-ready code with exceptional attention to aesthetic details.
`.trim();
}

function generateMarkdownSpec(spec: UxSpec): string {
  // Similar ao template que criei anteriormente
  return `# ${spec.name}

## Design Philosophy
- **Tone**: ${spec.philosophy.tone}
- **Purpose**: ${spec.philosophy.purpose}
- **Memorable Element**: ${spec.philosophy.memorable_element}

[... resto da spec em formato markdown ...]
`;
}

function generateComponentPrompt(
  spec: UxSpec,
  componentType: string,
  requirements?: string
): string {
  return `
Create a React component: ${componentType}

UX CONTEXT:
- Design tone: ${spec.philosophy.tone}
- Purpose: ${spec.philosophy.purpose}

STYLING REQUIREMENTS:
- Typography: ${JSON.stringify(spec.typography)}
- Colors: ${JSON.stringify(spec.color_system)}
- Layout: ${JSON.stringify(spec.layout_rules)}
- Animations: ${JSON.stringify(spec.animation)}

ADDITIONAL REQUIREMENTS:
${requirements || 'None'}

CONSTRAINTS:
${Object.entries(spec.constraints).map(([k, v]) => `- ${k}: ${v}`).join('\n')}

Generate production-ready React + Tailwind code following these specifications EXACTLY.
Do not use generic components - make this distinctive and memorable.
`.trim();
}

// Inicializa servidor
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('UX Spec MCP Server running on stdio');
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
