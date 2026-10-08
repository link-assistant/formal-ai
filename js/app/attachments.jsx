// Composer attachments: file classification, reading, optional OCR and how
// attachment content is folded into the prompt and the memory log.

import React from "react";
import { withAssetVersion } from "./application-constants.jsx";

const { useCallback } = React;

const OCR_BUNDLE_FILENAME = "ocr.bundle.js";

export const OCR_DOWNLOAD_WARNING =
  "Downloads about 6 MB on first use: OCR wrapper, worker, WebAssembly core, and English traineddata.";

let ocrBundlePromise = null;

function isImageAttachment(file) {
  if (!file) return false;
  const type = String(file.type || "").toLowerCase();
  if (type.startsWith("image/")) return true;
  return /\.(png|jpe?g|webp|gif|bmp|tiff?)$/i.test(String(file.name || ""));
}

const TEXT_ATTACHMENT_CONTEXT_LIMIT = 12000;

function isTextAttachment(file) {
  if (!file) return false;
  const type = String(file.type || "").toLowerCase();
  if (type.startsWith("text/")) return true;
  if (
    [
      "application/json",
      "application/ld+json",
      "application/javascript",
      "application/xml",
      "application/x-ndjson",
      "application/yaml",
      "application/x-yaml",
    ].includes(type)
  ) {
    return true;
  }
  return /\.(txt|md|markdown|csv|tsv|json|jsonl|lino|log|xml|html?|css|js|jsx|ts|tsx|rs|py|java|c|cc|cpp|h|hpp|go|rb|php|sh|ps1|sql|ya?ml|toml|ini|tex|rtf)$/i.test(
    String(file.name || ""),
  );
}

function formatFileSize(bytes) {
  const value = Number(bytes);
  if (!Number.isFinite(value) || value <= 0) return "0 B";
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

function readFileAsDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(reader.error || new Error("Unable to read file"));
    reader.readAsDataURL(file);
  });
}

function readFileAsText(file) {
  if (file && typeof file.text === "function") {
    return file.text();
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(reader.error || new Error("Unable to read file"));
    reader.readAsText(file);
  });
}

function sampleTextAttachmentContent(text, limit = TEXT_ATTACHMENT_CONTEXT_LIMIT) {
  const normalized = String(text || "").replace(/\r\n?/g, "\n").trim();
  if (normalized.length <= limit) {
    return { text: normalized, truncated: false };
  }
  const segment = Math.max(1000, Math.floor(limit / 3));
  const middleStart = Math.max(0, Math.floor((normalized.length - segment) / 2));
  return {
    text: [
      normalized.slice(0, segment).trim(),
      "[... omitted middle of text attachment ...]",
      normalized.slice(middleStart, middleStart + segment).trim(),
      "[... omitted middle of text attachment ...]",
      normalized.slice(normalized.length - segment).trim(),
    ].join("\n\n"),
    truncated: true,
  };
}

function loadOcrBundle() {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return Promise.reject(new Error("OCR is only available in the browser"));
  }
  if (window.FormalAiOcr && typeof window.FormalAiOcr.recognizeImage === "function") {
    return Promise.resolve(window.FormalAiOcr);
  }
  if (ocrBundlePromise) {
    return ocrBundlePromise;
  }
  ocrBundlePromise = new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = withAssetVersion(OCR_BUNDLE_FILENAME);
    script.async = true;
    script.onload = () => {
      if (
        window.FormalAiOcr &&
        typeof window.FormalAiOcr.recognizeImage === "function"
      ) {
        resolve(window.FormalAiOcr);
      } else {
        ocrBundlePromise = null;
        reject(new Error("OCR bundle loaded without an OCR API"));
      }
    };
    script.onerror = () => {
      ocrBundlePromise = null;
      reject(new Error("Unable to load OCR bundle"));
    };
    document.head.appendChild(script);
  });
  return ocrBundlePromise;
}

export function attachmentMemoryRecord(attachment) {
  const record = {
    name: String(attachment.name || "attachment"),
    size: Number(attachment.size || 0),
    type: String(attachment.type || "application/octet-stream"),
    kind: attachment.isImage ? "image" : "file",
  };
  if (attachment.dataUrl) record.dataUrl = attachment.dataUrl;
  if (attachment.text) record.text = attachment.text;
  if (attachment.textTruncated) record.textTruncated = true;
  if (attachment.textError) record.textError = attachment.textError;
  if (attachment.ocrText) record.ocrText = attachment.ocrText;
  if (attachment.ocrConfidence !== undefined && attachment.ocrConfidence !== null) {
    record.ocrConfidence = attachment.ocrConfidence;
  }
  if (attachment.ocrError) record.ocrError = attachment.ocrError;
  return record;
}

