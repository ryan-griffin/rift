import {
	Component,
	createContext,
	createSignal,
	JSX,
	onCleanup,
	onMount,
	useContext,
} from "solid-js";
import { resolveAddress, WsMessage } from "../apiUtils.ts";
import { useAuth } from "./Auth.tsx";

interface WebSocketContextType {
	onMessage: (handler: (event: MessageEvent) => void) => () => void;
	sendMessage: (message: WsMessage) => void;
}

const WebSocketContext = createContext<WebSocketContextType>();

export const useWebSocket = () => {
	const context = useContext(WebSocketContext);
	if (!context) {
		throw new Error("useWebSocket must be used within a WebSocketProvider");
	}
	return context;
};

const WebSocketProvider: Component<{ children: JSX.Element }> = (props) => {
	const { token } = useAuth();
	const [socket, setSocket] = createSignal<WebSocket | null>(null);
	const messageHandlers = new Set<(event: MessageEvent) => void>();

	const connect = () => {
		const address = resolveAddress();
		const ws = new WebSocket(`ws://${address}/api/ws?token=${token}`);

		ws.onopen = () => setSocket(ws);
		ws.onclose = () => setSocket(null);
		ws.onerror = (error) => console.error("WebSocket error:", error);
		ws.onmessage = (event) =>
			messageHandlers.forEach((handler) => handler(event));
	};

	const disconnect = () => {
		const ws = socket();
		if (ws) ws.close();
	};

	onMount(() => connect());
	onCleanup(() => disconnect());

	const onMessage = (handler: (event: MessageEvent) => void) => {
		messageHandlers.add(handler);
		return () => messageHandlers.delete(handler);
	};

	const sendMessage = (message: WsMessage) => {
		const ws = socket();
		if (ws) ws.send(JSON.stringify(message));
	};

	return (
		<WebSocketContext.Provider
			value={{
				onMessage,
				sendMessage,
			}}
		>
			{props.children}
		</WebSocketContext.Provider>
	);
};

export default WebSocketProvider;
