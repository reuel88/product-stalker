import {
	CartesianGrid,
	Line,
	LineChart,
	ResponsiveContainer,
	Tooltip,
	XAxis,
	YAxis,
} from "recharts";

import { formatPrice } from "@/lib/utils";
import type { PriceDataPoint } from "@/modules/products/types";

interface PriceHistoryChartProps {
	data: PriceDataPoint[];
}

interface CustomTooltipProps {
	active?: boolean;
	payload?: Array<{
		payload: PriceDataPoint;
	}>;
}

function formatDate(dateString: string): string {
	return new Date(dateString).toLocaleDateString(undefined, {
		month: "short",
		day: "numeric",
	});
}

function formatTooltipDate(dateString: string): string {
	return new Date(dateString).toLocaleDateString(undefined, {
		year: "numeric",
		month: "short",
		day: "numeric",
		hour: "2-digit",
		minute: "2-digit",
	});
}

function CustomTooltip({ active, payload }: CustomTooltipProps) {
	if (!active || !payload || payload.length === 0) {
		return null;
	}

	const data = payload[0].payload;
	const formattedPrice = formatPrice(data.price, data.currency);

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
					{formatPrice(point.price, point.currency)}
				</p>
				<p className="text-muted-foreground text-sm">
					Recorded on {formatTooltipDate(point.date)}
				</p>
				<p className="text-muted-foreground text-xs">
					More data points needed to show price trend
				</p>
			</div>
		);
	}

	const currency = data[0].currency;
	const prices = data.map((d) => d.price);
	const minPrice = Math.min(...prices);
	const maxPrice = Math.max(...prices);
	const padding = Math.max((maxPrice - minPrice) * 0.1, 100);

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
					tickFormatter={formatDate}
					tick={{ fontSize: 10 }}
					tickLine={false}
					axisLine={false}
					className="fill-muted-foreground"
				/>
				<YAxis
					domain={[minPrice - padding, maxPrice + padding]}
					tickFormatter={(value) => formatPrice(value, currency)}
					tick={{ fontSize: 10 }}
					tickLine={false}
					axisLine={false}
					width={70}
					className="fill-muted-foreground"
				/>
				<Tooltip content={<CustomTooltip />} />
				<Line
					type="monotone"
					dataKey="price"
					stroke="oklch(0.546 0.245 262.881)"
					strokeWidth={2}
					dot={{
						r: 4,
						fill: "oklch(0.546 0.245 262.881)",
						stroke: "oklch(0.546 0.245 262.881)",
						strokeWidth: 0,
					}}
					activeDot={{
						r: 6,
						fill: "oklch(0.488 0.243 264.376)",
						stroke: "oklch(0.488 0.243 264.376)",
						strokeWidth: 0,
					}}
				/>
			</LineChart>
		</ResponsiveContainer>
	);
}
