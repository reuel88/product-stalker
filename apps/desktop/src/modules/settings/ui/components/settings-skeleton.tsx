import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

export function SettingsSkeleton() {
	return (
		<div className="container mx-auto max-w-2xl px-4 py-6">
			<Skeleton className="mb-6 h-7 w-24" />
			<div className="space-y-4">
				{[1, 2, 3, 4, 5].map((i) => (
					<Card key={i}>
						<CardHeader>
							<Skeleton className="h-5 w-32" />
							<Skeleton className="h-4 w-48" />
						</CardHeader>
						<CardContent>
							<div className="flex items-center justify-between">
								<Skeleton className="h-4 w-24" />
								<Skeleton className="h-5 w-9 rounded-full" />
							</div>
						</CardContent>
					</Card>
				))}
			</div>
		</div>
	);
}
