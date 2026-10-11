// Ratings arithmetic and text predicates the portable fragment translates,
// beside the items it carries with the construct map's reason.
import { readFileSync } from 'node:fs';

/** The largest score a rating holds. */
export const MAXIMUM_SCORE = 10;

/** The word a rating is filed under. */
export const RATING_UNIT = 'stars';

/**
 * The sum of `a` and `b`.
 * @param {number} a
 * @param {number} b
 * @returns {number}
 */
export function add(a, b) {
  return a + b;
}

/**
 * The mean of `count` scores summing to `total`; zero when there are none.
 * @param {number} total
 * @param {number} count
 * @returns {number}
 */
export function average(total, count) {
  if (count <= 0) {
    return 0;
  }
  return total / count;
}

/**
 * The distance of the point (`x`, `y`) from the origin.
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function distance(x, y) {
  const squares = x * x + y * y;
  return Math.sqrt(squares);
}

/**
 * The band a score falls in.
 * @param {number} score
 * @returns {string}
 */
export function scoreBand(score) {
  return score >= MAXIMUM_SCORE ? 'top' : score > 5 ? 'high' : 'low';
}

/**
 * Whether `word` names the rating unit or a word built on it.
 * @param {string} word
 * @returns {boolean}
 */
export function isRatingUnit(word) {
  return word === RATING_UNIT || (word.startsWith('star') && word.endsWith('rs'));
}

/**
 * Whether `text` mentions the rating unit inside a longer text.
 * @param {string} text
 * @returns {boolean}
 */
export function mentionsRating(text) {
  return text.includes(RATING_UNIT) && text !== RATING_UNIT;
}

/**
 * The label of a score: its band when it is the top one.
 * @param {number} score
 * @returns {string}
 */
export function ratingLabel(score) {
  const band = scoreBand(score);
  return band === 'top' ? band : 'rated';
}

/**
 * Whether a score counts as rated: shown and above zero.
 * @param {number} score
 * @param {boolean} hidden
 * @returns {boolean}
 */
export function isRated(score, hidden) {
  return !hidden && -score < 0;
}

/**
 * How far apart two scores are.
 * @param {number} a
 * @param {number} b
 * @returns {number}
 */
export function spread(a, b) {
  return Math.abs(a - b);
}

/**
 * The whole stars shown for a score: odd floors round up to at most the
 * maximum, even ones hold.
 * @param {number} score
 * @returns {number}
 */
export function wholeStars(score) {
  return Math.floor(score) % 2 !== 0 ? Math.min(Math.ceil(score), MAXIMUM_SCORE) : Math.max(Math.floor(score), 1);
}

// The items below sit outside portable-pure-v1; each is carried with the
// construct map's reason.

//! An inner doc comment that no item follows.

export class Rating {
  constructor(score) {
    this.score = score;
  }
}

export function untypedScore(score) {
  return score;
}

/**
 * @param {number} score
 * @returns {number}
 */
export function mutableScore(score) {
  let doubled = score;
  doubled += score;
  return doubled;
}

/**
 * @param {number} count
 * @returns {number}
 */
export function countDown(count) {
  while (count > 0) {
    return count;
  }
  return 0;
}

/**
 * @param {number} score
 * @returns {number}
 */
export function scorePair(score) {
  return [score, score];
}

/**
 * @param {number} score
 * @returns {string}
 */
export function templateLabel(score) {
  return `${score} stars`;
}

/**
 * @param {string} word
 * @returns {string}
 */
export function concatenated(word) {
  return word + ' stars';
}

/**
 * @param {number} score
 * @returns {number}
 */
export function roundedScore(score) {
  return Math.round(score);
}

/**
 * @param {string} word
 * @returns {number}
 */
export function wordLength(word) {
  return word.length;
}

/**
 * @param {number} score
 * @returns {number}
 */
export function helperScore(score) {
  return undefinedHelper(score);
}

/**
 * @param {number} score
 * @returns {boolean}
 */
export function mixedTypes(score) {
  return score === RATING_UNIT;
}

/**
 * @param {number} score
 * @returns {number}
 */
export function partialScore(score) {
  if (score > 0) {
    return score;
  }
}

/**
 * @param {number} score
 * @returns {number}
 */
export async function laterScore(score) {
  return score;
}

/**
 * @param {number} score
 * @returns {number}
 */
export function snake_score(score) {
  return score;
}

/**
 * @param {number} score
 * @returns {number}
 */
export function defaultedScore(score) {
  return score ?? 0;
}

Object.freeze(Rating);
