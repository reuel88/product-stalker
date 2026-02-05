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
		CHECK_ALL_COMPLETE: "All products checked",
		CHECK_ALL_FAILED: "Failed to check all products",
		BACK_IN_STOCK: "Back in stock!",
		IN_STOCK: "In Stock",
		OUT_OF_STOCK: "Out of Stock",
		BACK_ORDER: "Back Order",
		UNKNOWN: "Unknown",
		BOT_PROTECTION:
			"This site has bot protection. Enable headless browser in settings to check this site.",
		CHROME_NOT_FOUND:
			"Chrome/Chromium not found. Please install Chrome to check this site.",
		CAPTCHA_REQUIRED:
			"This site requires manual verification. Please check the product page directly.",
	},
	PRICE: {
		NO_PRICE: "-",
		PRICE_DROP: "Price drop!",
	},
	SETTINGS: {
		SAVED: "Settings saved",
		SAVE_FAILED: "Failed to save settings",
	},
	VALIDATION: {
		NAME_URL_REQUIRED: "Name and URL are required",
	},
	UPDATE: {
		AVAILABLE: (version: string) => `Update available: v${version}`,
		LATEST: "You're running the latest version",
		DOWNLOADING: "Downloading update...",
		CHECK_FAILED: "Failed to check for updates",
		INSTALL_FAILED: "Failed to install update",
	},
} as const;
