export const MESSAGES = {
	PRODUCT: {
		CREATED: "Product created",
		UPDATED: "Product updated",
		DELETED: "Product deleted",
		CREATE_FAILED: "Failed to create product",
		UPDATE_FAILED: "Failed to update product",
		DELETE_FAILED: "Failed to delete product",
	},
	AVAILABILITY: {
		CHECKED: "Availability checked",
		CHECK_FAILED: "Failed to check availability",
		IN_STOCK: "In Stock",
		OUT_OF_STOCK: "Out of Stock",
		BACK_ORDER: "Back Order",
		UNKNOWN: "Unknown",
	},
	SETTINGS: {
		SAVED: "Settings saved",
		SAVE_FAILED: "Failed to save settings",
	},
	VALIDATION: {
		NAME_URL_REQUIRED: "Name and URL are required",
	},
} as const;
