import { scaleLinear, scalePoint } from "@visx/scale";
import { LinePath } from "@visx/shape";
import { ParentSize } from "@visx/responsive";
import { localPoint } from "@visx/event";
import { TooltipWithBounds, defaultStyles, useTooltip } from "@visx/tooltip";
import { formatMinor } from "../lib/money";
import { CHART_MARGIN, pickIndices, shortAxisLabel, yTickValues } from "../lib/chartTicks";

export default function TrendChart({
  points,
  color,
  locale,
  mode,
}: {
  points: { key: string; amountMinor: number }[];
  color: string;
  locale: string;
  mode: string;
}) {
  const { tooltipData, tooltipLeft, tooltipTop, tooltipOpen, showTooltip, hideTooltip } = useTooltip<{
    key: string;
    amountMinor: number;
  }>();

  return (
    <div className="chart">
      <ParentSize>
        {({ width, height }) => {
          if (width < 80 || height < 60 || points.length === 0) return null;
          const x = scalePoint({
            domain: points.map((p) => p.key),
            range: [CHART_MARGIN.left, width - CHART_MARGIN.right],
          });
          const max = Math.max(...points.map((p) => p.amountMinor), 0);
          const y = scaleLinear({
            domain: [0, Math.max(max, 1)],
            range: [height - CHART_MARGIN.bottom, CHART_MARGIN.top],
          });
          const yTicks = yTickValues(max);
          const xIdx = pickIndices(points.length, 8);
          const monthly =
            mode === "custom" && points.length > 1 && points.every((p) => p.key.endsWith("-01"));
          const axisY = height - CHART_MARGIN.bottom;
          return (
            <>
              <svg
                width={width}
                height={height}
                onMouseLeave={hideTooltip}
                onMouseMove={(event) => {
                  const pt = localPoint(event);
                  if (!pt) return;
                  let best = points[0];
                  let bestDist = Infinity;
                  for (const p of points) {
                    const dist = Math.abs((x(p.key) ?? 0) - pt.x);
                    if (dist < bestDist) {
                      bestDist = dist;
                      best = p;
                    }
                  }
                  showTooltip({
                    tooltipData: best,
                    tooltipLeft: pt.x,
                    tooltipTop: pt.y,
                  });
                }}
              >
                <line
                  x1={CHART_MARGIN.left}
                  x2={CHART_MARGIN.left}
                  y1={CHART_MARGIN.top}
                  y2={axisY}
                  stroke="var(--line)"
                />
                <line
                  x1={CHART_MARGIN.left}
                  x2={width - CHART_MARGIN.right}
                  y1={axisY}
                  y2={axisY}
                  stroke="var(--line)"
                />
                {yTicks.map((tick) => (
                  <g key={tick}>
                    <line
                      x1={CHART_MARGIN.left - 4}
                      x2={CHART_MARGIN.left}
                      y1={y(tick)}
                      y2={y(tick)}
                      stroke="var(--line)"
                    />
                    <text
                      className="chart-axis mono"
                      x={CHART_MARGIN.left - 6}
                      y={y(tick)}
                      dy="0.35em"
                      textAnchor="end"
                    >
                      {formatMinor(tick, locale)}
                    </text>
                  </g>
                ))}
                {xIdx.map((i) => {
                  const p = points[i];
                  const cx = x(p.key) ?? 0;
                  return (
                    <g key={p.key}>
                      <line x1={cx} x2={cx} y1={axisY} y2={axisY + 4} stroke="var(--line)" />
                      <text className="chart-axis mono" x={cx} y={axisY + 16} textAnchor="middle">
                        {shortAxisLabel(p.key, mode, "trend", monthly)}
                      </text>
                    </g>
                  );
                })}
                <LinePath
                  data={points}
                  x={(d) => x(d.key) ?? 0}
                  y={(d) => y(d.amountMinor)}
                  stroke={color}
                  strokeWidth={1.75}
                  fill="transparent"
                />
                {points.map((p) => (
                  <circle
                    key={p.key}
                    cx={x(p.key) ?? 0}
                    cy={y(p.amountMinor)}
                    r={tooltipData?.key === p.key ? 4 : 3}
                    fill={color}
                  />
                ))}
              </svg>
              {tooltipOpen && tooltipData && (
                <TooltipWithBounds
                  top={tooltipTop}
                  left={tooltipLeft}
                  className="chart-tooltip"
                  style={{
                    ...defaultStyles,
                    background: "var(--surface)",
                    color: "var(--text)",
                    border: "1px solid var(--line)",
                    boxShadow: "none",
                    borderRadius: "var(--radius)",
                    padding: "4px 8px",
                    fontSize: 12,
                  }}
                >
                  <span className="mono">
                    {tooltipData.key} {formatMinor(tooltipData.amountMinor, locale)}
                  </span>
                </TooltipWithBounds>
              )}
            </>
          );
        }}
      </ParentSize>
    </div>
  );
}
