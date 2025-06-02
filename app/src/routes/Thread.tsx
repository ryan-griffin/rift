import { Component, createSignal, For, Suspense } from "solid-js";
import { createAsync, useAction, useParams } from "@solidjs/router";
import { CreateMessage, Message, useGetApi, usePostApi } from "../apiUtils.ts";
import Button from "../components/Button.tsx";
import { useAuth } from "../components/Auth.tsx";

const Thread: Component = () => {
	const { token } = useAuth();
	const params = useParams<{ id: string }>();

	const messages = createAsync<Message[]>(() =>
		useGetApi(token, `/thread/${params.id}`)
	);

	const createMessage = useAction(usePostApi);

	const [newMessage, setNewMessage] = createSignal("");
	const handleSend = async () => {
		if (!newMessage()) return;

		const message: CreateMessage = {
			content: newMessage(),
			directory_id: Number(params.id),
			parent_id: null,
		};

		const result: Message = await createMessage(
			token,
			"/message",
			message,
		);
		if (result) setNewMessage("");
	};

	return (
		<>
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
			<input
				placeholder="Message"
				value={newMessage()}
				onInput={(e) => setNewMessage(e.currentTarget.value)}
			/>
			<Button
				type="submit"
				variant="suggested"
				text="Send"
				onClick={handleSend}
			/>
		</>
	);
};

export default Thread;
