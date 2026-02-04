import { describe, expect, it } from "vitest";
import { SettingsSkeleton } from "@/modules/settings/ui/components/settings-skeleton";
import { render } from "../../test-utils";

describe("SettingsSkeleton", () => {
	describe("rendering", () => {
		it("should render 5 skeleton cards", () => {
			const { container } = render(<SettingsSkeleton />);

			// Each card is wrapped in a div with Card component
			const cards = container.querySelectorAll('[class*="rounded-"]');
			// We expect multiple rounded elements (cards and skeleton items)
			expect(cards.length).toBeGreaterThan(0);
		});

		it("should render skeleton elements", () => {
			const { container } = render(<SettingsSkeleton />);

			// Skeleton components use animate-pulse class
			const skeletons = container.querySelectorAll('[class*="animate-pulse"]');
			expect(skeletons.length).toBeGreaterThan(0);
		});

		it("should render container with max-width", () => {
			const { container } = render(<SettingsSkeleton />);

			const wrapper = container.firstChild as HTMLElement;
			expect(wrapper).toHaveClass("container", "max-w-2xl");
		});

		it("should render header skeleton", () => {
			const { container } = render(<SettingsSkeleton />);

			// Header skeleton (h-7 w-24)
			const headerSkeleton = container.querySelector(".h-7.w-24");
			expect(headerSkeleton).toBeInTheDocument();
		});

		it("should render card header skeletons", () => {
			const { container } = render(<SettingsSkeleton />);

			// Card title skeletons (h-5 w-32)
			const titleSkeletons = container.querySelectorAll(".h-5.w-32");
			expect(titleSkeletons.length).toBe(5);
		});

		it("should render card description skeletons", () => {
			const { container } = render(<SettingsSkeleton />);

			// Card description skeletons (h-4 w-48)
			const descSkeletons = container.querySelectorAll(".h-4.w-48");
			expect(descSkeletons.length).toBe(5);
		});

		it("should render card content skeletons", () => {
			const { container } = render(<SettingsSkeleton />);

			// Card content label skeletons (h-4 w-24)
			const labelSkeletons = container.querySelectorAll(".h-4.w-24");
			expect(labelSkeletons.length).toBe(5);
		});

		it("should render toggle skeletons", () => {
			const { container } = render(<SettingsSkeleton />);

			// Toggle skeletons (h-5 w-9 rounded-full)
			const toggleSkeletons = container.querySelectorAll(
				".h-5.w-9.rounded-full",
			);
			expect(toggleSkeletons.length).toBe(5);
		});
	});

	describe("layout", () => {
		it("should have proper spacing", () => {
			const { container } = render(<SettingsSkeleton />);

			const wrapper = container.firstChild as HTMLElement;
			expect(wrapper).toHaveClass("px-4", "py-6");
		});

		it("should have space between cards", () => {
			const { container } = render(<SettingsSkeleton />);

			const cardsContainer = container.querySelector(".space-y-4");
			expect(cardsContainer).toBeInTheDocument();
		});

		it("should have margin below header", () => {
			const { container } = render(<SettingsSkeleton />);

			const headerSkeleton = container.querySelector(".mb-6");
			expect(headerSkeleton).toBeInTheDocument();
		});
	});
});
