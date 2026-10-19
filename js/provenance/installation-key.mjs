// Installation continuity authenticates a configured key, never a person or autonomous authorship.
import fs from 'node:fs';
import path from 'node:path';
import {constants} from 'node:fs';
import {createPrivateKey,createPublicKey,generateKeyPairSync} from 'node:crypto';
import {blake3Hex} from '../crypto/blake3.mjs';
import {provenancePrimitives} from './session.mjs';
import {encodeTyped,decodeTyped} from '../logging/typed-codec.mjs';
const primitives=provenancePrimitives({encodeTyped,decodeTyped});
const privateName='installation-ed25519.pk8';
const publicName='installation-ed25519.spki';
function keyDirectory(directory,{create=false,excludedDirectory}={}){
 if(typeof directory!=='string'||!directory)throw Error('installation_key_directory_required');
 const resolved=path.resolve(directory),parent=path.dirname(resolved);
 if(resolved===parent)throw Error('installation_key_directory_required');
 if(create)fs.mkdirSync(parent,{recursive:true,mode:0o700});
 const destination=path.join(fs.realpathSync(parent),path.basename(resolved));
 if(excludedDirectory&&(destination===excludedDirectory||destination.startsWith(excludedDirectory+path.sep)))throw Error('installation_key_inside_workspace');
 if(create){try{fs.mkdirSync(destination,{mode:0o700});}catch(error){if(error.code!=='EEXIST')throw error;}}
 const status=fs.lstatSync(destination);
 if(!status.isDirectory()||status.isSymbolicLink()||(status.mode&0o077)!==0
   ||typeof process.getuid==='function'&&status.uid!==process.getuid())throw Error('installation_key_directory_custody');
 return destination;
}
function keyBytes(file,privateKey=false){
 const descriptor=fs.openSync(file,constants.O_RDONLY|(constants.O_NOFOLLOW??0));
 try{
  const status=fs.fstatSync(descriptor);
  if(!status.isFile()||status.nlink!==1||status.size>16384
    ||privateKey&&(status.mode&0o077)!==0
    ||typeof process.getuid==='function'&&status.uid!==process.getuid())throw Error('installation_key_file_custody');
  return fs.readFileSync(descriptor);
 }finally{fs.closeSync(descriptor);}
}
function writeKey(file,bytes){
 const descriptor=fs.openSync(file,'wx',0o600);
 try{fs.writeFileSync(descriptor,bytes);fs.fsyncSync(descriptor);}finally{fs.closeSync(descriptor);}
}
/** Reads a caller-designated trust store. It never trusts a key carried by evidence alone. */
export function trustedInstallationKey(directory){
 const destination=keyDirectory(directory),bytes=keyBytes(path.join(destination,publicName));
 const publicKey=createPublicKey({key:bytes,format:'der',type:'spki'});
 if(publicKey.asymmetricKeyType!=='ed25519')throw Error('installation_key_algorithm');
 return bytes.toString('base64');
}
/** An exclusive installation lock prevents accidental concurrent creation/rotation. */
export function openInstallationSigner(directory,{excludedDirectory}={}){
 const destination=keyDirectory(directory,{create:true,excludedDirectory}),lock=path.join(destination,'installation.lock');
 try{fs.mkdirSync(lock,{mode:0o700});}catch(error){if(error.code==='EEXIST')throw Error('installation_key_busy');throw error;}
 try{
  const privatePath=path.join(destination,privateName),publicPath=path.join(destination,publicName);
  let privateBytes;
  if(fs.existsSync(privatePath))privateBytes=keyBytes(privatePath,true);
  else{
   if(fs.existsSync(publicPath))throw Error('installation_key_incomplete');
   const pair=generateKeyPairSync('ed25519');
   privateBytes=pair.privateKey.export({format:'der',type:'pkcs8'});
   writeKey(privatePath,privateBytes);
  }
  const privateKey=createPrivateKey({key:privateBytes,format:'der',type:'pkcs8'});
  if(privateKey.asymmetricKeyType!=='ed25519')throw Error('installation_key_algorithm');
  const publicBytes=createPublicKey(privateKey).export({format:'der',type:'spki'});
  if(fs.existsSync(publicPath)){
   if(!keyBytes(publicPath).equals(publicBytes))throw Error('installation_key_pair_mismatch');
  }else writeKey(publicPath,publicBytes);
  const publicKey=publicBytes.toString('base64'),keyId=blake3Hex(publicBytes);
  return Object.freeze({keyId,publicKey,seal(lines,anchor){
   return {schema:'formal-ai-installation-seal-v1',algorithm:'Ed25519',keyId,publicKey,anchor,
    signature:Buffer.from(primitives.seal(lines,anchor,privateKey)).toString('base64')};
  }});
 }finally{fs.rmdirSync(lock);}
}
export function verifyInstallationSeal(lines,anchor,seal,trustedPublicKey){
 if(typeof trustedPublicKey!=='string'||!trustedPublicKey)throw Error('trusted_installation_key_required');
 // Canonical codec validation refuses getters, duplicate fields and unsupported scalar shapes.
 const record=decodeTyped(encodeTyped(seal));
 if(Object.keys(record).sort().join(',')!=='algorithm,anchor,keyId,publicKey,schema,signature'
   ||record.schema!=='formal-ai-installation-seal-v1'||record.algorithm!=='Ed25519'
   ||record.publicKey!==trustedPublicKey||encodeTyped(record.anchor)!==encodeTyped(anchor))throw Error('installation_seal_binding');
 const publicBytes=Buffer.from(trustedPublicKey,'base64'),signature=Buffer.from(record.signature,'base64');
 if(publicBytes.toString('base64')!==trustedPublicKey||signature.toString('base64')!==record.signature
   ||record.keyId!==blake3Hex(publicBytes))throw Error('installation_seal_encoding');
 const key=createPublicKey({key:publicBytes,format:'der',type:'spki'});
 primitives.verifySeal(lines,anchor,signature,key);
 return {keyId:record.keyId,signature:'verified',trust:'caller-pinned-installation-key'};
}
