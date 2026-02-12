import { useMemo } from "react";
import {
	calculatePriceChangePercent,
	formatPrice,
	formatPriceChangePercent,
	getPriceChangeDirection,
	type PriceChangeDirection,
} from "@/modules/products/price-utils";

/**
 * Props for usePriceFormatting hook
 */
export interface UsePriceFormattingProps {
	/** Current price in minor units */
	currentPriceMinorUnits: number | null;
	/** Today's average price in minor units (for comparison) */
	todayAverageMinorUnits?: number | null;
	/** Yesterday's average price in minor units (for comparison) */
	yesterdayAverageMinorUnits?: number | null;
	/** ISO 4217 currency code (e.g., "USD", "EUR", "JPY") */
	currency: string | null;
	/** Number of decimal places for the currency (0 for JPY, 2 for USD, 3 for KWD). Defaults to 2. */
	currencyExponent?: number;
}

/**
 * Return type for usePriceFormatting hook
 */
export interface PriceFormattingResult {
	/** Formatted current price string (e.g., "$799.00") */
	formattedCurrentPrice: string;
	/** Formatted previous price string (e.g., "$899.00") */
	formattedPreviousPrice: string;
	/** Direction of price change */
	direction: PriceChangeDirection;
	/** Percentage change (positive for increase, negative for decrease) */
	percentChange: number | null;
	/** Formatted percentage change (e.g., "+15%", "-12%") */
	formattedPercentChange: string;
	/** Whether price comparison data is available */
	hasComparison: boolean;
}

/**
 * Hook for memoized price formatting and comparison calculations.
 *
 * Provides formatted price strings and comparison data (direction, percentage change)
 * with automatic memoization to avoid recalculations on every render.
 *
 * @param props - Price data and currency information
 * @returns Memoized formatted prices and comparison data
 *
 * @example
 * ```tsx
 * function PriceDisplay({ product, latestCheck }) {
 *   const {
 *     formattedCurrentPrice,
 *     direction,
 *     formattedPercentChange,
 *     hasComparison
 *   } = usePriceFormatting({
 *     currentPriceMinorUnits: latestCheck.price_minor_units,
 *     todayAverageMinorUnits: latestCheck.today_average_price_minor_units,
 *     yesterdayAverageMinorUnits: latestCheck.yesterday_average_price_minor_units,
 *     currency: latestCheck.price_currency,
 *     currencyExponent: latestCheck.currency_exponent ?? 2,
 *   });
 *
 *   return (
 *     <div>
 *       <p>{formattedCurrentPrice}</p>
 *       {hasComparison && direction !== "unchanged" && (
 *         <p>{direction} {formattedPercentChange}</p>
 *       )}
 *     </div>
 *   );
 * }
 * ```
 */
export function usePriceFormatting({
	currentPriceMinorUnits,
	todayAverageMinorUnits,
	yesterdayAverageMinorUnits,
	currency,
	currencyExponent = 2,
}: UsePriceFormattingProps): PriceFormattingResult {
	const formattedCurrentPrice = useMemo(
		() => formatPrice(currentPriceMinorUnits, currency, currencyExponent),
		[currentPriceMinorUnits, currency, currencyExponent],
	);

	const formattedPreviousPrice = useMemo(
		() =>
			formatPrice(
				yesterdayAverageMinorUnits ?? null,
				currency,
				currencyExponent,
			),
		[yesterdayAverageMinorUnits, currency, currencyExponent],
	);

	const direction = useMemo(
		() =>
			getPriceChangeDirection(
				todayAverageMinorUnits ?? null,
				yesterdayAverageMinorUnits ?? null,
			),
		[todayAverageMinorUnits, yesterdayAverageMinorUnits],
	);

	const percentChange = useMemo(
		() =>
			calculatePriceChangePercent(
				todayAverageMinorUnits ?? null,
				yesterdayAverageMinorUnits ?? null,
			),
		[todayAverageMinorUnits, yesterdayAverageMinorUnits],
	);

	const formattedPercentChange = useMemo(
		() => formatPriceChangePercent(percentChange),
		[percentChange],
	);

	const hasComparison = useMemo(
		() => direction !== "unknown" && direction !== "unchanged",
		[direction],
	);

	return {
		formattedCurrentPrice,
		formattedPreviousPrice,
		direction,
		percentChange,
		formattedPercentChange,
		hasComparison,
	};
}
