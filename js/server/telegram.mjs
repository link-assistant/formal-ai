// Telegram webhook (`POST /telegram/webhook`): rust/src/telegram.rs
// `handle_telegram_webhook` (update parsing, the public-chat gate,
// `compose_telegram_reply`, `telegram_html_from_markdown`, reply splitting and
// the thinking blockquote), rust/src/attachment_context.rs
// (`compose_prompt_with_attachments`) and, through ./telegram-execution.mjs,
// rust/src/telegram_runtime.rs `execute_telegram_code_request_from_environment`.

import { serverMessage } from './messages.mjs';
import { JSON_TYPE, errorResponse, jsonResponse, rawResponse } from './response.mjs';
import { packageVersion } from './seed.mjs';
import { solveSymbolic } from './solve.mjs';
import { executeTelegramCodeRequestFromEnvironment } from './telegram-execution.mjs';
import { naturalizeThinkingStep, thinkingAnswerLanguage } from './thinking.mjs';

const TELEGRAM_MAX_MESSAGE_LEN = 4096;
const PUBLIC_CHATS = ['group', 'supergroup', 'channel'];

const byteLength = (text) => Buffer.byteLength(text, 'utf8');
const isObject = (value) => Boolean(value) && typeof value === 'object' && !Array.isArray(value);

// ---------------------------------------------------------------- update shape

class UpdateShapeError extends Error {}

function typeName(value) {
  if (value === null) return 'null';
  if (Array.isArray(value)) return 'sequence';
  if (typeof value === 'object') return 'map';
  if (typeof value === 'number') return Number.isInteger(value) ? 'integer' : 'floating point';
  return typeof value;
}

function fail(expected, value) {
  throw new UpdateShapeError(`invalid type: ${typeName(value)}, expected ${expected}`);
}

function checkStruct(value, name) {
  if (!isObject(value)) fail(`struct ${name}`, value);
  return value;
}

function optional(object, key, check) {
  const value = object[key];
  return value === undefined || value === null ? null : check(value);
}

function defaulted(object, key, check, fallback) {
  const value = object[key];
  return value === undefined ? fallback : check(value);
}

function required(object, key, check) {
  if (object[key] === undefined) throw new UpdateShapeError(`missing field \`${key}\``);
  return check(object[key]);
}

const asString = (value) => (typeof value === 'string' ? value : fail('a string', value));
const asBool = (value) => (typeof value === 'boolean' ? value : fail('a boolean', value));
const asI64 = (value) => (Number.isInteger(value) ? value : fail('i64', value));
const asU64 = (value) => (Number.isInteger(value) && value >= 0 ? value : fail('u64', value));
const asArray = (check) => (value) => (Array.isArray(value) ? value.map(check) : fail('a sequence', value));

function asDocument(value) {
  const object = checkStruct(value, 'TelegramDocument');
  return {
    file_name: optional(object, 'file_name', asString),
    mime_type: optional(object, 'mime_type', asString),
    file_size: optional(object, 'file_size', asU64),
  };
}

function asMessage(value) {
  const object = checkStruct(value, 'TelegramMessage');
  const message = {
    message_id: required(object, 'message_id', asI64),
    chat: required(object, 'chat', (chat) => {
      const fields = checkStruct(chat, 'TelegramChat');
      return {
        id: required(fields, 'id', asI64),
        type: optional(fields, 'type', asString),
        title: optional(fields, 'title', asString),
      };
    }),
    text: optional(object, 'text', asString),
    caption: optional(object, 'caption', asString),
    entities: defaulted(object, 'entities', asArray((entity) => ({
      type: required(checkStruct(entity, 'TelegramEntity'), 'type', asString),
    })), []),
    reply_to_message: optional(object, 'reply_to_message', asMessage),
    from: optional(object, 'from', (user) => ({
      is_bot: defaulted(checkStruct(user, 'TelegramUser'), 'is_bot', asBool, false),
    })),
    document: optional(object, 'document', asDocument),
    photo: defaulted(object, 'photo', asArray((size) => ({
      file_size: optional(checkStruct(size, 'TelegramPhotoSize'), 'file_size', asU64),
    })), []),
    audio: optional(object, 'audio', asDocument),
    voice: optional(object, 'voice', asDocument),
    video: optional(object, 'video', asDocument),
  };
  return message;
}

/** `TelegramUpdate` deserialization + `into_message`. */
function parseUpdate(body) {
  let value;
  try {
    value = JSON.parse(body);
  } catch (error) {
    throw new UpdateShapeError(error.message);
  }
  const object = checkStruct(value, 'TelegramUpdate');
  optional(object, 'update_id', asI64);
  const candidates = ['message', 'edited_message', 'channel_post', 'edited_channel_post']
    .map((key) => optional(object, key, asMessage));
  return candidates.find((message) => message !== null) ?? null;
}

function messageAddressesBot(message) {
  if (message.entities.some((entity) => entity.type === 'mention' || entity.type === 'bot_command')) return true;
  if (message.reply_to_message?.from?.is_bot) return true;
  if (message.chat.title !== null) {
    const lower = message.chat.title.toLowerCase();
    if (lower.includes('formal')) return true;
  }
  return false;
}

