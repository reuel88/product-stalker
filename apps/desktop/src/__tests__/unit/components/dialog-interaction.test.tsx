import { describe, expect, it, vi } from "vitest";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { render, screen } from "../../test-utils";

function TestDialog({ open = true }: { open?: boolean }) {
	return (
		<Dialog open={open} onOpenChange={vi.fn()}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Test Dialog</DialogTitle>
				</DialogHeader>
				<p>Content</p>
			</DialogContent>
		</Dialog>
	);
}

describe("Dialog interaction", () => {
	describe("resize handles", () => {
		it("should render all 8 resize handles", () => {
			render(<TestDialog />);

			const directions = ["n", "s", "e", "w", "ne", "nw", "se", "sw"];
			for (const dir of directions) {
				expect(
					document.querySelector(`[data-resize-handle="${dir}"]`),
				).toBeInTheDocument();
			}
		});

		it("should have correct cursor styles on resize handles", () => {
			render(<TestDialog />);

			const cursorMap: Record<string, string> = {
				n: "ns-resize",
				s: "ns-resize",
				e: "ew-resize",
				w: "ew-resize",
				ne: "nesw-resize",
				sw: "nesw-resize",
				nw: "nwse-resize",
				se: "nwse-resize",
			};

			for (const [dir, cursor] of Object.entries(cursorMap)) {
				const handle = document.querySelector(
					`[data-resize-handle="${dir}"]`,
				) as HTMLElement;
				expect(handle.style.cursor).toBe(cursor);
			}
		});
	});

	describe("header drag behavior", () => {
		it("should have cursor-grab class on dialog header", () => {
			render(<TestDialog />);

			const header = document.querySelector('[data-slot="dialog-header"]');
			expect(header).toHaveClass("cursor-grab");
		});

		it("should not trigger drag when clicking close button", async () => {
			const { user } = render(<TestDialog />);

			const closeButton = screen.getByRole("button", { name: "Close" });
			await user.click(closeButton);

			// Header should still have cursor-grab (not cursor-grabbing)
			const header = document.querySelector('[data-slot="dialog-header"]');
			expect(header).toHaveClass("cursor-grab");
			expect(header).not.toHaveClass("cursor-grabbing");
		});
	});

	describe("dialog content", () => {
		it("should have translate style for centering", () => {
			render(<TestDialog />);

			const popup = document.querySelector('[data-slot="dialog-content"]');
			expect(popup).toBeInTheDocument();
			expect((popup as HTMLElement).style.translate).toBe(
				"calc(-50% + 0px) calc(-50% + 0px)",
			);
		});

		it("should have w-full and max-w-lg classes by default", () => {
			render(<TestDialog />);

			const popup = document.querySelector('[data-slot="dialog-content"]');
			expect(popup).toHaveClass("w-full");
			expect(popup).toHaveClass("max-w-lg");
		});
	});
});
