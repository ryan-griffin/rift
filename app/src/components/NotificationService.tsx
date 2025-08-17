import { Component, onCleanup } from "solid-js";
import { useWebSocket } from "./WebSocket.tsx";
import { DirectoryNode, useGetApi, WsMessage } from "../apiUtils.ts";
import {
	isPermissionGranted,
	requestPermission,
	sendNotification,
} from "@tauri-apps/plugin-notification";
import { useAuth } from "./Auth.tsx";
import { createAsync, useParams } from "@solidjs/router";

const NotificationService: Component = () => {
	const { onMessage } = useWebSocket();
	const { user, token } = useAuth();
	const params = useParams<{ id: string }>();

	const permissionGranted = createAsync(async () => {
		let granted = await isPermissionGranted();
		if (!granted) {
			const permission = await requestPermission();
			granted = permission === "granted";
		}
		return granted;
	});

	const removeHandler = onMessage(async (event) => {
		const message: WsMessage = JSON.parse(event.data);

		if (
			permissionGranted() && message.type === "message_created" &&
			(message.directory_id != Number(params.id) || !document.hasFocus()) &&
			message.author_username != user?.username
		) {
			const thread: DirectoryNode[] = await useGetApi(
				token!,
				`/directory/${message.directory_id}`,
			);

			sendNotification({
				title: `#${thread[0].name} — ${message.author_username}`,
				body: message.content,
				group: message.directory_id.toString(),
			});
		}
	});
	onCleanup(removeHandler);

	return null;
};

export default NotificationService;
