import { AlertTriangle } from "lucide-react";
import type { ReactNode } from "react";

interface ErrorStateProps {
	children?: ReactNode;
	title?: string;
	description?: string;
}

export function ErrorState({ children, title, description }: ErrorStateProps) {
	return (
		<div className="flex h-full flex-1 items-center justify-center px-8 py-4">
			<div className="flex flex-col items-center justify-center gap-y-6 rounded-lg bg-background p-10 text-foreground shadow-sm">
				<AlertTriangle className="text-destructive" size={48} />
				<div className="flex flex-col gap-y-2 text-center">
					{title && <h6 className="font-medium text-lg">{title}</h6>}
					{description && <p className="text-sm">{description}</p>}
				</div>
				{children}
			</div>
		</div>
	);
}
