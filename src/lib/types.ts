import type { ComponentProps } from "svelte";
import type { HTMLAttributes, HTMLAnchorAttributes, HTMLButtonAttributes, HTMLInputAttributes, HTMLTableAttributes, HTMLTdAttributes, HTMLThAttributes } from "svelte/elements";

export type WithElementRef<
	T,
	P extends Record<string, any> = {},
> = P & {
	ref?: any;
} & Omit<ComponentProps<any>, "ref" | keyof P>;

export type WithoutChildrenOrChild<T> = Omit<T, "children" | "child">;

export type WithoutChild<T> = Omit<T, "child">;
