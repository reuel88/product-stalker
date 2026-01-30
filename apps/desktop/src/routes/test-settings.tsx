import { createFileRoute } from "@tanstack/react-router";
import { TestSettingsView } from "@/modules/settings/ui/views/test-settings-view";

export const Route = createFileRoute("/test-settings")({
	component: TestSettingsView,
});
