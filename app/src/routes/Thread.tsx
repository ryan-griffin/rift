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
import { useParams } from "@solidjs/router";
import { useQuery, useQueryClient } from "@tanstack/solid-query";
import {
	CreateMessage,
	DirectoryNode,
	Message,
	WsServerMessage,
} from "../apiUtils.ts";
import Button from "../components/Button.tsx";
import { useApi } from "../components/Api.tsx";
import { useWebSocket } from "../components/WebSocket.tsx";
import SendHorizontal from "../assets/send-horizontal.svg";
import Avatar from "../components/Avatar.tsx";
import MessageSquareText from "../assets/message-square-text.svg";
import { setStorageItem } from "../storageUtils.ts";
import markdownit from "markdown-it";
import X from "../assets/x.svg";
import Reply from "../assets/reply.svg";

const MESSAGE_GROUP_WINDOW_MS = 60 * 1000;
const TYPING_TIMEOUT_MS = 3000;

interface MessagesState {
	byId: Record<number, Message>;
	groups: number[][];
	messageCount: number;
}

const md = markdownit({
	breaks: true,
	linkify: true,
	typographer: true,
});

const mdClasses = {
	"[&_h1,&_h2,&_h3,&_h4,&_h5,&_h6]:font-bold [&_h1]:text-4xl [&_h2]:text-3xl [&_h3]:text-2xl [&_h4]:text-xl [&_h5]:text-lg":
		true,
	"[&_a]:text-accent-500 [&_a:hover]:underline": true,
	"[&_ol,&_ul]:pl-6 [&_ol>li,&_ul>li]:pl-1": true,
	"[&_ol]:list-decimal [&_ul]:list-disc": true,
	"[&_hr]:text-background-400 dark:[&_hr]:text-background-500": true,
	"[&_img]:rounded-xl": true,
};

const shouldGroupMessage = (anchor: Message, message: Message) => {
	const timeDiff = new Date(message.created_at).getTime() -
		new Date(anchor.created_at).getTime();

	const parentBreak = message.parent_id !== null &&
		(anchor.parent_id === null || anchor.parent_id !== message.parent_id);

	return (
		message.author_username === anchor.author_username &&
		timeDiff <= MESSAGE_GROUP_WINDOW_MS &&
		!parentBreak
	);
};

const buildMessagesState = (messages: Message[]): MessagesState => {
	const byId: Record<number, Message> = {};
	const groups: number[][] = [];
	if (!messages.length) return { byId, groups, messageCount: 0 };

	let currentGroup: number[] = [];

	for (const message of messages) {
		byId[message.id] = message;

		if (
			currentGroup.length && shouldGroupMessage(byId[currentGroup[0]], message)
		) {
			currentGroup.push(message.id);
		} else {
			if (currentGroup.length) groups.push(currentGroup);
			currentGroup = [message.id];
		}
	}

	if (currentGroup.length) groups.push(currentGroup);

	return { byId, groups, messageCount: messages.length };
};

const appendMessageToState = (
	state: MessagesState,
	message: Message,
): MessagesState => {
	const byId = { ...state.byId, [message.id]: message };
	const messageCount = state.messageCount + 1;

	if (!state.groups.length) {
		return { byId, groups: [[message.id]], messageCount };
	}

	const lastGroup = state.groups[state.groups.length - 1];
	const anchor = state.byId[lastGroup[0]];

	if (shouldGroupMessage(anchor, message)) {
		const newGroups = [...state.groups];
		newGroups[newGroups.length - 1] = [...lastGroup, message.id];
		return { byId, groups: newGroups, messageCount };
	}

	const newGroups = [...state.groups, [message.id]];
	return { byId, groups: newGroups, messageCount };
};

const MessageGroup: Component<
	{
		messagesById: Record<number, Message>;
		groupIds: number[];
		onMessageClick: (id: number) => void;
	}
