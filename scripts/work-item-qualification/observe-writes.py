#!/usr/bin/env python3
"""Hosted Linux inotify observation of actual task writes, including write-and-revert."""
import ctypes, json, os, select, signal, struct, sys, time
root, output, ready = sys.argv[1:]
libc=ctypes.CDLL(None,use_errno=True)
fd=libc.inotify_init1(os.O_NONBLOCK|os.O_CLOEXEC)
if fd<0: raise OSError(ctypes.get_errno(),'inotify_init1')
watches={}; events=[]; running=True; overflow=False
mask=0x2|0x4|0x8|0x40|0x80|0x100|0x200|0x400|0x800
def watch(directory):
    relative=os.path.relpath(directory,root)
    if '.git' in relative.split(os.sep): return
    wd=libc.inotify_add_watch(fd,os.fsencode(directory),mask)
    if wd<0: raise OSError(ctypes.get_errno(),'inotify_add_watch')
    watches[wd]=directory
for directory,dirs,files in os.walk(root):
    dirs[:]=[name for name in dirs if name!='.git' and not os.path.islink(os.path.join(directory,name))]
    watch(directory)
def stop(*unused):
    global running
    running=False
signal.signal(signal.SIGTERM,stop);signal.signal(signal.SIGINT,stop)
open(ready,'w').write('ready\n')
while running:
    if not select.select([fd],[],[],0.1)[0]:continue
    data=os.read(fd,1024*1024);position=0
    while position<len(data):
        wd,event,cookie,length=struct.unpack_from('iIII',data,position);position+=16
        name=os.fsdecode(data[position:position+length].split(b'\0',1)[0]);position+=length
        if event&0x4000:overflow=True;continue
        if wd not in watches:continue
        target=os.path.join(watches[wd],name);relative=os.path.relpath(target,root)
        if '.git' in relative.split(os.sep):continue
        events.append({'path':relative,'mask':event})
        if len(events)>10000:overflow=True;running=False;break
        if event&0x40000000 and event&(0x80|0x100) and os.path.isdir(target) and not os.path.islink(target):
            for directory,dirs,files in os.walk(target):
                dirs[:]=[item for item in dirs if item!='.git'];watch(directory)
os.close(fd)
with open(output,'w') as target:json.dump({'schema':'formal-ai-task-write-observation/v1','events':events,'complete':not overflow},target)
