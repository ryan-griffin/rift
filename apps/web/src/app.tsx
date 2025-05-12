import { createSignal } from "solid-js";

export default function App() {
	const [count, setCount] = createSignal(0);

	return (
		<main>
			<h1>Hello world!</h1>
			<button type="button" onClick={() => setCount(count() + 1)}>
				Clicks: {count()}
			</button>
		</main>
	);
}