// ---------------------------------------------------------------- attachments

function humanReadableSize(bytes) {
  const KIB = 1024;
  const MIB = KIB * 1024;
  const GIB = MIB * 1024;
  if (bytes < KIB) return `${bytes} B`;
  if (bytes < MIB) return `${(bytes / KIB).toFixed(1)} KB`;
  if (bytes < GIB) return `${(bytes / MIB).toFixed(1)} MB`;
  return `${(bytes / GIB).toFixed(1)} GB`;
}

function documentAttachment(document, kind, messageId, fallbackMime) {
  const name = document.file_name !== null && document.file_name.trim() ? document.file_name : `${kind}_${messageId}`;
  const mime = document.mime_type !== null && document.mime_type.trim() ? document.mime_type : fallbackMime;
  return { name, mime, size: document.file_size };
}

/** `TelegramMessage::attachments`. */
function messageAttachments(message) {
  const out = [];
  if (message.document) out.push(documentAttachment(message.document, 'document', message.message_id, ''));
  if (message.audio) out.push(documentAttachment(message.audio, 'audio', message.message_id, 'audio/mpeg'));
  if (message.voice) out.push(documentAttachment(message.voice, 'voice', message.message_id, 'audio/ogg'));
  if (message.video) out.push(documentAttachment(message.video, 'video', message.message_id, 'video/mp4'));
  if (message.photo.length) {
    const largest = message.photo.reduce((best, size) => ((size.file_size ?? 0) >= (best.file_size ?? 0) ? size : best));
    out.push({ name: `photo_${message.message_id}.jpg`, mime: 'image/jpeg', size: largest.file_size });
  }
  return out;
}

/** `compose_prompt_with_attachments` (+ `build_attachment_context`). */
function composePromptWithAttachments(text, attachments) {
  const message = text === null ? null : text.trim() || null;
  let context = null;
  if (attachments.length) {
    context = attachments.reduce((block, attachment, index) => {
      const mime = attachment.mime.trim() || 'application/octet-stream';
      const descriptor = attachment.size === null ? mime : `${mime}, ${humanReadableSize(attachment.size)}`;
      return `${block}\n${index + 1}. ${attachment.name} (${descriptor})`;
    }, serverMessage('telegram_attached_files'));
  }
  if (message !== null && context !== null) return `${message}\n\n${context}`;
  return message ?? context;
}

// ---------------------------------------------------------------- code execution

/**
 * Mirrors rust/src/telegram_runtime.rs `execute_telegram_code_request_from_environment`
 * as rust/src/telegram.rs `compose_telegram_reply` reads it: the answer for an
 * explicit code-execution request (refused, failed or observed), or null.
 */
async function codeExecutionAnswer(ctx, prompt) {
  const language = String(await ctx.worker.run('detectLanguage(__telegramPrompt)', { __telegramPrompt: prompt }));
  const execution = await executeTelegramCodeRequestFromEnvironment(prompt, language);
  return execution.kind === 'not_requested' ? null : execution.answer;
}

// ---------------------------------------------------------------- HTML

