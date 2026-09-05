import type { Component } from "solid-js";
import { getBaseColor, updateTheme } from "../colorUtils.ts";
import { useAuth } from "../components/Auth.tsx";
import Button from "../components/Button.tsx";

const Settings: Component = () => {
	const { logout, logoutAll } = useAuth();

	const handleLogout = async (revoke: () => Promise<boolean>) => {
		if (!(await revoke())) {
			alert("The server could not confirm logout. Please try again.");
		}
	};

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
				onClick={() => void handleLogout(logout)}
			/>
			<Button
				variant="suggested"
				text="Logout all devices"
				onClick={() => void handleLogout(logoutAll)}
			/>
		</>
	);
};

export default Settings;
