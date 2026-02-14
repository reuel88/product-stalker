import { describe, expect, it } from "vitest";
import type { MultiRetailerChartData } from "@/modules/products/types";
import { PriceHistoryChart } from "@/modules/products/ui/components/price-history-chart";
import { render, screen } from "../../test-utils";

function createChartData(
	overrides: Partial<MultiRetailerChartData> = {},
): MultiRetailerChartData {
	return {
		data: [],
		series: [],
		currency: "USD",
		currencyExponent: 2,
		...overrides,
	};
}

describe("PriceHistoryChart", () => {
	describe("empty state", () => {
		it("should show empty message when data is empty", () => {
			render(<PriceHistoryChart chartData={createChartData()} />);

			expect(screen.getByText("No price data available")).toBeInTheDocument();
		});
	});

	describe("single data point state", () => {
		it("should show single price when there is only one data point and one series", () => {
			const chartData = createChartData({
				data: [{ date: "2024-01-15T10:00:00Z", "retailer-1": 9999 }],
				series: [
					{ id: "retailer-1", label: "amazon.com", color: "var(--chart-1)" },
				],
			});

			render(<PriceHistoryChart chartData={chartData} />);

			expect(screen.getByText("$99.99")).toBeInTheDocument();
			expect(
				screen.getByText(/More data points needed to show price trend/),
			).toBeInTheDocument();
		});

		it("should show recorded date for single data point", () => {
			const chartData = createChartData({
				data: [{ date: "2024-01-15T10:00:00Z", "retailer-1": 12500 }],
				series: [
					{ id: "retailer-1", label: "amazon.com", color: "var(--chart-1)" },
				],
			});

			render(<PriceHistoryChart chartData={chartData} />);

			expect(screen.getByText(/Recorded on/)).toBeInTheDocument();
		});
	});

	describe("chart rendering", () => {
		it("should render chart container for multiple data points", () => {
			const chartData = createChartData({
				data: [
					{ date: "2024-01-01T10:00:00Z", "retailer-1": 9999 },
					{ date: "2024-01-02T10:00:00Z", "retailer-1": 8999 },
				],
				series: [
					{ id: "retailer-1", label: "amazon.com", color: "var(--chart-1)" },
				],
			});

			const { container } = render(<PriceHistoryChart chartData={chartData} />);

			const responsiveContainer = container.querySelector(
				".recharts-responsive-container",
			);
			expect(responsiveContainer).toBeInTheDocument();
		});

		it("should render chart container for multiple series", () => {
			const chartData = createChartData({
				data: [
					{
						date: "2024-01-01T10:00:00Z",
						"retailer-1": 9999,
						"retailer-2": 10999,
					},
					{
						date: "2024-01-02T10:00:00Z",
						"retailer-1": 8999,
						"retailer-2": 9999,
					},
				],
				series: [
					{ id: "retailer-1", label: "amazon.com", color: "var(--chart-1)" },
					{
						id: "retailer-2",
						label: "bestbuy.com",
						color: "var(--chart-2)",
					},
				],
			});

			const { container } = render(<PriceHistoryChart chartData={chartData} />);

			const responsiveContainer = container.querySelector(
				".recharts-responsive-container",
			);
			expect(responsiveContainer).toBeInTheDocument();
		});

		it("should not show empty message for multiple data points", () => {
			const chartData = createChartData({
				data: [
					{ date: "2024-01-01T10:00:00Z", "retailer-1": 9999 },
					{ date: "2024-01-02T10:00:00Z", "retailer-1": 8999 },
				],
				series: [
					{ id: "retailer-1", label: "amazon.com", color: "var(--chart-1)" },
				],
			});

			render(<PriceHistoryChart chartData={chartData} />);

			expect(
				screen.queryByText("No price data available"),
			).not.toBeInTheDocument();
		});

		it("should not show single data point message for multiple data points", () => {
			const chartData = createChartData({
				data: [
					{ date: "2024-01-01T10:00:00Z", "retailer-1": 9999 },
					{ date: "2024-01-02T10:00:00Z", "retailer-1": 8999 },
				],
				series: [
					{ id: "retailer-1", label: "amazon.com", color: "var(--chart-1)" },
				],
			});

			render(<PriceHistoryChart chartData={chartData} />);

			expect(
				screen.queryByText(/More data points needed/),
			).not.toBeInTheDocument();
		});
	});

	describe("price formatting", () => {
		it("should format prices correctly in different currencies", () => {
			const chartData = createChartData({
				data: [{ date: "2024-01-15T10:00:00Z", "retailer-1": 15000 }],
				series: [
					{ id: "retailer-1", label: "amazon.de", color: "var(--chart-1)" },
				],
				currency: "EUR",
			});

			render(<PriceHistoryChart chartData={chartData} />);

			expect(screen.getByText(/150/)).toBeInTheDocument();
		});
	});
});
