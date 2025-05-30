import { Component, For, Suspense } from "solid-js";
import { createAsync, useParams } from "@solidjs/router";
import { Message, useApi } from "../apiUtils.ts";

const Thread: Component = () => {
	const params = useParams<{ id: string }>();
	const messages = createAsync<Message[]>(() =>
		useApi(`/thread/${params.id}`)
	);

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
