// Circuit parser for visualization
// Based on circuit.pest grammar

export type ASTNode =
  | { type: 'assignment'; variable: string; expression: ASTNode }
  | { type: 'binary'; op: string; left: ASTNode; right: ASTNode }
  | { type: 'unary'; op: string; operand: ASTNode }
  | { type: 'variable'; name: string }
  | { type: 'number'; value: string }
  | { type: 'boolean'; value: boolean };

export type GraphNode = {
  id: string;
  type: 'operation' | 'variable' | 'constant' | 'output';
  label: string;
  level: number;
  x?: number;
  y?: number;
};

export type GraphEdge = {
  from: string;
  to: string;
};

// Simple tokenizer
type Token = {
  type: 'op' | 'lparen' | 'rparen' | 'variable' | 'number' | 'boolean' | 'assign';
  value: string;
};

function tokenize(input: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;

  while (i < input.length) {
    const char = input[i];

    // Skip whitespace
    if (/\s/.test(char)) {
      i++;
      continue;
    }

    // Operators
    if (char === '(') {
      tokens.push({ type: 'lparen', value: '(' });
      i++;
    } else if (char === ')') {
      tokens.push({ type: 'rparen', value: ')' });
      i++;
    } else if (char === '+' || char === '-' || char === '*' || char === '/') {
      tokens.push({ type: 'op', value: char });
      i++;
    } else if (input.slice(i, i + 3) === '<==') {
      tokens.push({ type: 'assign', value: '<==' });
      i += 3;
    } else if (input.slice(i, i + 2) === '>=' || input.slice(i, i + 2) === '<=' ||
               input.slice(i, i + 2) === '==' || input.slice(i, i + 2) === '!=' ||
               input.slice(i, i + 2) === '||' || input.slice(i, i + 2) === '&&') {
      tokens.push({ type: 'op', value: input.slice(i, i + 2) });
      i += 2;
    } else if (char === '>' || char === '<' || char === '!') {
      tokens.push({ type: 'op', value: char });
      i++;
    } else if (/[A-Z]/i.test(char) || char === '_') {
      // Variable or keyword
      let j = i;
      while (j < input.length && (/[A-Z0-9_]/i.test(input[j]))) {
        j++;
      }
      const word = input.slice(i, j);

      // Check for keywords
      if (word === 'AND' || word === 'OR' || word === 'NOT') {
        tokens.push({ type: 'op', value: word });
      } else if (word === 'true' || word === 'TRUE' || word === 'false' || word === 'FALSE') {
        tokens.push({ type: 'boolean', value: word.toLowerCase() });
      } else {
        tokens.push({ type: 'variable', value: word });
      }
      i = j;
    } else if (/[0-9]/.test(char)) {
      // Number
      let j = i;
      while (j < input.length && /[0-9]/.test(input[j])) {
        j++;
      }
      tokens.push({ type: 'number', value: input.slice(i, j) });
      i = j;
    } else {
      // Unknown character, skip
      i++;
    }
  }

  return tokens;
}

// Precedence levels (higher = higher precedence)
const precedence: Record<string, number> = {
  'OR': 1, '||': 1,
  'AND': 2, '&&': 2,
  '>': 3, '<': 3, '>=': 3, '<=': 3, '==': 3, '!=': 3,
  '+': 4, '-': 4,
  '*': 5, '/': 5,
  'NOT': 6, '!': 6,
  'unary-': 6,
};

