import { action, query } from "@solidjs/router";

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

const port = import.meta.env.VITE_PORT;

export const useGetApi = query(async (token: string, url: string) => {
	const res = await fetch(`http://localhost:${port}/api${url}`, {
		headers: {
			"Content-Type": "application/json",
			Authorization: `Bearer ${token}`,
		},
	});
	return res.json();
}, "useGetApi");

export const usePostApi = action(async (
	token: string,
	url: string,
	body: unknown,
) => {
	const res = await fetch(`http://localhost:${port}/api${url}`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
			Authorization: `Bearer ${token}`,
		},
		body: JSON.stringify(body),
	});
	return res.json();
});
