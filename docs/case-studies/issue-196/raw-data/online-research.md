# Issue 196 Online Research

Collected on 2026-05-24 for issue
<https://github.com/link-assistant/formal-ai/issues/196>.

## Browser Storage Deletion

- MDN `IDBObjectStore.delete()`:
  <https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/delete>
  explains that a key or key range can delete one or multiple IndexedDB
  records, and points callers to `clear()` when the intended operation is to
  remove all records from an object store.
- MDN `IDBObjectStore.clear()`:
  <https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/clear>
  documents that `clear()` removes every record from an IndexedDB object
  store and from indexes that reference that store.

Implication for this repository: the browser memory backend should keep the
normal append-only write path, but expose explicit maintenance operations that
use cursor deletion for selected conversations and `clear()` for full reset.

## Erasure And Backup UX

- ICO right to erasure guidance:
  <https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/individual-rights/individual-rights/right-to-erasure/>
  says organizations need processes for erasure requests, appropriate methods
  to erase information, clear communication about backups, and special care
  when backup data cannot be overwritten immediately.
- NIST Privacy Framework FAQ:
  <https://www.nist.gov/privacy-framework/frequently-asked-questions>
  frames the Privacy Framework as a voluntary way to manage privacy risk and
  references policies and procedures for review, transfer, alteration, and
  deletion.
- NIST Privacy Framework v1.0:
  <https://www.nist.gov/system/files/documents/2020/01/16/NIST%20Privacy%20Framework_V1.0.pdf>
  includes Core subcategories requiring data elements to be accessible for
  deletion and data to be destroyed according to policy.

Implication for this repository: deletion controls should be explicit,
warn that the operation is irreversible, and give the user a chance to export
a full memory bundle before the destructive action runs.
