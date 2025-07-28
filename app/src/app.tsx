import "./styles.css";
import { onMount, Show } from "solid-js";
import { Route, Router } from "@solidjs/router";
import { isTauri } from "@tauri-apps/api/core";
import { initializeTheme } from "./colorUtils.ts";
import WindowControls from "./components/WindowControls.tsx";
import Splitter from "./components/Spiltter.tsx";
import Nav from "./components/Nav.tsx";
import Index from "./routes/Index.tsx";
import Members from "./components/Members.tsx";
import Thread from "./routes/Thread.tsx";
import Settings from "./routes/Settings.tsx";
import Login from "./routes/Login.tsx";
import { AuthProvider } from "./components/Auth.tsx";
import ProtectedRoute from "./components/ProtectedRoute.tsx";

export default function App() {
	onMount(() => {
		initializeTheme();
	});

	return (
		<Router
			root={(props) => (
				<AuthProvider>
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
						<div class={`h-full flex p-2 ${isTauri() ? "pt-0" : ""} gap-2`}>
							<Splitter
								a={<Nav />}
								b={
									<div class="h-full rounded-xl bg-background-50 dark:bg-background-900">
										{props.children}
									</div>
								}
							/>
							<Members />
						</div>
					</ProtectedRoute>
				)}
			>
				<Route path="/" component={Index} />
				<Route path="/directory/:id" component={Thread} />
				<Route path="/settings" component={Settings} />
			</Route>
		</Router>
	);
}
