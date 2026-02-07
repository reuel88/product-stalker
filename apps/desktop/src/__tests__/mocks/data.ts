import type {
	AvailabilityCheckResponse,
	BulkCheckSummary,
	ProductResponse,
} from "@/modules/products/types";
import type { Settings } from "@/modules/settings/hooks/useSettings";
import type { UpdateInfo } from "@/modules/settings/hooks/useUpdater";

let productIdCounter = 1;

/**
 * Create a mock product with default values
 */
export function createMockProduct(
	overrides: Partial<ProductResponse> = {},
): ProductResponse {
	const id = overrides.id ?? `product-${productIdCounter++}`;
	const now = new Date().toISOString();
	return {
		id,
		name: `Test Product ${id}`,
		url: `https://example.com/product/${id}`,
		description: null,
		notes: null,
		created_at: now,
		updated_at: now,
		...overrides,
	};
}

/**
 * Create multiple mock products
 */
export function createMockProducts(
	count: number,
	overrides: Partial<ProductResponse> = {},
): ProductResponse[] {
	return Array.from({ length: count }, () => createMockProduct(overrides));
}

/**
 * Create mock settings with default values
 */
export function createMockSettings(
	overrides: Partial<Settings> = {},
): Settings {
	return {
		theme: "system",
		show_in_tray: true,
		launch_at_login: false,
		enable_logging: true,
		log_level: "info",
		enable_notifications: true,
		sidebar_expanded: true,
		background_check_enabled: false,
		background_check_interval_minutes: 60,
		enable_headless_browser: true,
		updated_at: new Date().toISOString(),
		...overrides,
	};
}

/**
 * Create mock update info for no update available
 */
export function createMockUpdateInfo(): UpdateInfo {
	return {
		available: false,
		version: null,
		body: null,
	};
}

/**
 * Create mock update info with an available update
 */
export function createMockUpdateAvailable(
	version = "1.1.0",
	body = "Bug fixes and improvements",
): UpdateInfo {
	return {
		available: true,
		version,
		body,
	};
}

/**
 * Reset the product ID counter (useful between tests)
 */
export function resetMockCounters(): void {
	productIdCounter = 1;
}

let availabilityIdCounter = 1;

/**
 * Create a mock availability check with default values
 */
export function createMockAvailabilityCheck(
	overrides: Partial<AvailabilityCheckResponse> = {},
): AvailabilityCheckResponse {
	const id = overrides.id ?? `availability-${availabilityIdCounter++}`;
	return {
		id,
		product_id: `product-${id}`,
		status: "in_stock",
		raw_availability: null,
		error_message: null,
		checked_at: new Date().toISOString(),
		price_cents: null,
		price_currency: null,
		raw_price: null,
		today_average_price_cents: null,
		yesterday_average_price_cents: null,
		is_price_drop: false,
		...overrides,
	};
}

/**
 * Create a mock bulk check summary with default values
 */
export function createMockBulkCheckSummary(
	overrides: Partial<BulkCheckSummary> = {},
): BulkCheckSummary {
	return {
		total: 5,
		successful: 5,
		failed: 0,
		back_in_stock_count: 0,
		price_drop_count: 0,
		results: [],
		...overrides,
	};
}
