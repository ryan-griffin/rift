import { Component } from "solid-js";
import Button from "../components/Button.tsx";
import { useAuth } from "../components/Auth.tsx";
import { useNavigate } from "@solidjs/router";

const Settings: Component = () => {
	const navigate = useNavigate();
	const { logout } = useAuth();

	return (
		<div>
			<h1>Settings</h1>
			<Button
				variant="suggested"
				text="Logout"
				onClick={() => {
					logout();
					navigate("/login");
				}}
			/>
		</div>
	);
};

export default Settings;
