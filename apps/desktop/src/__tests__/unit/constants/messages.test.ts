import { describe, expect, it } from "vitest";
import { MESSAGES } from "@/constants/messages";

describe("MESSAGES constant", () => {
	it("should export PRODUCT messages", () => {
		expect(MESSAGES).toHaveProperty("PRODUCT");
	});

	it("should export AVAILABILITY messages", () => {
		expect(MESSAGES).toHaveProperty("AVAILABILITY");
	});

	it("should export SETTINGS messages", () => {
		expect(MESSAGES).toHaveProperty("SETTINGS");
	});

	it("should export VALIDATION messages", () => {
		expect(MESSAGES).toHaveProperty("VALIDATION");
	});

	describe("PRODUCT messages", () => {
		it("should have all CRUD operation messages", () => {
			expect(MESSAGES.PRODUCT).toHaveProperty("CREATED");
			expect(MESSAGES.PRODUCT).toHaveProperty("UPDATED");
			expect(MESSAGES.PRODUCT).toHaveProperty("DELETED");
		});

		it("should have all error messages", () => {
			expect(MESSAGES.PRODUCT).toHaveProperty("CREATE_FAILED");
			expect(MESSAGES.PRODUCT).toHaveProperty("UPDATE_FAILED");
			expect(MESSAGES.PRODUCT).toHaveProperty("DELETE_FAILED");
		});

		it("should have non-empty string messages", () => {
			expect(typeof MESSAGES.PRODUCT.CREATED).toBe("string");
			expect(MESSAGES.PRODUCT.CREATED.length).toBeGreaterThan(0);
			expect(typeof MESSAGES.PRODUCT.UPDATED).toBe("string");
			expect(MESSAGES.PRODUCT.UPDATED.length).toBeGreaterThan(0);
			expect(typeof MESSAGES.PRODUCT.DELETED).toBe("string");
			expect(MESSAGES.PRODUCT.DELETED.length).toBeGreaterThan(0);
		});

		it("should have descriptive error messages", () => {
			expect(MESSAGES.PRODUCT.CREATE_FAILED).toContain("Failed");
			expect(MESSAGES.PRODUCT.UPDATE_FAILED).toContain("Failed");
			expect(MESSAGES.PRODUCT.DELETE_FAILED).toContain("Failed");
		});
	});

	describe("SETTINGS messages", () => {
		it("should have success and error messages", () => {
			expect(MESSAGES.SETTINGS).toHaveProperty("SAVED");
			expect(MESSAGES.SETTINGS).toHaveProperty("SAVE_FAILED");
		});

		it("should have non-empty string messages", () => {
			expect(typeof MESSAGES.SETTINGS.SAVED).toBe("string");
			expect(MESSAGES.SETTINGS.SAVED.length).toBeGreaterThan(0);
			expect(typeof MESSAGES.SETTINGS.SAVE_FAILED).toBe("string");
			expect(MESSAGES.SETTINGS.SAVE_FAILED.length).toBeGreaterThan(0);
		});
	});

	describe("AVAILABILITY messages", () => {
		it("should have status messages", () => {
			expect(MESSAGES.AVAILABILITY).toHaveProperty("CHECKED");
			expect(MESSAGES.AVAILABILITY).toHaveProperty("CHECK_FAILED");
			expect(MESSAGES.AVAILABILITY).toHaveProperty("IN_STOCK");
			expect(MESSAGES.AVAILABILITY).toHaveProperty("OUT_OF_STOCK");
			expect(MESSAGES.AVAILABILITY).toHaveProperty("BACK_ORDER");
			expect(MESSAGES.AVAILABILITY).toHaveProperty("UNKNOWN");
		});

		it("should have non-empty string messages", () => {
			expect(typeof MESSAGES.AVAILABILITY.CHECKED).toBe("string");
			expect(MESSAGES.AVAILABILITY.CHECKED.length).toBeGreaterThan(0);
			expect(typeof MESSAGES.AVAILABILITY.IN_STOCK).toBe("string");
			expect(MESSAGES.AVAILABILITY.IN_STOCK.length).toBeGreaterThan(0);
		});
	});

	describe("VALIDATION messages", () => {
		it("should have required field messages", () => {
			expect(MESSAGES.VALIDATION).toHaveProperty("NAME_URL_REQUIRED");
		});

		it("should have non-empty validation messages", () => {
			expect(typeof MESSAGES.VALIDATION.NAME_URL_REQUIRED).toBe("string");
			expect(MESSAGES.VALIDATION.NAME_URL_REQUIRED.length).toBeGreaterThan(0);
		});

		it("should mention required fields in message", () => {
			expect(MESSAGES.VALIDATION.NAME_URL_REQUIRED.toLowerCase()).toContain(
				"required",
			);
		});
	});
});
