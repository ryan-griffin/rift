import { Component, createEffect } from "solid-js";
import { createAsync, useNavigate } from "@solidjs/router";
import { DirectoryNode } from "../apiUtils.ts";
import { useApi } from "../components/Api.tsx";
import { getStorageItem } from "../storageUtils.ts";

const Index: Component = () => {
	const navigate = useNavigate();
	const { getApi } = useApi();

	const lastThread = getStorageItem<number>("lastThread");
	if (lastThread) {
		navigate(`/thread/${lastThread}`);
	} else {
		const directory = createAsync<DirectoryNode[]>(() =>
			getApi("/directory/1")
		);

		createEffect(() => {
			const dir = directory();
			if (!dir || dir.length === 0) return;

			for (const node of dir) {
				if (node.type === "thread") {
					navigate(`/thread/${node.id}`);
					return;
				}
			}
		});
	}

	return null;
};

export default Index;
