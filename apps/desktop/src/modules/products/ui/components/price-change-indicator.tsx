import { TrendingDown, TrendingUp } from "lucide-react";
import { cn } from "@/lib/utils";
import {
	calculatePriceChangePercent,
	formatPrice,
	formatPriceChangePercent,
	getPriceChangeDirection,
} from "@/modules/products/price-utils";

interface PriceChangeIndicatorProps {
	currentPriceMinorUnits: number | null;
	/** Today's average price in minor units for comparison */
	todayAverageMinorUnits: number | null;
	/** Yesterday's average price in minor units for comparison */
	yesterdayAverageMinorUnits: number | null;
	currency: string | null;
	/** Currency exponent for formatting (0 for JPY, 2 for USD, 3 for KWD) */
	currencyExponent: number;
	variant: "compact" | "detailed";
}

/**
 * Displays price with optional change indicator based on daily averages.
 *
 * Compares today's average price vs yesterday's average price.
 *
 * Compact variant (for table):
 * - `$799 ↓-12%` (green for drops)
 * - `$899 ↑+15%` (red for increases)
 * - `$799` (no indicator for unchanged/first check)
 *
 * Detailed variant (for product card):
 * - `$799.00`
 * - `↓ Down 12% from $899.00`
 */
export function PriceChangeIndicator({
	currentPriceMinorUnits,
	todayAverageMinorUnits,
	yesterdayAverageMinorUnits,
	currency,
	currencyExponent,
	variant,
}: PriceChangeIndicatorProps) {
	if (currentPriceMinorUnits === null) {
		return <span className="text-muted-foreground">-</span>;
	}

	const currentPrice = formatPrice(
		currentPriceMinorUnits,
		currency,
		currencyExponent,
	);
	const direction = getPriceChangeDirection(
		todayAverageMinorUnits,
		yesterdayAverageMinorUnits,
	);
	const percent = calculatePriceChangePercent(
		todayAverageMinorUnits,
		yesterdayAverageMinorUnits,
	);

	if (direction === "unknown" || direction === "unchanged") {
		if (variant === "compact") {
			return <span className="text-muted-foreground">{currentPrice}</span>;
		}
		return <p className="font-semibold text-2xl">{currentPrice}</p>;
	}

	if (variant === "compact") {
		return (
			<CompactIndicator
				currentPrice={currentPrice}
				direction={direction}
				percent={percent}
			/>
		);
	}

	return (
		<DetailedIndicator
			currentPrice={currentPrice}
			yesterdayAverageMinorUnits={yesterdayAverageMinorUnits}
			currency={currency}
			currencyExponent={currencyExponent}
			direction={direction}
			percent={percent}
		/>
	);
}

const PRICE_DIRECTION_COLORS = {
	down: "text-green-600 dark:text-green-400",
	up: "text-red-600 dark:text-red-400",
} as const;

interface CompactIndicatorProps {
	currentPrice: string;
	direction: "up" | "down";
	percent: number | null;
}

function CompactIndicator({
	currentPrice,
	direction,
	percent,
}: CompactIndicatorProps) {
	const isDown = direction === "down";
	const Icon = isDown ? TrendingDown : TrendingUp;
	const formattedPercent = formatPriceChangePercent(percent);

	return (
		<span className="inline-flex items-center gap-1">
			<span className="text-muted-foreground">{currentPrice}</span>
			<span
				className={cn(
					"inline-flex items-center gap-0.5 font-medium text-xs",
					PRICE_DIRECTION_COLORS[direction],
				)}
			>
				<Icon className="size-3" />
				{formattedPercent}
			</span>
		</span>
	);
}

interface DetailedIndicatorProps {
	currentPrice: string;
	yesterdayAverageMinorUnits: number | null;
	currency: string | null;
	currencyExponent: number;
	direction: "up" | "down";
	percent: number | null;
}

function DetailedIndicator({
	currentPrice,
	yesterdayAverageMinorUnits,
	currency,
	currencyExponent,
	direction,
	percent,
}: DetailedIndicatorProps) {
	const isDown = direction === "down";
	const Icon = isDown ? TrendingDown : TrendingUp;
	const yesterdayPrice = formatPrice(
		yesterdayAverageMinorUnits,
		currency,
		currencyExponent,
	);
	const percentValue = percent !== null ? Math.abs(percent) : 0;
	const directionLabel = isDown ? "Down" : "Up";

	return (
		<div>
			<p className="font-semibold text-2xl">{currentPrice}</p>
			<p
				className={cn(
					"mt-1 inline-flex items-center gap-1 text-xs",
					PRICE_DIRECTION_COLORS[direction],
				)}
			>
				<Icon className="size-3" />
				{directionLabel} {percentValue}% from {yesterdayPrice}
			</p>
		</div>
	);
}
