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

export default function App() {
	onMount(() => {
		initializeTheme();
	});

	return (
		<>
			<Show when={isTauri()}>
				<WindowControls />
			</Show>
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
							<Router>
								<Route path="/" component={Index} />
							</Router>
						</main>
					}
				/>
				<Members />
			</div>
		</>
	);
}
