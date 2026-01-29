import { createFileRoute } from "@tanstack/react-router";
import { HomeComponent } from "@/modules/home/ui/views/home";

export const Route = createFileRoute("/")({
	component: HomeComponent,
});
