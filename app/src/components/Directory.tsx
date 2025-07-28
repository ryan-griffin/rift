import { Component, For, Show, Suspense } from "solid-js";
import { A, createAsync } from "@solidjs/router";
import { createTreeCollection, TreeView } from "@ark-ui/solid/tree-view";
import ChevronRight from "../assets/chevron-right.svg";
import MessageSquareText from "../assets/message-square-text.svg";
import { DirectoryNode, useGetApi } from "../apiUtils.ts";
import { useAuth } from "./Auth.tsx";

interface TreeNode {
	id: number;
	name: string;
	type: "folder" | "thread";
	children?: TreeNode[];
}

const buildDirectory = (nodes: DirectoryNode[]): TreeNode | undefined => {
	const nodeMap = new Map<number, TreeNode>();

	for (const node of nodes) {
		nodeMap.set(node.id, {
			id: node.id,
			name: node.name,
			type: node.type,
		});
	}

	const root = nodes.find((node) => node.parent_id === null);
	if (!root) return;

	for (const node of nodes) {
		if (node.parent_id !== null) {
			const parent = nodeMap.get(node.parent_id);
			const child = nodeMap.get(node.id);

			if (parent && child) {
				if (!parent.children) parent.children = [];
				parent.children.push(child);
			}
		}
	}

	return nodeMap.get(root.id);
};

const DirectoryItem: Component<TreeView.NodeProviderProps<TreeNode>> = (
	props,
) => {
	const { node, indexPath } = props;
	const nodeClass =
		"flex p-2 gap-2 rounded-lg transition-colors duration-100 cursor-pointer select-none";

	return (
		<TreeView.NodeProvider node={node} indexPath={indexPath}>
			<Show
				when={node.type === "folder"}
				fallback={
					<A
						href={`directory/${node.id}`}
						class={nodeClass}
						inactiveClass="hover:bg-background-200 dark:hover:bg-background-800"
						activeClass="bg-accent-100 dark:bg-accent-800"
					>
						<MessageSquareText />
						{node.name}
					</A>
				}
			>
				<TreeView.Branch class="flex flex-col gap-1">
					<TreeView.BranchControl>
						<TreeView.BranchText
							class={`${nodeClass} hover:bg-background-200 dark:hover:bg-background-800 group text-background-400 dark:text-background-500 font-bold`}
						>
							<ChevronRight class="group-data-[state=open]:rotate-90 transition-transform duration-200" />
							{node.name}
						</TreeView.BranchText>
					</TreeView.BranchControl>
					<TreeView.BranchContent class="flex gap-3 overflow-hidden data-[state=closed]:animate-[slideUp_200ms] data-[state=open]:animate-[slideDown_200ms]">
						<TreeView.BranchIndentGuide class="border-l-2 border-background-200 dark:border-background-900 ml-4" />
						<div class="flex flex-col gap-1 grow">
							<For each={node.children}>
								{(child, index) => (
									<DirectoryItem
										node={child}
										indexPath={[...indexPath, index()]}
									/>
								)}
							</For>
						</div>
					</TreeView.BranchContent>
				</TreeView.Branch>
			</Show>
		</TreeView.NodeProvider>
	);
};

const Directory = () => {
	const { token } = useAuth();
	const nodes = createAsync<DirectoryNode[]>(() =>
		useGetApi(token!, "/directory/1")
	);

	return (
		<Suspense fallback={<p>Loading...</p>}>
			<Show when={nodes()}>
				{(nodesData) => {
					const directory = buildDirectory(nodesData());
					if (!directory) return;

					return (
						<TreeView.Root
							class="h-full pb-14 overflow-y-auto"
							collection={createTreeCollection<TreeNode>({
								nodeToValue: (node) => node.id.toString(),
								nodeToString: (node) => node.name,
								rootNode: directory,
							})}
						>
							<TreeView.Tree class="flex flex-col gap-1">
								<For each={directory.children}>
									{(node, index) => (
										<DirectoryItem
											node={node}
											indexPath={[index()]}
										/>
									)}
								</For>
							</TreeView.Tree>
						</TreeView.Root>
					);
				}}
			</Show>
		</Suspense>
	);
};

export default Directory;
