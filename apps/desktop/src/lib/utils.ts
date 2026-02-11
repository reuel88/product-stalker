import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

/**
 * Format a price in minor units to a localized currency string.
 * @param minorUnits - Price in smallest currency unit (e.g., cents for USD, yen for JPY)
 * @param currency - ISO 4217 currency code (e.g., "USD", "EUR", "JPY")
 * @param exponent - Number of decimal places for the currency (0 for JPY, 2 for USD, 3 for KWD). Defaults to 2.
 * @returns Formatted price string or "-" if price is not available
 */
export function formatPrice(
	minorUnits: number | null,
	currency: string | null,
	exponent = 2,
): string {
	if (minorUnits === null || currency === null) return "-";
	return new Intl.NumberFormat("en-US", {
		style: "currency",
		currency,
	}).format(minorUnits / 10 ** exponent);
}
