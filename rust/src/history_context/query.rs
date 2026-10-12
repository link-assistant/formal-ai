//! Querying the repository-history store (issue #1180 R9).
//!
//! The store `import_incremental` writes is an ordinary memory store, so
//! a lineage question is an exact memory query over it — the same
//! compile/execute pair every other surface uses, read-only.

use super::{Path, RepositoryHistoryImportError};
use crate::memory::MemoryStore;
use crate::memory_program::{MemoryProgramAuthorization, MemoryProgramLimits};
use crate::memory_query_language::{QueryDialect, compile_memory_query, execute_memory_query};

/// Run an ANSI-SQL memory query over the repository-history store at
/// `store_path`, read-only, and render each row as tab-separated
/// `column=value` cells (values in the query language's canonical form).
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when the store cannot be read
/// or the query does not compile.
pub fn query_repository_history(
    store_path: &Path,
    sql: &str,
) -> Result<Vec<String>, RepositoryHistoryImportError> {
    let mut store = MemoryStore::load_from_file(store_path).map_err(|error| {
        RepositoryHistoryImportError::with_detail("memory_store_io", format!("{error}"))
    })?;
    let query = compile_memory_query(sql, QueryDialect::SqlAnsi, MemoryProgramLimits::default())
        .map_err(|error| {
            RepositoryHistoryImportError::with_detail("query_compile_failed", format!("{error:?}"))
        })?;
    let outcome = execute_memory_query(&query, &mut store, MemoryProgramAuthorization::ReadOnly);
    Ok(outcome
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|(column, value)| format!("{column}={}", value.canonical()))
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect())
}
