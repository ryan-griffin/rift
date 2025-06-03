import { Component, createSignal, For, Suspense } from "solid-js";
import { createAsync, useAction, useParams } from "@solidjs/router";
import { CreateMessage, Message, useGetApi, usePostApi } from "../apiUtils.ts";
import Button from "../components/Button.tsx";
import Input from "../components/Input.tsx";
import { useAuth } from "../components/Auth.tsx";
import SendHorizontal from "../assets/send-horizontal.svg";

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
		<div class="relative h-full">
			<Suspense fallback={<p>Loading...</p>}>
				<div class="flex flex-col p-4 pb-22 gap-4 h-full overflow-y-auto">
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
			<div class="absolute bottom-4 left-4 right-4 flex p-2 gap-1 bg-background-100 dark:bg-background-800 rounded-2xl">
				<Input
					className="grow"
					placeholder="Send a message..."
					value={newMessage()}
					onInput={setNewMessage}
					onKeyDown={(e) => {
						if (e.key === "Enter") {
							e.preventDefault();
							handleSend();
						}
					}}
				/>
				<Button
					type="submit"
					variant="suggested"
					icon={<SendHorizontal />}
					onClick={handleSend}
				/>
			</div>
		</div>
	);
};

export default Thread;
