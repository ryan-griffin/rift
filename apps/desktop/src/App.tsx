import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@ui";

function App() {
	const [greetMsg, setGreetMsg] = createSignal("");
	const [name, setName] = createSignal("");

	async function greet() {
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		setGreetMsg(await invoke("greet", { name: name() }));
	}

	return (
		<main>
			<h1>Welcome to Tauri + Solid</h1>

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
			<Button />
		</main>
	);
}

export default App;
