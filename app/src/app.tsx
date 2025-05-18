import { createSignal, onMount } from "solid-js";
import "./styles.css";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { isServer } from "solid-js/web";
import WindowControls from "./components/WindowControls.tsx";
import Button from "./components/Button.tsx";
import { initializeTheme } from "./colorUtils.ts";
import Tree from "./components/Tree.tsx";
import Splitter from "./components/Spiltter.tsx";

export default function App() {
	const [greetMsg, setGreetMsg] = createSignal("");
	const [name, setName] = createSignal("");

	async function greet() {
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		if (isTauri()) {
			setGreetMsg(await invoke("greet", { name: name() }));
		}
	}

	onMount(() => {
		initializeTheme();
	});

	return (
		<main class="flex">
			{isTauri() && <WindowControls />}
			<Splitter
				a={<Tree />}
				b={
					<div class="p-10 m-10">
						<div>{isServer ? "Server" : "Client"}</div>

						<h1 class="text-2xl text-accent-500">
							Welcome to Tauri + Solid
						</h1>
						<form
							onSubmit={(e) => {
								e.preventDefault();
								greet();
							}}
						>
							<input
								id="greet-input"
								onChange={(e) => setName(e.currentTarget.value)}
								placeholder="Enter a name..."
							/>
							<Button
								type="submit"
								variant="suggested"
								text="Greet"
							/>
						</form>
						<p>{greetMsg()}</p>
					</div>
				}
			/>
		</main>
	);
}
