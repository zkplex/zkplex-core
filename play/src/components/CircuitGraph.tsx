import { useMemo } from 'react';
import { parseCircuit, astToGraph, layoutGraph, GraphNode } from '../utils/circuitParser';

interface CircuitGraphProps {
  circuit: string;
  containerWidth: number;
  containerHeight: number;
  onClick?: () => void;
}

export function CircuitGraph({
  circuit,
  containerWidth,
  containerHeight,
  onClick
}: CircuitGraphProps) {
  // Parse circuit and generate graph
  const graph = useMemo(() => {
    try {
      const ast = parseCircuit(circuit);
      const graphData = astToGraph(ast);
      const positionedNodes = layoutGraph(graphData.nodes, graphData.edges);

      return {
        nodes: positionedNodes,
        edges: graphData.edges,
      };
    } catch (error) {
      console.error('Circuit parsing error:', error);
      return { nodes: [], edges: [] };
    }
  }, [circuit]);

  // Get node color based on type
  const getNodeColor = (type: GraphNode['type']) => {
    switch (type) {
      case 'output':
        return { fill: 'transparent', stroke: '#22c55e', text: '#166534' };
      case 'operation':
        return { fill: 'transparent', stroke: '#3b82f6', text: '#1e3a8a' };
      case 'variable':
        return { fill: 'transparent', stroke: '#a855f7', text: '#581c87' };
      case 'constant':
        return { fill: 'transparent', stroke: '#94a3b8', text: '#475569' };
      default:
        return { fill: 'transparent', stroke: '#6b7280', text: '#374151' };
    }
  };

  if (graph.nodes.length === 0) {
    return (
      <div
        className="flex items-center justify-center text-sm text-muted-foreground"
        style={{ width: containerWidth, height: containerHeight }}
      >
        Enter a circuit to visualize
      </div>
    );
  }

  // Step 1: Calculate bounding box
  const padding = 40;
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

  graph.nodes.forEach(node => {
    const nodeMinX = node.type === 'operation' || node.type === 'output'
      ? node.x! - 20  // circle radius
      : node.x! - 40; // rectangle half-width
    const nodeMaxX = node.type === 'operation' || node.type === 'output'
      ? node.x! + 20
      : node.x! + 40;
    const nodeMinY = node.type === 'output' ? node.y! - 40 : node.y! - 30; // include OUTPUT text
    const nodeMaxY = node.type === 'operation' || node.type === 'output'
      ? node.y! + 20
      : node.y! + 15;

    minX = Math.min(minX, nodeMinX);
    maxX = Math.max(maxX, nodeMaxX);
    minY = Math.min(minY, nodeMinY);
    maxY = Math.max(maxY, nodeMaxY);
  });

  // Add padding
  minX -= padding;
  minY -= padding;
  maxX += padding;
  maxY += padding;

  const graphWidth = maxX - minX;
  const graphHeight = maxY - minY;

  // Step 2: Calculate scale to fit container while preserving aspect ratio
  const scaleX = containerWidth / graphWidth;
  const scaleY = containerHeight / graphHeight;
  const scale = Math.min(scaleX, scaleY);

  const scaledWidth = graphWidth * scale;
  const scaledHeight = graphHeight * scale;

  return (
    <div
      className={`flex items-center justify-center ${onClick ? 'cursor-pointer' : ''}`}
      style={{ width: containerWidth, height: containerHeight }}
      onClick={onClick}
    >
      <svg
        width={scaledWidth}
        height={scaledHeight}
        viewBox={`${minX} ${minY} ${graphWidth} ${graphHeight}`}
        className="block"
      >
        {/* Define arrow marker */}
        <defs>
          <marker
            id="arrowhead"
            markerWidth="10"
            markerHeight="10"
            refX="9"
            refY="3"
            orient="auto"
          >
            <polygon points="0 0, 10 3, 0 6" fill="#9ca3af" />
          </marker>
        </defs>

        {/* Draw edges FIRST */}
        <g>
          {graph.edges.map((edge, i) => {
            const fromNode = graph.nodes.find(n => n.id === edge.from);
            const toNode = graph.nodes.find(n => n.id === edge.to);
            if (!fromNode || !toNode) return null;

            const dx = toNode.x! - fromNode.x!;
            const dy = toNode.y! - fromNode.y!;
            const distance = Math.sqrt(dx * dx + dy * dy);
            const normX = dx / distance;
            const normY = dy / distance;

            // Calculate start point (from node boundary)
            let x1, y1;
            if (fromNode.type === 'operation' || fromNode.type === 'output') {
              x1 = fromNode.x! + normX * 22;
              y1 = fromNode.y! + normY * 22;
            } else {
              const rectWidth = 80;
              const rectHeight = 30;
              const halfWidth = rectWidth / 2;
              const halfHeight = rectHeight / 2;
              const tx = normX !== 0 ? halfWidth / Math.abs(normX) : Infinity;
              const ty = normY !== 0 ? halfHeight / Math.abs(normY) : Infinity;
              const t = Math.min(tx, ty);
              x1 = fromNode.x! + normX * t;
              y1 = fromNode.y! + normY * t;
            }

            // Calculate end point (to node boundary)
            let x2, y2;
            if (toNode.type === 'operation' || toNode.type === 'output') {
              x2 = toNode.x! - normX * 22;
              y2 = toNode.y! - normY * 22;
            } else {
              const rectWidth = 80;
              const rectHeight = 30;
              const halfWidth = rectWidth / 2;
              const halfHeight = rectHeight / 2;
              const tx = normX !== 0 ? halfWidth / Math.abs(normX) : Infinity;
              const ty = normY !== 0 ? halfHeight / Math.abs(normY) : Infinity;
              const t = Math.min(tx, ty);
              x2 = toNode.x! - normX * t;
              y2 = toNode.y! - normY * t;
            }

            return (
              <line
                key={`edge-${i}`}
                x1={x1}
                y1={y1}
                x2={x2}
                y2={y2}
                stroke="#9ca3af"
                strokeWidth="1"
                markerEnd="url(#arrowhead)"
              />
            );
          })}
        </g>

        {/* Draw nodes SECOND */}
        <g>
          {graph.nodes.map(node => {
            const colors = getNodeColor(node.type);

            return (
              <g key={node.id}>
                {/* Node shape */}
                {node.type === 'operation' || node.type === 'output' ? (
                  <circle
                    cx={node.x}
                    cy={node.y}
                    r="20"
                    fill={colors.fill}
                    stroke={colors.stroke}
                    strokeWidth="1"
                  />
                ) : (
                  <rect
                    x={node.x! - 40}
                    y={node.y! - 15}
                    width="80"
                    height="30"
                    rx="5"
                    fill={colors.fill}
                    stroke={colors.stroke}
                    strokeWidth="1"
                  />
                )}

                {/* Node label */}
                <text
                  x={node.x}
                  y={node.y! + 5}
                  textAnchor="middle"
                  fill={colors.text}
                  fontSize={node.type === 'operation' || node.type === 'output' ? '14' : '12'}
                  fontWeight="600"
                  fontFamily="monospace"
                >
                  {node.label}
                </text>

                {/* Type badge */}
                <text
                  x={node.x}
                  y={node.y! - 30}
                  textAnchor="middle"
                  fill="#9ca3af"
                  fontSize="10"
                  fontFamily="sans-serif"
                >
                  {node.type === 'output' ? 'OUTPUT' : ''}
                </text>
              </g>
            );
          })}
        </g>
      </svg>
    </div>
  );
}