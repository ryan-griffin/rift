import { useNavigate } from "@solidjs/router";
import { isTauri } from "@tauri-apps/api/core";
import { type Component, createSignal, Show } from "solid-js";
import { createStore } from "solid-js/store";
import { useAuth } from "../components/Auth.tsx";
import Button from "../components/Button.tsx";
import Input from "../components/Input.tsx";
import SegmentGroup from "../components/SegmentGroup.tsx";
import { getStorageItem, setStorageItem } from "../storageUtils.ts";

const AddressField: Component = () => {
	const [address, setAddress] = createSignal<string>(
		getStorageItem("address") || "",
	);

	return (
		<Input
			placeholder="Server Address"
			value={address()}
			onInput={(e) => {
				setAddress(e.currentTarget.value);
				setStorageItem("address", e.currentTarget.value);
			}}
		/>
	);
};

const Login: Component = () => {
	const navigate = useNavigate();
	const { login, signup } = useAuth();

	const [segment, setSegment] = createSignal<"Login" | "Sign Up">("Login");

	const [state, setState] = createStore({ username: "", password: "" });

	const handleSubmit = async (e: Event) => {
		e.preventDefault();
		switch (segment()) {
			case "Login":
				if (
					state.username &&
					state.password &&
					(!isTauri() || getStorageItem<string>("address")) &&
					(await login(state))
				) {
					navigate("/");
				} else {
					alert("Login failed. Please try again.");
				}
				break;
			case "Sign Up":
				if (
					state.username &&
					state.password &&
					(!isTauri() || getStorageItem<string>("address")) &&
					(await signup({ name: state.username, ...state }))
				) {
					navigate("/");
				} else {
					alert("Sign up failed. Please try again.");
				}
				break;
		}
	};

	return (
		<div class="h-full flex items-center justify-center">
			<form
				onSubmit={handleSubmit}
				class="flex flex-col p-8 gap-4 rounded-2xl bg-background-50 dark:bg-background-900"
			>
				<SegmentGroup
					value={segment()}
					setValue={setSegment}
					items={["Login", "Sign Up"]}
				/>
				<Show when={isTauri()}>
					<AddressField />
				</Show>
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
		</div>
	);
};

export default Login;
