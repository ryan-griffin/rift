import { Component, For, onCleanup, Suspense } from "solid-js";
import { useQuery, useQueryClient } from "@tanstack/solid-query";
import { User, WsServerMessage } from "../apiUtils.ts";
import { useApi } from "./Api.tsx";
import Avatar from "./Avatar.tsx";
import { useWebSocket } from "./WebSocket.tsx";

const UserCard: Component<{ user: User }> = (props) => {
	return (
		<div class="flex gap-2 items-center">
			<Avatar fallback={props.user.username[0]} className="w-10" />
			{props.user.username}
		</div>
	);
};

const Members: Component = () => {
	const { getApi } = useApi();
	const { onMessage } = useWebSocket();
	const queryClient = useQueryClient();

	const users = useQuery(() => ({
		queryKey: ["users"],
		queryFn: () => getApi<User[]>("/users"),
	}));

	const removeHandler = onMessage((event) => {
		const env: WsServerMessage = JSON.parse(event.data);

		if (env.module === "users" && env.type === "user_created") {
			queryClient.setQueryData<User[]>(
				["users"],
				(prev) => [...(prev ?? []), env.payload],
			);
		}
	});
	onCleanup(removeHandler);

	return (
		<div class="min-w-60 flex flex-col p-4 gap-2 overflow-y-auto rounded-xl bg-background-50 dark:bg-background-900">
			<p class="font-bold text-sm text-background-400 dark:text-background-500">
				Members
			</p>
			<Suspense>
				<For each={users.data}>
					{(user) => <UserCard user={user} />}
				</For>
			</Suspense>
		</div>
	);
};

export default Members;
