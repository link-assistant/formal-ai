// UI details over actual preceding message records; the solver answer stays intact.
import React from 'react';
import {conversationSummaryRows} from '../conversation-summary-history.js';
const {createElement:h}=React;

export function ConversationSummaryDetails({message,messages,t,className='',ariaHidden=false}) {
 const rows=conversationSummaryRows(message,messages);
 if(rows===null||rows.length===0)return null;
 return h('section',{'data-testid':'conversation-summary-details',className,'aria-hidden':ariaHidden?true:null},
  h('h3',null,t('message.conversationSummary.title')),
  h('table',null,
   h('thead',null,h('tr',null,...['role','intent','content'].map(key=>h('th',{key,scope:'col'},t('message.conversationSummary.'+key))))),
   h('tbody',null,...rows.map((row,index)=>h('tr',{key:row.id??index,'data-message-id':row.id},
    h('td',null,row.role),h('td',null,row.intent===null?t('message.conversationSummary.unrecorded'):row.intent),
    h('td',null,row.content))))));
}
