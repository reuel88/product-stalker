import { createFileRoute } from "@tanstack/react-router";
import { HomeView } from "@/modules/home/ui/views/home-view";

export const Route = createFileRoute("/")({
	component: HomeView,
});
