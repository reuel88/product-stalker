import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface AddRetailerDialogProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	onSubmit: (url: string, label: string | null) => void;
	isSubmitting: boolean;
}

export function AddRetailerDialog({
	open,
	onOpenChange,
	onSubmit,
	isSubmitting,
}: AddRetailerDialogProps) {
	const [url, setUrl] = useState("");
	const [label, setLabel] = useState("");

	const handleSubmit = () => {
		onSubmit(url, label || null);
		setUrl("");
		setLabel("");
	};

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Add Retailer</DialogTitle>
					<DialogDescription>
						Add a retailer URL to track this product's price and availability
					</DialogDescription>
				</DialogHeader>
				<div className="grid gap-4 py-4">
					<div className="grid gap-2">
						<Label htmlFor="retailer-url">URL</Label>
						<Input
							id="retailer-url"
							data-testid="retailer-url-input"
							value={url}
							onChange={(e) => setUrl(e.target.value)}
							placeholder="https://amazon.com/dp/B123..."
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor="retailer-label">Label (optional)</Label>
						<Input
							id="retailer-label"
							data-testid="retailer-label-input"
							value={label}
							onChange={(e) => setLabel(e.target.value)}
							placeholder="e.g., 64GB version"
						/>
					</div>
				</div>
				<DialogFooter>
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Cancel
					</Button>
					<Button onClick={handleSubmit} disabled={isSubmitting || !url}>
						{isSubmitting ? "Adding..." : "Add Retailer"}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
