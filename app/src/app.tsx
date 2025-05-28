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

export default function App() {
	onMount(() => {
		initializeTheme();
	});

	return (
		<AuthProvider>
			<Router
				root={(props) => (
					<>
						<Show when={isTauri()}>
							<WindowControls />
						</Show>
						{props.children}
					</>
				)}
			>
				<Route path="/login" component={Login} />
				<Route
					component={(props) => (
						<div
							class={`flex m-2 gap-2 ${
								isTauri()
									? "h-[calc(100vh-3.5rem)] mt-0"
									: "h-[calc(100vh-1rem)]"
							}`}
						>
							<Splitter
								a={<Nav />}
								b={
									<main class="h-full p-8 rounded-xl bg-background-50 dark:bg-background-900">
										{props.children}
									</main>
								}
							/>
							<Members />
						</div>
					)}
				>
					<Route path="/" component={Index} />
					<Route path="/directory/:id" component={Thread} />
					<Route path="/settings" component={Settings} />
				</Route>
			</Router>
		</AuthProvider>
	);
}
