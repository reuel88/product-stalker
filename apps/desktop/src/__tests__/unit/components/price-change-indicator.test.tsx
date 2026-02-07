import { describe, expect, it } from "vitest";
import { PriceChangeIndicator } from "@/modules/products/ui/components/price-change-indicator";
import { render, screen } from "../../test-utils";

describe("PriceChangeIndicator", () => {
	describe("compact variant", () => {
		it("should display price with no indicator when no yesterday average", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={79900}
					todayAverageCents={79900}
					yesterdayAverageCents={null}
					currency="USD"
					variant="compact"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
		});

		it("should display price drop indicator", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={79900}
					todayAverageCents={79900}
					yesterdayAverageCents={89900}
					currency="USD"
					variant="compact"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
			expect(screen.getByText("-11%")).toBeInTheDocument();
		});

		it("should display price increase indicator", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={89900}
					todayAverageCents={89900}
					yesterdayAverageCents={79900}
					currency="USD"
					variant="compact"
				/>,
			);

			expect(screen.getByText("$899.00")).toBeInTheDocument();
			expect(screen.getByText("+13%")).toBeInTheDocument();
		});

		it("should display dash when current price is null", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={null}
					todayAverageCents={null}
					yesterdayAverageCents={null}
					currency={null}
					variant="compact"
				/>,
			);

			expect(screen.getByText("-")).toBeInTheDocument();
		});

		it("should not show indicator when price is unchanged", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={79900}
					todayAverageCents={79900}
					yesterdayAverageCents={79900}
					currency="USD"
					variant="compact"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
			expect(screen.queryByText("+0%")).not.toBeInTheDocument();
			expect(screen.queryByText("-0%")).not.toBeInTheDocument();
		});
	});

	describe("detailed variant", () => {
		it("should display price with no indicator when no yesterday average", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={79900}
					todayAverageCents={79900}
					yesterdayAverageCents={null}
					currency="USD"
					variant="detailed"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
		});

		it("should display price drop with details", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={79900}
					todayAverageCents={79900}
					yesterdayAverageCents={89900}
					currency="USD"
					variant="detailed"
				/>,
			);

			expect(screen.getByText("$799.00")).toBeInTheDocument();
			expect(screen.getByText(/Down 11% from \$899\.00/)).toBeInTheDocument();
		});

		it("should display price increase with details", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={89900}
					todayAverageCents={89900}
					yesterdayAverageCents={79900}
					currency="USD"
					variant="detailed"
				/>,
			);

			expect(screen.getByText("$899.00")).toBeInTheDocument();
			expect(screen.getByText(/Up 13% from \$799\.00/)).toBeInTheDocument();
		});

		it("should display dash when current price is null", () => {
			render(
				<PriceChangeIndicator
					currentPriceCents={null}
					todayAverageCents={null}
					yesterdayAverageCents={null}
					currency={null}
					variant="detailed"
				/>,
			);

			expect(screen.getByText("-")).toBeInTheDocument();
		});
	});
});
