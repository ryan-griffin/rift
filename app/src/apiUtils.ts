import { useAuth } from "./components/Auth.tsx";
import { query, redirect } from "@solidjs/router";

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

export interface Message {
	id: number;
	content: string;
	author_username: string;
	directory_id: number;
	created_at: string;
	parent_id: number | null;
}

export const useApi = query(async (url: string) => {
	const { token } = useAuth();
	if (!token) throw redirect("/login");

	const res = await fetch(`http://localhost:3000/api${url}`, {
		headers: {
			"Content-Type": "application/json",
			Authorization: `Bearer ${token}`,
		},
	});
	return res.json();
}, "useApi");
