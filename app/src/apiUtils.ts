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

export type WsClientMessage =
	| {
		module: "messaging";
		type: "typing";
		payload: { thread_id: number };
	}
	| {
		module: "messaging";
		type: "stop_typing";
		payload: { thread_id: number };
	}
	| {
		module: "messaging";
		type: "create_message";
		payload: CreateMessage;
	};

export type WsServerMessage =
	| {
		module: "messaging";
		type: "user_typing";
		payload: { username: string; thread_id: number };
	}
	| {
		module: "messaging";
		type: "user_stopped_typing";
		payload: { username: string; thread_id: number };
	}
	| {
		module: "messaging";
		type: "message_created";
		payload: Message;
	}
	| {
		module: "system";
		type: "error";
		payload: string;
	};

export const resolveAddress = (): string =>
	(import.meta.env.VITE_API_CONTAINER_ADDRESS && isServer)
		? import.meta.env.VITE_API_CONTAINER_ADDRESS
		: import.meta.env.VITE_API_ADDRESS;
