// Supplied Playwright MCP experiment: zero independent authorship credit.
async (page) => {
  await page.setViewportSize({width:1120,height:1000});
  await page.context().route('**/*', route => {
    const url = new URL(route.request().url());
    return ['127.0.0.1','localhost'].includes(url.hostname) ? route.continue() : route.abort('blockedbyclient');
  });
  const results = [];
  for (const [phase,port] of [['before',8048],['after',8049]]) {
    await page.goto(`http://127.0.0.1:${port}/meaning-regression/`);
    await page.waitForFunction(() => window.harnessComplete || window.harnessError, {}, {timeout:50000});
    const observed = await page.evaluate(async (phase) => {
      if (window.harnessError || !window.harnessComplete) throw new Error(window.harnessError || 'Incomplete harness');
      const observation = {phase,complete:true,url:location.href,viewport:{width:innerWidth,height:innerHeight},
        userAgent:navigator.userAgent,observed:window.observed,body:document.body.innerText,
        nativeBehaviorVerified:false,productionAppUiVerified:false,independentAuthorshipCredit:0};
      const response = await fetch('/__experiment_observation', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(observation,null,2)+'\n'});
      if (!response.ok) throw new Error('Browser evidence save failed: '+response.status);
      return {receipt:await response.json(),summary:observation.observed.map(x=>({prompt:x.prompt,intent:x.intent,engine:x.engine}))};
    },phase);
    const expected = phase === 'before' ? ['unknown','unknown'] : ['web_search','unknown'];
    if (JSON.stringify(observed.summary.map(x=>x.intent)) !== JSON.stringify(expected)) throw new Error('Unexpected sibling routes');
    await page.screenshot({path:`/tmp/gh-issue-solver-1794900398048/docs/screenshots/issue-1580/meaning-hydration-${phase}.png`,fullPage:true,scale:'css'});
    results.push({phase,...observed});
  }
  return results;
}
