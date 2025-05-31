import { Component } from "solid-js";
import Directory from "./Directory.tsx";
import Button from "./Button.tsx";
import Settings from "../assets/settings.svg";
import { useNavigate } from "@solidjs/router";
import { useAuth } from "./Auth.tsx";

const Nav: Component = () => {
	const navigate = useNavigate();
	const { user } = useAuth();

	return (
		<nav class="relative h-full">
			<Directory />
			<div class="absolute bottom-0 w-full flex p-2 gap-2 items-center justify-between rounded-xl bg-background-50 dark:bg-background-900">
				<div class="flex gap-2 items-center">
					<div class="w-10 h-10 rounded-full bg-background-100 dark:bg-background-800" />
					{user?.name}
				</div>
				<Button
					variant="flat"
					icon={<Settings />}
					onClick={() => navigate("/settings")}
				/>
			</div>
		</nav>
	);
};

export default Nav;
