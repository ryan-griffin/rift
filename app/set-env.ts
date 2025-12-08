const env = { ...Deno.env.toObject() };

const [task] = Deno.args;

if (task === "dev" || task === "start") {
	env.HOST = env.APP_HOST;
	env.PORT = env.APP_PORT;
}

if (task === "dev" || task === "build") {
	env.VITE_API_ADDRESS = `${env.API_HOST}:${env.API_PORT}`;
}

new Deno.Command("deno", {
	args: ["run", "-A", "npm:vinxi", task],
	env,
	stdout: "inherit",
	stderr: "inherit",
}).spawn();
