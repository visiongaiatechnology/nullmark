// STATUS: DIAMANT VGT SUPREME

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(import.meta.dirname, "..");
const targets = [
  "src",
  "src-tauri/src",
  "src-tauri/Cargo.toml",
  "src-tauri/tauri.conf.json",
  "src-tauri/capabilities",
  "src-tauri/permissions"
];

const forbidden = [
  ["dangerouslySetInnerHTML", "React raw HTML injection"],
  ["tauri-plugin-shell", "native shell plugin"],
  ["@tauri-apps/plugin-shell", "frontend shell plugin"],
  ["tauri-plugin-fs", "native filesystem plugin"],
  ["@tauri-apps/plugin-fs", "frontend filesystem plugin"],
  ["tauri-plugin-http", "native HTTP plugin"],
  ["@tauri-apps/plugin-http", "frontend HTTP plugin"],
  ["child_process", "Node process spawning"],
  ["eval(", "dynamic JavaScript evaluation"],
  ["new Function(", "dynamic JavaScript function construction"],
  ["fetch(", "renderer network request"],
  ["XMLHttpRequest", "renderer network request"],
  ["WebSocket(", "renderer websocket connection"],
  ["std::process", "native process API"],
  ["Command::new", "native process spawning"],
  ["std::fs", "native filesystem API"],
  ["TcpStream", "native TCP networking"],
  ["reqwest", "native HTTP client"],
  ["std::net", "native networking"],
  ["unsafe {", "application unsafe block"],
  ["tauri_plugin_", "unreviewed native Tauri plugin"]
];

function filesUnder(entry) {
  const full = path.join(root, entry);
  if (!fs.existsSync(full)) return [];
  const stat = fs.statSync(full);
  if (stat.isFile()) return [full];
  return fs.readdirSync(full, { withFileTypes: true }).flatMap((item) => {
    const child = path.join(full, item.name);
    return item.isDirectory() ? filesUnder(path.relative(root, child)) : [child];
  });
}

const files = targets.flatMap(filesUnder);
const failures = [];

for (const lockfile of ["package-lock.json", "src-tauri/Cargo.lock"]) {
  if (!fs.existsSync(path.join(root, lockfile))) failures.push(`Required lockfile missing: ${lockfile}`);
}

const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
for (const lifecycle of ["preinstall", "install", "postinstall"]) {
  if (packageJson.scripts?.[lifecycle]) failures.push(`Lifecycle script is forbidden: ${lifecycle}`);
}
for (const [name, version] of Object.entries({ ...packageJson.dependencies, ...packageJson.devDependencies })) {
  if (typeof version !== "string" || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
    failures.push(`Direct npm dependency must use an exact version: ${name}@${version}`);
  }
}
for (const file of files) {
  const text = fs.readFileSync(file, "utf8");
  for (const [needle, reason] of forbidden) {
    if (text.includes(needle)) failures.push(`${path.relative(root, file)}: ${reason} (${needle})`);
  }
}

const conf = JSON.parse(fs.readFileSync(path.join(root, "src-tauri/tauri.conf.json"), "utf8"));
if (conf.version !== packageJson.version) failures.push("package.json and tauri.conf.json versions differ.");
const cargoToml = fs.readFileSync(path.join(root, "src-tauri/Cargo.toml"), "utf8");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (cargoVersion !== packageJson.version) failures.push("package.json and Cargo.toml versions differ.");
const prodCsp = conf?.app?.security?.csp ?? {};
if (prodCsp["default-src"] !== "'none'") failures.push("Production CSP default-src must remain 'none'.");
if (conf?.app?.withGlobalTauri !== false) failures.push("withGlobalTauri must remain false.");
if (conf?.app?.security?.freezePrototype !== true) failures.push("freezePrototype must remain true.");
if (conf?.app?.security?.dangerousDisableAssetCspModification !== false) failures.push("Tauri CSP modification must remain enabled.");
if (prodCsp["connect-src"] !== "ipc: http://ipc.localhost") failures.push("Production connect-src must allow only local Tauri IPC.");
for (const [directive, value] of Object.entries(prodCsp)) {
  const rendered = Array.isArray(value) ? value.join(" ") : String(value);
  if (rendered.includes("'unsafe-inline'") || rendered.includes("'unsafe-eval'")) {
    failures.push(`Production CSP ${directive} contains an unsafe source expression.`);
  }
}

const capability = JSON.parse(fs.readFileSync(path.join(root, "src-tauri/capabilities/main.json"), "utf8"));
const allowed = new Set(capability.permissions ?? []);
const expectedPermissions = new Set([
  "allow-analyze-text",
  "allow-sanitize-text",
  "allow-analyze-binary",
  "allow-sanitize-binary"
]);
if (allowed.size !== expectedPermissions.size || [...allowed].some((p) => !expectedPermissions.has(p))) {
  failures.push(`Main capability must contain only: ${[...expectedPermissions].join(", ")}`);
}
for (const permission of allowed) {
  if (/^(shell|fs|http|opener|process):/.test(permission)) {
    failures.push(`Forbidden high-risk capability: ${permission}`);
  }
}

if (failures.length) {
  console.error("NullMark security gate FAILED:\n- " + failures.join("\n- "));
  process.exit(1);
}

console.log(`NullMark security gate passed (${files.length} source/config files scanned).`);
