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
			module: "messages";
			type: "typing";
			payload: { thread_id: number };
	  }
	| {
			module: "messages";
			type: "stop_typing";
			payload: { thread_id: number };
	  }
	| {
			module: "messages";
			type: "create_message";
			payload: CreateMessage;
	  };

export type WsServerMessage =
	| {
			module: "messages";
			type: "user_typing";
			payload: { username: string; thread_id: number };
	  }
	| {
			module: "messages";
			type: "user_stopped_typing";
			payload: { username: string; thread_id: number };
	  }
	| {
			module: "messages";
			type: "message_created";
			payload: Message;
	  }
	| {
			module: "users";
			type: "user_created";
			payload: User;
	  }
	| {
			module: "system";
			type: "error";
			payload: string;
	  };

export const resolveAddress = () => {
	if (!isServer) {
		return window.__API_ADDRESS__ || import.meta.env.VITE_API_ADDRESS;
	}

	const { API_INTERNAL_HOST, API_INTERNAL_PORT, API_HOST, API_PORT } =
		process.env;

	if (API_INTERNAL_HOST && API_INTERNAL_PORT) {
		return `${API_INTERNAL_HOST}:${API_INTERNAL_PORT}`;
	}

	if (API_HOST && API_PORT) {
		return `${API_HOST}:${API_PORT}`;
	}

	throw new Error("API address not found");
};
