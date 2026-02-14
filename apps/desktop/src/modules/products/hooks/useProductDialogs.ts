import { useRef, useState } from "react";

import type { CreateProductInput } from "@/modules/products/hooks/useProducts";
import type { ProductResponse } from "@/modules/products/types";

export interface RetailerEntry {
	id: number;
	url: string;
	label: string;
}

/**
 * Discriminated union representing the current dialog state.
 *
 * - `closed`: No dialog is open
 * - `create`: Create product dialog is open with form data
 * - `edit`: Edit product dialog is open with the product being edited and form data
 * - `delete`: Delete confirmation dialog is open with the product to delete
 */
export type DialogState =
	| { type: "closed" }
	| {
			type: "create";
			formData: CreateProductInput;
			retailerEntries: RetailerEntry[];
	  }
	| { type: "edit"; product: ProductResponse; formData: CreateProductInput }
	| { type: "delete"; product: ProductResponse };

const initialFormData: CreateProductInput = {
	name: "",
	description: "",
	notes: "",
};

/**
 * Custom hook for managing product dialog state.
 *
 * Encapsulates all dialog-related state and transitions for create, edit,
 * and delete product dialogs. This keeps the main view component focused
 * on rendering and business logic.
 *
 * @returns Object containing:
 *   - dialogState: Current state of the dialog (closed, create, edit, or delete)
 *   - openCreateDialog: Opens the create product dialog with empty form
 *   - openEditDialog: Opens the edit dialog pre-filled with product data
 *   - openDeleteDialog: Opens the delete confirmation dialog
 *   - closeDialog: Closes any open dialog
 *   - updateFormData: Updates form data for create/edit dialogs
 *   - initialFormData: Default empty form data for external use
 */
export function useProductDialogs() {
	const [dialogState, setDialogState] = useState<DialogState>({
		type: "closed",
	});
	const nextEntryId = useRef(0);

	const openCreateDialog = () => {
		setDialogState({
			type: "create",
			formData: initialFormData,
			retailerEntries: [],
		});
	};

	const openEditDialog = (product: ProductResponse) => {
		setDialogState({
			type: "edit",
			product,
			formData: {
				name: product.name,
				description: product.description || "",
				notes: product.notes || "",
			},
		});
	};

	const openDeleteDialog = (product: ProductResponse) => {
		setDialogState({ type: "delete", product });
	};

	const closeDialog = () => {
		setDialogState({ type: "closed" });
	};

	const updateFormData = (formData: CreateProductInput) => {
		setDialogState((prev) => {
			if (prev.type === "create" || prev.type === "edit") {
				return { ...prev, formData };
			}
			return prev;
		});
	};

	const addRetailerEntry = () => {
		const id = nextEntryId.current++;
		setDialogState((prev) => {
			if (prev.type !== "create") return prev;
			return {
				...prev,
				retailerEntries: [...prev.retailerEntries, { id, url: "", label: "" }],
			};
		});
	};

	const updateRetailerEntry = (index: number, entry: RetailerEntry) => {
		setDialogState((prev) => {
			if (prev.type !== "create") return prev;
			const updated = [...prev.retailerEntries];
			updated[index] = entry;
			return { ...prev, retailerEntries: updated };
		});
	};

	const removeRetailerEntry = (index: number) => {
		setDialogState((prev) => {
			if (prev.type !== "create") return prev;
			return {
				...prev,
				retailerEntries: prev.retailerEntries.filter((_, i) => i !== index),
			};
		});
	};

	return {
		dialogState,
		openCreateDialog,
		openEditDialog,
		openDeleteDialog,
		closeDialog,
		updateFormData,
		addRetailerEntry,
		updateRetailerEntry,
		removeRetailerEntry,
		initialFormData,
	};
}
