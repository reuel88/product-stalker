import { MESSAGES } from "@/constants";
import type { AvailabilityStatus } from "@/modules/products/types";

export const STATUS_BADGE_CONFIG: Record<
	AvailabilityStatus,
	{ label: string; className: string }
> = {
	in_stock: {
		label: MESSAGES.AVAILABILITY.IN_STOCK,
		className:
			"bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
	},
	out_of_stock: {
		label: MESSAGES.AVAILABILITY.OUT_OF_STOCK,
		className: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
	},
	back_order: {
		label: MESSAGES.AVAILABILITY.BACK_ORDER,
		className:
			"bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200",
	},
	unknown: {
		label: MESSAGES.AVAILABILITY.UNKNOWN,
		className: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
	},
};
