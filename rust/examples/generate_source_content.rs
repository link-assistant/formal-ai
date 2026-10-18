//! Native readable-source producer; the optional engine must really be enabled.
#[cfg(feature = "meta-language")]
mod enabled {
    //! Stream full readable models into CI artifacts; receipts are navigation only.
    use formal_ai::agentic_coding::self_ast::source_document;
    use formal_ai::readable_source::{
        read_source_links, source_from_links, source_owner, verify_source_links,
    };
    use formal_ai::self_source_links::owned_source_files;
    use serde_json::json;
    use std::error::Error;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn git_identity(root: &std::path::Path, reference: &str) -> Result<String, Box<dyn Error>> {
        let output = Command::new("git")
            .args(["rev-parse", reference])
            .current_dir(root)
            .output()?;
        if !output.status.success() {
            return Err("SourceContentGitIdentityUnavailable".into());
        }
        let identity = String::from_utf8(output.stdout)?.trim().to_owned();
        if identity.len() != 40 || !identity.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("SourceContentGitIdentityMalformed".into());
        }
        Ok(identity)
    }

    pub(super) fn run() -> Result<(), Box<dyn Error>> {
        let arguments = std::env::args().skip(1).collect::<Vec<_>>();
        if arguments.len() != 3 {
            return Err("usage: generate_source_content OUTPUT SHARD_INDEX SHARDS".into());
        }
        let output = PathBuf::from(&arguments[0]);
        let shard_index = arguments[1].parse::<usize>()?;
        let shards = arguments[2].parse::<usize>()?;
        if shards == 0 || shards > 32 || shard_index >= shards {
            return Err("InvalidSourceContentShard".into());
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("RepositoryRootRequired")?
            .to_path_buf();
        let identity = json!({"commit":git_identity(&root,"HEAD")?,"tree":git_identity(&root,"HEAD^{tree}")?,
        "run":std::env::var("GITHUB_RUN_ID").ok(),"attempt":std::env::var("GITHUB_RUN_ATTEMPT").ok()});
        fs::create_dir(&output)?;
        let mut rows = Vec::new();
        for (index, (relative, source)) in owned_source_files().iter().enumerate() {
            if index % shards != shard_index {
                continue;
            }
            let document = source_document(relative, source).map_err(std::io::Error::other)?;
            let retained = read_source_links(&document).map_err(std::io::Error::other)?;
            if source_owner(&retained).map_err(std::io::Error::other)? != Some(*relative)
                || source_from_links(&retained).map_err(std::io::Error::other)? != *source
            {
                return Err("SourceContentRestorationMismatch".into());
            }
            verify_source_links(&retained).map_err(std::io::Error::other)?;
            let member = format!(
                "source-content/{}.lino",
                relative
                    .strip_suffix(".rs")
                    .ok_or("RustSourceExtensionRequired")?
            );
            let target = output.join(&member);
            fs::create_dir_all(target.parent().ok_or("SourceContentParentRequired")?)?;
            fs::write(&target, document.as_bytes())?;
            rows.push(json!({"path":format!("rust/{relative}"),"member":member.strip_prefix("source-content/").ok_or("SourceContentArtifactRootRequired")?,
            "sourceBlake3":blake3::hash(source.as_bytes()).to_hex().to_string(),
            "modelBlake3":blake3::hash(document.as_bytes()).to_hex().to_string(),
            "sourceFreeRestoration":true,"independentGrammarReparse":true}));
        }
        let receipt = json!({"schema":"native-readable-source-shard/v1","identity":identity,
        "shardIndex":shard_index,"shards":shards,"models":rows,
        "representation":"full UTF-8 recursive Links models are separate artifact members",
        "receiptAuthority":"navigation-and-producer-evidence-only"});
        fs::write(
            output.join("receipt.json"),
            serde_json::to_vec_pretty(&receipt)?,
        )?;
        Ok(())
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "meta-language")]
    {
        enabled::run()
    }
    #[cfg(not(feature = "meta-language"))]
    {
        Err("SourceModelEngineUnavailable".into())
    }
}
