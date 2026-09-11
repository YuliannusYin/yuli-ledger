import { scaleBand, scaleLinear } from "@visx/scale";
import { ParentSize } from "@visx/responsive";
import { localPoint } from "@visx/event";
import { TooltipWithBounds, defaultStyles, useTooltip } from "@visx/tooltip";
import { formatMinor } from "../lib/money";
import { CHART_MARGIN, shortAxisLabel, yTickValues } from "../lib/chartTicks";

export default function BarChart({
  bars,
  color,
  locale,
  mode,
}: {
  bars: { key: string; amountMinor: number; isCurrent: boolean }[];
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
          if (width < 80 || height < 60) return null;
          const x = scaleBand({
            domain: bars.map((b) => b.key),
            range: [CHART_MARGIN.left, width - CHART_MARGIN.right],
            padding: 0.25,
          });
          const max = Math.max(...bars.map((b) => b.amountMinor), 0);
          const y = scaleLinear({
            domain: [0, Math.max(max, 1)],
            range: [height - CHART_MARGIN.bottom, CHART_MARGIN.top],
          });
          const yTicks = yTickValues(max);
          const axisY = height - CHART_MARGIN.bottom;
          const bw = x.bandwidth();
          return (
            <>
              <svg
                width={width}
                height={height}
                onMouseLeave={hideTooltip}
                onMouseMove={(event) => {
                  const pt = localPoint(event);
                  if (!pt) return;
                  const hit = bars.find((b) => {
                    const left = x(b.key) ?? 0;
                    return pt.x >= left && pt.x <= left + bw;
                  });
                  if (!hit) {
                    hideTooltip();
                    return;
                  }
                  showTooltip({
                    tooltipData: hit,
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
                {bars.map((b) => {
                  const cx = (x(b.key) ?? 0) + bw / 2;
                  return (
                    <g key={`tick-${b.key}`}>
                      <line x1={cx} x2={cx} y1={axisY} y2={axisY + 4} stroke="var(--line)" />
                      <text className="chart-axis mono" x={cx} y={axisY + 16} textAnchor="middle">
                        {shortAxisLabel(b.key, mode, "comparison")}
                      </text>
                    </g>
                  );
                })}
                {bars.map((b) => {
                  const bh = Math.max(1, y(0) - y(b.amountMinor));
                  return (
                    <rect
                      key={b.key}
                      x={x(b.key)}
                      y={y(b.amountMinor)}
                      width={bw}
                      height={bh}
                      fill={color}
                      opacity={b.isCurrent ? 1 : 0.7}
                      stroke={b.isCurrent ? "var(--text)" : "none"}
                      strokeWidth={b.isCurrent ? 1 : 0}
                    />
                  );
                })}
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
                    borderRadius: 2,
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
