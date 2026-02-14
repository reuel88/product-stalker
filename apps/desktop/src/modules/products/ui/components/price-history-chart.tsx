import {
	CartesianGrid,
	Legend,
	Line,
	LineChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";

import { formatPrice } from "@/modules/products/price-utils";
import type {
	MultiRetailerChartData,
	RetailerChartSeries,
} from "@/modules/products/types";
import { useDateFormat } from "@/modules/shared/hooks/useDateFormat";

interface PriceHistoryChartProps {
	chartData: MultiRetailerChartData;
}

interface CustomTooltipProps {
	active?: boolean;
	payload?: Array<{
		name: string;
		value: number;
		color: string;
	}>;
	label?: string;
	currency: string;
	currencyExponent: number;
	formatTooltipDate: (dateString: string) => string;
}

function calculateYAxisDomain(
	data: Array<Record<string, string | number>>,
	series: RetailerChartSeries[],
): [number, number] {
	const prices: number[] = [];
	const seriesIds = new Set(series.map((s) => s.id));

	for (const row of data) {
		for (const [key, value] of Object.entries(row)) {
			if (seriesIds.has(key) && typeof value === "number") {
				prices.push(value);
			}
		}
	}

	if (prices.length === 0) return [0, 100];

	const minPrice = Math.min(...prices);
	const maxPrice = Math.max(...prices);
	const padding = Math.max((maxPrice - minPrice) * 0.1, 100);

	return [minPrice - padding, maxPrice + padding];
}

function CustomTooltip({
	active,
	payload,
	label,
	currency,
	currencyExponent,
	formatTooltipDate,
}: CustomTooltipProps) {
	if (!active || !payload || payload.length === 0 || !label) {
		return null;
	}

	return (
		<div className="rounded-none border bg-background px-3 py-2 shadow-sm">
			<p className="mb-1 text-muted-foreground text-xs">
				{formatTooltipDate(label)}
			</p>
			{payload.map((entry) => (
				<div key={entry.name} className="flex items-center gap-2">
					<span
						className="inline-block size-2 rounded-full"
						style={{ backgroundColor: entry.color }}
					/>
					<span className="font-medium text-sm">
						{formatPrice(entry.value, currency, currencyExponent)}
					</span>
					{payload.length > 1 && (
						<span className="text-muted-foreground text-xs">{entry.name}</span>
					)}
				</div>
			))}
		</div>
	);
}

export function PriceHistoryChart({ chartData }: PriceHistoryChartProps) {
	const { formatChartAxisDate, formatChartTooltipDate } = useDateFormat();
	const { data, series, currency, currencyExponent } = chartData;

	if (data.length === 0) {
		return (
			<div className="flex h-[200px] items-center justify-center text-muted-foreground">
				No price data available
			</div>
		);
	}

	if (data.length === 1 && series.length === 1) {
		const point = data[0];
		const price = point[series[0].id];
		return (
			<div className="flex h-[200px] flex-col items-center justify-center gap-2">
				<p className="font-medium text-2xl">
					{formatPrice(
						typeof price === "number" ? price : null,
						currency,
						currencyExponent,
					)}
				</p>
				<p className="text-muted-foreground text-sm">
					Recorded on {formatChartTooltipDate(point.date as string)}
				</p>
				<p className="text-muted-foreground text-xs">
					More data points needed to show price trend
				</p>
			</div>
		);
	}

	const yAxisDomain = calculateYAxisDomain(data, series);

	return (
		<ResponsiveContainer width="100%" height={200}>
			<LineChart data={data} margin={{ top: 5, right: 5, left: 5, bottom: 5 }}>
				<CartesianGrid
					strokeDasharray="3 3"
					className="stroke-muted"
					vertical={false}
				/>
				<XAxis
					dataKey="date"
					tickFormatter={formatChartAxisDate}
					tick={{ fontSize: 10 }}
					tickLine={false}
					axisLine={false}
					className="fill-muted-foreground"
				/>
				<YAxis
					domain={yAxisDomain}
					tickFormatter={(value) =>
						formatPrice(value, currency, currencyExponent)
					}
					tick={{ fontSize: 10 }}
					tickLine={false}
					axisLine={false}
					width={70}
					className="fill-muted-foreground"
				/>
				<Tooltip
					content={
						<CustomTooltip
							currency={currency}
							currencyExponent={currencyExponent}
							formatTooltipDate={formatChartTooltipDate}
						/>
					}
				/>
				{series.length > 1 && <Legend />}
				{series.map((s) => (
					<Line
						key={s.id}
						type="monotone"
						dataKey={s.id}
						name={s.label}
						stroke={s.color}
						strokeWidth={2}
						connectNulls
						dot={{
							r: 4,
							fill: s.color,
							stroke: s.color,
							strokeWidth: 0,
						}}
						activeDot={{
							r: 6,
							fill: s.color,
							stroke: s.color,
							strokeWidth: 0,
						}}
					/>
				))}
			</LineChart>
		</ResponsiveContainer>
	);
}
