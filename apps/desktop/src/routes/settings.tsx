import { createFileRoute } from "@tanstack/react-router";
import { SettingsComponent } from "@/modules/settings/ui/views/settings";

export const Route = createFileRoute("/settings")({
	component: SettingsComponent,
});
