export const QUERY_KEYS = {
	PRODUCTS: ["products"],
	SETTINGS: ["settings"],
	availability: (productId: string) => ["availability", productId] as const,
	availabilityHistory: (productId: string, limit?: number) =>
		limit !== undefined
			? (["availability", productId, "history", limit] as const)
			: (["availability", productId, "history"] as const),
} as const;
