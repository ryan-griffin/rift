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
	clearAuthIfCurrent: (token: string | null) => void;
}

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

	const clearAuthIfCurrent = (token: string | null) => {
		// Delayed responses and socket closures must not clear a newer login.
		if (state().token === token) clearAuth();
	};

	const revoke = async (endpoint: "logout" | "logout-all") => {
		const token = state().token;
		const address = resolveAddress();

		if (!token) {
			clearAuth();
			return true;
		}
		if (!address) return false;

		try {
			const response = await fetch(`http://${address}/api/${endpoint}`, {
				method: "POST",
				headers: { Authorization: `Bearer ${token}` },
				keepalive: true,
			});

			// A 401 confirms only that this token is invalid, not that other
			// sessions were revoked by logout-all.
			const revoked =
				response.ok ||
				(endpoint === "logout" && response.status === 401);
			if (revoked) clearAuthIfCurrent(token);
			return revoked;
		} catch {
			return false;
		}
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
		clearAuthIfCurrent,
	};

	return (
		<AuthContext.Provider value={contextValue}>
			{props.children}
		</AuthContext.Provider>
	);
};

export default AuthProvider;
