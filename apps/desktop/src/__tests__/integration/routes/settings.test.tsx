import { beforeEach, describe, expect, it, vi } from "vitest";
import { COMMANDS, MESSAGES } from "@/constants";
import { SettingsSkeleton } from "@/modules/settings/ui/components/settings-skeleton";
import { SettingsView } from "@/modules/settings/ui/views/settings-view";
import {
	createMockSettings,
	createMockUpdateAvailable,
	createMockUpdateInfo,
} from "../../mocks/data";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { render, screen, waitFor, within } from "../../test-utils";

/**
 * Helper to find a switch by its label text.
 * Finds the label, then gets the parent row container and returns the switch within it.
 */
function getSwitchByLabel(labelText: RegExp) {
	const label = screen.getByText(labelText);
	// The parent row contains both the label and the switch
	const row = label.closest(".flex.items-center.justify-between");
	if (!row)
		throw new Error(`Could not find row container for label: ${labelText}`);
	return within(row as HTMLElement).getByRole("switch");
}

/**
 * Helper to find a combobox by its label text.
 * Finds the label, then gets the parent row container and returns the combobox within it.
 */
function getComboboxByLabel(labelText: RegExp) {
	const label = screen.getByText(labelText);
	const row = label.closest(".flex.items-center.justify-between");
	if (!row)
		throw new Error(`Could not find row container for label: ${labelText}`);
	return within(row as HTMLElement).getByRole("combobox");
}

// Mock sonner toast
vi.mock("sonner", () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn(),
		info: vi.fn(),
	},
}));

// Mock next-themes
const mockSetTheme = vi.fn();
vi.mock("next-themes", () => ({
	useTheme: () => ({
		theme: "system",
		setTheme: mockSetTheme,
	}),
}));

// Mock the palette provider
const mockSetPalette = vi.fn();
vi.mock("@/modules/shared/providers/palette-provider", () => ({
	usePalette: () => ({
		paletteId: "default",
		setPalette: mockSetPalette,
		palettes: [
			{
				id: "default",
				name: "Default",
				description: "Clean neutral theme",
				preview: { primary: "#000", accent: "#eee", background: "#fff" },
			},
			{
				id: "ocean",
				name: "Ocean",
				description: "Cool blue tones",
				preview: { primary: "#005", accent: "#aad", background: "#eef" },
			},
			{
				id: "rose",
				name: "Rose",
				description: "Warm pink tones",
				preview: { primary: "#500", accent: "#daa", background: "#fee" },
			},
		],
	}),
}));

import { toast } from "sonner";

