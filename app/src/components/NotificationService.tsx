import { Component, onCleanup } from "solid-js";
import { useWebSocket } from "./WebSocket.tsx";
import { WsMessage } from "../apiUtils.ts";
import {
	createChannel,
	Importance,
	isPermissionGranted,
	requestPermission,
	sendNotification,
	Visibility,
} from "@tauri-apps/plugin-notification";
import { useAuth } from "./Auth.tsx";
import { useParams } from "@solidjs/router";

const NotificationService: Component = () => {
	const { onMessage } = useWebSocket();
	const { user } = useAuth();
	const params = useParams<{ id: string }>();
	let permissionGranted: boolean;

	(async () => {
		permissionGranted = await isPermissionGranted();
		if (!permissionGranted) {
			const permission = await requestPermission();
			permissionGranted = permission === "granted";
		}

		await createChannel({
			id: "messages",
			name: "Messages",
			description: "Notifications for new messages",
			importance: Importance.High,
			visibility: Visibility.Private,
			lights: true,
			lightColor: "#ffffff",
			vibration: true,
			sound: "notification_sound",
		});
	})();

	const removeHandler = onMessage((event) => {
		const message: WsMessage = JSON.parse(event.data);

		if (
			permissionGranted && message.type === "message_created" &&
			(message.directory_id != Number(params.id) || !document.hasFocus()) &&
			message.author_username != user?.username
		) {
			console.log(message);
			sendNotification({
				title: message.author_username,
				body: message.content,
				channelId: "messages",
			});
		}
	});
	onCleanup(removeHandler);

	return null;
};

export default NotificationService;
