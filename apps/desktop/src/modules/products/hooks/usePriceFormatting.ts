import { useMemo } from "react";
import {
	calculatePriceChangePercent,
	formatPrice,
	formatPriceChangePercent,
	getPriceChangeDirection,
	isRoundedZero,
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
	/** Whether the percentage change rounds to 0% but is not exactly zero */
	isRoundedZero: boolean;
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
	// Price formatting (current + previous)
	const { formattedCurrentPrice, formattedPreviousPrice } = useMemo(
		() => ({
			formattedCurrentPrice: formatPrice(
				currentPriceMinorUnits,
				currency,
				currencyExponent,
			),
			formattedPreviousPrice: formatPrice(
				yesterdayAverageMinorUnits ?? null,
				currency,
				currencyExponent,
			),
		}),
		[
			currentPriceMinorUnits,
			yesterdayAverageMinorUnits,
			currency,
			currencyExponent,
		],
	);

	// Price comparison calculations (direction + percent + formatted)
	const { direction, percentChange, formattedPercentChange } = useMemo(() => {
		const today = todayAverageMinorUnits ?? null;
		const yesterday = yesterdayAverageMinorUnits ?? null;
		const dir = getPriceChangeDirection(today, yesterday);
		const pct = calculatePriceChangePercent(today, yesterday);
		return {
			direction: dir,
			percentChange: pct,
			formattedPercentChange: formatPriceChangePercent(pct),
		};
	}, [todayAverageMinorUnits, yesterdayAverageMinorUnits]);

	// Comparison flags (hasComparison + isRoundedZero)
	const { hasComparison, isRoundedZero: isRoundedZeroValue } = useMemo(
		() => ({
			hasComparison: direction !== "unknown" && direction !== "unchanged",
			isRoundedZero: isRoundedZero(
				todayAverageMinorUnits ?? null,
				yesterdayAverageMinorUnits ?? null,
			),
		}),
		[direction, todayAverageMinorUnits, yesterdayAverageMinorUnits],
	);

	return {
		formattedCurrentPrice,
		formattedPreviousPrice,
		direction,
		percentChange,
		formattedPercentChange,
		hasComparison,
		isRoundedZero: isRoundedZeroValue,
	};
}
