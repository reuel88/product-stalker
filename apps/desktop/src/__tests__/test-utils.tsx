import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { type RenderOptions, render } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";

function createTestQueryClient() {
	return new QueryClient({
		defaultOptions: {
			queries: {
				retry: false,
				gcTime: 0,
			},
			mutations: {
				retry: false,
			},
		},
	});
}

interface WrapperProps {
	children: ReactNode;
}

function createWrapper() {
	const queryClient = createTestQueryClient();
	return function Wrapper({ children }: WrapperProps) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		);
	};
}

function customRender(
	ui: ReactElement,
	options?: Omit<RenderOptions, "wrapper">,
) {
	return {
		...render(ui, { wrapper: createWrapper(), ...options }),
		// Disable pointer events check for Radix UI compatibility
		user: userEvent.setup({ pointerEventsCheck: 0 }),
	};
}

export * from "@testing-library/react";
export { customRender as render, createTestQueryClient };
