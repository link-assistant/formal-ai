from pathlib import Path
import json,hashlib
root=Path('/Users/konard/Code/Archive/link-assistant/formal-ai');d=Path('/private/tmp/spec-recurrence-1188/release-control-repair');replacements=[]
def change(path,old,new):
 source=(root/path).read_text();assert source.count(old)==1,path
 (d/Path(path).name).write_text(source.replace(old,new));(d/('original-'+Path(path).name)).write_text(source)
 replacements.append({'path':path,'sha256':hashlib.sha256((root/path).read_bytes()).hexdigest(),'content':source.replace(old,new)})
change('rust/tests/unit/ci-cd/release_recovery.rs','''        let step = workflow_step_block(auto_release, step_name);''','''        // The prepared image factory now owns the metadata operation; keep the
        // original logical operand and its publication-state assertions.
        let current_step_name = if step_name == "Extract GHCR Docker metadata" {
            "Publish Docker image to GHCR"
        } else {
            step_name
        };
        let step = workflow_step_block(auto_release, current_step_name);
        if step_name == "Extract GHCR Docker metadata" {
            assert!(
                step.contains("node scripts/release-image-factory.mjs publish prepared-release"),
                "metadata must bind to the actual prepared publication operation"
            );
            let factory = fs::read_to_string(format!(
                "{}/../scripts/release-image-factory.mjs",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap();
            assert!(factory.contains("for(const suffix of [version,'latest'])"));
            let tag = factory.find("invoke(['tag',local,image+':'+suffix])").unwrap();
            let push = factory.find("invoke(['push',image+':'+suffix])").unwrap();
            assert!(tag < push, "both metadata suffixes must be tagged before pushing");
        }''')
# Manual operation uses the same logical metadata ownership.
p='rust/tests/unit/ci-cd/release_recovery.rs';new=replacements[-1]['content'];old='        let step = workflow_step_block(manual_release, step_name);';assert new.count(old)==1
new=new.replace(old,'''        let current_step_name = if step_name == "Extract GHCR Docker metadata" {
            "Publish Docker image to GHCR"
        } else {
            step_name
        };
        let step = workflow_step_block(manual_release, current_step_name);
        if step_name == "Extract GHCR Docker metadata" {
            assert!(step.contains("node scripts/release-image-factory.mjs publish prepared-release"));
        }''');replacements[-1]['content']=new;(d/'release_recovery.rs').write_text(new)
assets=[f'formal-ai-desktop-{x}-${{version}}.{ext}' for x,ext in [('macos-arm64','dmg'),('macos-arm64','zip'),('macos-x64','dmg'),('macos-x64','zip'),('windows-installer-x64','exe'),('windows-installer-arm64','exe'),('windows-portable-x64','exe'),('windows-portable-arm64','exe'),('linux-x64','AppImage'),('linux-arm64','AppImage'),('linux-x64','deb'),('linux-arm64','deb'),('linux-x64','tar.gz'),('linux-arm64','tar.gz')]]+['latest.yml','latest-mac.yml','latest-linux.yml']
replacement='''    // The original seventeen obligations remain a minimum; CLI archives and
    // authenticated provenance now add required assets beyond that old count.
    let original_assets = [\n'''+''.join('        '+json.dumps(x)+',\n' for x in assets)+'''    ];
    assert_eq!(original_assets.len(), 17);
    for asset in original_assets {
        assert!(resolve_script.contains(asset), "original required asset missing: {asset}");
    }
    for obligation in [
        "expected_desktop_assets \\\"$release_version\\\"",
        "expected_cli_assets",
        "expected_native_targets",
        "formal-ai-vscode-",
        "SHA256SUMS.txt",
        "BUILD-PROVENANCE.txt",
        "formal-ai-native-source-",
        "formal-ai-native-protocol-",
        "formal-ai-signing-macos-",
        "verify_durable_release",
    ] {
        assert!(resolve_script.contains(obligation), "required release obligation missing: {obligation}");
    }
    assert!(
        resolve_script.contains("latest.yml")
            && resolve_script.contains("latest-mac.yml")
            && resolve_script.contains("latest-linux.yml")
            && resolve_script.contains("done < <(expected_desktop_assets \\\"$release_version\\\")"),
        "release resolver should require update metadata and the complete asset inventory before skipping an automatic build"
    );'''
old='''    assert!(
        resolve_script.contains("latest.yml")
            && resolve_script.contains("latest-mac.yml")
            && resolve_script.contains("latest-linux.yml")
            && resolve_script.contains("required desktop assets: 17"),
        "release resolver should require update metadata before skipping an automatic build"
    );'''
change('rust/tests/unit/ci-cd/workflow_release_desktop.rs',old,replacement)
new=replacements[-1]['content'];name='fn desktop_release_uploads_auto_update_metadata()';assert name in new;new=new.replace(name,'#[allow(clippy::literal_string_with_formatting_args)]\n'+name);replacements[-1]['content']=new;(d/'workflow_release_desktop.rs').write_text(new)
old=" for(const image of ['ghcr.io/fixture/formal-ai','fixture/mirror'])for(const suffix of ['1.2.3','latest'])assert.ok(docker.some(call=>call.args[0]==='push'&&call.args[1]===image+':'+suffix));"
extra='''
 const publicationOrder=[];
 for(const image of ['ghcr.io/fixture/formal-ai','fixture/mirror'])for(const suffix of ['1.2.3','latest']) {
  const tagged=docker.findIndex(call=>call.args[0]==='tag'&&call.args[2]===image+':'+suffix);
  const pushed=docker.findIndex(call=>call.args[0]==='push'&&call.args[1]===image+':'+suffix);
  assert.ok(tagged>=0&&pushed>tagged,'actual metadata suffix must be tagged before push: '+image+':'+suffix);
  publicationOrder.push(pushed);
 }
 assert.ok(publicationOrder[0]<publicationOrder[1]&&publicationOrder[1]<publicationOrder[2]&&publicationOrder[2]<publicationOrder[3],
  'actual full version/latest publication must precede mirror version/latest publication');'''
change('rust/tests/web/prepared-release-factory.test.mjs',old,old+extra)
(d/'replacements.json').write_text(json.dumps({'replacements':replacements,'originalAssets':assets},indent=2)+'\n')
print('reviewed drafts ready')
