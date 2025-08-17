import { Component, onCleanup } from "solid-js";
import { useWebSocket } from "./WebSocket.tsx";
import { DirectoryNode, useGetApi, WsMessage } from "../apiUtils.ts";
import {
	isPermissionGranted,
	requestPermission,
	sendNotification,
} from "@tauri-apps/plugin-notification";
import { useAuth } from "./Auth.tsx";
import { useParams } from "@solidjs/router";

const NotificationService: Component = () => {
	const { onMessage } = useWebSocket();
	const { user, token } = useAuth();
	const params = useParams<{ id: string }>();
	let permissionGranted: boolean;

	(async () => {
		permissionGranted = await isPermissionGranted();
		if (!permissionGranted) {
			const permission = await requestPermission();
			permissionGranted = permission === "granted";
		}
	})();

	const removeHandler = onMessage(async (event) => {
		const message: WsMessage = JSON.parse(event.data);

		if (
			permissionGranted && message.type === "message_created" &&
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
