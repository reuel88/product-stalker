import { describe, expect, it, vi } from "vitest";
import { ProductFormDialog } from "@/modules/products/ui/components/product-form-dialog";
import { render, screen } from "../../test-utils";

const defaultProps = {
	open: true,
	onOpenChange: vi.fn(),
	title: "Add Product",
	description: "Enter product details below",
	formData: {
		name: "",
		url: "",
		description: null,
		notes: null,
	},
	onFormChange: vi.fn(),
	onSubmit: vi.fn(),
	isSubmitting: false,
	submitLabel: "Save",
	submittingLabel: "Saving...",
	idPrefix: "add-product",
};

describe("ProductFormDialog", () => {
	describe("rendering", () => {
		it("should render dialog with title", () => {
			render(<ProductFormDialog {...defaultProps} />);

			expect(screen.getByText("Add Product")).toBeInTheDocument();
		});

		it("should render dialog with description", () => {
			render(<ProductFormDialog {...defaultProps} />);

			expect(
				screen.getByText("Enter product details below"),
			).toBeInTheDocument();
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

		it("should render submit button with label", () => {
			render(<ProductFormDialog {...defaultProps} />);

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

			const urlInput = screen.getByLabelText("URL");
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

			const descInput = screen.getByLabelText("Description");
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

			const notesInput = screen.getByLabelText("Notes");
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

			const submitButton = screen.getByRole("button", { name: "Save" });
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
				screen.getByRole("button", { name: "Saving..." }),
			).toBeInTheDocument();
			expect(
				screen.queryByRole("button", { name: "Save" }),
			).not.toBeInTheDocument();
		});

		it("should disable submit button when isSubmitting is true", () => {
			render(<ProductFormDialog {...defaultProps} isSubmitting={true} />);

			const submitButton = screen.getByRole("button", { name: "Saving..." });
			expect(submitButton).toBeDisabled();
		});
	});

	describe("closed state", () => {
		it("should not render dialog content when open is false", () => {
			render(<ProductFormDialog {...defaultProps} open={false} />);

			expect(screen.queryByText("Add Product")).not.toBeInTheDocument();
		});
	});

	describe("custom id prefix", () => {
		it("should use idPrefix for form field ids", () => {
			render(<ProductFormDialog {...defaultProps} idPrefix="edit-product" />);

			expect(screen.getByLabelText("Name")).toHaveAttribute(
				"id",
				"edit-product-name",
			);
			expect(screen.getByLabelText("URL")).toHaveAttribute(
				"id",
				"edit-product-url",
			);
			expect(screen.getByLabelText("Description")).toHaveAttribute(
				"id",
				"edit-product-description",
			);
			expect(screen.getByLabelText("Notes")).toHaveAttribute(
				"id",
				"edit-product-notes",
			);
		});
	});
});
