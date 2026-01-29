import { describe, expect, it } from "vitest";
import { QUERY_KEYS } from "@/constants/queryKeys";

describe("QUERY_KEYS constant", () => {
	it("should export all required query keys", () => {
		expect(QUERY_KEYS).toHaveProperty("PRODUCTS");
		expect(QUERY_KEYS).toHaveProperty("SETTINGS");
	});

	it("should have array values for query keys", () => {
		expect(Array.isArray(QUERY_KEYS.PRODUCTS)).toBe(true);
		expect(Array.isArray(QUERY_KEYS.SETTINGS)).toBe(true);
	});

	it("should have non-empty arrays", () => {
		expect(QUERY_KEYS.PRODUCTS.length).toBeGreaterThan(0);
		expect(QUERY_KEYS.SETTINGS.length).toBeGreaterThan(0);
	});

	it("should have expected key structures", () => {
		expect(QUERY_KEYS.PRODUCTS).toEqual(["products"]);
		expect(QUERY_KEYS.SETTINGS).toEqual(["settings"]);
	});

	it("should have unique top-level keys", () => {
		const keys = Object.keys(QUERY_KEYS);
		const uniqueKeys = new Set(keys);
		expect(uniqueKeys.size).toBe(keys.length);
	});

	it("should have unique key values", () => {
		const values = Object.values(QUERY_KEYS).map((arr) => arr.join("-"));
		const uniqueValues = new Set(values);
		expect(uniqueValues.size).toBe(values.length);
	});
});
