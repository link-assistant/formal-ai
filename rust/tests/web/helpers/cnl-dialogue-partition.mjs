// Stable ownership keeps every fixture language while bounding each test file.
export function dialogueLanguagesForShard(languages, shard) {
  if (!Number.isInteger(shard) || shard < 0 || shard >= 3)
    throw new RangeError('invalid dialogue shard');
  return languages.filter((_, index) => index % 3 === shard);
}
