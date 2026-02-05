const AVAILABILITY_PREFIX = "availability";

export const QUERY_KEYS = {
	PRODUCTS: ["products"],
	SETTINGS: ["settings"],
	AVAILABILITY_PREFIX,
	availability: (productId: string) =>
		[AVAILABILITY_PREFIX, productId] as const,
	availabilityHistory: (productId: string, limit?: number) =>
		limit !== undefined
			? ([AVAILABILITY_PREFIX, productId, "history", limit] as const)
			: ([AVAILABILITY_PREFIX, productId, "history"] as const),
} as const;
