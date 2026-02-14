import { describe, expect, it } from "vitest";
import {
	calculatePriceChangePercent,
	extractDomain,
	filterByTimeRange,
	findCheapestRetailerId,
	formatPrice,
	formatPriceChangePercent,
	getDateRangeLabel,
	getDisplayPrice,
	getLatestPriceByRetailer,
	getPriceChangeDirection,
	isRoundedZero,
	type RetailerPrice,
	transformToMultiRetailerChartData,
	transformToPriceDataPoints,
} from "@/modules/products/price-utils";
import type {
	AvailabilityCheckResponse,
	ProductRetailerResponse,
} from "@/modules/products/types";

function createCheck(
	overrides: Partial<AvailabilityCheckResponse> = {},
): AvailabilityCheckResponse {
	return {
		id: "check-1",
		product_id: "product-1",
		product_retailer_id: null,
		status: "in_stock",
		raw_availability: null,
		error_message: null,
		checked_at: new Date().toISOString(),
		price_minor_units: 9999,
		price_currency: "USD",
		raw_price: "99.99",
		currency_exponent: 2,
		today_average_price_minor_units: null,
		yesterday_average_price_minor_units: null,
		is_price_drop: false,
		lowest_price_minor_units: null,
		lowest_price_currency: null,
		lowest_currency_exponent: null,
		normalized_price_minor_units: null,
		normalized_currency: null,
		normalized_currency_exponent: null,
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
				price_minor_units: 9999,
				price_currency: "USD",
			}),
			createCheck({
				checked_at: "2024-01-02T10:00:00Z",
				price_minor_units: 8999,
				price_currency: "USD",
			}),
		];

		const result = transformToPriceDataPoints(checks);

		expect(result).toHaveLength(2);
		expect(result[0]).toEqual({
			date: "2024-01-01T10:00:00Z",
			price: 9999,
			currency: "USD",
			currencyExponent: 2,
		});
		expect(result[1]).toEqual({
			date: "2024-01-02T10:00:00Z",
			price: 8999,
			currency: "USD",
			currencyExponent: 2,
		});
	});

	it("should filter out checks with null prices", () => {
		const checks = [
			createCheck({
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
			createCheck({
				checked_at: "2024-01-02T10:00:00Z",
				price_minor_units: null,
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
				price_minor_units: 9999,
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
				price_minor_units: 7999,
				price_currency: "USD",
			}),
			createCheck({
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
			createCheck({
				checked_at: "2024-01-02T10:00:00Z",
				price_minor_units: 8999,
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
	const mockFormatDate = (dateString: string) =>
		new Date(dateString).toLocaleDateString("en-US");

	it("should return empty string for empty array", () => {
		const result = getDateRangeLabel([], mockFormatDate);

		expect(result).toBe("");
	});

	it("should return single date for one data point", () => {
		const dataPoints = [
			{
				date: "2024-01-15T10:00:00Z",
				price: 9999,
				currency: "USD",
				currencyExponent: 2,
			},
		];

		const result = getDateRangeLabel(dataPoints, mockFormatDate);

		expect(result).toBe(mockFormatDate("2024-01-15T10:00:00Z"));
	});

	it("should return date range for multiple data points", () => {
		const dataPoints = [
			{
				date: "2024-01-01T10:00:00Z",
				price: 9999,
				currency: "USD",
				currencyExponent: 2,
			},
			{
				date: "2024-01-15T10:00:00Z",
				price: 8999,
				currency: "USD",
				currencyExponent: 2,
			},
			{
				date: "2024-01-30T10:00:00Z",
				price: 7999,
				currency: "USD",
				currencyExponent: 2,
			},
		];

		const result = getDateRangeLabel(dataPoints, mockFormatDate);

		const firstDate = mockFormatDate("2024-01-01T10:00:00Z");
		const lastDate = mockFormatDate("2024-01-30T10:00:00Z");
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

describe("formatPrice", () => {
	it("should format USD price with default exponent (2)", () => {
		expect(formatPrice(9999, "USD")).toBe("$99.99");
	});

	it("should format price with explicit exponent 2", () => {
		expect(formatPrice(78900, "USD", 2)).toBe("$789.00");
	});

	it("should format JPY price with exponent 0", () => {
		expect(formatPrice(1500, "JPY", 0)).toBe("\u00a51,500");
	});

	it("should format KWD price with exponent 3", () => {
		expect(formatPrice(29990, "KWD", 3)).toBe("KWD\u00a029.990");
	});

	it("should return dash when minor units is null", () => {
		expect(formatPrice(null, "USD")).toBe("-");
	});

	it("should return dash when currency is null", () => {
		expect(formatPrice(9999, null)).toBe("-");
	});

	it("should return dash when both are null", () => {
		expect(formatPrice(null, null)).toBe("-");
	});
});

describe("isRoundedZero", () => {
	it("should return true when change rounds to 0% but is not exactly 0 (positive)", () => {
		// 0.05% increase rounds to 0%
		expect(isRoundedZero(80000, 79960)).toBe(true);
		// 0.4% increase rounds to 0%
		expect(isRoundedZero(80320, 80000)).toBe(true);
	});

	it("should return true when change rounds to 0% but is not exactly 0 (negative)", () => {
		// -0.05% decrease rounds to 0%
		expect(isRoundedZero(79960, 80000)).toBe(true);
		// -0.4% decrease rounds to 0%
		expect(isRoundedZero(80000, 80320)).toBe(true);
	});

	it("should return false when change is exactly 0%", () => {
		expect(isRoundedZero(80000, 80000)).toBe(false);
	});

	it("should return false when change is 1% or more", () => {
		// 1% increase
		expect(isRoundedZero(80800, 80000)).toBe(false);
		// -1% decrease
		expect(isRoundedZero(79200, 80000)).toBe(false);
		// 5% increase
		expect(isRoundedZero(84000, 80000)).toBe(false);
	});

	it("should return false when today value is null", () => {
		expect(isRoundedZero(null, 80000)).toBe(false);
	});

	it("should return false when yesterday value is null", () => {
		expect(isRoundedZero(80000, null)).toBe(false);
	});

	it("should return false when both values are null", () => {
		expect(isRoundedZero(null, null)).toBe(false);
	});

	it("should return false when yesterday value is zero", () => {
		expect(isRoundedZero(100, 0)).toBe(false);
	});
});

function createRetailer(
	overrides: Partial<ProductRetailerResponse> = {},
): ProductRetailerResponse {
	return {
		id: "pr-1",
		product_id: "product-1",
		retailer_id: "amazon.com",
		url: "https://www.amazon.com/dp/B123",
		label: null,
		sort_order: 0,
		created_at: new Date().toISOString(),
		...overrides,
	};
}

describe("extractDomain", () => {
	it("should extract hostname from a valid URL", () => {
		expect(extractDomain("https://www.amazon.com/dp/B123")).toBe(
			"www.amazon.com",
		);
	});

	it("should return raw string for invalid URL", () => {
		expect(extractDomain("not-a-url")).toBe("not-a-url");
	});
});

describe("getDisplayPrice", () => {
	it("should prefer lowest_price fields when available", () => {
		const check = createCheck({
			price_minor_units: 9999,
			price_currency: "USD",
			currency_exponent: 2,
			lowest_price_minor_units: 7999,
			lowest_price_currency: "AUD",
			lowest_currency_exponent: 2,
		});

		const result = getDisplayPrice(check);

		expect(result.price).toBe(7999);
		expect(result.currency).toBe("AUD");
		expect(result.exponent).toBe(2);
	});

	it("should fall back to single-check price fields", () => {
		const check = createCheck({
			price_minor_units: 9999,
			price_currency: "USD",
			currency_exponent: 3,
			lowest_price_minor_units: null,
			lowest_price_currency: null,
			lowest_currency_exponent: null,
		});

		const result = getDisplayPrice(check);

		expect(result.price).toBe(9999);
		expect(result.currency).toBe("USD");
		expect(result.exponent).toBe(3);
	});

	it("should return nulls and default exponent for null check", () => {
		const result = getDisplayPrice(null);

		expect(result.price).toBeNull();
		expect(result.currency).toBeNull();
		expect(result.exponent).toBe(2);
	});

	it("should return nulls and default exponent for undefined check", () => {
		const result = getDisplayPrice(undefined);

		expect(result.price).toBeNull();
		expect(result.currency).toBeNull();
		expect(result.exponent).toBe(2);
	});

	it("should default exponent to 2 when both are null", () => {
		const check = createCheck({
			currency_exponent: null,
			lowest_currency_exponent: null,
		});

		const result = getDisplayPrice(check);

		expect(result.exponent).toBe(2);
	});
});

describe("transformToMultiRetailerChartData", () => {
	it("should return empty data for empty checks", () => {
		const result = transformToMultiRetailerChartData([], []);

		expect(result.data).toHaveLength(0);
		expect(result.series).toHaveLength(0);
	});

	it("should produce two series for two retailers", () => {
		const retailers = [
			createRetailer({
				id: "pr-1",
				url: "https://www.amazon.com/dp/B123",
			}),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
				retailer_id: "bestbuy.com",
			}),
		];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-01T10:00:05Z",
				price_minor_units: 9999,
				price_currency: "USD",
				currency_exponent: 2,
			}),
			createCheck({
				id: "c2",
				product_retailer_id: "pr-2",
				checked_at: "2024-01-01T10:00:10Z",
				price_minor_units: 10999,
				price_currency: "USD",
				currency_exponent: 2,
			}),
		];

		const result = transformToMultiRetailerChartData(checks, retailers);

		expect(result.series).toHaveLength(2);
		expect(result.series[0].label).toBe("www.amazon.com");
		expect(result.series[1].label).toBe("www.bestbuy.com");
		expect(result.series[0].color).toBe("var(--chart-1)");
		expect(result.series[1].color).toBe("var(--chart-2)");
		expect(result.currency).toBe("USD");
		expect(result.currencyExponent).toBe(2);
	});

	it("should bucket checks from same minute into one row", () => {
		const retailers = [
			createRetailer({ id: "pr-1" }),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
			}),
		];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-01T10:00:05Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
			createCheck({
				id: "c2",
				product_retailer_id: "pr-2",
				checked_at: "2024-01-01T10:00:30Z",
				price_minor_units: 10999,
				price_currency: "USD",
			}),
		];

		const result = transformToMultiRetailerChartData(checks, retailers);

		expect(result.data).toHaveLength(1);
		expect(result.data[0]["pr-1"]).toBe(9999);
		expect(result.data[0]["pr-2"]).toBe(10999);
	});

	it("should produce single 'Price' series for legacy checks (null product_retailer_id)", () => {
		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: null,
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
			createCheck({
				id: "c2",
				product_retailer_id: null,
				checked_at: "2024-01-02T10:00:00Z",
				price_minor_units: 8999,
				price_currency: "USD",
			}),
		];

		const result = transformToMultiRetailerChartData(checks, []);

		expect(result.series).toHaveLength(1);
		expect(result.series[0].id).toBe("legacy");
		expect(result.series[0].label).toBe("Price");
		expect(result.data).toHaveLength(2);
		expect(result.data[0].legacy).toBe(9999);
		expect(result.data[1].legacy).toBe(8999);
	});

	it("should filter out checks with null prices", () => {
		const retailers = [createRetailer({ id: "pr-1" })];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
			createCheck({
				id: "c2",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-02T10:00:00Z",
				price_minor_units: null,
				price_currency: null,
			}),
		];

		const result = transformToMultiRetailerChartData(checks, retailers);

		expect(result.data).toHaveLength(1);
	});

	it("should sort data rows by date ascending", () => {
		const retailers = [createRetailer({ id: "pr-1" })];

		const checks = [
			createCheck({
				id: "c2",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-03T10:00:00Z",
				price_minor_units: 7999,
				price_currency: "USD",
			}),
			createCheck({
				id: "c1",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
		];

		const result = transformToMultiRetailerChartData(checks, retailers);

		expect(new Date(result.data[0].date as string).getTime()).toBeLessThan(
			new Date(result.data[1].date as string).getTime(),
		);
	});

	it("should include label in series when retailer has a label", () => {
		const retailers = [
			createRetailer({
				id: "pr-1",
				url: "https://www.amazon.com/dp/B123",
				label: "64GB",
			}),
		];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
		];

		const result = transformToMultiRetailerChartData(checks, retailers);

		expect(result.series[0].label).toBe("www.amazon.com (64GB)");
	});
});

describe("getLatestPriceByRetailer", () => {
	it("should return latest price for each retailer", () => {
		const retailers = [
			createRetailer({ id: "pr-1" }),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
			}),
		];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
				currency_exponent: 2,
			}),
			createCheck({
				id: "c2",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-02T10:00:00Z",
				price_minor_units: 8999,
				price_currency: "USD",
				currency_exponent: 2,
			}),
			createCheck({
				id: "c3",
				product_retailer_id: "pr-2",
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 10999,
				price_currency: "USD",
				currency_exponent: 2,
			}),
		];

		const result = getLatestPriceByRetailer(checks, retailers);

		expect(result.size).toBe(2);
		expect(result.get("pr-1")).toEqual({
			priceMinorUnits: 8999,
			currency: "USD",
			currencyExponent: 2,
		});
		expect(result.get("pr-2")).toEqual({
			priceMinorUnits: 10999,
			currency: "USD",
			currencyExponent: 2,
		});
	});

	it("should skip checks with null prices", () => {
		const retailers = [createRetailer({ id: "pr-1" })];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-02T10:00:00Z",
				price_minor_units: null,
				price_currency: null,
			}),
			createCheck({
				id: "c2",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
				currency_exponent: 2,
			}),
		];

		const result = getLatestPriceByRetailer(checks, retailers);

		expect(result.size).toBe(1);
		expect(result.get("pr-1")?.priceMinorUnits).toBe(9999);
	});

	it("should ignore checks with null product_retailer_id", () => {
		const retailers = [createRetailer({ id: "pr-1" })];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: null,
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
		];

		const result = getLatestPriceByRetailer(checks, retailers);

		expect(result.size).toBe(0);
	});

	it("should ignore checks for unknown retailer IDs", () => {
		const retailers = [createRetailer({ id: "pr-1" })];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: "pr-unknown",
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
			}),
		];

		const result = getLatestPriceByRetailer(checks, retailers);

		expect(result.size).toBe(0);
	});

	it("should return empty map for empty inputs", () => {
		const result = getLatestPriceByRetailer([], []);

		expect(result.size).toBe(0);
	});

	it("should use default exponent of 2 when currency_exponent is null", () => {
		const retailers = [createRetailer({ id: "pr-1" })];

		const checks = [
			createCheck({
				id: "c1",
				product_retailer_id: "pr-1",
				checked_at: "2024-01-01T10:00:00Z",
				price_minor_units: 9999,
				price_currency: "USD",
				currency_exponent: null,
			}),
		];

		const result = getLatestPriceByRetailer(checks, retailers);

		expect(result.get("pr-1")?.currencyExponent).toBe(2);
	});
});

