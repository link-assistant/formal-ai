import pathlib,json,hashlib
root=pathlib.Path('/Users/konard/Code/Archive/link-assistant/formal-ai'); p=pathlib.Path('/private/tmp/pr1188-ci-T4024'); changes=[]
def change(path,fn):
 b=(root/path).read_text(); a=fn(b); assert a!=b
 changes.append(dict(path=str(root/path),before_sha256=hashlib.sha256(b.encode()).hexdigest(),after_sha256=hashlib.sha256(a.encode()).hexdigest(),content=a,before=b))
def once(s,b,a):
 assert s.count(b)==1,(b,s.count(b)); return s.replace(b,a)
change('rust/src/agentic_coding/general_planner/content_shape.rs',lambda s:once(once(s,'    if let Some((start, end)) = first_raw_prefix_lead_end(request, "file_write_content_lead") {\n        if outside(start)','    if let Some((start, end)) = first_raw_prefix_lead_end(request, "file_write_content_lead")\n        && outside(start)'), '            return true;\n        }\n    }','        return true;\n    }'))
def slots(s):
 assert s.count('&[("{target}", &shell_quote(&target))]')==2
 return s.replace('&[("{target}", &shell_quote(&target))]','&[(concat!("{", "target", "}"), &shell_quote(&target))]')
change('rust/src/agentic_coding/general_planner.rs',slots)
for path,names in [('shell_command.rs',['shell_command_for_task']),('shell_command_policy.rs',['named_shell_command_in_sentence','is_prose_word']),('workspace_inspection.rs',['asks_about_the_workspace'])]:
 def transform(s):
  for name in names:s=once(s,'pub(crate) fn '+name,'pub fn '+name)
  return s
 change('rust/src/agentic_coding/'+path,transform)
change('rust/src/repository_workspace/operation.rs',lambda s:once(s,'Ok(nearest[0].to_owned())','Ok(nearest[0].clone())'))
def invert(s):
 start=s.index('        if !error.speaks_for_the_service() {'); end=s.index('\n    } else {\n        "fallback_failed"',start)
 block=s[start:end]; left,right=block.split('        } else {\n'); right=right.removesuffix('        }')
 return s[:start]+'        if error.speaks_for_the_service() {\n'+right+'        } else {\n'+left.split('\n',1)[1]+'        }'+s[end:]
change('rust/src/source_walk.rs',invert)
(p/'request.json').write_text(json.dumps({'changes':changes,'original_job':113995032444},indent=2)+'\n')
