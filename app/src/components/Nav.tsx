import { Component } from "solid-js";
import Directory from "./Directory.tsx";
import Button from "./Button.tsx";
import Settings from "../assets/settings.svg";

const Nav: Component = () => {
	return (
		<nav class="relative h-screen bg-background-50 dark:bg-background-900">
			<div class="p-4 font-bold">Rift</div>
			<Directory />
			<div class="absolute bottom-0 w-[calc(100%-2rem)] flex p-2 gap-2 m-4 items-center justify-between rounded-xl bg-background-100 dark:bg-background-800 shadow-sm">
				<div class="flex gap-2 items-center">
					<div class="w-10 h-10 rounded-full bg-background-200 dark:bg-background-700" />
					User
				</div>
				<Button variant="regular" icon={<Settings />} />
			</div>
		</nav>
	);
};

export default Nav;
