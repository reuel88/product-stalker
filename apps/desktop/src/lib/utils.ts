import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

/**
 * Format a price in cents to a localized currency string
 * @param cents - Price in smallest currency unit (e.g., cents)
 * @param currency - ISO 4217 currency code (e.g., "USD", "EUR", "AUD")
 * @returns Formatted price string or "-" if price is not available
 */
export function formatPrice(
	cents: number | null,
	currency: string | null,
): string {
	if (cents === null || currency === null) return "-";
	return new Intl.NumberFormat("en-US", {
		style: "currency",
		currency,
	}).format(cents / 100);
}
