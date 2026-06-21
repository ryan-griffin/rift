import { useNavigate } from "@solidjs/router";
import { type Component, createEffect, type JSX, Show } from "solid-js";
import { useAuth } from "./Auth.tsx";

const ProtectedRoute: Component<{ children: JSX.Element }> = (props) => {
	const auth = useAuth();
	const navigate = useNavigate();

	createEffect(() => {
		if (!auth.token || !auth.user) navigate("/login");
	});

	return <Show when={auth.token && auth.user}>{props.children}</Show>;
};

export default ProtectedRoute;
