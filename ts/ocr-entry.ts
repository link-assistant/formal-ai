import { createWorker } from "tesseract.js";

const DEFAULT_LANGUAGE = "eng";
const DEFAULT_LANG_PATH = "https://tessdata.projectnaptha.com/4.0.0_fast";

let workerPromise = null;

function toMessage(error) {
  return error && error.message ? error.message : String(error || "OCR failed");
}

async function getWorker(options = {}) {
  if (workerPromise) {
    return workerPromise;
  }
  const language = options.language || options.lang || DEFAULT_LANGUAGE;
  workerPromise = createWorker(language, 1, {
    langPath: options.langPath || DEFAULT_LANG_PATH,
    logger:
      typeof options.logger === "function"
        ? options.logger
        : () => undefined,
  }).catch((error) => {
    workerPromise = null;
    throw error;
  });
  return workerPromise;
}

async function recognizeImage(image, options = {}) {
  const worker = await getWorker(options);
  try {
    const result = await worker.recognize(image);
    const data = result && result.data ? result.data : {};
    return {
      text: String(data.text || "").trim(),
      confidence:
        typeof data.confidence === "number" && Number.isFinite(data.confidence)
          ? data.confidence
          : null,
    };
  } catch (error) {
    throw new Error(toMessage(error));
  }
}

window.FormalAiOcr = {
  VERSION: "7.0.0",
  DATA_WARNING:
    "Downloads about 6 MB on first use: OCR wrapper, worker, WebAssembly core, and English traineddata.",
  recognizeImage,
};
