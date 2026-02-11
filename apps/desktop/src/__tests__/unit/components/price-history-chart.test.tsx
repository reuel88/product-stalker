import { describe, expect, it } from "vitest";
import type { PriceDataPoint } from "@/modules/products/types";
import { PriceHistoryChart } from "@/modules/products/ui/components/price-history-chart";
import { render, screen } from "../../test-utils";

describe("PriceHistoryChart", () => {
	describe("empty state", () => {
		it("should show empty message when data is empty", () => {
			render(<PriceHistoryChart data={[]} />);

			expect(screen.getByText("No price data available")).toBeInTheDocument();
		});
	});

	describe("single data point state", () => {
		it("should show single price when there is only one data point", () => {
			const data: PriceDataPoint[] = [
				{
					date: "2024-01-15T10:00:00Z",
					price: 9999,
					currency: "USD",
					currencyExponent: 2,
				},
			];

			render(<PriceHistoryChart data={data} />);

			expect(screen.getByText("$99.99")).toBeInTheDocument();
			expect(
				screen.getByText(/More data points needed to show price trend/),
			).toBeInTheDocument();
		});

		it("should show recorded date for single data point", () => {
			const data: PriceDataPoint[] = [
				{
					date: "2024-01-15T10:00:00Z",
					price: 12500,
					currency: "USD",
					currencyExponent: 2,
				},
			];

			render(<PriceHistoryChart data={data} />);

			expect(screen.getByText(/Recorded on/)).toBeInTheDocument();
		});
	});

	describe("chart rendering", () => {
		it("should render chart container for multiple data points", () => {
			const data: PriceDataPoint[] = [
				{
					date: "2024-01-01T10:00:00Z",
					price: 9999,
					currency: "USD",
					currencyExponent: 2,
				},
				{
					date: "2024-01-02T10:00:00Z",
					price: 8999,
					currency: "USD",
					currencyExponent: 2,
				},
			];

			const { container } = render(<PriceHistoryChart data={data} />);

			// Check that RecahrtResponsiveContainer is rendered
			const responsiveContainer = container.querySelector(
				".recharts-responsive-container",
			);
			expect(responsiveContainer).toBeInTheDocument();
		});

		it("should not show empty message for multiple data points", () => {
			const data: PriceDataPoint[] = [
				{
					date: "2024-01-01T10:00:00Z",
					price: 9999,
					currency: "USD",
					currencyExponent: 2,
				},
				{
					date: "2024-01-02T10:00:00Z",
					price: 8999,
					currency: "USD",
					currencyExponent: 2,
				},
			];

			render(<PriceHistoryChart data={data} />);

			expect(
				screen.queryByText("No price data available"),
			).not.toBeInTheDocument();
		});

		it("should not show single data point message for multiple data points", () => {
			const data: PriceDataPoint[] = [
				{
					date: "2024-01-01T10:00:00Z",
					price: 9999,
					currency: "USD",
					currencyExponent: 2,
				},
				{
					date: "2024-01-02T10:00:00Z",
					price: 8999,
					currency: "USD",
					currencyExponent: 2,
				},
			];

			render(<PriceHistoryChart data={data} />);

			expect(
				screen.queryByText(/More data points needed/),
			).not.toBeInTheDocument();
		});
	});

	describe("price formatting", () => {
		it("should format prices correctly in different currencies", () => {
			const data: PriceDataPoint[] = [
				{
					date: "2024-01-15T10:00:00Z",
					price: 15000,
					currency: "EUR",
					currencyExponent: 2,
				},
			];

			render(<PriceHistoryChart data={data} />);

			// EUR formatting may vary by locale, but should contain the amount
			expect(screen.getByText(/150/)).toBeInTheDocument();
		});
	});
});
