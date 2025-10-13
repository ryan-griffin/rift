import { Component, For, Suspense } from "solid-js";
import { createAsync } from "@solidjs/router";
import { User } from "../apiUtils.ts";
import { useApi } from "./Api.tsx";
import Avatar from "./Avatar.tsx";

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
	const users = createAsync<User[]>(() => getApi("/users"));

	return (
		<div class="w-1/4 flex flex-col p-4 gap-2 rounded-xl bg-background-50 dark:bg-background-900">
			<p class="font-bold text-sm text-background-400 dark:text-background-500">
				Members
			</p>
			<Suspense fallback={<p>Loading...</p>}>
				<For each={users()}>
					{(user) => <UserCard user={user} />}
				</For>
			</Suspense>
		</div>
	);
};

export default Members;
