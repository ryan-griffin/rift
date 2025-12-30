import { Component, createEffect } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { useQuery } from "@tanstack/solid-query";
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
		const directory = useQuery(() => ({
			queryKey: ["directory", 1],
			queryFn: () => getApi<DirectoryNode[]>("/directory/1"),
		}));

		createEffect(() => {
			const dir = directory.data;
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
