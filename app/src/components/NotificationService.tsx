import { Component, createEffect, onCleanup } from "solid-js";
import { useWebSocket } from "./WebSocket.tsx";
import { DirectoryNode, Message, WsServerMessage } from "../apiUtils.ts";
import {
	isPermissionGranted,
	requestPermission,
	sendNotification,
} from "@tauri-apps/plugin-notification";
import { useAuth } from "./Auth.tsx";
import { useApi } from "./Api.tsx";
import { useParams } from "@solidjs/router";
import { useQuery } from "@tanstack/solid-query";

const NotificationService: Component = () => {
	const { onMessage } = useWebSocket();
	const { user } = useAuth();
	const { getApi } = useApi();
	const params = useParams<{ id: string }>();

	const permissionGranted = async () => {
		let granted = await isPermissionGranted();
		if (!granted) {
			const permission = await requestPermission();
			granted = permission === "granted";
		}
		return granted;
	};

	const removeHandler = onMessage(async (event) => {
		const env: WsServerMessage = JSON.parse(event.data);
		if (env.module !== "messages" || env.type !== "message_created") return;

		const message = env.payload as Message;

		if (
			await permissionGranted() &&
			(message.directory_id != Number(params.id) || !document.hasFocus()) &&
			message.author_username != user?.username
		) {
			const thread = useQuery(() => ({
				queryKey: ["directory", message.directory_id],
				queryFn: () =>
					getApi<DirectoryNode[]>(`/directory/${message.directory_id}`),
			}));

			createEffect(() => {
				if (!thread.data) return;
				sendNotification({
					title: `#${thread.data[0].name} — ${message.author_username}`,
					body: message.content,
					group: message.directory_id.toString(),
				});
			});
		}
	});
	onCleanup(removeHandler);

	return null;
};

export default NotificationService;
