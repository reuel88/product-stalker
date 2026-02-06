export interface ProductResponse {
	id: string;
	name: string;
	url: string;
	description: string | null;
	notes: string | null;
	created_at: string;
	updated_at: string;
}

export type AvailabilityStatus =
	| "in_stock"
	| "out_of_stock"
	| "back_order"
	| "unknown";

export interface AvailabilityCheckResponse {
	id: string;
	product_id: string;
	status: AvailabilityStatus;
	raw_availability: string | null;
	error_message: string | null;
	checked_at: string;
	price_cents: number | null;
	price_currency: string | null;
	raw_price: string | null;
}

export interface BulkCheckResult {
	product_id: string;
	product_name: string;
	status: AvailabilityStatus;
	previous_status: AvailabilityStatus | null;
	is_back_in_stock: boolean;
	price_cents: number | null;
	price_currency: string | null;
	previous_price_cents: number | null;
	is_price_drop: boolean;
	error: string | null;
}

export interface BulkCheckSummary {
	total: number;
	successful: number;
	failed: number;
	back_in_stock_count: number;
	price_drop_count: number;
	results: BulkCheckResult[];
}

export interface CheckProgressEvent {
	current_index: number;
	total_count: number;
	result: BulkCheckResult;
}

export interface CheckCompleteEvent {
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
