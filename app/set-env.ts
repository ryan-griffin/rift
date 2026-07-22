const env = { ...Bun.env };

const task = Bun.argv[2];

if (task === "dev" || task === "start") {
	env.HOST = env.APP_HOST;
	env.PORT = env.APP_PORT;
}

Bun.spawn(["bun", "run", "vinxi", task as string], {
	env,
	stdout: "inherit",
	stderr: "inherit",
});
