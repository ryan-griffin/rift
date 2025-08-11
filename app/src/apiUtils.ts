import { action, query } from "@solidjs/router";
import { createSignal } from "solid-js";
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

export const useWebSocket = (token: string) => {
	const [socket, setSocket] = createSignal<WebSocket | null>(null);

	const connect = () => {
		const address = resolveAddress();
		const ws = new WebSocket(`ws://${address}/api/ws?token=${token}`);

		ws.onopen = () => setSocket(ws);
		ws.onclose = () => setSocket(null);
		ws.onerror = (error) => console.error("WebSocket error:", error);

		return ws;
	};

	const disconnect = () => {
		const ws = socket();
		if (ws) ws.close();
	};

	const sendMessage = (message: WsMessage) => {
		const ws = socket();
		if (ws) ws.send(JSON.stringify(message));
	};

	return {
		isConnected: () => socket() !== null,
		connect,
		disconnect,
		sendMessage,
	};
};