describe("SettingsComponent", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
		vi.clearAllMocks();
	});

	describe("loading state", () => {
		it("should show skeleton while loading", async () => {
			// Never resolve to keep loading state
			getMockedInvoke().mockImplementation(() => new Promise(() => {}));

			render(<SettingsView />);

			// SettingsComponent shows SettingsSkeleton when loading
			// The skeleton doesn't have the Settings title visible
			await waitFor(() => {
				expect(screen.queryByText("Settings")).not.toBeInTheDocument();
			});
		});
	});

	describe("error state", () => {
		it("should show error message when settings fail to load", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS) {
					return Promise.reject(new Error("Failed"));
				}
				if (cmd === COMMANDS.GET_CURRENT_VERSION) {
					return Promise.resolve("1.0.0");
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Failed to load settings")).toBeInTheDocument();
			});
		});
	});

	describe("settings display", () => {
		it("should render all setting sections", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Settings")).toBeInTheDocument();
			});

			expect(screen.getByText("Appearance")).toBeInTheDocument();
			expect(
				screen.getByText("System integration settings"),
			).toBeInTheDocument();
			expect(screen.getByText("Logging")).toBeInTheDocument();
			expect(screen.getByText("Notifications")).toBeInTheDocument();
			expect(screen.getByText("Interface")).toBeInTheDocument();
			expect(screen.getByText("Updates")).toBeInTheDocument();
		});

		it("should display current version", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.GET_CURRENT_VERSION]: "1.2.3",
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("v1.2.3")).toBeInTheDocument();
			});
		});
	});

	describe("theme switching", () => {
		it("should change theme when selected", async () => {
			const settings = createMockSettings({ theme: "system" });
			const updatedSettings = createMockSettings({ theme: "dark" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Settings")).toBeInTheDocument();
			});

			const themeSelect = getComboboxByLabel(/^Theme$/);

			// Use keyboard navigation for Radix UI Select compatibility
			await user.click(themeSelect);
			await user.keyboard("{ArrowUp}"); // Move to Dark option (from System, items are: Light, Dark, System)
			await user.keyboard("{Enter}");

			await waitFor(() => {
				expect(mockSetTheme).toHaveBeenCalledWith("dark");
			});

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith(MESSAGES.SETTINGS.SAVED);
			});
		});
	});

	describe("toggle switches", () => {
		it("should toggle show_in_tray setting", async () => {
			const settings = createMockSettings({ show_in_tray: false });
			const updatedSettings = createMockSettings({ show_in_tray: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Show in tray")).toBeInTheDocument();
			});

			const showInTraySwitch = getSwitchByLabel(/^Show in tray$/);
			await user.click(showInTraySwitch);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { show_in_tray: true } },
				);
			});
		});

		it("should toggle enable_notifications setting", async () => {
			const settings = createMockSettings({ enable_notifications: true });
			const updatedSettings = createMockSettings({
				enable_notifications: false,
			});
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Enable notifications")).toBeInTheDocument();
			});

			const notificationsSwitch = getSwitchByLabel(/^Enable notifications$/);
			await user.click(notificationsSwitch);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { enable_notifications: false } },
				);
			});
		});

		it("should toggle background_check_enabled setting", async () => {
			const settings = createMockSettings({ background_check_enabled: false });
			const updatedSettings = createMockSettings({
				background_check_enabled: true,
			});
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByText("Enable background checking"),
				).toBeInTheDocument();
			});

			const backgroundCheckSwitch = getSwitchByLabel(
				/^Enable background checking$/,
			);
			await user.click(backgroundCheckSwitch);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { background_check_enabled: true } },
				);
			});
		});

		it("should toggle launch_at_login setting", async () => {
			const settings = createMockSettings({ launch_at_login: false });
			const updatedSettings = createMockSettings({ launch_at_login: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Launch at login")).toBeInTheDocument();
			});

			const launchAtLoginSwitch = getSwitchByLabel(/^Launch at login$/);
			await user.click(launchAtLoginSwitch);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { launch_at_login: true } },
				);
			});
		});

		it("should toggle enable_logging setting", async () => {
			const settings = createMockSettings({ enable_logging: true });
			const updatedSettings = createMockSettings({ enable_logging: false });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Enable logging")).toBeInTheDocument();
			});

			const loggingSwitch = getSwitchByLabel(/^Enable logging$/);
			await user.click(loggingSwitch);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { enable_logging: false } },
				);
			});
		});

		it("should toggle sidebar_expanded setting", async () => {
			const settings = createMockSettings({ sidebar_expanded: true });
			const updatedSettings = createMockSettings({ sidebar_expanded: false });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Sidebar expanded")).toBeInTheDocument();
			});

			const sidebarSwitch = getSwitchByLabel(/^Sidebar expanded$/);
			await user.click(sidebarSwitch);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { sidebar_expanded: false } },
				);
			});
		});
	});

	describe("log level", () => {
		it("should change log level when selected", async () => {
			const settings = createMockSettings({
				log_level: "info",
				enable_logging: true,
			});
			const updatedSettings = createMockSettings({ log_level: "debug" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Log level")).toBeInTheDocument();
			});

			const logLevelSelect = getComboboxByLabel(/^Log level$/);

			// Use keyboard navigation for Radix UI Select compatibility
			await user.click(logLevelSelect);
			await user.keyboard("{ArrowDown}"); // Move to Debug option (from Info)
			await user.keyboard("{Enter}");

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { log_level: "debug" } },
				);
			});
		});
	});

	describe("update checking", () => {
		it("should check for updates when button clicked", async () => {
			const noUpdate = createMockUpdateInfo();
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: noUpdate,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(toast.info).toHaveBeenCalledWith(
					"You're running the latest version",
				);
			});
		});

		it("should show update available toast", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: updateAvailable,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith("Update available: v2.0.0");
			});
		});

		it("should show error toast when check fails", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.GET_CURRENT_VERSION)
					return Promise.resolve("1.0.0");
				if (cmd === COMMANDS.CHECK_FOR_UPDATE)
					return Promise.reject(new Error("Network error"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith("Failed to check for updates");
			});
		});
	});

	describe("update installation", () => {
		it("should show Update Now button when update available", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.GET_CURRENT_VERSION)
					return Promise.resolve("1.0.0");
				if (cmd === COMMANDS.CHECK_FOR_UPDATE)
					return Promise.resolve(updateAvailable);
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(screen.getByText("Update Now")).toBeInTheDocument();
			});

			expect(screen.getByText("v2.0.0")).toBeInTheDocument();
		});

		it("should install update when Update Now clicked", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.GET_CURRENT_VERSION)
					return Promise.resolve("1.0.0");
				if (cmd === COMMANDS.CHECK_FOR_UPDATE)
					return Promise.resolve(updateAvailable);
				if (cmd === COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE)
					return Promise.resolve(undefined);
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(screen.getByText("Update Now")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Update Now"));

			await waitFor(() => {
				expect(toast.info).toHaveBeenCalledWith("Downloading update...");
			});
		});

		it("should show error toast when install fails", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.GET_CURRENT_VERSION)
					return Promise.resolve("1.0.0");
				if (cmd === COMMANDS.CHECK_FOR_UPDATE)
					return Promise.resolve(updateAvailable);
				if (cmd === COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE)
					return Promise.reject(new Error("Download failed"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(screen.getByText("Update Now")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Update Now"));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith("Failed to install update");
			});
		});
	});
});

describe("SettingsSkeleton", () => {
	it("should render skeleton cards", () => {
		render(<SettingsSkeleton />);

		// The skeleton renders 5 cards using data-slot attribute
		const cards = document.querySelectorAll('[data-slot="card"]');
		expect(cards.length).toBe(5);
	});
});
