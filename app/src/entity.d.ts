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
