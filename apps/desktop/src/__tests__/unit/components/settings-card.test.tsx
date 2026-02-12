import { describe, expect, it } from "vitest";
import { SettingsCard } from "@/modules/settings/ui/components/settings-card";
import { render, screen } from "../../test-utils";

describe("SettingsCard", () => {
	it("should render card with title and description", () => {
		render(
			<SettingsCard title="Test Title" description="Test description">
				<div>Test content</div>
			</SettingsCard>,
		);

		expect(screen.getByText("Test Title")).toBeInTheDocument();
		expect(screen.getByText("Test description")).toBeInTheDocument();
		expect(screen.getByText("Test content")).toBeInTheDocument();
	});

	it("should render children in card content", () => {
		render(
			<SettingsCard title="Settings" description="Description">
				<button type="button">Click me</button>
			</SettingsCard>,
		);

		expect(
			screen.getByRole("button", { name: "Click me" }),
		).toBeInTheDocument();
	});

	it("should apply contentClassName when provided", () => {
		const { container } = render(
			<SettingsCard
				title="Settings"
				description="Description"
				contentClassName="space-y-4"
			>
				<div>Content</div>
			</SettingsCard>,
		);

		// CardContent should have the space-y-4 class
		const cardContent = container.querySelector('[class*="space-y-4"]');
		expect(cardContent).toBeInTheDocument();
	});

	it("should render without contentClassName", () => {
		render(
			<SettingsCard title="Settings" description="Description">
				<div>Content</div>
			</SettingsCard>,
		);

		expect(screen.getByText("Settings")).toBeInTheDocument();
		expect(screen.getByText("Content")).toBeInTheDocument();
	});
});
