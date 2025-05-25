import { Component, For, Suspense } from "solid-js";
import { createAsync, useParams } from "@solidjs/router";

interface Message {
	id: number;
	content: string;
	author_username: string;
	directory_id: number;
	created_at: string;
	parent_id: number | null;
}

const Thread: Component = () => {
	const params = useParams<{ id: string }>();
	const messages = createAsync<Message[]>(async () => {
		const res = await fetch(
			`http://localhost:3000/api/thread/${params.id}`,
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
