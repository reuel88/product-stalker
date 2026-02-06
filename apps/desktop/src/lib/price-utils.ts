import type {
	AvailabilityCheckResponse,
	PriceDataPoint,
	TimeRange,
} from "@/modules/products/types";

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
	const cutoff = new Date(now.getTime() - daysToSubtract * 24 * 60 * 60 * 1000);

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
		.filter(
			(check) => check.price_cents !== null && check.price_currency !== null,
		)
		.map((check) => ({
			date: check.checked_at,
			price: check.price_cents as number,
			currency: check.price_currency as string,
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