export function attachmentOnlyPrompt(attachments) {
  const count = attachments.length;
  if (count === 1) {
    return `Attached ${attachments[0].isImage ? "image" : "file"}: ${attachments[0].name}`;
  }
  return `Attached ${count} files`;
}

function attachmentContextText(attachments) {
  if (!attachments.length) return "";
  const lines = ["Attached files:"];
  attachments.forEach((attachment, index) => {
    lines.push(
      `${index + 1}. ${attachment.name} (${attachment.type}, ${formatFileSize(attachment.size)})`,
    );
    if (attachment.text) {
      lines.push(`Text excerpt: ${attachment.text}`);
      if (attachment.textTruncated) {
        lines.push("Text omitted: attachment excerpt was truncated before solver context.");
      }
    } else if (attachment.textError) {
      lines.push(`Text unavailable: ${attachment.textError}`);
    } else if (attachment.ocrText) {
      lines.push(`OCR text: ${attachment.ocrText}`);
    } else if (attachment.ocrError) {
      lines.push(`OCR unavailable: ${attachment.ocrError}`);
    } else if (attachment.isImage && attachment.dataUrl) {
      lines.push("Image data is stored in memory as a base64 data URL.");
    }
  });
  return lines.join("\n");
}

export function buildPromptWithAttachments(text, attachments) {
  const context = attachmentContextText(attachments);
  if (!context) return text;
  const promptText = String(text || "").trim();
  return `${promptText}\n\n${context}`.trim();
}

// Composer attachment picking and preparation (including optional OCR).
export function useAttachments({
  attachmentInputRef, setComposerMenuOpen, setAttachments, experimentalOcr,
}) {
  const triggerAttachFiles = useCallback(() => {
    if (attachmentInputRef.current) {
      attachmentInputRef.current.click();
    }
    setComposerMenuOpen(false);
  }, []);

  const handleAttachFiles = useCallback((event) => {
    const files = Array.from(event.target.files || []);
    event.target.value = "";
    setAttachments(
      files.map((file) => ({
        id: `attachment-${Date.now()}-${Math.random().toString(16).slice(2)}`,
        sourceFile: file,
        name: file.name,
        size: file.size,
        type: file.type || "application/octet-stream",
        isImage: isImageAttachment(file),
      })),
    );
    setComposerMenuOpen(false);
  }, []);

  const prepareAttachmentsForSend = useCallback(
    async (items) => {
      const safe = Array.isArray(items) ? items : [];
      const prepared = [];
      for (const attachment of safe) {
        const next = {
          id: attachment.id,
          name: attachment.name,
          size: attachment.size,
          type: attachment.type || "application/octet-stream",
          isImage: Boolean(attachment.isImage),
        };
        if (experimentalOcr && next.isImage && attachment.sourceFile) {
          try {
            next.dataUrl = await readFileAsDataUrl(attachment.sourceFile);
            try {
              const ocr = await loadOcrBundle();
              const result = await ocr.recognizeImage(next.dataUrl, { language: "eng" });
              next.ocrText = result && result.text ? String(result.text).trim() : "";
              if (
                result &&
                typeof result.confidence === "number" &&
                Number.isFinite(result.confidence)
              ) {
                next.ocrConfidence = result.confidence;
              }
            } catch (error) {
              next.ocrError =
                error && error.message ? error.message : "OCR recognition failed";
            }
          } catch (error) {
            next.ocrError = error && error.message ? error.message : "File read failed";
          }
        }
        if (!next.isImage && attachment.sourceFile && isTextAttachment(attachment.sourceFile)) {
          try {
            const fullText = await readFileAsText(attachment.sourceFile);
            const sample = sampleTextAttachmentContent(fullText);
            next.text = sample.text;
            next.textTruncated = sample.truncated;
          } catch (error) {
            next.textError = error && error.message ? error.message : "File read failed";
          }
        }
        prepared.push(next);
      }
      return prepared;
    },
    [experimentalOcr],
  );

  return { triggerAttachFiles, handleAttachFiles, prepareAttachmentsForSend };
}
