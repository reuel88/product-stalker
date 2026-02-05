import { ErrorState } from "./error-state";

type FullPageErrorProps = {
	title: string;
	description: string;
};

export function FullPageError({ title, description }: FullPageErrorProps) {
	return (
		<div className="flex h-screen w-full flex-col items-center justify-center">
			<ErrorState title={title} description={description} />
		</div>
	);
}
