import { createFileRoute } from "@tanstack/react-router";
import { ProductsComponent } from "@/modules/products/ui/views/products";

export const Route = createFileRoute("/products")({
	component: ProductsComponent,
});
