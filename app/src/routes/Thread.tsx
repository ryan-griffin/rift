import {
	Component,
	createEffect,
	createMemo,
	createSignal,
	For,
	onCleanup,
	onMount,
	Show,
	Suspense,
} from "solid-js";
import { createStore } from "solid-js/store";
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
import { useAuth } from "../components/Auth.tsx";
import SendHorizontal from "../assets/send-horizontal.svg";
import Avatar from "../components/Avatar.tsx";
import MessageSquareText from "../assets/message-square-text.svg";
import { setStorageItem } from "../storageUtils.ts";

const MessageCard: Component<{ messages: Message[] }> = (props) => {
	return (
		<div class="flex gap-4">
			<Avatar
				fallback={props.messages[0].author_username[0]}
				className="h-12"
			/>
			<div class="flex flex-col">
				<div class="flex gap-2 items-center">
					<p class="text-accent-500 font-semibold">
						{props.messages[0].author_username}
					</p>
					<p class="text-sm text-background-400 dark:text-background-500">
						{new Date(props.messages[0].created_at).toLocaleString()}
					</p>
				</div>
				<For each={props.messages}>
					{(message) => <p>{message.content}</p>}
				</For>
			</div>
		</div>
	);
};

const TypingIndicator: Component<{ users: string[] }> = (props) => {
	const text = () => {
		const users = props.users;
		const count = users.length;

		if (count === 0) return "";
		if (count === 1) {
			return (
				<>
					<b>{users[0]}</b> is typing...
				</>
			);
		}
		if (count === 2) {
			return (
				<>
					<b>{users[0]}</b> and <b>{users[1]}</b> are typing...
				</>
			);
		}
		if (count <= 4) {
			return (
				<>
					{users.slice(0, -1).map((user, index) => (
						<>
							<b>{user}</b>
							{index < count - 2 ? ", " : ", and "}
						</>
					))}
					<b>{users[count - 1]}</b> are typing...
				</>
			);
		}
		return "Several people are typing...";
	};

	return (
		<p class="min-h-5 ml-2 text-sm text-background-400 dark:text-background-500">
			{text()}
		</p>
	);
};

