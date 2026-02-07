import { describe, expect, it } from "vitest";
import {
	calculatePriceChangePercent,
	filterByTimeRange,
	formatPriceChangePercent,
	getDateRangeLabel,
	getPriceChangeDirection,
	transformToPriceDataPoints,
} from "@/lib/price-utils";
import type { AvailabilityCheckResponse } from "@/modules/products/types";

function createCheck(
	overrides: Partial<AvailabilityCheckResponse> = {},
): AvailabilityCheckResponse {
	return {
		id: "check-1",
		product_id: "product-1",
		status: "in_stock",
		raw_availability: null,
		error_message: null,
		checked_at: new Date().toISOString(),
		price_cents: 9999,
		price_currency: "USD",
		raw_price: "99.99",
		previous_price_cents: null,
		is_price_drop: false,
		...overrides,
	};
}

describe("filterByTimeRange", () => {
	it("should return all checks when range is 'all'", () => {
		const checks = [
			createCheck({ checked_at: "2024-01-01T00:00:00Z" }),
			createCheck({ checked_at: "2024-06-01T00:00:00Z" }),
			createCheck({ checked_at: "2024-12-01T00:00:00Z" }),
		];

		const result = filterByTimeRange(checks, "all");

		expect(result).toHaveLength(3);
		expect(result).toEqual(checks);
	});

	it("should filter checks within last 7 days", () => {
		const now = new Date();
		const fiveDaysAgo = new Date(now.getTime() - 5 * 24 * 60 * 60 * 1000);
		const tenDaysAgo = new Date(now.getTime() - 10 * 24 * 60 * 60 * 1000);

		const checks = [
			createCheck({ id: "recent", checked_at: fiveDaysAgo.toISOString() }),
			createCheck({ id: "old", checked_at: tenDaysAgo.toISOString() }),
		];

		const result = filterByTimeRange(checks, "7d");

		expect(result).toHaveLength(1);
		expect(result[0].id).toBe("recent");
	});

	it("should filter checks within last 30 days", () => {
		const now = new Date();
		const twentyDaysAgo = new Date(now.getTime() - 20 * 24 * 60 * 60 * 1000);
		const sixtyDaysAgo = new Date(now.getTime() - 60 * 24 * 60 * 60 * 1000);

		const checks = [
			createCheck({ id: "recent", checked_at: twentyDaysAgo.toISOString() }),
			createCheck({ id: "old", checked_at: sixtyDaysAgo.toISOString() }),
		];

		const result = filterByTimeRange(checks, "30d");

		expect(result).toHaveLength(1);
		expect(result[0].id).toBe("recent");
	});

	it("should return empty array when no checks match time range", () => {
		const now = new Date();
		const thirtyDaysAgo = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);

		const checks = [createCheck({ checked_at: thirtyDaysAgo.toISOString() })];

		const result = filterByTimeRange(checks, "7d");

		expect(result).toHaveLength(0);
	});

	it("should handle empty input array", () => {
		const result = filterByTimeRange([], "7d");

		expect(result).toHaveLength(0);
	});
});

describe("transformToPriceDataPoints", () => {
	it("should transform checks with prices into data points", () => {
		const checks = [
			createCheck({
				checked_at: "2024-01-01T10:00:00Z",
				price_cents: 9999,
				price_currency: "USD",
			}),
			createCheck({
				checked_at: "2024-01-02T10:00:00Z",
				price_cents: 8999,
				price_currency: "USD",
			}),
		];

		const result = transformToPriceDataPoints(checks);

		expect(result).toHaveLength(2);
		expect(result[0]).toEqual({
			date: "2024-01-01T10:00:00Z",
			price: 9999,
			currency: "USD",
		});
		expect(result[1]).toEqual({
			date: "2024-01-02T10:00:00Z",
			price: 8999,
			currency: "USD",
		});
	});

	it("should filter out checks with null prices", () => {
		const checks = [
			createCheck({
				checked_at: "2024-01-01T10:00:00Z",
				price_cents: 9999,
				price_currency: "USD",
			}),
			createCheck({
				checked_at: "2024-01-02T10:00:00Z",
				price_cents: null,
				price_currency: null,
			}),
		];

		const result = transformToPriceDataPoints(checks);

		expect(result).toHaveLength(1);
		expect(result[0].price).toBe(9999);
	});

	it("should filter out checks with null currency", () => {
		const checks = [
			createCheck({
				checked_at: "2024-01-01T10:00:00Z",
				price_cents: 9999,
				price_currency: null,
			}),
		];

		const result = transformToPriceDataPoints(checks);

		expect(result).toHaveLength(0);
	});

	it("should sort data points by date ascending", () => {
		const checks = [
			createCheck({
				checked_at: "2024-01-03T10:00:00Z",
				price_cents: 7999,
				price_currency: "USD",
			}),
			createCheck({
				checked_at: "2024-01-01T10:00:00Z",
				price_cents: 9999,
				price_currency: "USD",
			}),
			createCheck({
				checked_at: "2024-01-02T10:00:00Z",
				price_cents: 8999,
				price_currency: "USD",
			}),
		];

		const result = transformToPriceDataPoints(checks);

		expect(result[0].date).toBe("2024-01-01T10:00:00Z");
		expect(result[1].date).toBe("2024-01-02T10:00:00Z");
		expect(result[2].date).toBe("2024-01-03T10:00:00Z");
	});

	it("should handle empty input array", () => {
		const result = transformToPriceDataPoints([]);

		expect(result).toHaveLength(0);
	});
});

