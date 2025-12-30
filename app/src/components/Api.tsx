import { Component, createContext, JSX, useContext } from "solid-js";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
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
	const queryClient = new QueryClient();
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

	const getApi = <T,>(url: string) => api<T>("GET", url);

	const postApi = <T,>(url: string, body: unknown) => api<T>("POST", url, body);

	return (
		<QueryClientProvider client={queryClient}>
			<ApiContext.Provider value={{ getApi, postApi }}>
				{props.children}
			</ApiContext.Provider>
		</QueryClientProvider>
	);
};

export default ApiProvider;
