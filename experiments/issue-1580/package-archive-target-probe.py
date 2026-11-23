# Bounded, offline Cargo packaging probe; never compile or access registry dependencies.
import hashlib,json,os,subprocess,tempfile
from pathlib import Path
with tempfile.TemporaryDirectory(prefix="formal-ai-package-target-")as temporary:
 root=Path(temporary)
 observations=[]
 for label,workspace,mode in [
  ("single-default",False,"default"),
  ("single-environment-absolute",False,"absolute"),
  ("single-environment-relative",False,"relative"),
  ("single-config",False,"config"),
  ("workspace-member-default",True,"default")
 ]:
  case=root/label;package=case/"rust"
  (package/"src").mkdir(parents=True)
  (package/"src/lib.rs").write_text("pub fn fixture()->u8 { 7 }\n")
  (package/"Cargo.toml").write_text('[package]\nname="formal-ai-target-probe"\nversion="0.1.0"\nedition="2021"\nlicense="MIT"\ndescription="Finite offline archive path probe"\n')
  if workspace:(case/"Cargo.toml").write_text('[workspace]\nmembers=["rust"]\nresolver="2"\n')
  environment={k:v for k,v in os.environ.items()if k not in ["CARGO_TARGET_DIR","CARGO_BUILD_TARGET_DIR","CARGO_ENCODED_RUSTFLAGS","RUSTFLAGS","RUSTC_WRAPPER","RUSTC_WORKSPACE_WRAPPER"]}
  environment.update({"CARGO_NET_OFFLINE":"true","CARGO_BUILD_JOBS":"1","CARGO_INCREMENTAL":"0","CARGO_HOME":str(root/"cargo-home")})
  if mode=="absolute":environment["CARGO_TARGET_DIR"]=str(case/"shared-target")
  if mode=="relative":environment["CARGO_TARGET_DIR"]="relative-target"
  if mode=="config":
   (case/".cargo").mkdir();(case/".cargo/config.toml").write_text('[build]\ntarget-dir="configured-target"\n')
  metadata=subprocess.run(["cargo","metadata","--offline","--no-deps","--format-version","1"],cwd=package,env=environment,capture_output=True,text=True)
  if metadata.returncode:raise RuntimeError(metadata.stderr)
  data=json.loads(metadata.stdout);target=Path(data["target_directory"])
  result=subprocess.run(["cargo","package","--offline","--allow-dirty","--no-verify","-p","formal-ai-target-probe"],cwd=package,env=environment,capture_output=True,text=True)
  if result.returncode:raise RuntimeError(result.stderr)
  actual=target/"package/formal-ai-target-probe-0.1.0.crate"
  assumed=package/"target/package/formal-ai-target-probe-0.1.0.crate"
  assert actual.is_file()
  observations.append({"case":label,"cargoMetadataExit":metadata.returncode,"cargoPackageExit":result.returncode,"targetDirectoryRelativeToFixture":str(target.relative_to(case)),"actualArchiveExists":actual.is_file(),"hardcodedArchiveExists":assumed.exists(),"hardcodedPathMatchesActual":actual==assumed,"archiveBytes":actual.stat().st_size,"archiveSha256":hashlib.sha256(actual.read_bytes()).hexdigest(),"metadataStderr":metadata.stderr,"packageStdout":result.stdout,"packageStderr":result.stderr})
 print(json.dumps({"observations":observations,"cases":len(observations),"rustCompilationExecuted":False,"fullIssueComplete":False,"independentAuthorshipCredit":0},indent=2))