describe("getDateRangeLabel", () => {
	it("should return empty string for empty array", () => {
		const result = getDateRangeLabel([]);

		expect(result).toBe("");
	});

	it("should return single date for one data point", () => {
		const dataPoints = [
			{ date: "2024-01-15T10:00:00Z", price: 9999, currency: "USD" },
		];

		const result = getDateRangeLabel(dataPoints);

		expect(result).toBe(new Date("2024-01-15T10:00:00Z").toLocaleDateString());
	});

	it("should return date range for multiple data points", () => {
		const dataPoints = [
			{ date: "2024-01-01T10:00:00Z", price: 9999, currency: "USD" },
			{ date: "2024-01-15T10:00:00Z", price: 8999, currency: "USD" },
			{ date: "2024-01-30T10:00:00Z", price: 7999, currency: "USD" },
		];

		const result = getDateRangeLabel(dataPoints);

		const firstDate = new Date("2024-01-01T10:00:00Z").toLocaleDateString();
		const lastDate = new Date("2024-01-30T10:00:00Z").toLocaleDateString();
		expect(result).toBe(`${firstDate} - ${lastDate}`);
	});
});

describe("getPriceChangeDirection", () => {
	it("should return 'down' when current price is lower", () => {
		expect(getPriceChangeDirection(8000, 10000)).toBe("down");
	});

	it("should return 'up' when current price is higher", () => {
		expect(getPriceChangeDirection(12000, 10000)).toBe("up");
	});

	it("should return 'unchanged' when prices are equal", () => {
		expect(getPriceChangeDirection(10000, 10000)).toBe("unchanged");
	});

	it("should return 'unknown' when current price is null", () => {
		expect(getPriceChangeDirection(null, 10000)).toBe("unknown");
	});

	it("should return 'unknown' when previous price is null", () => {
		expect(getPriceChangeDirection(10000, null)).toBe("unknown");
	});

	it("should return 'unknown' when both prices are null", () => {
		expect(getPriceChangeDirection(null, null)).toBe("unknown");
	});
});

describe("calculatePriceChangePercent", () => {
	it("should calculate positive percentage for price increase", () => {
		expect(calculatePriceChangePercent(11000, 10000)).toBe(10);
	});

	it("should calculate negative percentage for price decrease", () => {
		expect(calculatePriceChangePercent(9000, 10000)).toBe(-10);
	});

	it("should return 0 when prices are equal", () => {
		expect(calculatePriceChangePercent(10000, 10000)).toBe(0);
	});

	it("should return null when current price is null", () => {
		expect(calculatePriceChangePercent(null, 10000)).toBeNull();
	});

	it("should return null when previous price is null", () => {
		expect(calculatePriceChangePercent(10000, null)).toBeNull();
	});

	it("should return null when previous price is zero", () => {
		expect(calculatePriceChangePercent(10000, 0)).toBeNull();
	});

	it("should round percentage to integer", () => {
		expect(calculatePriceChangePercent(10333, 10000)).toBe(3);
		expect(calculatePriceChangePercent(10666, 10000)).toBe(7);
	});
});

describe("formatPriceChangePercent", () => {
	it("should format positive percentage with plus sign", () => {
		expect(formatPriceChangePercent(15)).toBe("+15%");
	});

	it("should format negative percentage", () => {
		expect(formatPriceChangePercent(-12)).toBe("-12%");
	});

	it("should format zero with plus sign", () => {
		expect(formatPriceChangePercent(0)).toBe("+0%");
	});

	it("should return empty string for null", () => {
		expect(formatPriceChangePercent(null)).toBe("");
	});
});
