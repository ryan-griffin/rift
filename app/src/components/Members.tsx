import { Component, createResource, For, Show, Suspense } from "solid-js";

const Members: Component = () => {
	const [users] = createResource(async () => {
		const res = await fetch("http://localhost:3000/api/users");
		return res.json();
	});

	return (
		<div class="h-screen w-1/4 flex flex-col p-4 gap-2 bg-background-50 dark:bg-background-900">
			<p class="font-bold text-sm text-background-400 dark:text-background-500">
				Members
			</p>
			<Suspense fallback={<p>Loading...</p>}>
				<Show when={users()}>
					{(userData) => (
						<For each={userData()}>
							{(user) => <p>{user.name}</p>}
						</For>
					)}
				</Show>
			</Suspense>
		</div>
	);
};

export default Members;
