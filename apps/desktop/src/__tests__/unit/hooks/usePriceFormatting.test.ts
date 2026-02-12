import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { usePriceFormatting } from "@/modules/products/hooks/usePriceFormatting";

describe("usePriceFormatting", () => {
	describe("basic price formatting", () => {
		it("should format USD price correctly", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: 79900,
					currency: "USD",
					currencyExponent: 2,
				}),
			);

			expect(result.current.formattedCurrentPrice).toBe("$799.00");
		});

		it("should format JPY price correctly", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: 1500,
					currency: "JPY",
					currencyExponent: 0,
				}),
			);

			expect(result.current.formattedCurrentPrice).toBe("¥1,500");
		});

		it("should return dash when price is null", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: null,
					currency: "USD",
					currencyExponent: 2,
				}),
			);

			expect(result.current.formattedCurrentPrice).toBe("-");
		});
	});

	describe("price comparison", () => {
		it("should detect price decrease", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: 79900,
					todayAverageMinorUnits: 79900,
					yesterdayAverageMinorUnits: 89900,
					currency: "USD",
					currencyExponent: 2,
				}),
			);

			expect(result.current.direction).toBe("down");
			expect(result.current.percentChange).toBe(-11);
			expect(result.current.formattedPercentChange).toBe("-11%");
			expect(result.current.hasComparison).toBe(true);
		});

		it("should detect price increase", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: 89900,
					todayAverageMinorUnits: 89900,
					yesterdayAverageMinorUnits: 79900,
					currency: "USD",
					currencyExponent: 2,
				}),
			);

			expect(result.current.direction).toBe("up");
			expect(result.current.percentChange).toBe(13);
			expect(result.current.formattedPercentChange).toBe("+13%");
			expect(result.current.hasComparison).toBe(true);
		});

		it("should detect unchanged price", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: 79900,
					todayAverageMinorUnits: 79900,
					yesterdayAverageMinorUnits: 79900,
					currency: "USD",
					currencyExponent: 2,
				}),
			);

			expect(result.current.direction).toBe("unchanged");
			expect(result.current.percentChange).toBe(0);
			expect(result.current.hasComparison).toBe(false);
		});

		it("should handle unknown comparison when data is missing", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: 79900,
					todayAverageMinorUnits: null,
					yesterdayAverageMinorUnits: null,
					currency: "USD",
					currencyExponent: 2,
				}),
			);

			expect(result.current.direction).toBe("unknown");
			expect(result.current.percentChange).toBe(null);
			expect(result.current.hasComparison).toBe(false);
		});
	});

	describe("previous price formatting", () => {
		it("should format previous price correctly", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: 79900,
					todayAverageMinorUnits: 79900,
					yesterdayAverageMinorUnits: 89900,
					currency: "USD",
					currencyExponent: 2,
				}),
			);

			expect(result.current.formattedPreviousPrice).toBe("$899.00");
		});

		it("should return dash when previous price is null", () => {
			const { result } = renderHook(() =>
				usePriceFormatting({
					currentPriceMinorUnits: 79900,
					todayAverageMinorUnits: 79900,
					yesterdayAverageMinorUnits: null,
					currency: "USD",
					currencyExponent: 2,
				}),
			);

			expect(result.current.formattedPreviousPrice).toBe("-");
		});
	});

	describe("memoization", () => {
		it("should memoize formatted values when props don't change", () => {
			const { result, rerender } = renderHook(
				(props) => usePriceFormatting(props),
				{
					initialProps: {
						currentPriceMinorUnits: 79900,
						todayAverageMinorUnits: 79900,
						yesterdayAverageMinorUnits: 89900,
						currency: "USD",
						currencyExponent: 2,
					},
				},
			);

			const firstResult = result.current;

			// Re-render with same props
			rerender({
				currentPriceMinorUnits: 79900,
				todayAverageMinorUnits: 79900,
				yesterdayAverageMinorUnits: 89900,
				currency: "USD",
				currencyExponent: 2,
			});

			// References should be the same (memoized)
			expect(result.current.formattedCurrentPrice).toBe(
				firstResult.formattedCurrentPrice,
			);
			expect(result.current.direction).toBe(firstResult.direction);
			expect(result.current.percentChange).toBe(firstResult.percentChange);
		});
	});
});
