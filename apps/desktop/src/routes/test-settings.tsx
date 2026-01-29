import { createFileRoute } from "@tanstack/react-router";
import { TestSettingsComponent } from "@/modules/settings/ui/views/test-settings";

export const Route = createFileRoute("/test-settings")({
	component: TestSettingsComponent,
});
