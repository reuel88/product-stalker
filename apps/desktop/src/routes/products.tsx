import { createFileRoute } from "@tanstack/react-router";
import { ProductsView } from "@/modules/products/ui/views/products-view";

export const Route = createFileRoute("/products")({
	component: ProductsView,
});
