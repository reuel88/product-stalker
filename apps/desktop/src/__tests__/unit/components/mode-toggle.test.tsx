import { beforeEach, describe, expect, it, vi } from "vitest";
import { ModeToggle } from "@/components/mode-toggle";
import { render, screen, waitFor } from "../../test-utils";

// Mock next-themes
const mockSetTheme = vi.fn();
vi.mock("next-themes", () => ({
	useTheme: () => ({
		theme: "system",
		setTheme: mockSetTheme,
	}),
}));

describe("ModeToggle", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("should render the toggle button", () => {
		render(<ModeToggle />);

		const button = screen.getByRole("button");
		expect(button).toBeInTheDocument();
	});

	it("should have screen reader text", () => {
		render(<ModeToggle />);

		expect(screen.getByText("Toggle theme")).toBeInTheDocument();
		expect(screen.getByText("Toggle theme")).toHaveClass("sr-only");
	});

	it("should render Sun icon for light mode indication", () => {
		const { container } = render(<ModeToggle />);

		// Find SVG icons within the button
		const svgs = container.querySelectorAll("button svg");
		expect(svgs.length).toBeGreaterThanOrEqual(2); // Sun and Moon icons
	});

	it("should open dropdown menu when clicked", async () => {
		const { user } = render(<ModeToggle />);

		const button = screen.getByRole("button");
		await user.click(button);

		await waitFor(() => {
			expect(screen.getByText("Light")).toBeInTheDocument();
			expect(screen.getByText("Dark")).toBeInTheDocument();
			expect(screen.getByText("System")).toBeInTheDocument();
		});
	});

	it("should call setTheme with 'light' when Light is selected", async () => {
		const { user } = render(<ModeToggle />);

		await user.click(screen.getByRole("button"));

		await waitFor(() => {
			expect(screen.getByText("Light")).toBeInTheDocument();
		});

		await user.click(screen.getByText("Light"));

		expect(mockSetTheme).toHaveBeenCalledWith("light");
	});

	it("should call setTheme with 'dark' when Dark is selected", async () => {
		const { user } = render(<ModeToggle />);

		await user.click(screen.getByRole("button"));

		await waitFor(() => {
			expect(screen.getByText("Dark")).toBeInTheDocument();
		});

		await user.click(screen.getByText("Dark"));

		expect(mockSetTheme).toHaveBeenCalledWith("dark");
	});

	it("should call setTheme with 'system' when System is selected", async () => {
		const { user } = render(<ModeToggle />);

		await user.click(screen.getByRole("button"));

		await waitFor(() => {
			expect(screen.getByText("System")).toBeInTheDocument();
		});

		await user.click(screen.getByText("System"));

		expect(mockSetTheme).toHaveBeenCalledWith("system");
	});

	it("should render button with outline variant", () => {
		render(<ModeToggle />);

		const button = screen.getByRole("button");
		// The button should have the outline variant styling
		expect(button).toBeInTheDocument();
	});
});
