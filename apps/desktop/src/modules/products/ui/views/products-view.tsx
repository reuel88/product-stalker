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

	const [createDialogOpen, setCreateDialogOpen] = useState(false);
	const [editDialogOpen, setEditDialogOpen] = useState(false);
	const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
	const [selectedProduct, setSelectedProduct] =
		useState<ProductResponse | null>(null);

	const [formData, setFormData] = useState<CreateProductInput>({
		name: "",
		url: "",
		description: "",
		notes: "",
	});

	const resetForm = () => {
		setFormData({ name: "", url: "", description: "", notes: "" });
	};

	const handleCreate = async () => {
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
			setCreateDialogOpen(false);
			resetForm();
		} catch {
			toast.error(MESSAGES.PRODUCT.CREATE_FAILED);
		}
	};

	const handleEdit = (product: ProductResponse) => {
		setSelectedProduct(product);
		setFormData({
			name: product.name,
			url: product.url,
			description: product.description || "",
			notes: product.notes || "",
		});
		setEditDialogOpen(true);
	};

	const handleUpdate = async () => {
		if (!selectedProduct) return;
		if (!formData.name || !formData.url) {
			toast.error(MESSAGES.VALIDATION.NAME_URL_REQUIRED);
			return;
		}

		try {
			await updateProduct({
				id: selectedProduct.id,
				input: {
					name: formData.name,
					url: formData.url,
					description: formData.description || null,
					notes: formData.notes || null,
				},
			});
			toast.success(MESSAGES.PRODUCT.UPDATED);
			setEditDialogOpen(false);
			setSelectedProduct(null);
			resetForm();
		} catch {
			toast.error(MESSAGES.PRODUCT.UPDATE_FAILED);
		}
	};

	const handleDeleteClick = (product: ProductResponse) => {
		setSelectedProduct(product);
		setDeleteDialogOpen(true);
	};

	const handleDelete = async () => {
		if (!selectedProduct) return;

		try {
			await deleteProduct(selectedProduct.id);
			toast.success(MESSAGES.PRODUCT.DELETED);
			setDeleteDialogOpen(false);
			setSelectedProduct(null);
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
					<Button size="sm" onClick={() => setCreateDialogOpen(true)}>
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
				open={createDialogOpen}
				onOpenChange={setCreateDialogOpen}
				title="Add Product"
				description="Add a new product to track"
				formData={formData}
				onFormChange={setFormData}
				onSubmit={handleCreate}
				isSubmitting={isCreating}
				submitLabel="Create"
				submittingLabel="Creating..."
				idPrefix="create"
			/>

			{/* Edit Dialog */}
			<ProductFormDialog
				open={editDialogOpen}
				onOpenChange={setEditDialogOpen}
				title="Edit Product"
				description="Update product details"
				formData={formData}
				onFormChange={setFormData}
				onSubmit={handleUpdate}
				isSubmitting={isUpdating}
				submitLabel="Save"
				submittingLabel="Saving..."
				idPrefix="edit"
			/>

			{/* Delete Confirmation Dialog */}
			<Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Delete Product</DialogTitle>
						<DialogDescription>
							Are you sure you want to delete "{selectedProduct?.name}"? This
							action cannot be undone.
						</DialogDescription>
					</DialogHeader>
					<DialogFooter>
						<Button
							variant="outline"
							onClick={() => setDeleteDialogOpen(false)}
						>
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
