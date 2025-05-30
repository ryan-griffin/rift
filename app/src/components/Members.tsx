import { Component, For, Suspense } from "solid-js";
import { createAsync } from "@solidjs/router";
import { useApi, User } from "../apiUtils.ts";

const Members: Component = () => {
	const users = createAsync<User[]>(() => useApi("/users"));

	return (
		<div class="w-1/4 flex flex-col p-4 gap-2 rounded-xl bg-background-50 dark:bg-background-900">
			<p class="font-bold text-sm text-background-400 dark:text-background-500">
				Members
			</p>
			<Suspense fallback={<p>Loading...</p>}>
				<For each={users()}>
					{(user) => <p>{user.name}</p>}
				</For>
			</Suspense>
		</div>
	);
};

export default Members;
