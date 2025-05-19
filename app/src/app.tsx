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
		<div class="flex">
			<Show when={isTauri()}>
				<WindowControls />
			</Show>
			<Splitter
				a={<Nav />}
				b={
					<Router>
						<Route path="/" component={Index} />
					</Router>
				}
			/>
			<Members />
		</div>
	);
}
