import { describe, expect, it } from "vitest";
import { CONFIG } from "@/constants/config";

describe("CONFIG constant", () => {
	it("should export QUERY configuration", () => {
		expect(CONFIG).toHaveProperty("QUERY");
	});

	describe("QUERY config", () => {
		it("should have STALE_TIME defined", () => {
			expect(CONFIG.QUERY).toHaveProperty("STALE_TIME");
			expect(typeof CONFIG.QUERY.STALE_TIME).toBe("number");
		});

		it("should have reasonable STALE_TIME (1-30 minutes)", () => {
			const oneMinute = 1000 * 60;
			const thirtyMinutes = 1000 * 60 * 30;
			expect(CONFIG.QUERY.STALE_TIME).toBeGreaterThanOrEqual(oneMinute);
			expect(CONFIG.QUERY.STALE_TIME).toBeLessThanOrEqual(thirtyMinutes);
		});

		it("should have STALE_TIME of 5 minutes", () => {
			expect(CONFIG.QUERY.STALE_TIME).toBe(1000 * 60 * 5);
		});

		it("should have RETRY defined", () => {
			expect(CONFIG.QUERY).toHaveProperty("RETRY");
			expect(typeof CONFIG.QUERY.RETRY).toBe("number");
		});

		it("should have reasonable RETRY count (0-5)", () => {
			expect(CONFIG.QUERY.RETRY).toBeGreaterThanOrEqual(0);
			expect(CONFIG.QUERY.RETRY).toBeLessThanOrEqual(5);
		});

		it("should have RETRY of 1", () => {
			expect(CONFIG.QUERY.RETRY).toBe(1);
		});
	});
});