const Thread: Component = () => {
	const params = useParams<{ id: string }>();
	const { token } = useAuth();
	const { connect, disconnect, sendMessage } = useWebSocket(token!);

	const thread = createAsync<DirectoryNode[]>(() =>
		useGetApi(token!, `/directory/${params.id}`)
	);

	const initialMessages = createAsync<Message[]>(() =>
		useGetApi(token!, `/thread/${params.id}`)
	);

	const [messages, setMessages] = createSignal<Message[]>([]);
	const [typingUsers, setTypingUsers] = createSignal<string[]>([]);

	createEffect(() => {
		const initial = initialMessages();
		if (initial) setMessages(initial);
	});

	const groupedMessages = createMemo(() => {
		const msgs = messages();
		if (msgs.length === 0) return [];

		const grouped: Message[][] = [];
		let currentGroup: Message[] = [msgs[0]];

		for (const currentMessage of msgs.slice(1)) {
			const firstMessage = currentGroup[0];

			const timeDiff = new Date(currentMessage.created_at).getTime() -
				new Date(firstMessage.created_at).getTime();
			const duration = 60 * 1000;

			if (
				currentMessage.author_username === firstMessage.author_username &&
				timeDiff <= duration
			) {
				currentGroup.push(currentMessage);
			} else {
				grouped.push(currentGroup);
				currentGroup = [currentMessage];
			}
		}

		grouped.push(currentGroup);
		return grouped;
	});

	onMount(() => {
		const ws = connect();

		ws.onmessage = (event) => {
			const message: WsMessage = JSON.parse(event.data);

			switch (message.type) {
				case "message_created": {
					if (message.directory_id !== Number(params.id)) return;
					const { type: _type, ...messageData } = message;
					setMessages((prev) => [...prev, messageData]);
					break;
				}
				case "user_typing": {
					if (message.thread_id !== Number(params.id)) return;
					setTypingUsers((prev) =>
						prev.includes(message.username) ? prev : [...prev, message.username]
					);
					break;
				}
				case "user_stopped_typing": {
					if (message.thread_id !== Number(params.id)) return;
					setTypingUsers((prev) =>
						prev.filter((user) => user !== message.username)
					);
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
		if (prevId && prevId !== id) setTypingUsers([]);
		setStorageItem("lastThread", id);
		return id;
	});

	const [isTyping, setIsTyping] = createSignal(false);
	let isTypingTimeout: number | undefined;

	const startTyping = () => {
		if (!isTyping()) {
			setIsTyping(true);
			sendMessage({
				type: "typing",
				thread_id: Number(params.id),
			});
		}

		if (isTypingTimeout) {
			clearTimeout(isTypingTimeout);
		}

		isTypingTimeout = setTimeout(() => {
			stopTyping();
		}, 3000);
	};

	const stopTyping = () => {
		if (isTyping()) {
			setIsTyping(false);
			sendMessage({
				type: "stop_typing",
				thread_id: Number(params.id),
			});
		}

		if (isTypingTimeout) {
			clearTimeout(isTypingTimeout);
			isTypingTimeout = undefined;
		}
	};

	const [newMessage, setNewMessage] = createSignal("");
	const handleSend = () => {
		if (!newMessage()) return;

		stopTyping();

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

	onCleanup(() => {
		stopTyping();
		disconnect();
	});

	let messagesContainer: HTMLDivElement | undefined;
	const [scrollState, setScrollState] = createStore({
		isTop: false,
		isBottom: true,
	});

	const handleScroll = () => {
		if (messagesContainer) {
			const { scrollTop, scrollHeight, clientHeight } = messagesContainer;
			setScrollState({
				isTop: scrollTop === 0,
				isBottom: scrollTop + clientHeight >= scrollHeight - 1,
			});
		}
	};

	onMount(() => {
		createEffect(() => {
			const messageCount = messages().length;
			if (messagesContainer && scrollState.isBottom && messageCount > 0) {
				messagesContainer.scrollTop = messagesContainer.scrollHeight;
			}
		});
	});

	let inputRef: HTMLInputElement | undefined;
	const handleGlobalKeydown = (e: KeyboardEvent) => {
		if (document.activeElement === inputRef) return;
		if (
			e.ctrlKey || e.altKey || e.metaKey || e.key === "Tab" ||
			e.key === "Escape" || e.key === "Enter" || e.key.startsWith("Arrow") ||
			e.key.startsWith("F") || e.key === "Backspace" || e.key === "Delete"
		) return;

		if (inputRef && e.key.length === 1) inputRef.focus();
	};

	onMount(() => {
		document.addEventListener("keydown", handleGlobalKeydown);
		onCleanup(() =>
			document.removeEventListener("keydown", handleGlobalKeydown)
		);
	});

	return (
		<Suspense>
			<Show
				when={thread()?.[0].type === "thread"}
				fallback={
					<div class="h-full flex items-center justify-center">
						<p class="text-xl font-bold text-background-400 dark:text-background-500">
							Thread not found
						</p>
					</div>
				}
			>
				<div class="relative h-full">
					<header class="flex p-4 gap-2">
						<MessageSquareText />
						<p class="font-bold">{thread()?.[0].name}</p>
					</header>
					<div
						class="flex flex-col p-4 pb-27 h-[calc(100%-3.5rem)] overflow-y-auto"
						style={{
							"mask-image": !scrollState.isTop
								? "linear-gradient(to bottom, transparent 0%, black 5%, black 100%)"
								: "none",
						}}
						ref={messagesContainer}
						onScroll={handleScroll}
					>
						<div class="flex-1" />
						<div class="flex flex-col gap-6">
							<For each={groupedMessages()}>
								{(messages) => <MessageCard messages={messages} />}
							</For>
						</div>
					</div>
					<div class="absolute z-10 bottom-0 left-0 right-0 flex flex-col px-4 pb-1 gap-1 before:absolute before:inset-0 before:bg-gradient-to-t before:from-background-50 dark:before:from-background-900 before:to-transparent before:-z-10 before:rounded-b-xl">
						<div class="flex items-center bg-background-100 dark:bg-background-800 rounded-2xl shadow-sm has-[input:focus]:outline-2 -outline-offset-1 outline-accent-500">
							<input
								class="grow p-4 rounded-l-2xl outline-0"
								ref={inputRef}
								placeholder={`Message ${thread()?.[0].name}`}
								value={newMessage()}
								onInput={(e) => {
									setNewMessage(e.currentTarget.value);
									e.currentTarget.value.trim() ? startTyping() : stopTyping();
								}}
								onKeyDown={(e) => {
									if (e.key === "Enter") {
										e.preventDefault();
										handleSend();
									}
								}}
								onBlur={stopTyping}
							/>
							<Button
								className="mr-2"
								type="submit"
								variant="suggested"
								icon={<SendHorizontal />}
								onClick={handleSend}
							/>
						</div>
						<TypingIndicator users={typingUsers()} />
					</div>
				</div>
			</Show>
		</Suspense>
	);
};

export default Thread;
