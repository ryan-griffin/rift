import { Component } from "solid-js";
import { createStore } from "solid-js/store";
import { useNavigate } from "@solidjs/router";
import { useAuth } from "../components/Auth.tsx";
import Button from "../components/Button.tsx";
import Input from "../components/Input.tsx";

const Login: Component = () => {
	const navigate = useNavigate();
	const { login } = useAuth();

	const [state, setState] = createStore({ username: "", password: "" });

	const handleSubmit = async (e: Event) => {
		e.preventDefault();
		if (state.username && await login(state.username)) {
			navigate("/");
		} else {
			alert("Login failed. Please try again.");
		}
	};

	return (
		<main class="flex items-center justify-center h-screen">
			<form
				onSubmit={handleSubmit}
				class="flex flex-col p-8 gap-4 rounded-2xl bg-background-50 dark:bg-background-900"
			>
				<Input
					placeholder="Username"
					value={state.username}
					onInput={(e) => setState("username", e.currentTarget.value)}
				/>
				<Input
					placeholder="Password"
					type="password"
					value={state.password}
					onInput={(e) => setState("password", e.currentTarget.value)}
				/>
				<Button type="submit" variant="suggested" text="Login" />
			</form>
		</main>
	);
};

export default Login;
