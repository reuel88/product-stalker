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
	price_cents: number;
	price_currency: string;
} {
	return check.price_cents !== null && check.price_currency !== null;
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
			price: check.price_cents,
			currency: check.price_currency,
		}))
		.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());
}

/**
 * Get the date range string for display.
 * @param dataPoints - Array of price data points
 * @returns Formatted date range string or empty string if no data
 */
export function getDateRangeLabel(dataPoints: PriceDataPoint[]): string {
	if (dataPoints.length === 0) {
		return "";
	}

	if (dataPoints.length === 1) {
		return new Date(dataPoints[0].date).toLocaleDateString();
	}

	const first = new Date(dataPoints[0].date).toLocaleDateString();
	const last = new Date(
		dataPoints[dataPoints.length - 1].date,
	).toLocaleDateString();

	return `${first} - ${last}`;
}

/** Price change direction */
export type PriceChangeDirection = "up" | "down" | "unchanged" | "unknown";

/**
 * Determine the direction of price change.
 * @param currentPriceCents - Current price in cents
 * @param previousPriceCents - Previous price in cents
 * @returns Direction of price change
 */
export function getPriceChangeDirection(
	currentPriceCents: number | null,
	previousPriceCents: number | null,
): PriceChangeDirection {
	if (currentPriceCents === null || previousPriceCents === null) {
		return "unknown";
	}

	if (currentPriceCents < previousPriceCents) {
		return "down";
	}

	if (currentPriceCents > previousPriceCents) {
		return "up";
	}

	return "unchanged";
}

/**
 * Calculate the percentage change between two prices.
 * @param currentPriceCents - Current price in cents
 * @param previousPriceCents - Previous price in cents
 * @returns Percentage change (positive for increase, negative for decrease), or null if cannot be calculated
 */
export function calculatePriceChangePercent(
	currentPriceCents: number | null,
	previousPriceCents: number | null,
): number | null {
	if (
		currentPriceCents === null ||
		previousPriceCents === null ||
		previousPriceCents === 0
	) {
		return null;
	}

	const change = currentPriceCents - previousPriceCents;
	const percentChange = (change / previousPriceCents) * 100;

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
