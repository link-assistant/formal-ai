import { readFileSync } from "node:fs";

const root = new URL("../../fixtures/procedure-publishers/", import.meta.url);
const api = "https://en.wikibooks.org/w/api.php";
const escapeTitle = title => encodeURIComponent(title).replace(/[()]/gu, character =>
  "%" + character.codePointAt(0).toString(16).toUpperCase());
export const bookPage = title =>
  `${api}?action=parse&page=${escapeTitle(title)}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`;
export const bookSearch = (query, namespace = "") =>
  `${api}?action=query&list=search&srsearch=${encodeURIComponent(query)}&srlimit=5&format=json&origin=*${namespace}`;

export const procedureBooks = [
  {
    task: "prepare basic bread dough",
    direct: bookPage("Prepare Basic Bread Dough"),
    search: bookSearch("prepare basic bread dough"),
    scoped: bookSearch("basic bread dough", "&srnamespace=102"),
    page: bookPage("Cookbook:Basic Bread Dough"),
    files: ["prepare-basic-bread-dough", "basic-bread-dough-cookbook", "basic-bread-dough-parse"],
    firstStep: /^Add ingredients to a 2-\s*pound capacity/u,
  },
  {
    task: "prepare pancakes north american",
    direct: bookPage("Prepare Pancakes North American"),
    search: bookSearch("prepare pancakes north american"),
    scoped: bookSearch("pancakes north american", "&srnamespace=102"),
    page: bookPage("Cookbook:Pancakes (North American)"),
    files: ["prepare-pancakes-north-american", "pancakes-north-american-cookbook", "pancakes-north-american-parse"],
    firstStep: /^In a large bowl, mix dry ingredients/u,
  },
];

export function procedureBookCaptures(book, wrongTitle = false) {
  const [search, scoped, page] = book.files.map(name => readFileSync(new URL(name + ".json", root), "utf8"));
  const initial = wrongTitle ? JSON.stringify({
    parse: {
      title: "Unrelated book",
      text: { "*": "<ol><li>Repeat an unrelated instruction.</li><li>Store the unrelated result.</li></ol>" },
    },
  }) : JSON.stringify({ error: { code: "missingtitle" } });
  return new Map([
    [book.direct, initial],
    [book.search, search],
    [book.scoped, scoped],
    [book.page, page],
  ]);
}
