import { useNavigate } from "@solidjs/router";
import type { Component } from "solid-js";
import Settings from "../assets/settings.svg";
import { useAuth } from "./Auth.tsx";
import Avatar from "./Avatar.tsx";
import Button from "./Button.tsx";
import Directory from "./Directory.tsx";

const Nav: Component = () => {
	const navigate = useNavigate();
	const { user } = useAuth();

	return (
		<nav class="relative flex-1 min-w-65">
			<Directory />
			<div class="absolute bottom-0 w-full flex p-2 gap-2 items-center justify-between rounded-2xl bg-background-50 dark:bg-background-900">
				<div class="flex gap-2 items-center">
					<Avatar fallback={user?.username[0]} className="w-10" />
					{user?.username}
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
