import { readFileSync } from "node:fs";
import { gunzipSync } from "node:zlib";

export const indexedProcedure = {
  task: "create a new repository on github from the web ui",
  page: "https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-new-repository",
  sha256: "5af1ce3c57977984936b14a02740a819be9684d61f75915bf70054ad05a4fedd",
  firstStep: /select Create something new, then click New repository/u,
  finalAction: /Click Create repository\./u,
};

export const unrelatedProcedure = {
  page: "https://example.org/unrelated",
  body: "<main><h1>Archive an old repository</h1><ol><li>Find the old project.</li><li>Archive it.</li></ol></main>",
};

export function indexedProcedureBody() {
  return gunzipSync(readFileSync(new URL("../../fixtures/issue-1613/github-create-repository.html.gz", import.meta.url))).toString("utf8");
}
