import { Plus, RefreshCw } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { MESSAGES } from "@/constants";
import { useCheckAllAvailability } from "@/modules/products/hooks/useAvailability";
import {
	type CreateProductInput,
	useProducts,
} from "@/modules/products/hooks/useProducts";
import type { ProductResponse } from "@/modules/products/types";
import { ProductFormDialog } from "@/modules/products/ui/components/product-form-dialog";
import { ProductsTable } from "@/modules/products/ui/components/products-table";
import { ErrorState } from "@/modules/shared/ui/components/error-state";

type DialogState =
	| { type: "closed" }
	| { type: "create"; formData: CreateProductInput }
	| { type: "edit"; product: ProductResponse; formData: CreateProductInput }
	| { type: "delete"; product: ProductResponse };

const initialFormData: CreateProductInput = {
	name: "",
	url: "",
	description: "",
	notes: "",
};

export function ProductsView() {
	const {
		products,
		isLoading,
		error,
		createProduct,
		isCreating,
		updateProduct,
		isUpdating,
		deleteProduct,
		isDeleting,
	} = useProducts();

	const { checkAllAvailability, isCheckingAll } = useCheckAllAvailability();

	const [dialogState, setDialogState] = useState<DialogState>({
		type: "closed",
	});

	const closeDialog = () => setDialogState({ type: "closed" });

	const updateFormData = (formData: CreateProductInput) => {
		if (dialogState.type === "create") {
			setDialogState({ type: "create", formData });
		} else if (dialogState.type === "edit") {
			setDialogState({ ...dialogState, formData });
		}
	};

	const handleCreate = async () => {
		if (dialogState.type !== "create") return;
		const { formData } = dialogState;

		if (!formData.name || !formData.url) {
			toast.error(MESSAGES.VALIDATION.NAME_URL_REQUIRED);
			return;
		}

		try {
			await createProduct({
				name: formData.name,
				url: formData.url,
				description: formData.description || null,
				notes: formData.notes || null,
			});
			toast.success(MESSAGES.PRODUCT.CREATED);
			closeDialog();
		} catch {
			toast.error(MESSAGES.PRODUCT.CREATE_FAILED);
		}
	};

	const handleEdit = (product: ProductResponse) => {
		setDialogState({
			type: "edit",
			product,
			formData: {
				name: product.name,
				url: product.url,
				description: product.description || "",
				notes: product.notes || "",
			},
		});
	};

	const handleUpdate = async () => {
		if (dialogState.type !== "edit") return;
		const { product, formData } = dialogState;

		if (!formData.name || !formData.url) {
			toast.error(MESSAGES.VALIDATION.NAME_URL_REQUIRED);
			return;
		}

		try {
			await updateProduct({
				id: product.id,
				input: {
					name: formData.name,
					url: formData.url,
					description: formData.description || null,
					notes: formData.notes || null,
				},
			});
			toast.success(MESSAGES.PRODUCT.UPDATED);
			closeDialog();
		} catch {
			toast.error(MESSAGES.PRODUCT.UPDATE_FAILED);
		}
	};

	const handleDeleteClick = (product: ProductResponse) => {
		setDialogState({ type: "delete", product });
	};

	const handleDelete = async () => {
		if (dialogState.type !== "delete") return;

		try {
			await deleteProduct(dialogState.product.id);
			toast.success(MESSAGES.PRODUCT.DELETED);
			closeDialog();
		} catch {
			toast.error(MESSAGES.PRODUCT.DELETE_FAILED);
		}
	};

	const handleCheckAll = async () => {
		try {
			const summary = await checkAllAvailability();
			if (summary.back_in_stock_count > 0) {
				toast.success(
					`${MESSAGES.AVAILABILITY.CHECK_ALL_COMPLETE} - ${summary.back_in_stock_count} product(s) back in stock!`,
				);
			} else {
				toast.success(
					`${MESSAGES.AVAILABILITY.CHECK_ALL_COMPLETE} (${summary.successful}/${summary.total} successful)`,
				);
			}
		} catch {
			toast.error(MESSAGES.AVAILABILITY.CHECK_ALL_FAILED);
		}
	};

	if (error) {
		return (
			<div className="flex h-screen w-full flex-col items-center justify-center">
				<ErrorState
					title="Failed to load products"
					description="Please try again later"
				/>
			</div>
		);
	}

	return (
		<div className="container mx-auto max-w-4xl overflow-y-auto px-4 py-6">
			<div className="mb-6 flex items-center justify-between">
				<h1 className="font-semibold text-xl">Products</h1>
				<div className="flex gap-2">
					<Button
						variant="outline"
						size="sm"
						onClick={handleCheckAll}
						disabled={isCheckingAll || !products?.length}
					>
						<RefreshCw
							className={`size-4 ${isCheckingAll ? "animate-spin" : ""}`}
						/>
						{isCheckingAll ? "Checking..." : "Check All"}
					</Button>
					<Button
						size="sm"
						onClick={() =>
							setDialogState({ type: "create", formData: initialFormData })
						}
					>
						<Plus className="size-4" />
						Add Product
					</Button>
				</div>
			</div>

			<Card>
				<CardHeader>
					<CardTitle>All Products</CardTitle>
					<CardDescription>
						View and manage your tracked products
					</CardDescription>
				</CardHeader>
				<CardContent>
					<ProductsTable
						products={products ?? []}
						isLoading={isLoading}
						onEdit={handleEdit}
						onDelete={handleDeleteClick}
					/>
				</CardContent>
			</Card>

			{/* Create Dialog */}
			<ProductFormDialog
				open={dialogState.type === "create"}
				onOpenChange={(open) => !open && closeDialog()}
				title="Add Product"
				description="Add a new product to track"
				formData={
					dialogState.type === "create" ? dialogState.formData : initialFormData
				}
				onFormChange={updateFormData}
				onSubmit={handleCreate}
				isSubmitting={isCreating}
				submitLabel="Create"
				submittingLabel="Creating..."
				idPrefix="create"
			/>

			{/* Edit Dialog */}
			<ProductFormDialog
				open={dialogState.type === "edit"}
				onOpenChange={(open) => !open && closeDialog()}
				title="Edit Product"
				description="Update product details"
				formData={
					dialogState.type === "edit" ? dialogState.formData : initialFormData
				}
				onFormChange={updateFormData}
				onSubmit={handleUpdate}
				isSubmitting={isUpdating}
				submitLabel="Save"
				submittingLabel="Saving..."
				idPrefix="edit"
			/>

			{/* Delete Confirmation Dialog */}
			<Dialog
				open={dialogState.type === "delete"}
				onOpenChange={(open) => !open && closeDialog()}
			>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Delete Product</DialogTitle>
						<DialogDescription>
							Are you sure you want to delete "
							{dialogState.type === "delete" ? dialogState.product.name : ""}
							"? This action cannot be undone.
						</DialogDescription>
					</DialogHeader>
					<DialogFooter>
						<Button variant="outline" onClick={closeDialog}>
							Cancel
						</Button>
						<Button
							variant="destructive"
							onClick={handleDelete}
							disabled={isDeleting}
						>
							{isDeleting ? "Deleting..." : "Delete"}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</div>
	);
}