function htmlEscape(text) {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function telegramInlineHtml(markdown) {
  let rendered = '';
  let rest = markdown;
  while (rest) {
    if (rest.startsWith('**')) {
      const end = rest.indexOf('**', 2);
      if (end >= 0) {
        rendered += `<b>${telegramInlineHtml(rest.slice(2, end))}</b>`;
        rest = rest.slice(end + 2);
        continue;
      }
    }
    if (rest.startsWith('`')) {
      const end = rest.indexOf('`', 1);
      if (end >= 0) {
        rendered += `<code>${htmlEscape(rest.slice(1, end))}</code>`;
        rest = rest.slice(end + 1);
        continue;
      }
    }
    if (rest.startsWith('[')) {
      const labelEnd = rest.indexOf('](', 1);
      if (labelEnd >= 0) {
        const afterLabel = rest.slice(labelEnd + 2);
        const urlEnd = afterLabel.indexOf(')');
        if (urlEnd >= 0) {
          const url = afterLabel.slice(0, urlEnd);
          if (url.startsWith('https://') || url.startsWith('http://')) {
            rendered += `<a href="${htmlEscape(url)}">${htmlEscape(rest.slice(1, labelEnd))}</a>`;
            rest = afterLabel.slice(urlEnd + 1);
            continue;
          }
        }
      }
    }
    const character = String.fromCodePoint(rest.codePointAt(0));
    rendered += htmlEscape(character);
    rest = rest.slice(character.length);
  }
  return rendered;
}

function openPreCodeTag(language) {
  return language && /^[A-Za-z0-9_-]+$/.test(language) ? `<pre><code class="language-${language}">` : '<pre><code>';
}

/** `str::lines`. */
function lines(text) {
  if (!text) return [];
  const parts = text.split('\n');
  if (parts[parts.length - 1] === '') parts.pop();
  return parts.map((line) => line.replace(/\r$/, ''));
}

/** `telegram_html_from_markdown`. */
export function telegramHtmlFromMarkdown(markdown) {
  let rendered = '';
  let inCodeBlock = false;
  for (const line of lines(markdown)) {
    const trimmed = line.trimStart();
    if (trimmed.startsWith('```')) {
      if (inCodeBlock) {
        rendered += '</code></pre>\n';
        inCodeBlock = false;
      } else {
        rendered += openPreCodeTag(trimmed.slice(3).trim());
        inCodeBlock = true;
      }
      continue;
    }
    if (inCodeBlock) {
      rendered += `${htmlEscape(line)}\n`;
    } else if (trimmed.startsWith('>')) {
      rendered += `<blockquote>${telegramInlineHtml(trimmed.slice(1).trimStart())}</blockquote>\n`;
    } else {
      rendered += `${telegramInlineHtml(line)}\n`;
    }
  }
  if (inCodeBlock) rendered += '</code></pre>\n';
  return rendered.trimEnd();
}

/** `split_telegram_reply_parts`: byte-budgeted, line-preferring split. */
function splitTelegramReplyParts(text) {
  if (byteLength(text) <= TELEGRAM_MAX_MESSAGE_LEN) return [text];
  const parts = [];
  let current = '';
  for (const line of text.split(/(?<=\n)/)) {
    let rest = line;
    while (rest) {
      if (byteLength(rest) <= TELEGRAM_MAX_MESSAGE_LEN - byteLength(current)) {
        current += rest;
        break;
      }
      if (!current) {
        let taken = '';
        let budget = TELEGRAM_MAX_MESSAGE_LEN;
        for (const character of rest) {
          const size = byteLength(character);
          if (size > budget) break;
          taken += character;
          budget -= size;
        }
        current = taken;
        rest = rest.slice(taken.length);
      }
      parts.push(current);
      current = '';
    }
  }
  if (current) parts.push(current);
  return parts;
}

/** `telegram_thinking_blockquote`. */
function telegramThinkingBlockquote(steps) {
  if (!steps.length) return null;
  const language = thinkingAnswerLanguage(steps);
  const body = steps.map((step) => htmlEscape(naturalizeThinkingStep(language, step.step, step.detail))).join('\n');
  return `<blockquote expandable>\u{1F4AD} ${body}</blockquote>`;
}

function isVersionCommand(text) {
  const first = text.split(/\s+/).filter(Boolean)[0] ?? '';
  return first.split('@')[0].toLowerCase() === '/version';
}

// ---------------------------------------------------------------- reply

/** `compose_telegram_reply(...).reply`. */
async function composeTelegramReply(ctx, message) {
  const prompt = composePromptWithAttachments(message.text ?? message.caption, messageAttachments(message));
  let replyText = serverMessage('telegram_text_only');
  let traceId = null;
  let thinking = null;
  if (prompt !== null && prompt.trim()) {
    const trimmed = prompt.trim();
    const execution = isVersionCommand(trimmed) ? null : await codeExecutionAnswer(ctx, trimmed);
    if (isVersionCommand(trimmed)) {
      replyText = `formal-ai ${packageVersion()}`;
    } else if (execution !== null) {
      replyText = execution;
    } else {
      const symbolic = await solveSymbolic(ctx, trimmed, []);
      const trace = symbolic.evidence_links.find((link) => link.startsWith('trace:'));
      traceId = trace === undefined ? null : trace.slice(6);
      thinking = telegramThinkingBlockquote(symbolic.thinking_steps);
      replyText = symbolic.answer;
    }
  }
  let text = telegramHtmlFromMarkdown(replyText);
  const traceFooter = traceId === null ? null : `\n\n/trace ${traceId}`;
  if (thinking !== null) {
    const traceLength = traceFooter === null ? 0 : byteLength(traceFooter);
    if (byteLength(text) + 2 + byteLength(thinking) + traceLength <= TELEGRAM_MAX_MESSAGE_LEN) {
      text += `\n\n${thinking}`;
    }
  }
  if (traceFooter !== null) text += traceFooter;
  const parts = splitTelegramReplyParts(text);
  if (parts.length > 1) text = `${parts[0]}\n\n[part 1/${parts.length}]`;
  return {
    method: 'sendMessage',
    chat_id: message.chat.id,
    text,
    parse_mode: 'HTML',
    reply_parameters: { message_id: message.message_id },
  };
}

/** `handle_telegram_webhook`: the reply, or null for an update to ignore. */
async function telegramWebhookReply(ctx, body) {
  const message = parseUpdate(body);
  if (message === null) return null;
  if (message.chat.type !== null && PUBLIC_CHATS.includes(message.chat.type) && !messageAddressesBot(message)) {
    return null;
  }
  return composeTelegramReply(ctx, message);
}

/** `POST /telegram/webhook`. */
export async function handleTelegramWebhook(ctx, request) {
  let reply;
  try {
    reply = await telegramWebhookReply(ctx, request.body);
  } catch (error) {
    if (!(error instanceof UpdateShapeError)) throw error;
    return errorResponse(400, serverMessage('telegram_invalid_update', { message: error.message }));
  }
  if (reply === null) return rawResponse(200, JSON_TYPE, '');
  return jsonResponse(200, reply);
}
