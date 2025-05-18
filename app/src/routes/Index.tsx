import { Component, createResource, For, Suspense } from "solid-js";
import { isServer } from "solid-js/web";

const Index: Component = () => {
	const [users] = createResource(async () => {
		const res = await fetch("http://localhost:3000/api/users");
		return res.json();
	});

	return (
		<main class="p-10 m-10">
			<div>{isServer ? "Server" : "Client"}</div>

			<h1 class="text-2xl text-accent-500">
				Welcome to Tauri + Solid
			</h1>

			<Suspense fallback={<p>Loading...</p>}>
				<For each={users()}>
					{(user) => (
						<div>
							<p>{user.user_name}</p>
							<p>{user.name}</p>
						</div>
					)}
				</For>
			</Suspense>
		</main>
	);
};

export default Index;
