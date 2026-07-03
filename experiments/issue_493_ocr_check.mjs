import { createWorker } from "tesseract.js";
import { writeFile } from "node:fs/promises";

const [, , imagePath, outputPath] = process.argv;

if (!imagePath || !outputPath) {
  console.error(
    "usage: node experiments/issue_493_ocr_check.mjs <image> <output-json>",
  );
  process.exit(2);
}

const worker = await createWorker("eng", 1, {
  langPath: "https://tessdata.projectnaptha.com/4.0.0_fast",
  logger: () => {},
});

try {
  const result = await worker.recognize(imagePath);
  const data = result?.data ?? {};
  const text = String(data.text ?? "").trim();
  const payload = {
    image: imagePath,
    language: "eng",
    tesseract: "7.0.0",
    confidence: typeof data.confidence === "number" ? data.confidence : null,
    text,
    checks: {
      containsEth2024Claim: /ETH\s+in\s+2024:\s*\$?1,?700/i.test(text),
      containsRepeatedEthClaims:
        (text.match(/ETH\s+in\s+20(21|22|23|24|25|26)/gi) ?? []).length >= 6,
    },
  };
  await writeFile(outputPath, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
} finally {
  await worker.terminate();
}
