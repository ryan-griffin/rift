import { Component, createContext, JSX, useContext } from "solid-js";
import { action, query } from "@solidjs/router";
import { useAuth } from "./Auth.tsx";
import { resolveAddress } from "../apiUtils.ts";

type GetApi = <T>(url: string) => Promise<T>;
type PostApi = <T>(url: string, body: unknown) => Promise<T>;

interface ApiContextType {
	getApi: GetApi;
	postApi: PostApi;
}

const ApiContext = createContext<ApiContextType>();

export const useApi = () => {
	const context = useContext(ApiContext);
	if (!context) {
		throw new Error(
			"useApi must be used within an ApiProvider",
		);
	}
	return context;
};

const ApiProvider: Component<{ children: JSX.Element }> = (
	props,
) => {
	const { token, logout } = useAuth();

	const api = async <T,>(
		method: "GET" | "POST" = "GET",
		url: string,
		body?: unknown,
	): Promise<T> => {
		const address = resolveAddress();
		const options: RequestInit = {
			method,
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${token}`,
			},
		};

		if (method === "POST" && body !== undefined) {
			options.body = JSON.stringify(body);
		}

		const res = await fetch(`http://${address}/api${url}`, options);
		if (res.status === 401) logout();
		return await res.json();
	};

	const getApi = query((url: string) => api("GET", url), "getApi") as GetApi;

	const postApi = action((url: string, body: unknown) =>
		api("POST", url, body)
	) as PostApi;

	return (
		<ApiContext.Provider value={{ getApi, postApi }}>
			{props.children}
		</ApiContext.Provider>
	);
};

export default ApiProvider;
