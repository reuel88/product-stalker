import { describe, expect, it } from "vitest";
import { UI } from "@/constants/ui";

describe("UI constant", () => {
	it("should export TRUNCATE configuration", () => {
		expect(UI).toHaveProperty("TRUNCATE");
	});

	it("should export PAGINATION configuration", () => {
		expect(UI).toHaveProperty("PAGINATION");
	});

	it("should export LOG_LEVELS", () => {
		expect(UI).toHaveProperty("LOG_LEVELS");
	});

	describe("TRUNCATE config", () => {
		it("should have URL_LENGTH defined", () => {
			expect(UI.TRUNCATE).toHaveProperty("URL_LENGTH");
			expect(typeof UI.TRUNCATE.URL_LENGTH).toBe("number");
		});

		it("should have reasonable URL_LENGTH (20-100 characters)", () => {
			expect(UI.TRUNCATE.URL_LENGTH).toBeGreaterThanOrEqual(20);
			expect(UI.TRUNCATE.URL_LENGTH).toBeLessThanOrEqual(100);
		});

		it("should have DESCRIPTION_LENGTH defined", () => {
			expect(UI.TRUNCATE).toHaveProperty("DESCRIPTION_LENGTH");
			expect(typeof UI.TRUNCATE.DESCRIPTION_LENGTH).toBe("number");
		});

		it("should have reasonable DESCRIPTION_LENGTH (20-200 characters)", () => {
			expect(UI.TRUNCATE.DESCRIPTION_LENGTH).toBeGreaterThanOrEqual(20);
			expect(UI.TRUNCATE.DESCRIPTION_LENGTH).toBeLessThanOrEqual(200);
		});
	});

	describe("PAGINATION config", () => {
		it("should have DEFAULT_PAGE_SIZE defined", () => {
			expect(UI.PAGINATION).toHaveProperty("DEFAULT_PAGE_SIZE");
			expect(typeof UI.PAGINATION.DEFAULT_PAGE_SIZE).toBe("number");
		});

		it("should have reasonable DEFAULT_PAGE_SIZE (5-100)", () => {
			expect(UI.PAGINATION.DEFAULT_PAGE_SIZE).toBeGreaterThanOrEqual(5);
			expect(UI.PAGINATION.DEFAULT_PAGE_SIZE).toBeLessThanOrEqual(100);
		});

		it("should have DEFAULT_PAGE_SIZE of 10", () => {
			expect(UI.PAGINATION.DEFAULT_PAGE_SIZE).toBe(10);
		});
	});

	describe("LOG_LEVELS", () => {
		it("should be an array", () => {
			expect(Array.isArray(UI.LOG_LEVELS)).toBe(true);
		});

		it("should have common log levels", () => {
			expect(UI.LOG_LEVELS).toContain("error");
			expect(UI.LOG_LEVELS).toContain("warn");
			expect(UI.LOG_LEVELS).toContain("info");
			expect(UI.LOG_LEVELS).toContain("debug");
		});

		it("should have log levels in order of severity", () => {
			const errorIndex = UI.LOG_LEVELS.indexOf("error");
			const warnIndex = UI.LOG_LEVELS.indexOf("warn");
			const infoIndex = UI.LOG_LEVELS.indexOf("info");
			const debugIndex = UI.LOG_LEVELS.indexOf("debug");

			expect(errorIndex).toBeLessThan(warnIndex);
			expect(warnIndex).toBeLessThan(infoIndex);
			expect(infoIndex).toBeLessThan(debugIndex);
		});

		it("should include trace level", () => {
			expect(UI.LOG_LEVELS).toContain("trace");
		});

		it("should have 5 log levels", () => {
			expect(UI.LOG_LEVELS).toHaveLength(5);
		});
	});
});
