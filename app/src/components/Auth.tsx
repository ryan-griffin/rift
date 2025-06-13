import {
	Component,
	createContext,
	createSignal,
	JSX,
	useContext,
} from "solid-js";
import { isServer } from "solid-js/web";
import { getCookie, setCookie } from "vinxi/http";
import { User } from "../apiUtils.ts";

interface AuthState {
	token: string | null;
	user: User | null;
}

interface AuthContextType extends AuthState {
	login: (username: string) => Promise<boolean>;
	logout: () => void;
}

const setServerAuthCookie = (value: AuthState) => {
	"use server";
	setCookie("auth", JSON.stringify(value), {
		httpOnly: false,
		maxAge: 60 * 60 * 24,
	});
};

const getServerAuthCookie = (): AuthState | null => {
	"use server";
	const cookie = getCookie("auth");
	return cookie ? JSON.parse(cookie) : null;
};

const deleteServerAuthCookie = () => {
	"use server";
	setCookie("auth", "", { maxAge: -1 });
};

const setAuthCookie = (value: AuthState) => {
	if (isServer) {
		setServerAuthCookie(value);
	} else {
		document.cookie = `auth=${JSON.stringify(value)}; path=/; max-age=${
			60 * 60 * 24
		};`;
	}
};

const getAuthCookie = (): AuthState | null => {
	if (isServer) {
		return getServerAuthCookie();
	} else {
		const cookies = document.cookie.split(";");
		const authCookie = cookies.find((cookie) =>
			cookie.trim().startsWith("auth=")
		);
		if (!authCookie) return null;

		const value = authCookie.split("=")[1];
		return value ? JSON.parse(decodeURIComponent(value)) : null;
	}
};

const deleteAuthCookie = () => {
	if (isServer) {
		deleteServerAuthCookie();
	} else {
		document.cookie = "auth=; path=/; max-age=-1;";
	}
};

const AuthContext = createContext<AuthContextType>();

export const useAuth = () => {
	const context = useContext(AuthContext);
	if (!context) {
		throw new Error("useAuth must be used within an AuthProvider");
	}
	return context;
};

export const AuthProvider: Component<{ children: JSX.Element }> = (props) => {
	const [state, setState] = createSignal<AuthState>(
		getAuthCookie() || { token: null, user: null },
	);

	const login = async (username: string) => {
		const res = await fetch(
			`http://${import.meta.env.VITE_ADDRESS}/api/login`,
			{
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ username }),
			},
		);

		if (res.ok) {
			const data: AuthState = await res.json();
			setState(data);
			setAuthCookie(data);
			return true;
		}

		return false;
	};

	const logout = () => {
		setState({ token: null, user: null });
		deleteAuthCookie();
	};

	const contextValue: AuthContextType = {
		get token() {
			return state().token;
		},
		get user() {
			return state().user;
		},
		login,
		logout,
	};

	return (
		<AuthContext.Provider value={contextValue}>
			{props.children}
		</AuthContext.Provider>
	);
};
