// Built-in prompt templates library

import type { PromptTemplate, PromptCategory } from "./types"

export const PROMPT_TEMPLATES: PromptTemplate[] = [
  {
    id: "code-review-expert",
    name: "Expert Code Review",
    description: "Comprehensive code review with security, performance, and best practices analysis",
    category: "code-review",
    template: `You are a senior software engineer conducting a thorough code review.

## Code to Review
\`\`\`{{language}}
{{code}}
\`\`\`

## Review Focus
{{focus}}

## Analysis Required
1. **Security Issues**: Identify potential vulnerabilities
2. **Performance**: Analyze time/space complexity and bottlenecks
3. **Best Practices**: Check adherence to {{language}} conventions
4. **Maintainability**: Assess readability and documentation
5. **Edge Cases**: Identify unhandled scenarios

Provide specific line references and actionable recommendations.`,
    variables: [
      {
        name: "language",
        type: "select",
        description: "Programming language",
        required: true,
        options: ["typescript", "python", "rust", "go", "javascript"],
      },
      { name: "code", type: "code", description: "Code to review", required: true },
      {
        name: "focus",
        type: "text",
        description: "Specific areas to focus on",
        required: false,
        default: "general review",
      },
    ],
    tags: ["review", "security", "performance", "quality"],
    rating: 4.8,
    usageCount: 1247,
    createdAt: new Date("2024-01-15"),
    updatedAt: new Date("2024-06-01"),
    author: "FORGE",
  },
  {
    id: "system-prompt-architect",
    name: "System Prompt Architect",
    description: "Generate powerful system prompts for AI agents",
    category: "system",
    template: `Design a system prompt for an AI agent with the following specifications:

## Agent Profile
- **Role**: {{role}}
- **Primary Function**: {{function}}
- **Target Users**: {{users}}
- **Tone**: {{tone}}

## Requirements
{{requirements}}

## Output Format
Create a comprehensive system prompt that includes:
1. Identity and role definition
2. Core capabilities and limitations
3. Response format guidelines
4. Guardrails and safety considerations
5. Example interaction patterns

The prompt should be:
- Clear and unambiguous
- Comprehensive but concise
- Aligned with best practices for {{role}} agents`,
    variables: [
      { name: "role", type: "text", description: "Agent role (e.g., coding assistant, data analyst)", required: true },
      { name: "function", type: "text", description: "Primary function", required: true },
      { name: "users", type: "text", description: "Target user audience", required: true },
      {
        name: "tone",
        type: "select",
        description: "Communication tone",
        required: true,
        options: ["professional", "friendly", "technical", "casual", "formal"],
      },
      {
        name: "requirements",
        type: "text",
        description: "Specific requirements and constraints",
        required: false,
        default: "None specified",
      },
    ],
    tags: ["system-prompt", "agent", "architecture"],
    rating: 4.9,
    usageCount: 892,
    createdAt: new Date("2024-02-01"),
    updatedAt: new Date("2024-05-15"),
    author: "FORGE",
  },
  {
    id: "debug-detective",
    name: "Debug Detective",
    description: "Systematic debugging analysis with root cause identification",
    category: "debugging",
    template: `You are a debugging expert. Analyze the following issue systematically.

## Error Context
**Error Message**: 
\`\`\`
{{error}}
\`\`\`

**Relevant Code**:
\`\`\`{{language}}
{{code}}
\`\`\`

**Environment**: {{environment}}
**Steps to Reproduce**: {{steps}}

## Analysis Framework
1. **Error Classification**: Categorize the error type
2. **Root Cause Analysis**: Identify the fundamental cause
3. **Impact Assessment**: Determine affected components
4. **Solution Paths**: Provide multiple fix approaches ranked by:
   - Implementation effort
   - Risk level
   - Long-term maintainability
5. **Prevention**: Suggest measures to prevent recurrence

Provide code fixes with explanations.`,
    variables: [
      { name: "error", type: "text", description: "Error message or stack trace", required: true },
      {
        name: "language",
        type: "select",
        description: "Programming language",
        required: true,
        options: ["typescript", "python", "rust", "go", "javascript", "shell"],
      },
      { name: "code", type: "code", description: "Relevant code snippet", required: true },
      { name: "environment", type: "text", description: "Runtime environment", required: false, default: "production" },
      { name: "steps", type: "text", description: "Steps to reproduce", required: false, default: "Not specified" },
    ],
    tags: ["debugging", "error", "troubleshooting"],
    rating: 4.7,
    usageCount: 2341,
    createdAt: new Date("2024-01-20"),
    updatedAt: new Date("2024-06-10"),
    author: "CIPHER",
  },
  {
    id: "doc-generator",
    name: "Documentation Generator",
    description: "Generate comprehensive documentation from code",
    category: "documentation",
    template: `Generate {{docType}} documentation for the following code.

## Code
\`\`\`{{language}}
{{code}}
\`\`\`

## Documentation Requirements
- **Style**: {{style}}
- **Audience**: {{audience}}
- **Include**: 
  - Purpose and overview
  - Parameters/arguments with types
  - Return values
  - Usage examples
  - Edge cases and error handling
  - Related functions/methods

Format the documentation according to {{language}} conventions (JSDoc, docstrings, rustdoc, etc.)`,
    variables: [
      {
        name: "docType",
        type: "select",
        description: "Documentation type",
        required: true,
        options: ["API reference", "tutorial", "README", "inline comments", "technical spec"],
      },
      {
        name: "language",
        type: "select",
        description: "Programming language",
        required: true,
        options: ["typescript", "python", "rust", "go"],
      },
      { name: "code", type: "code", description: "Code to document", required: true },
      {
        name: "style",
        type: "select",
        description: "Documentation style",
        required: false,
        default: "comprehensive",
        options: ["minimal", "comprehensive", "tutorial-style"],
      },
      {
        name: "audience",
        type: "select",
        description: "Target audience",
        required: false,
        default: "developers",
        options: ["beginners", "developers", "experts"],
      },
    ],
    tags: ["documentation", "api", "readme"],
    rating: 4.6,
    usageCount: 1876,
    createdAt: new Date("2024-02-15"),
    updatedAt: new Date("2024-05-20"),
    author: "SCRIBE",
  },
  {
    id: "refactor-advisor",
    name: "Refactor Advisor",
    description: "Intelligent code refactoring suggestions with patterns",
    category: "refactoring",
    template: `Analyze and suggest refactoring for the following code.

## Current Code
\`\`\`{{language}}
{{code}}
\`\`\`

## Refactoring Goals
{{goals}}

## Constraints
- Must maintain backward compatibility: {{backwardCompat}}
- Test coverage required: {{testRequired}}
- Performance critical: {{perfCritical}}

## Analysis
Provide:
1. **Code Smells**: Identify anti-patterns and issues
2. **Design Patterns**: Suggest applicable patterns
3. **Refactoring Steps**: Ordered list of safe refactoring operations
4. **Refactored Code**: Complete refactored version
5. **Migration Guide**: Steps to adopt the new code

Consider SOLID principles and {{language}} idioms.`,
    variables: [
      {
        name: "language",
        type: "select",
        description: "Programming language",
        required: true,
        options: ["typescript", "python", "rust", "go"],
      },
      { name: "code", type: "code", description: "Code to refactor", required: true },
      { name: "goals", type: "text", description: "Refactoring goals", required: true },
      {
        name: "backwardCompat",
        type: "boolean",
        description: "Maintain backward compatibility",
        required: false,
        default: "true",
      },
      { name: "testRequired", type: "boolean", description: "Generate tests", required: false, default: "true" },
      {
        name: "perfCritical",
        type: "boolean",
        description: "Performance critical code",
        required: false,
        default: "false",
      },
    ],
    tags: ["refactoring", "patterns", "clean-code"],
    rating: 4.5,
    usageCount: 1123,
    createdAt: new Date("2024-03-01"),
    updatedAt: new Date("2024-06-05"),
    author: "CIPHER",
  },
  {
    id: "test-generator",
    name: "Test Suite Generator",
    description: "Generate comprehensive test suites with edge cases",
    category: "testing",
    template: `Generate a comprehensive test suite for the following code.

## Code Under Test
\`\`\`{{language}}
{{code}}
\`\`\`

## Testing Framework
{{framework}}

## Test Requirements
- Coverage target: {{coverage}}%
- Include: {{testTypes}}

## Generate Tests For
1. **Happy Path**: Normal operation scenarios
2. **Edge Cases**: Boundary conditions and limits
3. **Error Handling**: Invalid inputs and error states
4. **Integration**: Component interactions (if applicable)
5. **Performance**: Benchmarks for critical paths (if {{perfTests}})

Provide complete, runnable test code with descriptive names.`,
    variables: [
      {
        name: "language",
        type: "select",
        description: "Programming language",
        required: true,
        options: ["typescript", "python", "rust", "go"],
      },
      { name: "code", type: "code", description: "Code to test", required: true },
      {
        name: "framework",
        type: "select",
        description: "Testing framework",
        required: true,
        options: ["vitest", "jest", "pytest", "cargo test", "go test"],
      },
      { name: "coverage", type: "number", description: "Target coverage percentage", required: false, default: "80" },
      {
        name: "testTypes",
        type: "text",
        description: "Types of tests to generate",
        required: false,
        default: "unit, integration",
      },
      {
        name: "perfTests",
        type: "boolean",
        description: "Include performance tests",
        required: false,
        default: "false",
      },
    ],
    tags: ["testing", "unit-tests", "coverage"],
    rating: 4.7,
    usageCount: 1654,
    createdAt: new Date("2024-02-20"),
    updatedAt: new Date("2024-06-01"),
    author: "CIPHER",
  },
  {
    id: "chain-cot-analysis",
    name: "Chain of Thought Analysis",
    description: "Multi-step reasoning chain for complex problems",
    category: "chain",
    template: `Analyze the following problem using chain-of-thought reasoning.

## Problem
{{problem}}

## Context
{{context}}

## Reasoning Chain
Work through this step by step:

### Step 1: Problem Decomposition
Break down the problem into smaller components.

### Step 2: Information Gathering
Identify what information is available and what's missing.

### Step 3: Hypothesis Generation
Generate potential solutions or approaches.

### Step 4: Evaluation
Evaluate each hypothesis against criteria.

### Step 5: Synthesis
Combine insights into a coherent solution.

### Step 6: Validation
Verify the solution addresses the original problem.

Show your reasoning at each step explicitly.`,
    variables: [
      { name: "problem", type: "text", description: "Problem statement", required: true },
      { name: "context", type: "text", description: "Additional context", required: false, default: "None provided" },
    ],
    tags: ["reasoning", "analysis", "chain-of-thought"],
    rating: 4.8,
    usageCount: 987,
    createdAt: new Date("2024-03-15"),
    updatedAt: new Date("2024-05-25"),
    author: "ATLAS",
  },
  {
    id: "prompt-optimizer",
    name: "Prompt Optimizer",
    description: "Enhance and optimize prompts for better results",
    category: "system",
    template: `Optimize the following prompt for better AI responses.

## Original Prompt
{{prompt}}

## Target Model
{{model}}

## Optimization Goals
{{goals}}

## Optimization Analysis
1. **Clarity Score**: Rate current clarity (1-10)
2. **Specificity Score**: Rate specificity (1-10)
3. **Structure Score**: Rate organization (1-10)

## Improvements
- **Clarity Enhancements**: Make instructions unambiguous
- **Context Addition**: Add necessary background
- **Constraint Specification**: Define boundaries and format
- **Example Inclusion**: Add relevant examples if helpful
- **Token Optimization**: Reduce while maintaining quality

## Optimized Prompt
[Provide the enhanced version]

## Explanation
[Explain key changes and expected improvements]`,
    variables: [
      { name: "prompt", type: "text", description: "Original prompt to optimize", required: true },
      {
        name: "model",
        type: "select",
        description: "Target model",
        required: false,
        default: "general",
        options: ["gpt-4", "claude", "gemini", "llama", "general"],
      },
      {
        name: "goals",
        type: "text",
        description: "Optimization goals",
        required: false,
        default: "improve clarity and response quality",
      },
    ],
    tags: ["optimization", "prompt-engineering", "enhancement"],
    rating: 4.9,
    usageCount: 2456,
    createdAt: new Date("2024-01-25"),
    updatedAt: new Date("2024-06-15"),
    author: "FORGE",
  },
]

export function getTemplateById(id: string): PromptTemplate | undefined {
  return PROMPT_TEMPLATES.find((t) => t.id === id)
}

export function getTemplatesByCategory(category: PromptCategory): PromptTemplate[] {
  return PROMPT_TEMPLATES.filter((t) => t.category === category)
}

export function searchTemplates(query: string): PromptTemplate[] {
  const lower = query.toLowerCase()
  return PROMPT_TEMPLATES.filter(
    (t) =>
      t.name.toLowerCase().includes(lower) ||
      t.description.toLowerCase().includes(lower) ||
      t.tags.some((tag) => tag.toLowerCase().includes(lower)),
  )
}