// Parse expression using Shunting Yard algorithm
function parseExpression(tokens: Token[]): ASTNode | null {
  if (tokens.length === 0) return null;

  const outputQueue: ASTNode[] = [];
  const operatorStack: Token[] = [];

  let i = 0;

  while (i < tokens.length) {
    const token = tokens[i];

    if (token.type === 'number') {
      outputQueue.push({ type: 'number', value: token.value });
    } else if (token.type === 'boolean') {
      outputQueue.push({ type: 'boolean', value: token.value === 'true' });
    } else if (token.type === 'variable') {
      outputQueue.push({ type: 'variable', name: token.value });
    } else if (token.type === 'op') {
      // Handle unary operators
      if ((token.value === '-' || token.value === 'NOT' || token.value === '!') &&
          (i === 0 || tokens[i - 1].type === 'op' || tokens[i - 1].type === 'lparen')) {
        operatorStack.push({ type: 'op', value: token.value === '-' ? 'unary-' : token.value });
      } else {
        // Binary operator
        while (
          operatorStack.length > 0 &&
          operatorStack[operatorStack.length - 1].type === 'op' &&
          precedence[operatorStack[operatorStack.length - 1].value] >= precedence[token.value]
        ) {
          const op = operatorStack.pop()!;
          const isUnary = op.value === 'NOT' || op.value === '!' || op.value === 'unary-';

          if (isUnary) {
            const operand = outputQueue.pop()!;
            outputQueue.push({ type: 'unary', op: op.value === 'unary-' ? '-' : op.value, operand });
          } else {
            const right = outputQueue.pop()!;
            const left = outputQueue.pop()!;
            outputQueue.push({ type: 'binary', op: op.value, left, right });
          }
        }
        operatorStack.push(token);
      }
    } else if (token.type === 'lparen') {
      operatorStack.push(token);
    } else if (token.type === 'rparen') {
      while (operatorStack.length > 0 && operatorStack[operatorStack.length - 1].type !== 'lparen') {
        const op = operatorStack.pop()!;
        const isUnary = op.value === 'NOT' || op.value === '!' || op.value === 'unary-';

        if (isUnary) {
          const operand = outputQueue.pop()!;
          outputQueue.push({ type: 'unary', op: op.value === 'unary-' ? '-' : op.value, operand });
        } else {
          const right = outputQueue.pop()!;
          const left = outputQueue.pop()!;
          outputQueue.push({ type: 'binary', op: op.value, left, right });
        }
      }
      operatorStack.pop(); // Remove '('
    }

    i++;
  }

  while (operatorStack.length > 0) {
    const op = operatorStack.pop()!;
    const isUnary = op.value === 'NOT' || op.value === '!' || op.value === 'unary-';

    if (isUnary) {
      const operand = outputQueue.pop()!;
      outputQueue.push({ type: 'unary', op: op.value === 'unary-' ? '-' : op.value, operand });
    } else {
      const right = outputQueue.pop()!;
      const left = outputQueue.pop()!;
      outputQueue.push({ type: 'binary', op: op.value, left, right });
    }
  }

  return outputQueue.length > 0 ? outputQueue[0] : null;
}

export function parseCircuit(circuit: string): ASTNode | null {
  try {
    const tokens = tokenize(circuit);

    // Check if this is an assignment statement (variable <== expression)
    const assignIndex = tokens.findIndex(t => t.type === 'assign');
    if (assignIndex > 0 && tokens[0].type === 'variable') {
      const variable = tokens[0].value;
      const exprTokens = tokens.slice(assignIndex + 1);
      const expression = parseExpression(exprTokens);
      if (expression) {
        return { type: 'assignment', variable, expression };
      }
    }

    return parseExpression(tokens);
  } catch (e) {
    console.error('Parse error:', e);
    return null;
  }
}

// Convert AST to graph representation
let nodeIdCounter = 0;

export function astToGraph(ast: ASTNode | null): { nodes: GraphNode[]; edges: GraphEdge[] } {
  nodeIdCounter = 0;
  const nodes: GraphNode[] = [];
  const edges: GraphEdge[] = [];

  if (!ast) return { nodes, edges };

  function traverse(node: ASTNode, level: number): string {
    const id = `node_${nodeIdCounter++}`;

    if (node.type === 'assignment') {
      // Create output node for the assignment variable
      nodes.push({ id, type: 'output', label: node.variable, level });

      // Traverse the expression
      const exprId = traverse(node.expression, level + 1);
      edges.push({ from: exprId, to: id });
    } else if (node.type === 'binary') {
      nodes.push({ id, type: 'operation', label: node.op, level });

      const leftId = traverse(node.left, level + 1);
      const rightId = traverse(node.right, level + 1);

      edges.push({ from: leftId, to: id });
      edges.push({ from: rightId, to: id });
    } else if (node.type === 'unary') {
      nodes.push({ id, type: 'operation', label: node.op, level });

      const operandId = traverse(node.operand, level + 1);
      edges.push({ from: operandId, to: id });
    } else if (node.type === 'variable') {
      nodes.push({ id, type: 'variable', label: node.name, level });
    } else if (node.type === 'number') {
      nodes.push({ id, type: 'constant', label: node.value, level });
    } else if (node.type === 'boolean') {
      nodes.push({ id, type: 'constant', label: String(node.value), level });
    }

    return id;
  }

  traverse(ast, 0);

  // Mark root node as output if it's not already marked (for non-assignment statements)
  if (nodes.length > 0 && nodes[0].type !== 'output') {
    nodes[0].type = 'output';
  }

  return { nodes, edges };
}

// Calculate node positions for visualization
export function layoutGraph(nodes: GraphNode[], _edges: GraphEdge[]): GraphNode[] {
  const levelWidth = 120;
  const nodeHeight = 60;

  // Group nodes by level
  const levelGroups: Record<number, GraphNode[]> = {};
  nodes.forEach(node => {
    if (!levelGroups[node.level]) levelGroups[node.level] = [];
    levelGroups[node.level].push(node);
  });

  // Calculate positions
  const positionedNodes = nodes.map(node => {
    const levelNodes = levelGroups[node.level];
    const indexInLevel = levelNodes.indexOf(node);
    const totalInLevel = levelNodes.length;

    return {
      ...node,
      x: 250 + (indexInLevel - (totalInLevel - 1) / 2) * levelWidth,
      y: 50 + node.level * nodeHeight,
    };
  });

  return positionedNodes;
}