import { action, query } from "@solidjs/router";
import { isServer } from "solid-js/web";

export interface User {
	username: string;
	name: string;
}

export interface DirectoryNode {
	id: number;
	name: string;
	type: "folder" | "thread";
	parent_id: number | null;
}

export interface CreateMessage {
	content: string;
	directory_id: number;
	parent_id: number | null;
}

export interface Message extends CreateMessage {
	id: number;
	author_username: string;
	created_at: string;
}

export type WsMessage =
	| { type: "typing"; thread_id: number }
	| { type: "stop_typing"; thread_id: number }
	| ({ type: "create_message" } & CreateMessage)
	| { type: "user_typing"; username: string; thread_id: number }
	| { type: "user_stopped_typing"; username: string; thread_id: number }
	| ({ type: "message_created" } & Message)
	| { type: "error"; message: string };

export const resolveAddress = (): string =>
	(import.meta.env.VITE_CONTAINER_ADDRESS && isServer)
		? import.meta.env.VITE_CONTAINER_ADDRESS
		: import.meta.env.VITE_ADDRESS;

export const useGetApi = query(async (token: string, url: string) => {
	const address = resolveAddress();
	const res = await fetch(`http://${address}/api${url}`, {
		headers: {
			"Content-Type": "application/json",
			Authorization: `Bearer ${token}`,
		},
	});
	if (res.ok) return await res.json();
}, "useGetApi");

export const usePostApi = action(async (
	token: string,
	url: string,
	body: unknown,
) => {
	const address = resolveAddress();
	const res = await fetch(`http://${address}/api${url}`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
			Authorization: `Bearer ${token}`,
		},
		body: JSON.stringify(body),
	});
	if (res.ok) return await res.json();
});
