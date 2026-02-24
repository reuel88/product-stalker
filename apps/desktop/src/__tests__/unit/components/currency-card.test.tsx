import { beforeEach, describe, expect, it, vi } from "vitest";
import { CurrencyCard } from "@/modules/settings/ui/components/currency-card";
import { createMockSettings } from "../../mocks/data";
import { render, screen, waitFor } from "../../test-utils";

describe("CurrencyCard", () => {
	const mockOnUpdate = vi.fn();

	beforeEach(() => {
		mockOnUpdate.mockClear();
	});

	it("should render with current currency setting", () => {
		const settings = createMockSettings({ preferred_currency: "AUD" });

		render(<CurrencyCard settings={settings} onUpdate={mockOnUpdate} />);

		expect(screen.getByText("Currency")).toBeInTheDocument();
		expect(
			screen.getByText("Set your preferred currency for price comparisons"),
		).toBeInTheDocument();
		expect(screen.getByText("Preferred Currency")).toBeInTheDocument();
	});

	it("should call onUpdate when currency is changed", async () => {
		const settings = createMockSettings({ preferred_currency: "AUD" });

		const { user } = render(
			<CurrencyCard settings={settings} onUpdate={mockOnUpdate} />,
		);

		const combobox = screen.getByRole("combobox");
		await user.click(combobox);

		// Select USD from the dropdown
		const usdOption = await screen.findByText("USD ($) - US Dollar");
		await user.click(usdOption);

		await waitFor(() => {
			expect(mockOnUpdate).toHaveBeenCalledWith({
				preferred_currency: "USD",
			});
		});
	});

	it("should render the select trigger with id for accessibility", () => {
		const settings = createMockSettings({ preferred_currency: "EUR" });

		render(<CurrencyCard settings={settings} onUpdate={mockOnUpdate} />);

		const trigger = document.getElementById("preferred-currency");
		expect(trigger).toBeInTheDocument();
	});
});
