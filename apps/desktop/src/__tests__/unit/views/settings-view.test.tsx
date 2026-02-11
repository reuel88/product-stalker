import { beforeEach, describe, expect, it, vi } from "vitest";
import { COMMANDS } from "@/constants";
import { SettingsView } from "@/modules/settings/ui/views/settings-view";
import {
	createMockSettings,
	createMockUpdateAvailable,
} from "../../mocks/data";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { render, screen, waitFor } from "../../test-utils";

// Mock the theme provider
const mockSetTheme = vi.fn();
vi.mock("@/modules/shared/providers/theme-provider", () => ({
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

describe("SettingsView", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
		mockSetTheme.mockClear();
	});

	describe("loading state", () => {
		it("should render settings skeleton while loading", () => {
			// Never resolve the settings fetch to keep loading state
			getMockedInvoke().mockImplementation(() => new Promise(() => {}));

			const { container } = render(<SettingsView />);

			// Skeleton should have animated elements
			const skeletons = container.querySelectorAll('[class*="animate-pulse"]');
			expect(skeletons.length).toBeGreaterThan(0);
		});
	});

	describe("error state", () => {
		it("should render error state when settings fail to load", async () => {
			// Mock settings to return null (simulating undefined settings after load)
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: null,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Failed to load settings")).toBeInTheDocument();
			});
			expect(screen.getByText("Please try again later")).toBeInTheDocument();
		});
	});

	describe("rendering with settings", () => {
		beforeEach(() => {
			const mockSettings = createMockSettings();
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: mockSettings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: mockSettings,
			});
		});

		it("should render settings page title", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Settings")).toBeInTheDocument();
			});
		});

		it("should render Appearance card", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Appearance")).toBeInTheDocument();
			});
			expect(
				screen.getByText("Customize how the app looks"),
			).toBeInTheDocument();
		});

		it("should render System card", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByText("System integration settings"),
				).toBeInTheDocument();
			});
		});

		it("should render Logging card", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Logging")).toBeInTheDocument();
			});
			expect(
				screen.getByText("Configure application logging"),
			).toBeInTheDocument();
		});

		it("should render Notifications card", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Notifications")).toBeInTheDocument();
			});
			expect(
				screen.getByText("Configure notification preferences"),
			).toBeInTheDocument();
		});

		it("should render Background Checking card", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Background Checking")).toBeInTheDocument();
			});
			expect(
				screen.getByText(
					"Automatically check product availability periodically",
				),
			).toBeInTheDocument();
		});

		it("should render Advanced card", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Advanced")).toBeInTheDocument();
			});
			expect(
				screen.getByText(
					"Advanced settings for compatibility with protected sites",
				),
			).toBeInTheDocument();
		});

		it("should render Interface card", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Interface")).toBeInTheDocument();
			});
			expect(screen.getByText("UI preferences")).toBeInTheDocument();
		});

		it("should render Updates card", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Updates")).toBeInTheDocument();
			});
			expect(
				screen.getByText("Check for application updates"),
			).toBeInTheDocument();
		});

		it("should display current version", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("v1.0.0")).toBeInTheDocument();
			});
		});

		it("should render Check for Updates button", async () => {
			render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Check for Updates" }),
				).toBeInTheDocument();
			});
		});
	});

	describe("theme settings", () => {
		it("should render theme selector with current value", async () => {
			const settings = createMockSettings({ theme: "dark" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Theme")).toBeInTheDocument();
			});
		});

		it("should call setTheme and updateSettings when theme changes", async () => {
			const settings = createMockSettings({ theme: "light" });
			const updatedSettings = createMockSettings({ theme: "dark" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Theme")).toBeInTheDocument();
			});

			// Click on the first combobox (theme select)
			const comboboxes = screen.getAllByRole("combobox");
			const themeSelect = comboboxes[0];

			// Use keyboard navigation for Radix UI Select compatibility
			await user.click(themeSelect);
			await user.keyboard("{ArrowDown}"); // Move to Dark option (from Light)
			await user.keyboard("{Enter}");

			await waitFor(() => {
				expect(mockSetTheme).toHaveBeenCalledWith("dark");
			});
		});
	});

	describe("system settings", () => {
		it("should render show in tray label", async () => {
			const settings = createMockSettings({ show_in_tray: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Show in tray")).toBeInTheDocument();
			});

			// Verify switch element exists
			const switchEl = document.getElementById("show-in-tray");
			expect(switchEl).toBeInTheDocument();
		});

		it("should render launch at login switch", async () => {
			const settings = createMockSettings({ launch_at_login: false });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Launch at login")).toBeInTheDocument();
			});

			const switchEl = document.getElementById("launch-at-login");
			expect(switchEl).toBeInTheDocument();
		});

		it("should call updateSettings when show in tray is toggled", async () => {
			const settings = createMockSettings({ show_in_tray: true });
			const updatedSettings = createMockSettings({ show_in_tray: false });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Show in tray")).toBeInTheDocument();
			});

			const switchEl = document.getElementById("show-in-tray");
			expect(switchEl).toBeInTheDocument();
			if (switchEl) {
				await user.click(switchEl);
			}

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					expect.objectContaining({
						input: expect.objectContaining({ show_in_tray: false }),
					}),
				);
			});
		});
	});

	describe("logging settings", () => {
		it("should render enable logging switch", async () => {
			const settings = createMockSettings({ enable_logging: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Enable logging")).toBeInTheDocument();
			});

			const switchEl = document.getElementById("enable-logging");
			expect(switchEl).toBeInTheDocument();
		});

		it("should disable log level select when logging is disabled", async () => {
			const settings = createMockSettings({ enable_logging: false });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				const logLevelTriggers = screen.getAllByRole("combobox");
				// Log level select should be the second combobox (after theme)
				const logLevelSelect = logLevelTriggers[1];
				expect(logLevelSelect).toBeDisabled();
			});
		});
	});

	describe("notification settings", () => {
		it("should render enable notifications switch", async () => {
			const settings = createMockSettings({ enable_notifications: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Enable notifications")).toBeInTheDocument();
			});

			const switchEl = document.getElementById("enable-notifications");
			expect(switchEl).toBeInTheDocument();
		});
	});

	describe("background checking settings", () => {
		it("should render background checking switch", async () => {
			const settings = createMockSettings({ background_check_enabled: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByText("Enable background checking"),
				).toBeInTheDocument();
			});

			const switchEl = document.getElementById("background-check");
			expect(switchEl).toBeInTheDocument();
		});

		it("should disable check interval select when background checking is disabled", async () => {
			const settings = createMockSettings({ background_check_enabled: false });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				const selects = screen.getAllByRole("combobox");
				// Check interval is the third combobox (theme, log level, interval)
				const intervalSelect = selects[2];
				expect(intervalSelect).toBeDisabled();
			});
		});
	});

	describe("advanced settings", () => {
		it("should render headless browser switch", async () => {
			const settings = createMockSettings({ enable_headless_browser: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Headless browser")).toBeInTheDocument();
			});

			const switchEl = document.getElementById("headless-browser");
			expect(switchEl).toBeInTheDocument();
		});

		it("should show description for headless browser setting", async () => {
			const settings = createMockSettings();
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByText(
						"Use Chrome to check sites with bot protection (Cloudflare)",
					),
				).toBeInTheDocument();
			});
		});
	});

	describe("interface settings", () => {
		it("should render sidebar expanded switch", async () => {
			const settings = createMockSettings({ sidebar_expanded: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			render(<SettingsView />);

			await waitFor(() => {
				expect(screen.getByText("Sidebar expanded")).toBeInTheDocument();
			});

			const switchEl = document.getElementById("sidebar-expanded");
			expect(switchEl).toBeInTheDocument();
		});
	});

	describe("update functionality", () => {
		it("should show available version when update exists", async () => {
			const settings = createMockSettings();
			const updateInfo = createMockUpdateAvailable("2.0.0");
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: updateInfo,
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Check for Updates" }),
				).toBeInTheDocument();
			});

			// Click check for updates
			await user.click(
				screen.getByRole("button", { name: "Check for Updates" }),
			);

			await waitFor(() => {
				expect(screen.getByText("v2.0.0")).toBeInTheDocument();
			});
		});

		it("should show Update Now button when update is available", async () => {
			const settings = createMockSettings();
			const updateInfo = createMockUpdateAvailable("2.0.0");
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: updateInfo,
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Check for Updates" }),
				).toBeInTheDocument();
			});

			await user.click(
				screen.getByRole("button", { name: "Check for Updates" }),
			);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Update Now" }),
				).toBeInTheDocument();
			});
		});

		it("should call install update when Update Now is clicked", async () => {
			const settings = createMockSettings();
			const updateInfo = createMockUpdateAvailable("2.0.0");
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: updateInfo,
				[COMMANDS.UPDATE_SETTINGS]: settings,
				[COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE]: undefined,
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Check for Updates" }),
				).toBeInTheDocument();
			});

			await user.click(
				screen.getByRole("button", { name: "Check for Updates" }),
			);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Update Now" }),
				).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: "Update Now" }));

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE,
				);
			});
		});

		it("should show Checking... while checking for updates", async () => {
			const settings = createMockSettings();
			// Never resolve to keep checking state
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS) {
					return Promise.resolve(settings);
				}
				if (cmd === COMMANDS.GET_CURRENT_VERSION) {
					return Promise.resolve("1.0.0");
				}
				if (cmd === COMMANDS.CHECK_FOR_UPDATE) {
					return new Promise(() => {}); // Never resolves
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Check for Updates" }),
				).toBeInTheDocument();
			});

			await user.click(
				screen.getByRole("button", { name: "Check for Updates" }),
			);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Checking..." }),
				).toBeInTheDocument();
			});
		});
	});
});
