import { describe, expect, it } from "vitest";
import {
	filterByTimeRange,
	getDateRangeLabel,
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
