// Host observes bytes only. Never execute Git against candidate-controlled .git/config/hooks.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
export function workspaceSnapshot(root){
  const rows=[];let bytes=0,count=0;
  const visit=(relative,depth)=>{
    assert(depth<=12);
    for(const entry of fs.readdirSync(path.join(root,relative),{withFileTypes:true})){
      assert(++count<=2048);if(relative===''&&entry.name==='.git')continue;
      assert(!entry.isSymbolicLink(),'task workspace symlink refused');
      const name=relative?relative+'/'+entry.name:entry.name,file=path.join(root,name);
      if(entry.isDirectory()){visit(name,depth+1);continue;}
      assert(entry.isFile());const fd=fs.openSync(file,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
      try{const before=fs.fstatSync(fd);assert(before.isFile()&&before.size<=1024*1024);bytes+=before.size;assert(bytes<=64*1024*1024);
        const content=fs.readFileSync(fd),after=fs.fstatSync(fd);assert.equal(after.size,before.size);assert.equal(after.mtimeMs,before.mtimeMs);
        rows.push({path:name,sha256:createHash('sha256').update(content).digest('hex'),bytes:content.length});
      }finally{fs.closeSync(fd);}
    }
  };
  visit('',0);return rows.sort((left,right)=>left.path<right.path?-1:left.path>right.path?1:0);
}
