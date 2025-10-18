import { Component } from "solid-js";
import Button from "../components/Button.tsx";
import { useAuth } from "../components/Auth.tsx";
import { getBaseColor, updateTheme } from "../colorUtils.ts";

const Settings: Component = () => {
	const { logout } = useAuth();

	return (
		<>
			<h1>Settings</h1>
			<input
				type="color"
				value={getBaseColor()}
				onInput={(e) => updateTheme(e.currentTarget.value)}
			/>
			<Button
				variant="suggested"
				text="Logout"
				onClick={() => logout()}
			/>
		</>
	);
};

export default Settings;
