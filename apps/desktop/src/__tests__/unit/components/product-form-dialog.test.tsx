import { describe, expect, it, vi } from "vitest";
import { ProductFormDialog } from "@/modules/products/ui/components/product-form-dialog";
import { render, screen } from "../../test-utils";

const defaultProps = {
	open: true,
	onOpenChange: vi.fn(),
	mode: "create" as const,
	formData: {
		name: "",
		url: "",
		description: null,
		notes: null,
	},
	onFormChange: vi.fn(),
	onSubmit: vi.fn(),
	isSubmitting: false,
};

describe("ProductFormDialog", () => {
	describe("rendering", () => {
		it("should render dialog with title for create mode", () => {
			render(<ProductFormDialog {...defaultProps} />);

			expect(screen.getByText("Add Product")).toBeInTheDocument();
		});

		it("should render dialog with title for edit mode", () => {
			render(<ProductFormDialog {...defaultProps} mode="edit" />);

			expect(screen.getByText("Edit Product")).toBeInTheDocument();
		});

		it("should render all form fields", () => {
			render(<ProductFormDialog {...defaultProps} />);

			expect(screen.getByLabelText("Name")).toBeInTheDocument();
			expect(screen.getByLabelText("URL")).toBeInTheDocument();
			expect(screen.getByLabelText("Description")).toBeInTheDocument();
			expect(screen.getByLabelText("Notes")).toBeInTheDocument();
		});

		it("should render form fields with initial values", () => {
			const formData = {
				name: "Test Product",
				url: "https://example.com",
				description: "A description",
				notes: "Some notes",
			};
			render(<ProductFormDialog {...defaultProps} formData={formData} />);

			expect(screen.getByLabelText("Name")).toHaveValue("Test Product");
			expect(screen.getByLabelText("URL")).toHaveValue("https://example.com");
			expect(screen.getByLabelText("Description")).toHaveValue("A description");
			expect(screen.getByLabelText("Notes")).toHaveValue("Some notes");
		});

		it("should render submit button with create label", () => {
			render(<ProductFormDialog {...defaultProps} />);

			expect(
				screen.getByRole("button", { name: "Create" }),
			).toBeInTheDocument();
		});

		it("should render submit button with save label for edit mode", () => {
			render(<ProductFormDialog {...defaultProps} mode="edit" />);

			expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
		});

		it("should render cancel button", () => {
			render(<ProductFormDialog {...defaultProps} />);

			expect(
				screen.getByRole("button", { name: "Cancel" }),
			).toBeInTheDocument();
		});
	});

	describe("form interactions", () => {
		it("should call onFormChange when name input changes", async () => {
			const onFormChange = vi.fn();
			const { user } = render(
				<ProductFormDialog {...defaultProps} onFormChange={onFormChange} />,
			);

			const nameInput = screen.getByLabelText("Name");
			await user.type(nameInput, "Test");

			expect(onFormChange).toHaveBeenCalled();
			expect(onFormChange).toHaveBeenCalledWith(
				expect.objectContaining({ name: "T" }),
			);
		});

		it("should call onFormChange when url input changes", async () => {
			const onFormChange = vi.fn();
			const { user } = render(
				<ProductFormDialog {...defaultProps} onFormChange={onFormChange} />,
			);

			const urlInput = await screen.findByLabelText("URL");
			await user.click(urlInput);
			await user.type(urlInput, "h");

			expect(onFormChange).toHaveBeenCalled();
			expect(onFormChange).toHaveBeenCalledWith(
				expect.objectContaining({ url: "h" }),
			);
		});

		it("should call onFormChange when description textarea changes", async () => {
			const onFormChange = vi.fn();
			const { user } = render(
				<ProductFormDialog {...defaultProps} onFormChange={onFormChange} />,
			);

			const descInput = await screen.findByLabelText("Description");
			await user.click(descInput);
			await user.type(descInput, "D");

			expect(onFormChange).toHaveBeenCalled();
			expect(onFormChange).toHaveBeenCalledWith(
				expect.objectContaining({ description: "D" }),
			);
		});

		it("should call onFormChange when notes textarea changes", async () => {
			const onFormChange = vi.fn();
			const { user } = render(
				<ProductFormDialog {...defaultProps} onFormChange={onFormChange} />,
			);

			const notesInput = await screen.findByTestId("product-notes-input");
			await user.click(notesInput);
			await user.type(notesInput, "N");

			expect(onFormChange).toHaveBeenCalled();
			expect(onFormChange).toHaveBeenCalledWith(
				expect.objectContaining({ notes: "N" }),
			);
		});

		it("should call onSubmit when submit button is clicked", async () => {
			const onSubmit = vi.fn();
			const { user } = render(
				<ProductFormDialog {...defaultProps} onSubmit={onSubmit} />,
			);

			const submitButton = screen.getByRole("button", { name: "Create" });
			await user.click(submitButton);

			expect(onSubmit).toHaveBeenCalledTimes(1);
		});

		it("should call onOpenChange(false) when Cancel is clicked", async () => {
			const onOpenChange = vi.fn();
			const { user } = render(
				<ProductFormDialog {...defaultProps} onOpenChange={onOpenChange} />,
			);

			const cancelButton = screen.getByRole("button", { name: "Cancel" });
			await user.click(cancelButton);

			expect(onOpenChange).toHaveBeenCalledWith(false);
		});
	});

	describe("submitting state", () => {
		it("should show submittingLabel when isSubmitting is true", () => {
			render(<ProductFormDialog {...defaultProps} isSubmitting={true} />);

			expect(
				screen.getByRole("button", { name: "Creating..." }),
			).toBeInTheDocument();
			expect(
				screen.queryByRole("button", { name: "Create" }),
			).not.toBeInTheDocument();
		});

		it("should disable submit button when isSubmitting is true", () => {
			render(<ProductFormDialog {...defaultProps} isSubmitting={true} />);

			const submitButton = screen.getByRole("button", {
				name: "Creating...",
			});
			expect(submitButton).toBeDisabled();
		});
	});

	describe("closed state", () => {
		it("should not render dialog content when open is false", () => {
			render(<ProductFormDialog {...defaultProps} open={false} />);

			expect(screen.queryByText("Add Product")).not.toBeInTheDocument();
		});
	});

	describe("mode-based id prefix", () => {
		it("should use mode as id prefix for form fields in edit mode", () => {
			render(<ProductFormDialog {...defaultProps} mode="edit" />);

			expect(screen.getByLabelText("Name")).toHaveAttribute("id", "edit-name");
			expect(screen.getByLabelText("URL")).toHaveAttribute("id", "edit-url");
			expect(screen.getByLabelText("Description")).toHaveAttribute(
				"id",
				"edit-description",
			);
			expect(screen.getByLabelText("Notes")).toHaveAttribute(
				"id",
				"edit-notes",
			);
		});
	});
});
