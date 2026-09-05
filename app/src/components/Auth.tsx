import {
	type Component,
	createContext,
	createSignal,
	type JSX,
	useContext,
} from "solid-js";
import { resolveAddress, type User } from "../apiUtils.ts";
import {
	deleteStorageItem,
	getStorageItem,
	setStorageItem,
} from "../storageUtils.ts";

interface LoginCredentials {
	username: string;
	password: string;
}

interface SignUpCredentials extends LoginCredentials {
	name: string;
}

interface AuthState {
	token: string | null;
	user: User | null;
}

interface AuthContextType extends AuthState {
	login: (credentials: LoginCredentials) => Promise<boolean>;
	signup: (credentials: SignUpCredentials) => Promise<boolean>;
	logout: () => Promise<boolean>;
	logoutAll: () => Promise<boolean>;
	clearAuth: () => void;
}

const LOGOUT_ATTEMPTS = 2;
const LOGOUT_TIMEOUT_MS = 3000;

const AuthContext = createContext<AuthContextType>();

export const useAuth = () => {
	const context = useContext(AuthContext);
	if (!context) {
		throw new Error("useAuth must be used within an AuthProvider");
	}
	return context;
};

const AuthProvider: Component<{ children: JSX.Element }> = (props) => {
	const [state, setState] = createSignal<AuthState>(
		getStorageItem("auth") || { token: null, user: null },
	);

	const authenticate = async (
		endpoint: "login" | "signup",
		credentials: LoginCredentials | SignUpCredentials,
	) => {
		const address = resolveAddress();
		if (!address) return false;

		try {
			const res = await fetch(`http://${address}/api/${endpoint}`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(credentials),
			});

			if (res.ok) {
				const data: AuthState = await res.json();
				setState(data);
				setStorageItem("auth", data);
				return true;
			}

			return false;
		} catch {
			return false;
		}
	};

	const clearAuth = () => {
		setState({ token: null, user: null });
		deleteStorageItem("auth");
	};

	const revoke = async (endpoint: "logout" | "logout-all") => {
		const token = state().token;
		const address = resolveAddress();

		// Remove the local credential immediately, but retain this in-memory
		// copy long enough to retry server-side revocation.
		clearAuth();

		// Without a token there is nothing to revoke, and without an
		// address there is no server to revoke with: local logout is
		// the whole story either way.
		if (!token || !address) return true;

		for (let attempt = 0; attempt < LOGOUT_ATTEMPTS; attempt += 1) {
			const controller = new AbortController();
			const timeout = setTimeout(
				() => controller.abort(),
				LOGOUT_TIMEOUT_MS,
			);

			try {
				const response = await fetch(
					`http://${address}/api/${endpoint}`,
					{
						method: "POST",
						headers: { Authorization: `Bearer ${token}` },
						keepalive: true,
						signal: controller.signal,
					},
				);

				if (response.ok || response.status === 401) return true;
				if (response.status < 500) return false;
			} catch {
				// Retry network failures and timeouts while the token remains in memory.
			} finally {
				clearTimeout(timeout);
			}
		}

		return false;
	};

	const logout = () => revoke("logout");
	const logoutAll = () => revoke("logout-all");

	const contextValue: AuthContextType = {
		get token() {
			return state().token;
		},
		get user() {
			return state().user;
		},
		login: (credentials) => authenticate("login", credentials),
		signup: (credentials) => authenticate("signup", credentials),
		logout,
		logoutAll,
		clearAuth,
	};

	return (
		<AuthContext.Provider value={contextValue}>
			{props.children}
		</AuthContext.Provider>
	);
};

export default AuthProvider;
