import fs from "node:fs";
import path from "node:path";

const releaseDir = path.resolve(process.argv[2] || "release");

const rules = [
  {
    from: /^formal-ai-desktop-linux-x86_64-(.+\.AppImage)$/,
    to: "formal-ai-desktop-linux-x64-$1",
  },
  {
    from: /^formal-ai-desktop-linux-x86_64-(.+\.AppImage\.blockmap)$/,
    to: "formal-ai-desktop-linux-x64-$1",
  },
  {
    from: /^formal-ai-desktop-linux-amd64-(.+\.deb)$/,
    to: "formal-ai-desktop-linux-x64-$1",
  },
];

if (!fs.existsSync(releaseDir)) {
  console.log(`Desktop release directory not found: ${releaseDir}`);
  process.exit(0);
}

let renamed = 0;
for (const entry of fs.readdirSync(releaseDir, { withFileTypes: true })) {
  if (!entry.isFile()) {
    continue;
  }

  for (const rule of rules) {
    if (!rule.from.test(entry.name)) {
      continue;
    }

    const normalizedName = entry.name.replace(rule.from, rule.to);
    if (normalizedName === entry.name) {
      continue;
    }

    const source = path.join(releaseDir, entry.name);
    const target = path.join(releaseDir, normalizedName);
    if (fs.existsSync(target)) {
      throw new Error(`Cannot normalize ${entry.name}: target ${normalizedName} already exists`);
    }

    fs.renameSync(source, target);
    console.log(`Normalized desktop artifact: ${entry.name} -> ${normalizedName}`);
    renamed += 1;
    break;
  }
}

let metadataFilesUpdated = 0;
const metadataReplacements = [
  ["formal-ai-desktop-linux-x86_64-", "formal-ai-desktop-linux-x64-"],
  ["formal-ai-desktop-linux-amd64-", "formal-ai-desktop-linux-x64-"],
];
for (const entry of fs.readdirSync(releaseDir, { withFileTypes: true })) {
  if (!entry.isFile() || !/^latest(?:-mac|-linux)?\.yml$/.test(entry.name)) {
    continue;
  }

  const target = path.join(releaseDir, entry.name);
  let content = fs.readFileSync(target, "utf8");
  let normalized = content;
  for (const [from, to] of metadataReplacements) {
    normalized = normalized.split(from).join(to);
  }
  if (normalized !== content) {
    fs.writeFileSync(target, normalized);
    console.log(`Normalized desktop update metadata: ${entry.name}`);
    metadataFilesUpdated += 1;
  }
}

if (renamed === 0) {
  console.log(`No desktop artifact names required normalization in ${releaseDir}`);
}
if (metadataFilesUpdated === 0) {
  console.log(`No desktop update metadata required normalization in ${releaseDir}`);
}
