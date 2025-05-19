import { Component } from "solid-js";
import { isServer } from "solid-js/web";

const Index: Component = () => {
	return (
		<main class="p-10 m-10">
			<h1 class="text-3xl font-bold text-accent-500">
				Rift
			</h1>
			<div>{isServer ? "Server" : "Client"}</div>
		</main>
	);
};

export default Index;
