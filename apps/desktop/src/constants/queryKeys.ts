const AVAILABILITY_PREFIX = "availability";

// UPPER_CASE = static query keys, camelCase = factory functions that accept parameters
export const QUERY_KEYS = {
	// === DOMAIN ===
	PRODUCTS: ["products"],
	AVAILABILITY_PREFIX,
	product: (id: string) => ["product", id] as const,
	productRetailers: (productId: string) =>
		["productRetailers", productId] as const,
	availability: (productId: string) =>
		[AVAILABILITY_PREFIX, productId] as const,
	availabilityHistory: (productId: string, limit?: number) =>
		limit !== undefined
			? ([AVAILABILITY_PREFIX, productId, "history", limit] as const)
			: ([AVAILABILITY_PREFIX, productId, "history"] as const),
	// === INFRASTRUCTURE ===
	SETTINGS: ["settings"],
	EXCHANGE_RATES: ["exchange_rates"],
	CURRENT_VERSION: ["currentVersion"],
} as const;
