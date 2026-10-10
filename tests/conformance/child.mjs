// SPDX-License-Identifier: MIT
// One conformance file on the runtime: node child.mjs CACHE NAME
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { PyRuntime } from "../../dist/pyruntime.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const [cache, name] = process.argv.slice(2);
const rt = new PyRuntime();
const put = (dir, mount) => {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) put(full, mount + "/" + ent.name);
    else if (ent.name.endsWith(".py")) rt.writeFile(mount + "/" + ent.name, fs.readFileSync(full, "utf8"));
  }
};
put(path.join(cache, "test"), "/t/test");
put(path.join(HERE, "support/test/support"), "/t/test/support");
rt.addPath("/t");
rt.addArg("driver.py");
rt.addArg("test." + name);
rt.echo = true;
const ok = rt.run(fs.readFileSync(path.join(HERE, "driver.py"), "utf8"));
if (!ok) {
  console.error(rt.traceback);
  process.exit(1);
}
