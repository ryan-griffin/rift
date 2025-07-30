import {
	Component,
	createContext,
	createSignal,
	JSX,
	useContext,
} from "solid-js";
import { resolveAddress, User } from "../apiUtils.ts";
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
	logout: () => void;
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
		const res = await fetch(
			`http://${address}/api/${endpoint}`,
			{
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(credentials),
			},
		);

		if (res.ok) {
			const data: AuthState = await res.json();
			setState(data);
			setStorageItem("auth", data);
			return true;
		}

		return false;
	};

	const logout = () => {
		setState({ token: null, user: null });
		deleteStorageItem("auth");
	};

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
	};

	return (
		<AuthContext.Provider value={contextValue}>
			{props.children}
		</AuthContext.Provider>
	);
};

export default AuthProvider;
