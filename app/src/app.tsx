import "./styles.css";
import { onMount, Show } from "solid-js";
import { Route, Router } from "@solidjs/router";
import { isTauri } from "@tauri-apps/api/core";
import { initializeTheme } from "./colorUtils.ts";
import WindowControls from "./components/WindowControls.tsx";
import Splitter from "./components/Spiltter.tsx";
import Directory from "./components/Directory.tsx";
import Index from "./routes/Index.tsx";

export default function App() {
	onMount(() => {
		initializeTheme();
	});

	return (
		<>
			<Show when={isTauri()}>
				<WindowControls />
			</Show>
			<Splitter
				a={<Directory />}
				b={
					<Router>
						<Route path="/" component={Index} />
					</Router>
				}
			/>
		</>
	);
}
