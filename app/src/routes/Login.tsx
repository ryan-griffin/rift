import { Component, createSignal } from "solid-js";
import { createStore } from "solid-js/store";
import { useNavigate } from "@solidjs/router";
import { useAuth } from "../components/Auth.tsx";
import Button from "../components/Button.tsx";
import Input from "../components/Input.tsx";
import SegmentGroup from "../components/SegmentGroup.tsx";

const Login: Component = () => {
	const navigate = useNavigate();
	const { login, signup } = useAuth();

	const [segment, setSegment] = createSignal<"Login" | "Sign Up">("Login");

	const [state, setState] = createStore({ username: "", password: "" });

	const handleSubmit = async (e: Event) => {
		e.preventDefault();
		switch (segment()) {
			case "Login":
				if (state.username && state.password && await login(state)) {
					navigate("/");
				} else {
					alert("Login failed. Please try again.");
				}
				break;
			case "Sign Up":
				if (
					state.username && state.password &&
					await signup({ name: state.username, ...state })
				) {
					navigate("/");
				} else {
					alert("Sign up failed. Please try again.");
				}
				break;
		}
	};

	return (
		<main class="flex items-center justify-center h-screen">
			<form
				onSubmit={handleSubmit}
				class="flex flex-col p-8 gap-4 rounded-2xl bg-background-50 dark:bg-background-900"
			>
				<SegmentGroup
					value={segment()}
					setValue={setSegment}
					items={["Login", "Sign Up"]}
				/>
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
				<Button type="submit" variant="suggested" text={segment()} />
			</form>
		</main>
	);
};

export default Login;
