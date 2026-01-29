import { Link } from "@tanstack/react-router";
import { FlaskConical, Package, Settings } from "lucide-react";

import { ModeToggle } from "./mode-toggle";

export default function Header() {
	const links = [
		{ to: "/", label: "Home" },
		{ to: "/products", label: <Package className="size-4" /> },
		{ to: "/test-settings", label: <FlaskConical className="size-4" /> },
		{ to: "/settings", label: <Settings className="size-4" /> },
	] as const;

	return (
		<div>
			<div className="flex flex-row items-center justify-between px-2 py-1">
				<nav className="flex gap-4 text-lg">
					{links.map(({ to, label }) => {
						return (
							<Link key={to} to={to}>
								{label}
							</Link>
						);
					})}
				</nav>
				<div className="flex items-center gap-2">
					<ModeToggle />
				</div>
			</div>
			<hr />
		</div>
	);
}
