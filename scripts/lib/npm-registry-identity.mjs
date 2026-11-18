// Resolve the publisher behind an installed npm key; local workspace code has
// no published-latest version. Its own declared registry dependencies still do.
export function npmRegistryIdentity(name, declared) {
  if (['file:','link:','workspace:'].some(prefix=>declared.startsWith(prefix))) return null;
  if (!declared.startsWith('npm:')) return {name,requirement:declared};
  const alias=declared.slice(4), index=alias.lastIndexOf('@');
  return index>0 ? {name:alias.slice(0,index),requirement:alias.slice(index+1)} : {name:alias,requirement:'latest'};
}
