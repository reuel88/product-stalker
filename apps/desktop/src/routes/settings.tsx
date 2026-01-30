import { createFileRoute } from "@tanstack/react-router";
import { SettingsView } from "@/modules/settings/ui/views/settings-view";

export const Route = createFileRoute("/settings")({
	component: SettingsView,
});
