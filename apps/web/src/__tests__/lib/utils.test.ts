import { describe, expect, it } from "vitest";
import { cn } from "@/lib/utils";

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
