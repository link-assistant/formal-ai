// JavaScript adaptation of BLAKE3 team's 1.8.2 reference algorithm.
// Copyright 2019 Jack O'Connor and Samuel Neves. CC0-1.0 option.
// Changed: one-shot CPU-only JavaScript, unkeyed mode and XOF only.
const IV=[0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
const PERM=[2,6,3,10,7,0,4,13,1,11,12,5,9,14,15,8];
const rotr=(x,n)=>(x>>>n)|(x<<(32-n));
function g(s,a,b,c,d,x,y){s[a]=(s[a]+s[b]+x)>>>0;s[d]=rotr(s[d]^s[a],16);s[c]=(s[c]+s[d])>>>0;s[b]=rotr(s[b]^s[c],12);s[a]=(s[a]+s[b]+y)>>>0;s[d]=rotr(s[d]^s[a],8);s[c]=(s[c]+s[d])>>>0;s[b]=rotr(s[b]^s[c],7);}
function compress(cv,block,counter,len,flags){
 const s=[...cv,...IV.slice(0,4),Number(counter&0xffffffffn),Number(counter>>32n),len,flags];let m=block;
 for(let r=0;r<7;r++){
  g(s,0,4,8,12,m[0],m[1]);g(s,1,5,9,13,m[2],m[3]);g(s,2,6,10,14,m[4],m[5]);g(s,3,7,11,15,m[6],m[7]);
  g(s,0,5,10,15,m[8],m[9]);g(s,1,6,11,12,m[10],m[11]);g(s,2,7,8,13,m[12],m[13]);g(s,3,4,9,14,m[14],m[15]);m=PERM.map(i=>m[i]);
 }
 for(let i=0;i<8;i++){s[i]=(s[i]^s[i+8])>>>0;s[i+8]=(s[i+8]^cv[i])>>>0;}return s;
}
const chaining=o=>compress(o.cv,o.block,o.counter,o.len,o.flags).slice(0,8);
const parent=(left,right)=>({cv:IV,block:[...left,...right],counter:0n,len:64,flags:4});
function chunk(bytes,index){
 let cv=IV;const blocks=Math.max(1,Math.ceil(bytes.length/64));
 for(let b=0;b<blocks;b++){
  const part=bytes.subarray(b*64,(b+1)*64),padded=new Uint8Array(64);padded.set(part);const view=new DataView(padded.buffer);
  const block=Array.from({length:16},(_,i)=>view.getUint32(i*4,true));
  const out={cv,block,counter:BigInt(index),len:part.length,flags:(b===0?1:0)|(b===blocks-1?2:0)};
  if(b===blocks-1)return out;cv=chaining(out);
 }
}
/** No string coercion, implicit encoding, keyed mode, or mutable retained state. */
export function blake3(input,{length=32}={}){
 if(!(input instanceof Uint8Array))throw Error('blake3_bytes_required');
 if(!Number.isSafeInteger(length)||length<0||length>1024*1024)throw Error('blake3_output_limit');
 const chunks=Math.max(1,Math.ceil(input.length/1024)),stack=[];let out;
 for(let i=0;i<chunks;i++){
  out=chunk(input.subarray(i*1024,(i+1)*1024),i);
  if(i<chunks-1){let cv=chaining(out),total=i+1;while(total%2===0){cv=chaining(parent(stack.pop(),cv));total/=2;}stack.push(cv);}
 }
 while(stack.length)out=parent(stack.pop(),chaining(out));
 const result=new Uint8Array(length),view=new DataView(result.buffer);
 for(let offset=0,counter=0n;offset<length;offset+=64,counter++){
  const words=compress(out.cv,out.block,counter,out.len,out.flags|8);
  for(let j=0;j<Math.min(64,length-offset);j++)view.setUint8(offset+j,(words[j>>>2]>>>((j%4)*8))&255);
 }
 return result;
}
export const blake3Hex=input=>Buffer.from(blake3(input)).toString('hex');
