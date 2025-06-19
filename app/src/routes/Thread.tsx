import {
	Component,
	createEffect,
	createSignal,
	For,
	onCleanup,
	onMount,
	Suspense,
} from "solid-js";
import { createAsync, useParams } from "@solidjs/router";
import {
	CreateMessage,
	DirectoryNode,
	Message,
	useGetApi,
	useWebSocket,
	WsMessage,
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
	const params = useParams<{ id: string }>();
	const { token } = useAuth();
	const { isConnected, connect, disconnect, sendMessage } = useWebSocket(
		token!,
	);

	const thread = createAsync<DirectoryNode[]>(() =>
		useGetApi(token!, `/directory/${params.id}`)
	);

	const initialMessages = createAsync<Message[]>(() =>
		useGetApi(token!, `/thread/${params.id}`)
	);
	const [messages, setMessages] = createSignal<Message[]>([]);
	const allMessages = () => [...(initialMessages() || []), ...messages()];

	onMount(() => {
		const ws = connect();

		ws.onmessage = (event) => {
			const message: WsMessage = JSON.parse(event.data);

			switch (message.type) {
				case "message_created": {
					const { type: _type, ...messageData } = message;
					setMessages((prev) => [...prev, messageData]);
					break;
				}
				case "error":
					console.error("WebSocket error:", message.message);
					break;
			}
		};
	});

	createEffect((prevId: number | undefined) => {
		const id = Number(params.id);

		if (prevId && prevId !== id) {
			sendMessage({
				type: "leave_thread",
				thread_id: prevId,
			});
			setMessages([]);
		}

		sendMessage({
			type: "join_thread",
			thread_id: id,
		});

		return id;
	});

	onCleanup(() => {
		sendMessage({
			type: "leave_thread",
			thread_id: Number(params.id),
		});
		disconnect();
	});

	const [newMessage, setNewMessage] = createSignal("");
	const handleSend = () => {
		if (!newMessage() || !isConnected()) return;

		const message: CreateMessage = {
			content: newMessage(),
			directory_id: Number(params.id),
			parent_id: null,
		};

		sendMessage({
			type: "create_message",
			...message,
		});

		setNewMessage("");
	};

	let messagesContainer: HTMLDivElement | undefined;

	createEffect(() => {
		if (allMessages().length > 0 && messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}
	});

	const [isScrolledFromTop, setIsScrolledFromTop] = createSignal(false);
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
					class="flex flex-col p-4 pb-24 gap-6 h-[calc(100vh-4.5rem)] overflow-y-auto"
					style={{
						"mask-image": isScrolledFromTop()
							? "linear-gradient(to bottom, transparent 0%, black 5%, black 100%)"
							: "none",
					}}
					ref={messagesContainer}
					onScroll={handleScroll}
				>
					<For each={allMessages()}>
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
