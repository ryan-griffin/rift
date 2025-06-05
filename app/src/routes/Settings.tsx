import { Component } from "solid-js";
import Button from "../components/Button.tsx";
import { useAuth } from "../components/Auth.tsx";

const Settings: Component = () => {
	const { logout } = useAuth();

	return (
		<div>
			<h1>Settings</h1>
			<Button
				variant="suggested"
				text="Logout"
				onClick={() => logout()}
			/>
		</div>
	);
};

export default Settings;
