import { createSignal } from "solid-js";
import "./styles.css";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { isServer } from "solid-js/web";
import WindowControls from "./components/WindowControls.tsx";

export default function App() {
	const [greetMsg, setGreetMsg] = createSignal("");
	const [name, setName] = createSignal("");

	async function greet() {
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		if (isTauri()) {
			setGreetMsg(await invoke("greet", { name: name() }));
		}
	}

	return (
		<main>
			{isTauri() && <WindowControls />}
			<div>{isServer ? "Server" : "Client"}</div>

			<h1 class="text-2xl text-blue-500">Welcome to Tauri + Solid</h1>
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
				<button type="submit">Greet</button>
			</form>
			<p>{greetMsg()}</p>
		</main>
	);
}
