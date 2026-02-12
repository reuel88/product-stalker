import {
	CartesianGrid,
	Line,
	LineChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";

import { formatPrice } from "@/modules/products/price-utils";
import type { PriceDataPoint } from "@/modules/products/types";
import { useDateFormat } from "@/modules/shared/hooks/useDateFormat";

const CHART_COLORS = {
	line: "var(--chart-3)",
	activeDot: "var(--chart-4)",
} as const;

interface PriceHistoryChartProps {
	data: PriceDataPoint[];
}

interface CustomTooltipProps {
	active?: boolean;
	payload?: Array<{
		payload: PriceDataPoint;
	}>;
	formatTooltipDate: (dateString: string) => string;
}

function calculateYAxisDomain(prices: number[]): [number, number] {
	const minPrice = Math.min(...prices);
	const maxPrice = Math.max(...prices);
	const padding = Math.max((maxPrice - minPrice) * 0.1, 100);

	return [minPrice - padding, maxPrice + padding];
}

function CustomTooltip({
	active,
	payload,
	formatTooltipDate,
}: CustomTooltipProps) {
	if (!active || !payload || payload.length === 0) {
		return null;
	}

	const data = payload[0].payload;
	const formattedPrice = formatPrice(
		data.price,
		data.currency,
		data.currencyExponent,
	);

	return (
		<div className="rounded-none border bg-background px-3 py-2 shadow-sm">
			<p className="font-medium text-sm">{formattedPrice}</p>
			<p className="text-muted-foreground text-xs">
				{formatTooltipDate(data.date)}
			</p>
		</div>
	);
}

export function PriceHistoryChart({ data }: PriceHistoryChartProps) {
	const { formatChartAxisDate, formatChartTooltipDate } = useDateFormat();

	if (data.length === 0) {
		return (
			<div className="flex h-[200px] items-center justify-center text-muted-foreground">
				No price data available
			</div>
		);
	}

	if (data.length === 1) {
		const point = data[0];
		return (
			<div className="flex h-[200px] flex-col items-center justify-center gap-2">
				<p className="font-medium text-2xl">
					{formatPrice(point.price, point.currency, point.currencyExponent)}
				</p>
				<p className="text-muted-foreground text-sm">
					Recorded on {formatChartTooltipDate(point.date)}
				</p>
				<p className="text-muted-foreground text-xs">
					More data points needed to show price trend
				</p>
			</div>
		);
	}

	const currency = data[0].currency;
	const currencyExponent = data[0].currencyExponent;
	const prices = data.map((d) => d.price);
	const yAxisDomain = calculateYAxisDomain(prices);

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
					content={<CustomTooltip formatTooltipDate={formatChartTooltipDate} />}
				/>
				<Line
					type="monotone"
					dataKey="price"
					stroke={CHART_COLORS.line}
					strokeWidth={2}
					dot={{
						r: 4,
						fill: CHART_COLORS.line,
						stroke: CHART_COLORS.line,
						strokeWidth: 0,
					}}
					activeDot={{
						r: 6,
						fill: CHART_COLORS.activeDot,
						stroke: CHART_COLORS.activeDot,
						strokeWidth: 0,
					}}
				/>
			</LineChart>
		</ResponsiveContainer>
	);
}
