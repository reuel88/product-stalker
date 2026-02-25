import { Dialog as DialogPrimitive } from "@base-ui/react/dialog";
import { X } from "lucide-react";
import type * as React from "react";
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
} from "react";

import { cn } from "@/lib/utils";
import { useDialogDrag } from "./use-dialog-drag";
import { useDialogResize } from "./use-dialog-resize";

interface DialogInteractionContextValue {
	handlePointerDown: (e: React.PointerEvent) => void;
	isDragging: boolean;
	isResizing: boolean;
}

const DialogInteractionContext =
	createContext<DialogInteractionContextValue | null>(null);

function Dialog({ ...props }: DialogPrimitive.Root.Props) {
	return <DialogPrimitive.Root data-slot="dialog" {...props} />;
}

function DialogTrigger({ ...props }: DialogPrimitive.Trigger.Props) {
	return <DialogPrimitive.Trigger data-slot="dialog-trigger" {...props} />;
}

function DialogPortal({ ...props }: DialogPrimitive.Portal.Props) {
	return <DialogPrimitive.Portal data-slot="dialog-portal" {...props} />;
}

function DialogClose({ ...props }: DialogPrimitive.Close.Props) {
	return <DialogPrimitive.Close data-slot="dialog-close" {...props} />;
}

function DialogBackdrop({
	className,
	...props
}: DialogPrimitive.Backdrop.Props) {
	return (
		<DialogPrimitive.Backdrop
			data-slot="dialog-backdrop"
			className={cn(
				"data-closed:fade-out-0 data-open:fade-in-0 fixed inset-0 z-50 bg-black/50 data-closed:animate-out data-open:animate-in",
				className,
			)}
			{...props}
		/>
	);
}

function DialogContent({
	className,
	children,
	...props
}: DialogPrimitive.Popup.Props) {
	const drag = useDialogDrag();
	const resize = useDialogResize();

	const isInteracting = drag.isDragging || resize.isResizing;
	const totalX = drag.offset.x + resize.resizeOffset.x;
	const totalY = drag.offset.y + resize.resizeOffset.y;
	const hasCustomSize = resize.size.width !== null;

	const popupElRef = useRef<HTMLElement | null>(null);

	const setPopupRef = useCallback(
		(el: HTMLElement | null) => {
			popupElRef.current = el;
			resize.popupRef.current = el;
		},
		[resize.popupRef],
	);

	// Reset position/size when dialog closes
	useEffect(() => {
		const el = popupElRef.current;
		if (!el) return;

		const observer = new MutationObserver(() => {
			if (el.hasAttribute("data-closed")) {
				drag.reset();
				resize.reset();
			}
		});

		observer.observe(el, {
			attributes: true,
			attributeFilter: ["data-closed"],
		});
		return () => observer.disconnect();
	}, [drag.reset, resize.reset]);

	return (
		<DialogInteractionContext.Provider
			value={{
				handlePointerDown: drag.handlePointerDown,
				isDragging: drag.isDragging,
				isResizing: resize.isResizing,
			}}
		>
			<DialogPortal>
				<DialogBackdrop />
				<DialogPrimitive.Popup
					ref={setPopupRef}
					data-slot="dialog-content"
					className={cn(
						"data-closed:fade-out-0 data-open:fade-in-0 data-closed:zoom-out-95 data-open:zoom-in-95 fixed top-1/2 left-1/2 z-50 grid gap-4 rounded-none border bg-background p-6 shadow-lg duration-200 data-closed:animate-out data-open:animate-in",
						!hasCustomSize && "w-full max-w-lg",
						hasCustomSize && "overflow-auto",
						className,
					)}
					style={{
						translate: `calc(-50% + ${totalX}px) calc(-50% + ${totalY}px)`,
						...(hasCustomSize && {
							width: resize.size.width ?? undefined,
							height: resize.size.height ?? undefined,
						}),
						...(isInteracting && {
							userSelect: "none",
							transitionProperty: "none",
						}),
					}}
					{...props}
				>
					{children}
					<DialogPrimitive.Close className="absolute top-4 right-4 rounded-none opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-hidden focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground [&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0">
						<X />
						<span className="sr-only">Close</span>
					</DialogPrimitive.Close>
					{resize.directions.map((dir) => (
						<div key={dir} {...resize.getHandleProps(dir)} />
					))}
				</DialogPrimitive.Popup>
			</DialogPortal>
		</DialogInteractionContext.Provider>
	);
}

function DialogHeader({ className, ...props }: React.ComponentProps<"div">) {
	const interaction = useContext(DialogInteractionContext);

	return (
		<div
			data-slot="dialog-header"
			className={cn(
				"flex flex-col gap-1.5 text-center sm:text-left",
				interaction &&
					(interaction.isDragging ? "cursor-grabbing" : "cursor-grab"),
				className,
			)}
			onPointerDown={interaction?.handlePointerDown}
			{...props}
		/>
	);
}

function DialogFooter({ className, ...props }: React.ComponentProps<"div">) {
	return (
		<div
			data-slot="dialog-footer"
			className={cn(
				"flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
				className,
			)}
			{...props}
		/>
	);
}

function DialogTitle({ className, ...props }: DialogPrimitive.Title.Props) {
	return (
		<DialogPrimitive.Title
			data-slot="dialog-title"
			className={cn(
				"font-semibold text-lg leading-none tracking-tight",
				className,
			)}
			{...props}
		/>
	);
}

function DialogDescription({
	className,
	...props
}: DialogPrimitive.Description.Props) {
	return (
		<DialogPrimitive.Description
			data-slot="dialog-description"
			className={cn("text-muted-foreground text-sm", className)}
			{...props}
		/>
	);
}

export {
	Dialog,
	DialogPortal,
	DialogBackdrop,
	DialogClose,
	DialogTrigger,
	DialogContent,
	DialogHeader,
	DialogFooter,
	DialogTitle,
	DialogDescription,
};
