// Presentation data from actual preceding records; no solver or intent inference.
export function conversationSummaryRows(message,messages) {
 if(message?.role!=='assistant'||message.intent!=='summarize_conversation'
  ||!Array.isArray(message.evidence)||message.evidence.includes('summarization:format:plain')||!Array.isArray(messages))return null;
 const index=messages.indexOf(message);
 if(index<0)return null;
 return messages.slice(0,index)
  .filter(turn=>['user','assistant'].includes(turn?.role)&&typeof turn.content==='string'&&turn.content.trim()!=='')
  .map(turn=>({id:typeof turn.id==='string'?turn.id:null,role:turn.role,
   intent:typeof turn.intent==='string'&&turn.intent.trim()!==''?turn.intent:null,content:turn.content}));
}
