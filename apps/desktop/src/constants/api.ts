export const COMMANDS = {
	// === DOMAIN ===
	GET_PRODUCTS: "get_products",
	GET_PRODUCT: "get_product",
	CREATE_PRODUCT: "create_product",
	UPDATE_PRODUCT: "update_product",
	DELETE_PRODUCT: "delete_product",
	REORDER_PRODUCTS: "reorder_products",
	ADD_PRODUCT_RETAILER: "add_product_retailer",
	GET_PRODUCT_RETAILERS: "get_product_retailers",
	REORDER_PRODUCT_RETAILERS: "reorder_product_retailers",
	REMOVE_PRODUCT_RETAILER: "remove_product_retailer",
	CHECK_AVAILABILITY: "check_availability",
	GET_LATEST_AVAILABILITY: "get_latest_availability",
	GET_AVAILABILITY_HISTORY: "get_availability_history",
	CHECK_ALL_AVAILABILITY: "check_all_availability",
	// === INFRASTRUCTURE ===
	GET_SETTINGS: "get_settings",
	UPDATE_SETTINGS: "update_settings",
	SEND_NOTIFICATION: "send_notification",
	CLOSE_SPLASHSCREEN: "close_splashscreen",
	CHECK_FOR_UPDATE: "check_for_update",
	DOWNLOAD_AND_INSTALL_UPDATE: "download_and_install_update",
	GET_CURRENT_VERSION: "get_current_version",
} as const;

export const EVENTS = {
	// === DOMAIN ===
	AVAILABILITY_CHECK_PROGRESS: "availability:check-progress",
	MANUAL_VERIFICATION_REQUESTED: "availability:manual-verification-requested",
} as const;
