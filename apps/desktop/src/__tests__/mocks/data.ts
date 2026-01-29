import type { ProductResponse } from "@/modules/products/types";
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
		updated_at: new Date().toISOString(),
		...overrides,
	};
}

/**
 * Create mock update info
 */
export function createMockUpdateInfo(
	overrides: Partial<UpdateInfo> = {},
): UpdateInfo {
	return {
		available: false,
		version: null,
		body: null,
		...overrides,
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
