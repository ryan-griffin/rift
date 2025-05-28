import { Component, For, Suspense } from "solid-js";
import { createAsync, useParams } from "@solidjs/router";
import { Message } from "../entity.d.ts";
import { useAuth } from "../components/Auth.tsx";

const Thread: Component = () => {
	const params = useParams<{ id: string }>();
	const { token } = useAuth();
	const messages = createAsync<Message[]>(async () => {
		const res = await fetch(
			`http://localhost:3000/api/thread/${params.id}`,
			{
				headers: {
					"Content-Type": "application/json",
					Authorization: `Bearer ${token()}`,
				},
			},
		);
		return res.json();
	});

	return (
		<Suspense fallback={<p>Loading...</p>}>
			<div class="flex flex-col gap-4">
				<For each={messages()}>
					{(message) => (
						<div class="flex flex-col">
							<div class="flex gap-2">
								<p>{message.author_username}</p>
								<p>
									{new Date(message.created_at)
										.toLocaleString()}
								</p>
							</div>
							<p>{message.content}</p>
						</div>
					)}
				</For>
			</div>
		</Suspense>
	);
};

export default Thread;
