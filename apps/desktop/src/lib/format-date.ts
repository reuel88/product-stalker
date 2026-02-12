/**
 * Date formatting utilities using Intl.DateTimeFormat
 *
 * Provides consistent date formatting across the app based on user settings.
 * Maps date format preferences to locale codes for Intl.DateTimeFormat.
 */

type DateFormat = "system" | "MM/DD/YYYY" | "DD/MM/YYYY" | "YYYY-MM-DD";
type Timezone = string; // "auto" or IANA timezone string

/**
 * Maps date format setting to locale code for Intl.DateTimeFormat
 */
function getLocaleForFormat(format: DateFormat): string | undefined {
	switch (format) {
		case "system":
			return undefined; // Use browser default
		case "MM/DD/YYYY":
			return "en-US";
		case "DD/MM/YYYY":
			return "en-GB";
		case "YYYY-MM-DD":
			return "sv-SE";
		default:
			return undefined;
	}
}

/**
 * Gets timezone option for Intl.DateTimeFormat
 */
function getTimezoneOption(timezone: Timezone): string | undefined {
	return timezone === "auto" ? undefined : timezone;
}

/**
 * Formats a date string as date-only (no time)
 *
 * @param dateString - ISO date string
 * @param format - Date format preference
 * @param timezone - Timezone setting
 * @returns Formatted date string
 */
export function formatDate(
	dateString: string,
	format: DateFormat = "system",
	timezone: Timezone = "auto",
): string {
	try {
		const date = new Date(dateString);
		if (Number.isNaN(date.getTime())) {
			return "Invalid date";
		}

		const locale = getLocaleForFormat(format);
		const timeZone = getTimezoneOption(timezone);

		return new Intl.DateTimeFormat(locale, {
			year: "numeric",
			month: "2-digit",
			day: "2-digit",
			timeZone,
		}).format(date);
	} catch {
		return "Invalid date";
	}
}

/**
 * Formats a date string with both date and time
 *
 * @param dateString - ISO date string
 * @param format - Date format preference
 * @param timezone - Timezone setting
 * @returns Formatted date and time string
 */
export function formatDateTime(
	dateString: string,
	format: DateFormat = "system",
	timezone: Timezone = "auto",
): string {
	try {
		const date = new Date(dateString);
		if (Number.isNaN(date.getTime())) {
			return "Invalid date";
		}

		const locale = getLocaleForFormat(format);
		const timeZone = getTimezoneOption(timezone);

		return new Intl.DateTimeFormat(locale, {
			year: "numeric",
			month: "2-digit",
			day: "2-digit",
			hour: "2-digit",
			minute: "2-digit",
			timeZone,
		}).format(date);
	} catch {
		return "Invalid date";
	}
}

/**
 * Formats a date for chart axis labels (short format)
 *
 * @param dateString - ISO date string
 * @param timezone - Timezone setting
 * @returns Short formatted date (e.g., "12/25" or "Dec 25")
 */
export function formatChartAxisDate(
	dateString: string,
	timezone: Timezone = "auto",
): string {
	try {
		const date = new Date(dateString);
		if (Number.isNaN(date.getTime())) {
			return "";
		}

		const timeZone = getTimezoneOption(timezone);

		return new Intl.DateTimeFormat(undefined, {
			month: "short",
			day: "numeric",
			timeZone,
		}).format(date);
	} catch {
		return "";
	}
}

/**
 * Formats a date for chart tooltips (full format with time)
 *
 * @param dateString - ISO date string
 * @param format - Date format preference
 * @param timezone - Timezone setting
 * @returns Formatted date and time for tooltip
 */
export function formatChartTooltipDate(
	dateString: string,
	format: DateFormat = "system",
	timezone: Timezone = "auto",
): string {
	try {
		const date = new Date(dateString);
		if (Number.isNaN(date.getTime())) {
			return "Invalid date";
		}

		const locale = getLocaleForFormat(format);
		const timeZone = getTimezoneOption(timezone);

		return new Intl.DateTimeFormat(locale, {
			year: "numeric",
			month: "long",
			day: "numeric",
			hour: "2-digit",
			minute: "2-digit",
			timeZone,
		}).format(date);
	} catch {
		return "Invalid date";
	}
}

/**
 * Gets a date range label from an array of dates
 *
 * @param dates - Array of ISO date strings
 * @param format - Date format preference
 * @param timezone - Timezone setting
 * @returns Formatted date range string (e.g., "12/01/2023 - 12/31/2023")
 */
export function getDateRangeLabel(
	dates: string[],
	format: DateFormat = "system",
	timezone: Timezone = "auto",
): string {
	if (dates.length === 0) {
		return "No data";
	}

	if (dates.length === 1) {
		return formatDate(dates[0], format, timezone);
	}

	const firstDate = formatDate(dates[0], format, timezone);
	const lastDate = formatDate(dates[dates.length - 1], format, timezone);

	return `${firstDate} - ${lastDate}`;
}
