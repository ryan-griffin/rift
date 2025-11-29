import "./styles.css";
import { Show } from "solid-js";
import { Route, Router } from "@solidjs/router";
import { isTauri } from "@tauri-apps/api/core";
import WindowControls from "./components/WindowControls.tsx";
import Nav from "./components/Nav.tsx";
import Index from "./routes/Index.tsx";
import Members from "./components/Members.tsx";
import Thread from "./routes/Thread.tsx";
import Settings from "./routes/Settings.tsx";
import Login from "./routes/Login.tsx";
import AuthProvider from "./components/Auth.tsx";
import ProtectedRoute from "./components/ProtectedRoute.tsx";
import ApiProvider from "./components/Api.tsx";
import { generateThemeCSS, getTheme } from "./colorUtils.ts";
import WebSocketProvider from "./components/WebSocket.tsx";
import NotificationService from "./components/NotificationService.tsx";

const App = () => {
	const theme = getTheme();
	const themeCSS = generateThemeCSS(theme);

	return (
		<Router
			root={(props) => (
				<AuthProvider>
					<style innerHTML={themeCSS} />
					<Show when={isTauri()}>
						<WindowControls />
					</Show>
					<main
						class={isTauri() ? "h-[calc(100vh-3rem)]" : "h-screen"}
					>
						{props.children}
					</main>
				</AuthProvider>
			)}
		>
			<Route path="/login" component={Login} />
			<Route
				component={(props) => (
					<ProtectedRoute>
						<ApiProvider>
							<WebSocketProvider>
								<Show when={isTauri()}>
									<NotificationService />
								</Show>
								<div class={`h-full flex p-2 gap-2 ${isTauri() ? "pt-0" : ""}`}>
									<Nav />
									<div class="flex-3 rounded-xl bg-background-50 dark:bg-background-900">
										{props.children}
									</div>
									<Members />
								</div>
							</WebSocketProvider>
						</ApiProvider>
					</ProtectedRoute>
				)}
			>
				<Route path="/" component={Index} />
				<Route path="/thread/:id" component={Thread} />
				<Route path="/settings" component={Settings} />
			</Route>
		</Router>
	);
};

export default App;
