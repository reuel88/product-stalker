import { describe, expect, it } from "vitest";
import { QUERY_KEYS } from "@/constants/queryKeys";

describe("QUERY_KEYS constant", () => {
	it("should export all required query keys", () => {
		expect(QUERY_KEYS).toHaveProperty("PRODUCTS");
		expect(QUERY_KEYS).toHaveProperty("SETTINGS");
		expect(QUERY_KEYS).toHaveProperty("availability");
		expect(QUERY_KEYS).toHaveProperty("availabilityHistory");
	});

	it("should have array values for static query keys", () => {
		expect(Array.isArray(QUERY_KEYS.PRODUCTS)).toBe(true);
		expect(Array.isArray(QUERY_KEYS.SETTINGS)).toBe(true);
	});

	it("should have non-empty arrays for static query keys", () => {
		expect(QUERY_KEYS.PRODUCTS.length).toBeGreaterThan(0);
		expect(QUERY_KEYS.SETTINGS.length).toBeGreaterThan(0);
	});

	it("should have expected key structures", () => {
		expect(QUERY_KEYS.PRODUCTS).toEqual(["products"]);
		expect(QUERY_KEYS.SETTINGS).toEqual(["settings"]);
	});

	it("should generate availability keys correctly", () => {
		const productId = "test-123";
		expect(QUERY_KEYS.availability(productId)).toEqual([
			"availability",
			productId,
		]);
		expect(QUERY_KEYS.availabilityHistory(productId)).toEqual([
			"availability",
			productId,
			"history",
		]);
		expect(QUERY_KEYS.availabilityHistory(productId, 10)).toEqual([
			"availability",
			productId,
			"history",
			10,
		]);
	});

	it("should have unique top-level keys", () => {
		const keys = Object.keys(QUERY_KEYS);
		const uniqueKeys = new Set(keys);
		expect(uniqueKeys.size).toBe(keys.length);
	});
});
