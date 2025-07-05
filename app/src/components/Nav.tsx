import { Component } from "solid-js";
import Directory from "./Directory.tsx";
import Button from "./Button.tsx";
import Settings from "../assets/settings.svg";
import { useNavigate } from "@solidjs/router";
import { useAuth } from "./Auth.tsx";
import Avatar from "./Avatar.tsx";

const Nav: Component = () => {
	const navigate = useNavigate();
	const { user } = useAuth();

	return (
		<nav class="relative h-full">
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
