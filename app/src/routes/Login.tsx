import { Component, createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { useAuth } from "../components/Auth.tsx";
import Button from "../components/Button.tsx";

const Login: Component = () => {
	const navigate = useNavigate();
	const { login } = useAuth();
	const [username, setUsername] = createSignal("");

	const handleSubmit = async (e: Event) => {
		e.preventDefault();
		if (!username()) return;

		const success = await login(username());
		if (success) {
			navigate("/");
		} else {
			alert("Login failed. Please try again.");
		}
	};

	return (
		<form onSubmit={handleSubmit}>
			<input
				placeholder="Username"
				value={username()}
				onInput={(e) => setUsername(e.currentTarget.value)}
			/>
			<Button type="submit" variant="suggested" text="Login" />
		</form>
	);
};

export default Login;
