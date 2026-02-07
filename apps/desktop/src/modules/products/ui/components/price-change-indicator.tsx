import { TrendingDown, TrendingUp } from "lucide-react";
import {
	calculatePriceChangePercent,
	formatPriceChangePercent,
	getPriceChangeDirection,
} from "@/lib/price-utils";
import { cn, formatPrice } from "@/lib/utils";

interface PriceChangeIndicatorProps {
	currentPriceCents: number | null;
	/** Today's average price in cents for comparison */
	todayAverageCents: number | null;
	/** Yesterday's average price in cents for comparison */
	yesterdayAverageCents: number | null;
	currency: string | null;
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
	currentPriceCents,
	todayAverageCents,
	yesterdayAverageCents,
	currency,
	variant,
}: PriceChangeIndicatorProps) {
	const currentPrice = formatPrice(currentPriceCents, currency);
	const direction = getPriceChangeDirection(
		todayAverageCents,
		yesterdayAverageCents,
	);
	const percent = calculatePriceChangePercent(
		todayAverageCents,
		yesterdayAverageCents,
	);

	if (currentPriceCents === null) {
		return <span className="text-muted-foreground">-</span>;
	}

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
			yesterdayAverageCents={yesterdayAverageCents}
			currency={currency}
			direction={direction}
			percent={percent}
		/>
	);
}

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
					isDown
						? "text-green-600 dark:text-green-400"
						: "text-red-600 dark:text-red-400",
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
	yesterdayAverageCents: number | null;
	currency: string | null;
	direction: "up" | "down";
	percent: number | null;
}

function DetailedIndicator({
	currentPrice,
	yesterdayAverageCents,
	currency,
	direction,
	percent,
}: DetailedIndicatorProps) {
	const isDown = direction === "down";
	const Icon = isDown ? TrendingDown : TrendingUp;
	const yesterdayPrice = formatPrice(yesterdayAverageCents, currency);
	const percentValue = percent !== null ? Math.abs(percent) : 0;
	const directionLabel = isDown ? "Down" : "Up";

	return (
		<div>
			<p className="font-semibold text-2xl">{currentPrice}</p>
			<p
				className={cn(
					"mt-1 inline-flex items-center gap-1 text-xs",
					isDown
						? "text-green-600 dark:text-green-400"
						: "text-red-600 dark:text-red-400",
				)}
			>
				<Icon className="size-3" />
				{directionLabel} {percentValue}% from {yesterdayPrice}
			</p>
		</div>
	);
}
