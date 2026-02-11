import { describe, expect, it } from "vitest";
import { cn, formatPrice } from "@/lib/utils";

describe("cn utility", () => {
	it("should merge class names", () => {
		const result = cn("foo", "bar");
		expect(result).toBe("foo bar");
	});

	it("should handle conditional classes", () => {
		const result = cn("base", false && "hidden", true && "visible");
		expect(result).toBe("base visible");
	});

	it("should merge tailwind classes correctly", () => {
		const result = cn("px-2 py-1", "px-4");
		expect(result).toBe("py-1 px-4");
	});

	it("should handle undefined and null values", () => {
		const result = cn("foo", undefined, null, "bar");
		expect(result).toBe("foo bar");
	});

	it("should handle empty input", () => {
		const result = cn();
		expect(result).toBe("");
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
