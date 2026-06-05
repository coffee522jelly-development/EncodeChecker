import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import type { WithElementRef, WithoutChildrenOrChild, WithoutChild } from "./types.js";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export type { WithElementRef, WithoutChildrenOrChild, WithoutChild };

export interface FileResult {
    path: string;
    status: string;
    encoding: string;
    newline: string;
    size: number;
}

export function convertToCsv(results: FileResult[]): string {
    const header = "Status,Path,Encoding,Newline,Size\n";
    const rows = results.map(r =>
        `"${r.status}","${r.path}","${r.encoding}","${r.newline}",${r.size}`
    ).join("\n");
    return header + rows;
}
