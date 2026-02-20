import type {
	AvailabilityCheckResponse,
	AvailabilityStatus,
	MultiRetailerChartData,
	PriceDataPoint,
	ProductRetailerResponse,
	RetailerChartSeries,
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
 * @param todayComparisonMinorUnits - Today's comparison price in minor units
 * @param yesterdayComparisonMinorUnits - Yesterday's comparison price in minor units
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
	todayComparisonMinorUnits: number | null,
	yesterdayComparisonMinorUnits: number | null,
): boolean {
	if (
		todayComparisonMinorUnits === null ||
		yesterdayComparisonMinorUnits === null ||
		yesterdayComparisonMinorUnits === 0
	) {
		return false;
	}

	const change = todayComparisonMinorUnits - yesterdayComparisonMinorUnits;
	if (change === 0) {
		return false; // Exactly zero, not rounded
	}

	const percentChange = (change / yesterdayComparisonMinorUnits) * 100;
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

/**
 * Get the effective price from an availability check, preferring normalized over original.
 * Used for cross-currency comparisons where prices have been normalized to the preferred currency.
 */
export function getEffectivePrice(check: AvailabilityCheckResponse): {
	minorUnits: number | null;
	currency: string | null;
	exponent: number | null;
} {
	return {
		minorUnits: check.normalized_price_minor_units ?? check.price_minor_units,
		currency: check.normalized_currency ?? check.price_currency,
		exponent: check.normalized_currency_exponent ?? check.currency_exponent,
	};
}

/** Display price resolved from an availability check, preferring multi-retailer lowest price */
export interface DisplayPrice {
	price: number | null;
	currency: string | null;
	exponent: number;
}

/**
 * Resolve the best display price from an availability check.
 * Prefers `lowest_price_*` fields (multi-retailer cheapest) over the single-check `price_*` fields.
 */
export function getDisplayPrice(
	check: AvailabilityCheckResponse | null | undefined,
): DisplayPrice {
	return {
		price: check?.lowest_price_minor_units ?? check?.price_minor_units ?? null,
		currency: check?.lowest_price_currency ?? check?.price_currency ?? null,
		exponent: check?.lowest_currency_exponent ?? check?.currency_exponent ?? 2,
	};
}

/** Latest price info for a single retailer */
export interface RetailerPrice {
	priceMinorUnits: number;
	currency: string;
	currencyExponent: number;
	/** Original price (before normalization), only set when currency differs from normalized */
	originalPriceMinorUnits?: number;
	originalCurrency?: string;
	originalCurrencyExponent?: number;
}

/**
 * Get the latest valid price for each retailer from availability checks.
 * Groups checks by `product_retailer_id`, picks the most recent check
 * with a valid price for each retailer.
 * Uses normalized (effective) prices for cross-currency comparisons.
 *
 * @returns Map keyed by retailer ID → latest price info
 */
export function getLatestPriceByRetailer(
	checks: AvailabilityCheckResponse[],
	retailers: ProductRetailerResponse[],
): Map<string, RetailerPrice> {
	const retailerIds = new Set(retailers.map((r) => r.id));
	const result = new Map<string, RetailerPrice>();

	// Sort descending by checked_at so we encounter newest first
	const sorted = [...checks]
		.filter(hasValidPrice)
		.sort(
			(a, b) =>
				new Date(b.checked_at).getTime() - new Date(a.checked_at).getTime(),
		);

	for (const check of sorted) {
		const retailerId = check.product_retailer_id;
		if (!retailerId || !retailerIds.has(retailerId)) continue;
		if (result.has(retailerId)) continue;

		const effective = getEffectivePrice(check);
		const originalDiffers =
			check.normalized_currency !== null &&
			check.normalized_currency !== check.price_currency;

		// getEffectivePrice already falls back to check.price_*; the ?? here satisfies the non-null type
		const retailerPrice: RetailerPrice = {
			priceMinorUnits: effective.minorUnits ?? check.price_minor_units,
			currency: effective.currency ?? check.price_currency,
			currencyExponent: effective.exponent ?? 2,
		};

		if (originalDiffers) {
			retailerPrice.originalPriceMinorUnits = check.price_minor_units;
			retailerPrice.originalCurrency = check.price_currency;
			retailerPrice.originalCurrencyExponent = check.currency_exponent ?? 2;
		}

		result.set(retailerId, retailerPrice);
	}

	return result;
}

/** Detailed info for a single retailer including status and price comparison */
export interface RetailerDetails {
	priceMinorUnits: number | null;
	currency: string | null;
	currencyExponent: number;
	originalPriceMinorUnits?: number;
	originalCurrency?: string;
	originalCurrencyExponent?: number;
	status: AvailabilityStatus | null;
	checkedAt: string | null;
	todayAverageMinorUnits: number | null;
	yesterdayAverageMinorUnits: number | null;
}

/**
 * Get detailed info for each retailer from availability checks.
 * Single pass over sorted history, collecting per retailer:
 * - Latest status (from most recent check, regardless of price validity)
 * - Latest price (from most recent check with valid price)
 * - Today (0-24h) / yesterday (24-48h) price averages
 *
 * @returns Map keyed by retailer ID → details
 */
export function getRetailerDetails(
	checks: AvailabilityCheckResponse[],
	retailers: ProductRetailerResponse[],
): Map<string, RetailerDetails> {
	const retailerIds = new Set(retailers.map((r) => r.id));
	const result = new Map<string, RetailerDetails>();

	const now = Date.now();
	const todayCutoff = now - MILLISECONDS_PER_DAY;
	const yesterdayCutoff = now - 2 * MILLISECONDS_PER_DAY;

	const todayPrices = new Map<string, number[]>();
	const yesterdayPrices = new Map<string, number[]>();

	// --- Phase 1: Collect per-retailer status, latest price, and daily price buckets ---
	const sorted = [...checks].sort(
		(a, b) =>
			new Date(b.checked_at).getTime() - new Date(a.checked_at).getTime(),
	);

	for (const check of sorted) {
		const retailerId = check.product_retailer_id;
		if (!retailerId || !retailerIds.has(retailerId)) continue;

		if (!result.has(retailerId)) {
			result.set(retailerId, {
				priceMinorUnits: null,
				currency: null,
				currencyExponent: 2,
				status: check.status,
				checkedAt: check.checked_at,
				todayAverageMinorUnits: null,
				yesterdayAverageMinorUnits: null,
			});
		}

		const entry = result.get(retailerId);
		if (!entry) continue;

		if (entry.priceMinorUnits === null && hasValidPrice(check)) {
			const effective = getEffectivePrice(check);
			entry.priceMinorUnits = effective.minorUnits;
			entry.currency = effective.currency;
			entry.currencyExponent = effective.exponent ?? 2;

			const originalDiffers =
				check.normalized_currency !== null &&
				check.normalized_currency !== check.price_currency;

			if (originalDiffers) {
				entry.originalPriceMinorUnits = check.price_minor_units;
				entry.originalCurrency = check.price_currency;
				entry.originalCurrencyExponent = check.currency_exponent ?? 2;
			}
		}

		if (hasValidPrice(check)) {
			const checkTime = new Date(check.checked_at).getTime();
			// Use original price (not normalized) for daily comparison buckets.
			// Each retailer operates in a single currency, so using original prices
			// produces rate-stable percentage changes that reflect actual price
			// changes rather than exchange rate fluctuations.
			const price = check.price_minor_units;

			if (checkTime >= todayCutoff) {
				if (!todayPrices.has(retailerId)) todayPrices.set(retailerId, []);
				todayPrices.get(retailerId)?.push(price);
			} else if (checkTime >= yesterdayCutoff) {
				if (!yesterdayPrices.has(retailerId))
					yesterdayPrices.set(retailerId, []);
				yesterdayPrices.get(retailerId)?.push(price);
			}
		}
	}

	// --- Phase 2: Compute daily averages from collected price buckets ---
	computeDailyAverages(result, todayPrices, yesterdayPrices);

	return result;
}

/** Compute today/yesterday average prices from collected price buckets and write into entries. */
function computeDailyAverages(
	result: Map<string, RetailerDetails>,
	todayPrices: Map<string, number[]>,
	yesterdayPrices: Map<string, number[]>,
): void {
	for (const [retailerId, entry] of result) {
		const today = todayPrices.get(retailerId);
		if (today && today.length > 0) {
			entry.todayAverageMinorUnits = Math.round(
				today.reduce((sum, p) => sum + p, 0) / today.length,
			);
		}

		const yesterday = yesterdayPrices.get(retailerId);
		if (yesterday && yesterday.length > 0) {
			entry.yesterdayAverageMinorUnits = Math.round(
				yesterday.reduce((sum, p) => sum + p, 0) / yesterday.length,
			);
		}
	}
}

/** Lowest price comparison across all retailers for today vs yesterday */
export interface LowestPriceComparison {
	todayLowestMinorUnits: number | null;
	yesterdayLowestMinorUnits: number | null;
	currency: string | null;
	currencyExponent: number;
}

/**
 * Find the cheapest price across all retailers in today (0-24h) and
 * yesterday (24-48h) windows for an apples-to-apples comparison.
 */
export function getLowestPriceComparison(
	checks: AvailabilityCheckResponse[],
): LowestPriceComparison {
	const now = Date.now();
	const todayCutoff = now - MILLISECONDS_PER_DAY;
	const yesterdayCutoff = now - 2 * MILLISECONDS_PER_DAY;

	let todayLowest: number | null = null;
	let yesterdayLowest: number | null = null;
	let currency: string | null = null;
	let currencyExponent = 2;

	for (const check of checks) {
		if (!hasValidPrice(check)) continue;

		const effective = getEffectivePrice(check);
		const price = effective.minorUnits;
		if (price === null) continue;

		const checkTime = new Date(check.checked_at).getTime();

		if (currency === null) {
			currency = effective.currency;
			currencyExponent = effective.exponent ?? 2;
		}

		if (checkTime >= todayCutoff) {
			if (todayLowest === null || price < todayLowest) {
				todayLowest = price;
			}
		} else if (checkTime >= yesterdayCutoff) {
			if (yesterdayLowest === null || price < yesterdayLowest) {
				yesterdayLowest = price;
			}
		}
	}

	return {
		todayLowestMinorUnits: todayLowest,
		yesterdayLowestMinorUnits: yesterdayLowest,
		currency,
		currencyExponent,
	};
}

/**
 * Find the retailer ID with the lowest price from a retailer price/details map.
 * Returns null if fewer than 2 retailers have valid prices (no comparison possible).
 * Skips entries with null priceMinorUnits.
 */
export function findCheapestRetailerId(
	priceMap: Map<string, { priceMinorUnits: number | null }>,
): string | null {
	let pricedCount = 0;
	let cheapestId: string | null = null;
	let cheapestPrice = Number.POSITIVE_INFINITY;

	for (const [id, entry] of priceMap) {
		if (entry.priceMinorUnits === null) continue;
		pricedCount++;
		if (entry.priceMinorUnits < cheapestPrice) {
			cheapestPrice = entry.priceMinorUnits;
			cheapestId = id;
		}
	}

	if (pricedCount < 2) return null;
	return cheapestId;
}

const CHART_SERIES_COLORS = [
	"var(--chart-1)",
	"var(--chart-2)",
	"var(--chart-3)",
	"var(--chart-4)",
	"var(--chart-5)",
];

/** Extract hostname from a URL, falling back to the raw string on parse failure. */
export function extractDomain(url: string): string {
	try {
		return new URL(url).hostname;
	} catch {
		return url;
	}
}

/** Truncate an ISO timestamp to the nearest minute for bucketing. */
function bucketToMinute(isoDate: string): string {
	const d = new Date(isoDate);
	d.setSeconds(0, 0);
	return d.toISOString();
}

/**
 * Build a label for a retailer series from domain + optional user label.
 * e.g. "amazon.com" or "amazon.com (64GB)"
 */
function buildSeriesLabel(
	retailer: ProductRetailerResponse | undefined,
): string {
	if (!retailer) return "Price";
	const domain = extractDomain(retailer.url);
	return retailer.label ? `${domain} (${retailer.label})` : domain;
}

/**
 * Transform availability checks into pivoted multi-retailer chart data.
 *
 * Groups checks by `product_retailer_id`, assigns each retailer a color,
 * and produces rows keyed `{ date, [retailerId]: price }` for Recharts.
 *
 * Checks from the same check cycle (within the same minute) are bucketed together
 * so they share an x-axis position.
 */
export function transformToMultiRetailerChartData(
	checks: AvailabilityCheckResponse[],
	retailers: ProductRetailerResponse[],
): MultiRetailerChartData {
	const validChecks = checks.filter(hasValidPrice);

	if (validChecks.length === 0) {
		return { data: [], series: [], currency: "", currencyExponent: 2 };
	}

	const firstEffective = getEffectivePrice(validChecks[0]);
	const currency = firstEffective.currency ?? "";
	const currencyExponent = firstEffective.exponent ?? 2;

	// Group by product_retailer_id (null → "legacy" fallback)
	const grouped = new Map<string, typeof validChecks>();
	for (const check of validChecks) {
		const key = check.product_retailer_id ?? "legacy";
		const list = grouped.get(key);
		if (list) {
			list.push(check);
		} else {
			grouped.set(key, [check]);
		}
	}

	const retailerMap = new Map(retailers.map((r) => [r.id, r]));

	// Build series metadata
	const series: RetailerChartSeries[] = [];
	let colorIndex = 0;
	for (const retailerId of grouped.keys()) {
		const retailer = retailerMap.get(retailerId);
		const label =
			retailerId === "legacy" ? "Price" : buildSeriesLabel(retailer);
		series.push({
			id: retailerId,
			label,
			color: CHART_SERIES_COLORS[colorIndex % CHART_SERIES_COLORS.length],
		});
		colorIndex++;
	}

	// Build pivoted data rows bucketed by minute
	const rowMap = new Map<string, Record<string, string | number>>();
	for (const check of validChecks) {
		const bucketDate = bucketToMinute(check.checked_at);
		const retailerId = check.product_retailer_id ?? "legacy";

		let row = rowMap.get(bucketDate);
		if (!row) {
			row = { date: bucketDate };
			rowMap.set(bucketDate, row);
		}
		row[retailerId] = getEffectivePrice(check).minorUnits ?? 0;
	}

	// Sort rows by date ascending
	const data = Array.from(rowMap.values()).sort(
		(a, b) =>
			new Date(a.date as string).getTime() -
			new Date(b.date as string).getTime(),
	);

	return { data, series, currency, currencyExponent };
}
