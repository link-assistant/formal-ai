import React from "react";
import { OUTPUT_STANDARDS, normalizeLexicalDensity } from "./preferences.jsx";

const {createElement: h, Fragment} = React;

const DENSITIES = ["standard", "primes", "top-100", "top-200", "top-500", "top-1000", "max-precision"];

export function CnlSettings({t, outputStandard, setOutputStandard, lexicalDensity, setLexicalDensity}) {
  const tier = lexicalDensity.startsWith("top-") ? lexicalDensity.slice(4) : "";
  const [draft, setDraft] = React.useState(tier);
  React.useEffect(() => setDraft(tier), [tier]);
  return <>
    <p className="setting-note">{t("settings.outputVerification")}</p>
    <label className="setting-row"><span>{t("settings.outputStandard")}</span>
      <select data-testid="setting-output-standard" value={outputStandard} onChange={event => setOutputStandard(event.target.value)}>
        {OUTPUT_STANDARDS.map(standard => <option key={standard} value={standard}>{standard}</option>)}
      </select>
    </label>
    <label className="setting-row"><span>{t("settings.lexicalDensity")}</span>
      <select data-testid="setting-lexical-density" value={DENSITIES.includes(lexicalDensity) ? lexicalDensity : "custom"}
        onChange={event => setLexicalDensity(event.target.value === "custom" ? "top-37" : event.target.value)}>
        {DENSITIES.map(density => <option key={density} value={density}>{density}</option>)}
        <option value="custom">{t("settings.customVocabulary")}</option>
      </select>
    </label>
    {tier ? <label className="setting-row"><span>{t("settings.vocabularySize")}</span>
      <input data-testid="setting-vocabulary-size" type="text" inputMode="numeric" value={draft}
        onChange={event => {
          const value = event.target.value;
          setDraft(value);
          const density = "top-" + value;
          if (normalizeLexicalDensity(density) === density) setLexicalDensity(density);
        }} onBlur={() => setDraft(tier)} />
    </label> : null}
  </>;
}
