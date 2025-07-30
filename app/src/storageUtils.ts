import { isServer } from "solid-js/web";
import { getCookie, setCookie } from "vinxi/http";
import { isTauri } from "@tauri-apps/api/core";

const MAX_COOKIE_AGE = 60 * 60 * 24 * 30;

const getServerCookie = <T>(key: string): T | null => {
	"use server";
	const cookie = getCookie(key);
	return cookie ? JSON.parse(cookie) : null;
};

export const getStorageItem = <T>(key: string): T | null => {
	if (isServer) {
		return getServerCookie<T>(key);
	} else if (isTauri()) {
		const item = localStorage.getItem(key);
		return item ? JSON.parse(item) : null;
	} else {
		const cookies = document.cookie.split(";");
		const matchingCookie = cookies.find((cookie) =>
			cookie.trim().startsWith(`${key}=`)
		);
		if (!matchingCookie) return null;

		const value = matchingCookie.split("=")[1];
		return value ? JSON.parse(decodeURIComponent(value)) : null;
	}
};

const setServerCookie = <T>(key: string, value: T) => {
	"use server";
	setCookie(key, JSON.stringify(value), {
		httpOnly: false,
		maxAge: MAX_COOKIE_AGE,
	});
};

export const setStorageItem = <T>(key: string, value: T) => {
	if (isServer) {
		setServerCookie(key, value);
	} else if (isTauri()) {
		localStorage.setItem(key, JSON.stringify(value));
	} else {
		document.cookie = `${key}=${
			encodeURIComponent(JSON.stringify(value))
		}; path=/; max-age=${MAX_COOKIE_AGE};`;
	}
};

const deleteServerCookie = (key: string) => {
	"use server";
	setCookie(key, "", { maxAge: -1 });
};

export const deleteStorageItem = (key: string) => {
	if (isServer) {
		deleteServerCookie(key);
	} else if (isTauri()) {
		localStorage.removeItem(key);
	} else {
		document.cookie = `${key}=; path=/; max-age=-1;`;
	}
};
