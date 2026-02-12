import type {
	AvailabilityCheckResponse,
	PriceDataPoint,
	TimeRange,
} from "@/modules/products/types";

const MILLISECONDS_PER_DAY = 24 * 60 * 60 * 1000;

/** Type guard to check if an availability check has valid price data */
function hasValidPrice(
	check: AvailabilityCheckResponse,
): check is AvailabilityCheckResponse & {
	price_minor_units: number;
	price_currency: string;
	currency_exponent: number;
} {
	return check.price_minor_units !== null && check.price_currency !== null;
}

/**
 * Filter availability checks by time range.
 * @param checks - Array of availability check responses
 * @param range - Time range to filter by
 * @returns Filtered array of availability checks
 */
export function filterByTimeRange(
	checks: AvailabilityCheckResponse[],
	range: TimeRange,
): AvailabilityCheckResponse[] {
	if (range === "all") {
		return checks;
	}

	const now = new Date();
	const daysToSubtract = range === "7d" ? 7 : 30;
	const cutoff = new Date(
		now.getTime() - daysToSubtract * MILLISECONDS_PER_DAY,
	);

	return checks.filter((check) => new Date(check.checked_at) >= cutoff);
}

/**
 * Transform availability checks into price data points for charting.
 * Filters out checks with null prices.
 * @param checks - Array of availability check responses
 * @returns Array of price data points suitable for charting
 */
export function transformToPriceDataPoints(
	checks: AvailabilityCheckResponse[],
): PriceDataPoint[] {
	return checks
		.filter(hasValidPrice)
		.map((check) => ({
			date: check.checked_at,
			price: check.price_minor_units,
			currency: check.price_currency,
			currencyExponent: check.currency_exponent ?? 2,
		}))
		.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());
}

/**
 * Get the date range string for display.
 * @param dataPoints - Array of price data points
 * @param formatDate - Function to format dates
 * @returns Formatted date range string or empty string if no data
 */
export function getDateRangeLabel(
	dataPoints: PriceDataPoint[],
	formatDate: (dateString: string) => string,
): string {
	if (dataPoints.length === 0) {
		return "";
	}

	if (dataPoints.length === 1) {
		return formatDate(dataPoints[0].date);
	}

	const first = formatDate(dataPoints[0].date);
	const last = formatDate(dataPoints[dataPoints.length - 1].date);

	return `${first} - ${last}`;
}

/** Price change direction */
export type PriceChangeDirection = "up" | "down" | "unchanged" | "unknown";

/**
 * Determine the direction of price change.
 *
 * @param currentMinorUnits - Current price in minor units
 * @param previousMinorUnits - Previous price in minor units
 * @returns Direction of price change
 *
 * @example
 * ```ts
 * getPriceChangeDirection(79900, 89900) // "down" (price decreased)
 * getPriceChangeDirection(89900, 79900) // "up" (price increased)
 * getPriceChangeDirection(79900, 79900) // "unchanged"
 * getPriceChangeDirection(null, 79900) // "unknown"
 * ```
 */
export function getPriceChangeDirection(
	currentMinorUnits: number | null,
	previousMinorUnits: number | null,
): PriceChangeDirection {
	if (currentMinorUnits === null || previousMinorUnits === null) {
		return "unknown";
	}

	if (currentMinorUnits < previousMinorUnits) {
		return "down";
	}

	if (currentMinorUnits > previousMinorUnits) {
		return "up";
	}

	return "unchanged";
}

/**
 * Calculate the percentage change between two prices.
 *
 * @param currentMinorUnits - Current price in minor units
 * @param previousMinorUnits - Previous price in minor units
 * @returns Percentage change (positive for increase, negative for decrease), or null if cannot be calculated
 *
 * @example
 * ```ts
 * calculatePriceChangePercent(89900, 79900) // 13 (13% increase)
 * calculatePriceChangePercent(79900, 89900) // -11 (11% decrease)
 * calculatePriceChangePercent(79900, 79900) // 0 (no change)
 * calculatePriceChangePercent(null, 79900) // null (cannot calculate)
 * calculatePriceChangePercent(79900, 0) // null (avoid division by zero)
 * ```
 */
export function calculatePriceChangePercent(
	currentMinorUnits: number | null,
	previousMinorUnits: number | null,
): number | null {
	if (
		currentMinorUnits === null ||
		previousMinorUnits === null ||
		previousMinorUnits === 0
	) {
		return null;
	}

	const change = currentMinorUnits - previousMinorUnits;
	const percentChange = (change / previousMinorUnits) * 100;

	return Math.round(percentChange);
}

/**
 * Format a price change percentage for display.
 * @param percent - The percentage change (can be positive or negative)
 * @returns Formatted string like "+15%" or "-12%"
 */
export function formatPriceChangePercent(percent: number | null): string {
	if (percent === null) {
		return "";
	}

	const sign = percent >= 0 ? "+" : "";
	return `${sign}${percent}%`;
}

/**
 * Checks if a percentage change rounds to 0 but is not actually zero.
 * Used to determine whether to show just the icon without percentage text.
 *
 * @param todayAverageMinorUnits - Today's average price in minor units
 * @param yesterdayAverageMinorUnits - Yesterday's average price in minor units
 * @returns True if the change rounds to 0% but prices are different, false otherwise
 *
 * @example
 * ```ts
 * isRoundedZero(80000, 79960) // true (0.05% increase, rounds to 0%)
 * isRoundedZero(79960, 80000) // true (-0.05% decrease, rounds to 0%)
 * isRoundedZero(80000, 80000) // false (exactly 0%, no change)
 * isRoundedZero(80800, 80000) // false (1% increase)
 * isRoundedZero(null, 80000) // false (cannot calculate)
 * ```
 */
export function isRoundedZero(
	todayAverageMinorUnits: number | null,
	yesterdayAverageMinorUnits: number | null,
): boolean {
	if (
		todayAverageMinorUnits === null ||
		yesterdayAverageMinorUnits === null ||
		yesterdayAverageMinorUnits === 0
	) {
		return false;
	}

	const change = todayAverageMinorUnits - yesterdayAverageMinorUnits;
	if (change === 0) {
		return false; // Exactly zero, not rounded
	}

	const percentChange = (change / yesterdayAverageMinorUnits) * 100;
	return Math.round(percentChange) === 0; // Rounds to zero but isn't exactly zero
}

/**
 * Format a price in minor units to a localized currency string.
 *
 * @param minorUnits - Price in smallest currency unit (e.g., cents for USD, yen for JPY)
 * @param currency - ISO 4217 currency code (e.g., "USD", "EUR", "JPY")
 * @param exponent - Number of decimal places for the currency (0 for JPY, 2 for USD, 3 for KWD). Defaults to 2.
 * @returns Formatted price string or "-" if price is not available
 *
 * @example
 * ```ts
 * formatPrice(79900, "USD", 2) // "$799.00"
 * formatPrice(1500, "JPY", 0) // "¥1,500"
 * formatPrice(12345, "KWD", 3) // "KD 12.345"
 * formatPrice(null, "USD", 2) // "-"
 * ```
 */
export function formatPrice(
	minorUnits: number | null,
	currency: string | null,
	exponent = 2,
): string {
	if (minorUnits === null || currency === null) return "-";
	return new Intl.NumberFormat("en-US", {
		style: "currency",
		currency,
	}).format(minorUnits / 10 ** exponent);
}