> = (props) => {
	const group = createMemo(() =>
		props.groupIds.map((id) => props.messagesById[id])
	);
	const anchor = () => group()[0];

	const parentMessage = () => {
		const parentId = anchor().parent_id;
		if (parentId === null) return null;
		return props.messagesById[parentId] ?? null;
	};

	return (
		<div>
			<Show when={parentMessage()}>
				{(parent) => (
					<div class="flex h-7 gap-1">
						<div class="w-9 h-3.5 ml-6 mb-1 mt-auto border-l-2 border-t-2 rounded-tl-lg border-background-100 dark:border-background-800" />
						<div class="flex gap-2 mb-auto text-sm">
							<div class="text-background-400 dark:text-background-500">
								{parent().author_username}
							</div>
							<p class="truncate max-w-100">{parent().content}</p>
						</div>
					</div>
				)}
			</Show>
			<div class="flex gap-4">
				<Avatar
					fallback={anchor().author_username[0]}
					className="h-12"
				/>
				<div class="flex flex-col grow">
					<div class="flex gap-2 items-center">
						<p class="text-accent-500 font-semibold">
							{anchor().author_username}
						</p>
						<p class="text-sm text-background-400 dark:text-background-500">
							{new Date(anchor().created_at).toLocaleString()}
						</p>
					</div>
					<For each={group()}>
						{(message) => (
							<div class="group flex justify-between items-start gap-1 hover:bg-background-100 dark:hover:bg-background-800 rounded-sm">
								<div
									class="wrap-anywhere"
									classList={mdClasses}
									innerHTML={md.render(message.content)}
								/>
								<button
									type="button"
									class="hidden group-hover:block text-background-500 hover:text-background-600 dark:text-background-400 dark:hover:text-background-300 transition-colors duration-200 cursor-pointer"
									onClick={() => props.onMessageClick(message.id)}
								>
									<Reply />
								</button>
							</div>
						)}
					</For>
				</div>
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
	const { getApi } = useApi();
	const { onMessage, sendMessage } = useWebSocket();
	const queryClient = useQueryClient();

	const directoryNode = useQuery(() => ({
		queryKey: ["directory", Number(params.id)],
		queryFn: () => getApi<DirectoryNode[]>(`/directory/${params.id}`),
	}));
	const threadNode = () => directoryNode.data?.[0];

	const messages = useQuery(() => ({
		queryKey: ["thread", Number(params.id)],
		queryFn: async () => {
			const data = await getApi<Message[]>(`/thread/${params.id}`);
			return buildMessagesState(data);
		},
		staleTime: Infinity,
		gcTime: 1000 * 60 * 30,
	}));

	const [typingUsers, setTypingUsers] = createSignal<string[]>([]);

	const removeHandler = onMessage((event) => {
		const env: WsServerMessage = JSON.parse(event.data);

		if (env.module === "messages") {
			switch (env.type) {
				case "message_created": {
					const message = env.payload;

					queryClient.setQueryData<MessagesState>(
						["thread", message.directory_id],
						(prev) => {
							if (!prev) return buildMessagesState([message]);
							return appendMessageToState(prev, message);
						},
					);
					break;
				}
				case "user_typing": {
					const payload = env.payload;
					if (payload.thread_id !== Number(params.id)) return;
					setTypingUsers((prev) =>
						prev.includes(payload.username) ? prev : [...prev, payload.username]
					);
					break;
				}
				case "user_stopped_typing": {
					const payload = env.payload;
					if (payload.thread_id !== Number(params.id)) return;
					setTypingUsers((prev) =>
						prev.filter((user) => user !== payload.username)
					);
					break;
				}
			}
		}
	});

	onCleanup(removeHandler);

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
				module: "messages",
				type: "typing",
				payload: { thread_id: Number(params.id) },
			});
		}

		if (isTypingTimeout) {
			clearTimeout(isTypingTimeout);
		}

		isTypingTimeout = setTimeout(() => {
			stopTyping();
		}, TYPING_TIMEOUT_MS);
	};

	const stopTyping = () => {
		if (isTyping()) {
			setIsTyping(false);
			sendMessage({
				module: "messages",
				type: "stop_typing",
				payload: { thread_id: Number(params.id) },
			});
		}

		if (isTypingTimeout) {
			clearTimeout(isTypingTimeout);
			isTypingTimeout = undefined;
		}
	};

	const [newMessage, setNewMessage] = createStore<CreateMessage>({
		content: "",
		directory_id: Number(params.id),
		parent_id: null,
	});

	createEffect(() =>
		setNewMessage({
			directory_id: Number(params.id),
			parent_id: null,
		})
	);

	const replyTarget = () => {
		const parentId = newMessage.parent_id;
		if (parentId === null) return null;
		return messages.data?.byId[parentId] ?? null;
	};

	const handleSend = () => {
		if (!newMessage.content.trim()) return;

		stopTyping();

		sendMessage({
			module: "messages",
			type: "create_message",
			payload: newMessage,
		});

		setNewMessage({
			content: "",
			parent_id: null,
		});

		if (inputRef) inputRef.style.height = "auto";
	};

	onCleanup(() => stopTyping());

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

	const scrollToBottom = () => {
		if (messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}
	};

	createEffect(() => {
		if (messages.data?.messageCount && scrollState.isBottom) {
			requestAnimationFrame(scrollToBottom);
		}
	});

	createEffect(() => {
		if (newMessage.parent_id && scrollState.isBottom) {
			requestAnimationFrame(scrollToBottom);
		}
	});

	let inputRef: HTMLTextAreaElement | undefined;
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
				when={threadNode()?.type === "thread"}
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
						<p class="font-bold">{threadNode()?.name}</p>
					</header>
					<div
						class={`flex flex-col p-4 h-[calc(100%-3.5rem)] overflow-y-auto ${
							newMessage.parent_id ? "pb-37" : "pb-26"
						}`}
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
							<For each={messages.data?.groups}>
								{(groupIds) => (
									<MessageGroup
										messagesById={messages.data!.byId}
										groupIds={groupIds}
										onMessageClick={(id) => {
											setNewMessage("parent_id", id);
											if (inputRef) inputRef.focus();
										}}
									/>
								)}
							</For>
						</div>
					</div>
					<div class="absolute z-10 bottom-0 left-0 right-0 flex flex-col px-4 pb-1 before:absolute before:inset-0 before:bg-linear-to-t before:from-background-50 dark:before:from-background-900 before:via-background-50/85 dark:before:via-background-900/85 before:to-transparent before:-z-10 before:rounded-b-xl">
						<div class="bg-background-100 dark:bg-background-800 rounded-2xl shadow-sm has-[textarea:focus]:outline-2 -outline-offset-1 outline-accent-500">
							<Show when={newMessage.parent_id}>
								<div class="flex justify-between m-1 mb-0 p-2 bg-background-50 dark:bg-background-700 rounded-t-xl rounded-b-sm">
									<p>
										Replying to <b>{replyTarget()?.author_username}</b>
									</p>
									<button
										type="button"
										class="text-background-500 hover:text-background-600 dark:text-background-400 dark:hover:text-background-300 transition-colors duration-200 cursor-pointer"
										onClick={() => setNewMessage("parent_id", null)}
									>
										<X />
									</button>
								</div>
							</Show>
							<div class="flex items-end">
								<textarea
									class="grow p-4 rounded-l-2xl outline-0 placeholder-background-400 dark:placeholder-background-500 resize-none max-h-48"
									rows={1}
									ref={inputRef}
									placeholder={`Message ${threadNode()?.name}`}
									value={newMessage.content}
									onInput={(e) => {
										const textarea = e.currentTarget;
										setNewMessage("content", textarea.value);
										textarea.value.trim() ? startTyping() : stopTyping();
										textarea.style.height = "auto";
										textarea.style.height = `${textarea.scrollHeight}px`;
									}}
									onKeyDown={(e) => {
										if (e.key === "Enter" && !e.shiftKey) {
											e.preventDefault();
											handleSend();
										}
										if (e.key === "Escape") {
											e.currentTarget.blur();
										}
									}}
									onBlur={stopTyping}
								/>
								<Button
									className="m-2 ml-0"
									type="submit"
									variant="suggested"
									icon={<SendHorizontal />}
									onClick={handleSend}
								/>
							</div>
						</div>
						<TypingIndicator users={typingUsers()} />
					</div>
				</div>
			</Show>
		</Suspense>
	);
};

export default Thread;