describe("findCheapestRetailerId", () => {
	it("should return the cheapest retailer ID", () => {
		const priceMap = new Map<string, RetailerPrice>([
			["pr-1", { priceMinorUnits: 9999, currency: "USD", currencyExponent: 2 }],
			["pr-2", { priceMinorUnits: 7999, currency: "USD", currencyExponent: 2 }],
			[
				"pr-3",
				{ priceMinorUnits: 10999, currency: "USD", currencyExponent: 2 },
			],
		]);

		expect(findCheapestRetailerId(priceMap)).toBe("pr-2");
	});

	it("should return null when fewer than 2 retailers have prices", () => {
		const singlePrice = new Map<string, RetailerPrice>([
			["pr-1", { priceMinorUnits: 9999, currency: "USD", currencyExponent: 2 }],
		]);

		expect(findCheapestRetailerId(singlePrice)).toBeNull();
	});

	it("should return null for empty map", () => {
		expect(findCheapestRetailerId(new Map())).toBeNull();
	});

	it("should return first cheapest when prices are tied", () => {
		const priceMap = new Map<string, RetailerPrice>([
			["pr-1", { priceMinorUnits: 9999, currency: "USD", currencyExponent: 2 }],
			["pr-2", { priceMinorUnits: 9999, currency: "USD", currencyExponent: 2 }],
		]);

		const result = findCheapestRetailerId(priceMap);
		// Both have same price; first one wins since it's < not <=
		expect(result).toBe("pr-1");
	});
});
