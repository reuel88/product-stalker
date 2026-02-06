import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { TimeRange } from "@/modules/products/types";

interface TimeRangeSelectorProps {
	value: TimeRange;
	onChange: (range: TimeRange) => void;
}

const options: { value: TimeRange; label: string }[] = [
	{ value: "7d", label: "7 Days" },
	{ value: "30d", label: "30 Days" },
	{ value: "all", label: "All Time" },
];

export function TimeRangeSelector({ value, onChange }: TimeRangeSelectorProps) {
	return (
		<fieldset
			className="flex gap-1 border-none p-0"
			aria-label="Time range filter"
		>
			{options.map((option) => (
				<Button
					key={option.value}
					variant={value === option.value ? "secondary" : "ghost"}
					size="xs"
					onClick={() => onChange(option.value)}
					className={cn(
						"min-w-[60px]",
						value === option.value && "font-semibold",
					)}
					aria-pressed={value === option.value}
				>
					{option.label}
				</Button>
			))}
		</fieldset>
	);
}
