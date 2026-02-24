import { describe, expect, it } from "vitest";
import { PriceChangeIndicator } from "@/modules/products/ui/components/price-change-indicator";
import { render, screen } from "../../test-utils";

describe("PriceChangeIndicator", () => {
	describe("compact variant", () => {
		it("should display price with no indicator when no yesterday average", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={79900}
					todayComparisonMinorUnits={79900}
					yesterdayComparisonMinorUnits={null}
					currency="USD"
					currencyExponent={2}
					variant="compact"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
		});

		it("should display price drop indicator", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={79900}
					todayComparisonMinorUnits={79900}
					yesterdayComparisonMinorUnits={89900}
					currency="USD"
					currencyExponent={2}
					variant="compact"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
			expect(screen.getByText("-11%")).toBeInTheDocument();
		});

		it("should display price increase indicator", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={89900}
					todayComparisonMinorUnits={89900}
					yesterdayComparisonMinorUnits={79900}
					currency="USD"
					currencyExponent={2}
					variant="compact"
				/>,
			);

			expect(screen.getByText("$899.00")).toBeInTheDocument();
			expect(screen.getByText("+13%")).toBeInTheDocument();
		});

		it("should display dash when current price is null", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={null}
					todayComparisonMinorUnits={null}
					yesterdayComparisonMinorUnits={null}
					currency={null}
					currencyExponent={2}
					variant="compact"
				/>,
			);

			expect(screen.getByText("-")).toBeInTheDocument();
		});

		it("should not show indicator when price is unchanged", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={79900}
					todayComparisonMinorUnits={79900}
					yesterdayComparisonMinorUnits={79900}
					currency="USD"
					currencyExponent={2}
					variant="compact"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
			expect(screen.queryByText("+0%")).not.toBeInTheDocument();
			expect(screen.queryByText("-0%")).not.toBeInTheDocument();
		});

		it("should show icon only when price change rounds to 0% (increase)", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={80000}
					todayComparisonMinorUnits={80000}
					yesterdayComparisonMinorUnits={79960}
					currency="USD"
					currencyExponent={2}
					variant="compact"
				/>,
			);

			expect(screen.getByText("$800.00")).toBeInTheDocument();
			// Icon should be present but percentage text should not
			expect(screen.queryByText("+0%")).not.toBeInTheDocument();
			expect(screen.queryByText("0%")).not.toBeInTheDocument();
		});

		it("should show icon only when price change rounds to 0% (decrease)", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={79960}
					todayComparisonMinorUnits={79960}
					yesterdayComparisonMinorUnits={80000}
					currency="USD"
					currencyExponent={2}
					variant="compact"
				/>,
			);

			expect(screen.getByText("$799.60")).toBeInTheDocument();
			// Icon should be present but percentage text should not
			expect(screen.queryByText("-0%")).not.toBeInTheDocument();
			expect(screen.queryByText("0%")).not.toBeInTheDocument();
		});
	});

	describe("detailed variant", () => {
		it("should display price with no indicator when no yesterday average", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={79900}
					todayComparisonMinorUnits={79900}
					yesterdayComparisonMinorUnits={null}
					currency="USD"
					currencyExponent={2}
					variant="detailed"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
		});

		it("should display price drop with details", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={79900}
					todayComparisonMinorUnits={79900}
					yesterdayComparisonMinorUnits={89900}
					currency="USD"
					currencyExponent={2}
					variant="detailed"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
			expect(screen.getByText(/Down 11% from \$899\.00/)).toBeInTheDocument();
		});

		it("should display price increase with details", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={89900}
					todayComparisonMinorUnits={89900}
					yesterdayComparisonMinorUnits={79900}
					currency="USD"
					currencyExponent={2}
					variant="detailed"
				/>,
			);

			expect(screen.getByText("$899.00")).toBeInTheDocument();
			expect(screen.getByText(/Up 13% from \$799\.00/)).toBeInTheDocument();
		});

		it("should display dash when current price is null", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={null}
					todayComparisonMinorUnits={null}
					yesterdayComparisonMinorUnits={null}
					currency={null}
					currencyExponent={2}
					variant="detailed"
				/>,
			);

			expect(screen.getByText("-")).toBeInTheDocument();
		});

		it("should show minimal change text when price change rounds to 0% (increase)", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={80000}
					todayComparisonMinorUnits={80000}
					yesterdayComparisonMinorUnits={79960}
					currency="USD"
					currencyExponent={2}
					variant="detailed"
				/>,
			);

			expect(screen.getByText("$800.00")).toBeInTheDocument();
			expect(screen.getByText(/Minimal up from \$799\.60/)).toBeInTheDocument();
			expect(screen.queryByText(/0%/)).not.toBeInTheDocument();
		});

		it("should show minimal change text when price change rounds to 0% (decrease)", () => {
			render(
				<PriceChangeIndicator
					currentPriceMinorUnits={79960}
					todayComparisonMinorUnits={79960}
					yesterdayComparisonMinorUnits={80000}
					currency="USD"
					currencyExponent={2}
					variant="detailed"
				/>,
			);

			expect(screen.getByText("$799.60")).toBeInTheDocument();
			expect(
				screen.getByText(/Minimal down from \$800\.00/),
			).toBeInTheDocument();
			expect(screen.queryByText(/0%/)).not.toBeInTheDocument();
		});
	});
});
