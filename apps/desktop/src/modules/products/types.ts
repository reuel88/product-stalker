/**
 * Product entity returned from the backend.
 */
export interface ProductResponse {
	/** Unique identifier (UUID) */
	id: string;
	/** Display name of the product */
	name: string;
	/** Optional description for user notes */
	description: string | null;
	/** Optional private notes */
	notes: string | null;
	/** ISO 4217 currency code auto-set from first successful price scrape */
	currency: string | null;
	/** ISO 8601 timestamp when the product was added */
	created_at: string;
	/** ISO 8601 timestamp of the last update */
	updated_at: string;
}

/**
 * Product-retailer link returned from the backend.
 */
export interface ProductRetailerResponse {
	/** Unique identifier (UUID) */
	id: string;
	/** Product ID this link belongs to */
	product_id: string;
	/** Retailer ID (domain-based) */
	retailer_id: string;
	/** URL to the product page at this retailer */
	url: string;
	/** Optional user-provided label (e.g., "64GB version") */
	label: string | null;
	/** ISO 8601 timestamp when the link was created */
	created_at: string;
}

/**
 * Availability status parsed from Schema.org data.
 * - `in_stock`: Product is available for purchase
 * - `out_of_stock`: Product is not available
 * - `back_order`: Product can be ordered but ships later
 * - `unknown`: Status could not be determined
 */
export type AvailabilityStatus =
	| "in_stock"
	| "out_of_stock"
	| "back_order"
	| "unknown";

/**
 * Result of a single availability check for a product.
 */
export interface AvailabilityCheckResponse {
	/** Unique identifier of this check record */
	id: string;
	/** Product ID this check belongs to */
	product_id: string;
	/** Product-retailer link ID this check was performed against */
	product_retailer_id: string | null;
	/** Parsed availability status */
	status: AvailabilityStatus;
	/** Raw availability string from Schema.org (e.g., "http://schema.org/InStock") */
	raw_availability: string | null;
	/** Error message if the check failed */
	error_message: string | null;
	/** ISO 8601 timestamp when the check was performed */
	checked_at: string;
	/** Price in minor units (smallest currency unit) */
	price_minor_units: number | null;
	/** ISO 4217 currency code (e.g., "USD", "EUR") */
	price_currency: string | null;
	/** Raw price string from the page (e.g., "789.00") */
	raw_price: string | null;
	/** Currency exponent (number of decimal places: 0 for JPY, 2 for USD, 3 for KWD) */
	currency_exponent: number | null;
	/** Today's average price in minor units for daily comparison */
	today_average_price_minor_units: number | null;
	/** Yesterday's average price in minor units for daily comparison */
	yesterday_average_price_minor_units: number | null;
	/** True if today's average price is lower than yesterday's average */
	is_price_drop: boolean;
}

/**
 * Result of checking a single product during bulk check operation.
 */
export interface BulkCheckResult {
	/** Product ID that was checked */
	product_id: string;
	/** Product name for display */
	product_name: string;
	/** Product-retailer link ID that was checked */
	product_retailer_id: string | null;
	/** URL that was checked */
	url: string | null;
	/** Current availability status */
	status: AvailabilityStatus;
	/** Previous status before this check (null if first check) */
	previous_status: AvailabilityStatus | null;
	/** True if product changed from out_of_stock/back_order to in_stock */
	is_back_in_stock: boolean;
	/** Current price in minor units */
	price_minor_units: number | null;
	/** Currency code for current price */
	price_currency: string | null;
	/** Currency exponent (number of decimal places: 0 for JPY, 2 for USD, 3 for KWD) */
	currency_exponent: number | null;
	/** Today's average price in minor units for daily comparison */
	today_average_price_minor_units: number | null;
	/** Yesterday's average price in minor units for daily comparison */
	yesterday_average_price_minor_units: number | null;
	/** True if today's average price is lower than yesterday's average */
	is_price_drop: boolean;
	/** Error message if this product's check failed */
	error: string | null;
}

/**
 * Summary of a bulk availability check operation.
 */
export interface BulkCheckSummary {
	/** Total number of products checked */
	total: number;
	/** Number of products checked successfully */
	successful: number;
	/** Number of products that failed to check */
	failed: number;
	/** Count of products that came back in stock */
	back_in_stock_count: number;
	/** Count of products with price drops */
	price_drop_count: number;
	/** Individual results for each product */
	results: BulkCheckResult[];
}

/**
 * Event emitted during bulk check for progress updates.
 * Field names match the Rust `BulkCheckProgressEvent` struct.
 */
export interface CheckProgressEvent {
	/** Product ID that was just checked */
	product_id: string;
	/** Availability status result */
	status: AvailabilityStatus;
	/** 1-based index of the current product being checked */
	current: number;
	/** Total number of products to check */
	total: number;
}

/**
 * Event emitted when bulk check completes.
 */
export interface CheckCompleteEvent {
	/** Final summary of the bulk check operation */
	summary: BulkCheckSummary;
}

/**
 * Time range options for filtering price history charts.
 */
export type TimeRange = "7d" | "30d" | "all";

/**
 * Data point for price history charts.
 */
export interface PriceDataPoint {
	/** ISO 8601 timestamp of the price check */
	date: string;
	/** Price in smallest currency unit (minor units) */
	price: number;
	/** ISO 4217 currency code */
	currency: string;
	/** Currency exponent for formatting (0 for JPY, 2 for USD, 3 for KWD) */
	currencyExponent: number;
}

/**
 * Metadata for a single retailer line in the price history chart.
 */
export interface RetailerChartSeries {
	/** product_retailer_id used as the data key */
	id: string;
	/** Human-readable label (e.g., "amazon.com" or "amazon.com (64GB)") */
	label: string;
	/** CSS color for the line stroke */
	color: string;
}

/**
 * Pivoted chart data for multi-retailer price history.
 * Each row has `{ date, [seriesId]: price, ... }` for Recharts.
 */
export interface MultiRetailerChartData {
	/** Pivoted data rows keyed by date + retailer ID */
	data: Array<Record<string, string | number>>;
	/** One entry per retailer line */
	series: RetailerChartSeries[];
	/** ISO 4217 currency code for formatting */
	currency: string;
	/** Currency exponent for formatting */
	currencyExponent: number;
}
