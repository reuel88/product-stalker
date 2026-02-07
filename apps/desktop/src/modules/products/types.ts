/**
 * Product entity returned from the backend.
 */
export interface ProductResponse {
	/** Unique identifier (UUID) */
	id: string;
	/** Display name of the product */
	name: string;
	/** URL to the product page for availability checking */
	url: string;
	/** Optional description for user notes */
	description: string | null;
	/** Optional private notes */
	notes: string | null;
	/** ISO 8601 timestamp when the product was added */
	created_at: string;
	/** ISO 8601 timestamp of the last update */
	updated_at: string;
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
	/** Parsed availability status */
	status: AvailabilityStatus;
	/** Raw availability string from Schema.org (e.g., "http://schema.org/InStock") */
	raw_availability: string | null;
	/** Error message if the check failed */
	error_message: string | null;
	/** ISO 8601 timestamp when the check was performed */
	checked_at: string;
	/** Price in cents (smallest currency unit) */
	price_cents: number | null;
	/** ISO 4217 currency code (e.g., "USD", "EUR") */
	price_currency: string | null;
	/** Raw price string from the page (e.g., "789.00") */
	raw_price: string | null;
	/** Previous price in cents for comparison (null if first check) */
	previous_price_cents: number | null;
	/** True if current price is lower than previous price */
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
	/** Current availability status */
	status: AvailabilityStatus;
	/** Previous status before this check (null if first check) */
	previous_status: AvailabilityStatus | null;
	/** True if product changed from out_of_stock/back_order to in_stock */
	is_back_in_stock: boolean;
	/** Current price in cents */
	price_cents: number | null;
	/** Currency code for current price */
	price_currency: string | null;
	/** Previous price in cents for comparison */
	previous_price_cents: number | null;
	/** True if current price is lower than previous price */
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
 */
export interface CheckProgressEvent {
	/** Zero-based index of the current product being checked */
	current_index: number;
	/** Total number of products to check */
	total_count: number;
	/** Result of the current product check */
	result: BulkCheckResult;
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
	/** Price in smallest currency unit (cents) */
	price: number;
	/** ISO 4217 currency code */
	currency: string;
}
