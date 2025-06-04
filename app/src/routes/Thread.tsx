import { Component, createEffect, createSignal, For, Suspense } from "solid-js";
import { createAsync, useAction, useParams } from "@solidjs/router";
import {
	CreateMessage,
	DirectoryNode,
	Message,
	useGetApi,
	usePostApi,
} from "../apiUtils.ts";
import Button from "../components/Button.tsx";
import Input from "../components/Input.tsx";
import { useAuth } from "../components/Auth.tsx";
import SendHorizontal from "../assets/send-horizontal.svg";
import Avatar from "../components/Avatar.tsx";
import MessageSquareText from "../assets/message-square-text.svg";

const MessageCard: Component<{ message: Message }> = (props) => {
	return (
		<div class="flex gap-4">
			<Avatar
				fallback={props.message.author_username[0].toUpperCase()}
				className="h-12"
			/>
			<div class="flex flex-col">
				<div class="flex gap-2 items-center">
					<p class="text-accent-500 font-medium">
						{props.message.author_username}
					</p>
					<p class="text-sm text-background-400 dark:text-background-500">
						{new Date(props.message.created_at).toLocaleString()}
					</p>
				</div>
				<p>{props.message.content}</p>
			</div>
		</div>
	);
};

const Thread: Component = () => {
	const { token } = useAuth();
	const params = useParams<{ id: string }>();

	const thread = createAsync<DirectoryNode[]>(() =>
		useGetApi(token, `/directory/${params.id}`)
	);

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

	let messagesContainer: HTMLDivElement | undefined;
	const [isScrolledFromTop, setIsScrolledFromTop] = createSignal(false);

	createEffect(() => {
		const messageList = messages();
		if (messageList && messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}
	});

	const handleScroll = () => {
		if (messagesContainer) {
			setIsScrolledFromTop(messagesContainer.scrollTop > 0);
		}
	};

	return (
		<div class="relative h-full">
			<header class="flex p-4 gap-2">
				<MessageSquareText />
				<Suspense>
					<p class="font-bold">{thread()?.[0].name}</p>
				</Suspense>
			</header>
			<Suspense fallback={<p>Loading...</p>}>
				<div
					class="flex flex-col p-4 pb-38 gap-6 h-full overflow-y-auto"
					style={{
						"mask-image": isScrolledFromTop()
							? "linear-gradient(to bottom, transparent 0%, black 5%, black 100%)"
							: "none",
					}}
					ref={messagesContainer}
					onScroll={handleScroll}
				>
					<For each={messages()}>
						{(message) => <MessageCard message={message} />}
					</For>
				</div>
			</Suspense>
			<div class="absolute bottom-4 left-4 right-4 flex p-2 gap-1 bg-background-100 dark:bg-background-800 rounded-2xl shadow-sm">
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
