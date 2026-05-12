export type IssueReportEnvironmentValue = string | number | boolean | null | undefined;

export type IssueReportEnvironment = Record<string, IssueReportEnvironmentValue>;

export interface IssueReportResult {
  success: boolean;
  result?: string;
  linksNotation?: string;
  alternativeLinksNotations?: readonly string[];
  steps?: readonly string[];
  error?: string;
}

export interface CalculationResultLike extends IssueReportResult {
  result: string;
  lino_interpretation?: string;
  alternative_lino?: readonly string[];
  steps: readonly string[];
}

export interface PageState {
  expression: string;
  result: CalculationResultLike | null;
  wasmReady: boolean;
  version: string;
  theme: string;
  language: string;
  url: string;
  userAgent: string;
  timestamp: string;
}

/**
 * Translation labels for issue report sections.
 */
export interface IssueReportLabels {
  environment: string;
  version: string;
  url: string;
  userAgent: string;
  theme: string;
  language: string;
  wasmReady: string;
  timestamp: string;
  input: string;
  resultSection: string;
  resultLabel: string;
  linksNotation: string;
  alternativeInterpretations: string;
  stepsLabel: string;
  reproductionSteps: string;
  errorLabel: string;
  description: string;
  descriptionPlaceholder: string;
  yes: string;
  no: string;
  unknownVersion: string;
  unknownError: string;
}

export interface GenerateIssueReportOptions {
  input?: string;
  result?: IssueReportResult | null;
  linksNotation?: string;
  alternativeLinksNotations?: readonly string[];
  steps?: readonly string[];
  reproductionSteps?: readonly string[];
  environment?: IssueReportEnvironment;
  labels?: Partial<IssueReportLabels>;
}

export interface GenerateIssueUrlOptions extends GenerateIssueReportOptions {
  repository: string;
  title?: string;
  issueLabels?: string | readonly string[];
  githubBaseUrl?: string;
}

export interface IssueReportTranslator {
  (key: string, defaultValue: string): string;
  (key: string, options: Record<string, unknown>): string;
}

const DEFAULT_GITHUB_BASE_URL = 'https://github.com';
const CALCULATOR_REPOSITORY = 'link-assistant/calculator';
const DEFAULT_ISSUE_LABELS = ['bug'];
const ENVIRONMENT_ORDER = [
  'version',
  'url',
  'userAgent',
  'theme',
  'language',
  'wasmReady',
  'timestamp',
];

/**
 * Get default English labels for issue reports.
 */
export function getDefaultLabels(): IssueReportLabels {
  return {
    environment: 'Environment',
    version: 'Version',
    url: 'URL',
    userAgent: 'User Agent',
    theme: 'Theme',
    language: 'Language',
    wasmReady: 'WASM Ready',
    timestamp: 'Timestamp',
    input: 'Input',
    resultSection: 'Result',
    resultLabel: 'Result',
    linksNotation: 'Links Notation',
    alternativeInterpretations: 'Alternative interpretations',
    stepsLabel: 'Steps',
    reproductionSteps: 'Reproduction Steps',
    errorLabel: 'Error',
    description: 'Description',
    descriptionPlaceholder: 'Please describe the issue you encountered',
    yes: 'Yes',
    no: 'No',
    unknownVersion: 'Unknown',
    unknownError: 'Unknown error',
  };
}

/**
 * Get labels from an i18n-compatible translation function.
 */
export function getLabelsFromI18n(t: IssueReportTranslator): IssueReportLabels {
  return {
    environment: t('issueReport.environment', 'Environment'),
    version: t('issueReport.version', 'Version'),
    url: t('issueReport.url', 'URL'),
    userAgent: t('issueReport.userAgent', 'User Agent'),
    theme: t('issueReport.theme', 'Theme'),
    language: t('issueReport.language', 'Language'),
    wasmReady: t('issueReport.wasmReady', 'WASM Ready'),
    timestamp: t('issueReport.timestamp', 'Timestamp'),
    input: t('issueReport.input', 'Input'),
    resultSection: t('issueReport.resultSection', 'Result'),
    resultLabel: t('issueReport.resultLabel', 'Result'),
    linksNotation: t('issueReport.linksNotation', 'Links Notation'),
    alternativeInterpretations: t(
      'issueReport.alternativeInterpretations',
      'Alternative interpretations'
    ),
    stepsLabel: t('issueReport.stepsLabel', 'Steps'),
    reproductionSteps: t('issueReport.reproductionSteps', 'Reproduction Steps'),
    errorLabel: t('issueReport.errorLabel', 'Error'),
    description: t('issueReport.description', 'Description'),
    descriptionPlaceholder: t(
      'issueReport.descriptionPlaceholder',
      'Please describe the issue you encountered'
    ),
    yes: t('common.yes', 'Yes'),
    no: t('common.no', 'No'),
    unknownVersion: t('common.unknown', 'Unknown'),
    unknownError: t('errors.unknownError', 'Unknown error'),
  };
}

function mergeLabels(labels?: Partial<IssueReportLabels>): IssueReportLabels {
  return {
    ...getDefaultLabels(),
    ...labels,
  };
}

function isPageState(value: GenerateIssueReportOptions | PageState): value is PageState {
  return 'expression' in value || 'wasmReady' in value;
}

function getEnvironmentLabel(key: string, labels: IssueReportLabels): string {
  const knownLabels: Record<string, string> = {
    version: labels.version,
    url: labels.url,
    userAgent: labels.userAgent,
    theme: labels.theme,
    language: labels.language,
    wasmReady: labels.wasmReady,
    timestamp: labels.timestamp,
  };

  return knownLabels[key] || humanizeKey(key);
}

function humanizeKey(key: string): string {
  return key
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/[-_]+/g, ' ')
    .replace(/\b\w/g, char => char.toUpperCase());
}

function formatEnvironmentValue(
  key: string,
  value: IssueReportEnvironmentValue,
  labels: IssueReportLabels
): string {
  if (typeof value === 'boolean') {
    return value ? labels.yes : labels.no;
  }

  if (value === null || value === undefined || value === '') {
    return key === 'version' ? labels.unknownVersion : '';
  }

  return String(value);
}

function getOrderedEnvironmentEntries(environment: IssueReportEnvironment) {
  const knownEntries = ENVIRONMENT_ORDER
    .filter(key => Object.prototype.hasOwnProperty.call(environment, key))
    .map(key => [key, environment[key]] as const);
  const extraEntries = Object.entries(environment).filter(
    ([key]) => !ENVIRONMENT_ORDER.includes(key)
  );

  return [...knownEntries, ...extraEntries];
}

function normalizeResult(result: CalculationResultLike | IssueReportResult | null | undefined) {
  if (!result) {
    return null;
  }

  const legacyLinksNotation =
    'lino_interpretation' in result ? result.lino_interpretation : undefined;
  const legacyAlternativeLinksNotations =
    'alternative_lino' in result ? result.alternative_lino : undefined;

  return {
    success: result.success,
    result: result.result,
    linksNotation: result.linksNotation || legacyLinksNotation,
    alternativeLinksNotations:
      result.alternativeLinksNotations || legacyAlternativeLinksNotations,
    steps: result.steps || [],
    error: result.error,
  };
}

function getDefaultReproductionSteps(state: PageState): string[] {
  const steps = [`Open ${state.url}`];

  if (state.expression) {
    steps.push(`Enter ${state.expression}`);
    steps.push('Run the calculation');
  } else {
    steps.push('Use the calculator until the issue occurs');
  }

  steps.push('Click Report Issue');
  return steps;
}

function pageStateToReportOptions(
  state: PageState,
  labels?: IssueReportLabels
): GenerateIssueReportOptions {
  return {
    input: state.expression,
    result: normalizeResult(state.result),
    reproductionSteps: getDefaultReproductionSteps(state),
    environment: {
      version: state.version,
      url: state.url,
      userAgent: state.userAgent,
      theme: state.theme,
      language: state.language,
      wasmReady: state.wasmReady,
      timestamp: state.timestamp,
    },
    labels,
  };
}

function appendCodeBlock(sections: string[], value: string): void {
  sections.push('```');
  sections.push(value);
  sections.push('```');
}

function generateIssueReportFromOptions(options: GenerateIssueReportOptions): string {
  const labels = mergeLabels(options.labels);
  const result = normalizeResult(options.result);
  const linksNotation = options.linksNotation || result?.linksNotation;
  const alternativeLinksNotations =
    options.alternativeLinksNotations || result?.alternativeLinksNotations || [];
  const steps = options.steps || result?.steps || [];
  const sections: string[] = [];

  const environmentEntries = getOrderedEnvironmentEntries(options.environment || {});
  if (environmentEntries.length > 0) {
    sections.push(`## ${labels.environment}`);
    sections.push('');
    for (const [key, value] of environmentEntries) {
      const label = getEnvironmentLabel(key, labels);
      const formattedValue = formatEnvironmentValue(key, value, labels);
      sections.push(`- **${label}**: ${formattedValue}`);
    }
  }

  if (options.input) {
    sections.push('');
    sections.push(`## ${labels.input}`);
    sections.push('');
    appendCodeBlock(sections, options.input);
  }

  if (result || linksNotation || alternativeLinksNotations.length > 0 || steps.length > 0) {
    sections.push('');
    sections.push(`## ${labels.resultSection}`);
    sections.push('');

    if (result) {
      if (result.success) {
        if (result.result !== undefined && result.result !== '') {
          sections.push(`**${labels.resultLabel}**: ${result.result}`);
        }
      } else {
        sections.push(`**${labels.errorLabel}**: ${result.error || labels.unknownError}`);
      }
    }

    if (linksNotation) {
      sections.push('');
      sections.push(`**${labels.linksNotation}**:`);
      appendCodeBlock(sections, linksNotation);
    }

    if (alternativeLinksNotations.length > 1) {
      sections.push('');
      sections.push(`**${labels.alternativeInterpretations}**:`);
      for (const alternative of alternativeLinksNotations) {
        appendCodeBlock(sections, alternative);
      }
    }

    if (steps.length > 0) {
      sections.push('');
      sections.push(`**${labels.stepsLabel}**:`);
      steps.forEach((step, i) => {
        sections.push(`${i + 1}. ${step}`);
      });
    }
  }

  if (options.reproductionSteps && options.reproductionSteps.length > 0) {
    sections.push('');
    sections.push(`## ${labels.reproductionSteps}`);
    sections.push('');
    options.reproductionSteps.forEach((step, i) => {
      sections.push(`${i + 1}. ${step}`);
    });
  }

  sections.push('');
  sections.push(`## ${labels.description}`);
  sections.push('');
  sections.push(`<!-- ${labels.descriptionPlaceholder} -->`);
  sections.push('');

  return sections.join('\n');
}

/**
 * Generate a markdown report for issue reporting.
 * @param optionsOrState - Reusable report options or calculator page state
 * @param labels - Translation labels for legacy page-state calls
 */
export function generateIssueReport(
  optionsOrState: GenerateIssueReportOptions | PageState,
  labels?: IssueReportLabels
): string {
  const options = isPageState(optionsOrState)
    ? pageStateToReportOptions(optionsOrState, labels || getDefaultLabels())
    : optionsOrState;

  return generateIssueReportFromOptions(options);
}

function translateIssueTitle(
  t: IssueReportTranslator | undefined,
  key: string,
  defaultValue: string,
  interpolation?: Record<string, string>
): string {
  if (!t) {
    return defaultValue;
  }

  if (interpolation) {
    return t(key, {
      ...interpolation,
      defaultValue,
    });
  }

  return t(key, defaultValue);
}

/**
 * Generate issue title with optional i18n-compatible translation support.
 * @param expression - The expression that caused the issue
 * @param t - Translation function (optional, defaults to English)
 */
export function generateIssueTitle(expression: string = '', t?: IssueReportTranslator): string {
  if (expression) {
    const truncated = expression.slice(0, 50);
    const ellipsis = expression.length > 50 ? '...' : '';
    const title = `Issue with expression: ${truncated}${ellipsis}`;

    return translateIssueTitle(t, 'issueReport.issueTitle', title, {
      expression: `${truncated}${ellipsis}`,
    });
  }

  return translateIssueTitle(t, 'issueReport.defaultIssueTitle', 'Issue report');
}

function normalizeRepository(repository: string): string {
  const normalized = repository.trim().replace(/^\/+|\/+$/g, '');

  if (!/^[^/]+\/[^/]+$/.test(normalized)) {
    throw new Error(`Invalid GitHub repository: ${repository}`);
  }

  return normalized;
}

function normalizeIssueLabels(issueLabels: string | readonly string[] | undefined): string {
  if (typeof issueLabels === 'string') {
    return issueLabels.trim();
  }

  if (issueLabels) {
    return issueLabels.map(label => label.trim()).filter(Boolean).join(',');
  }

  return '';
}

function generateIssueUrlFromOptions(options: GenerateIssueUrlOptions): string {
  const report = generateIssueReportFromOptions(options);
  const title = options.title || generateIssueTitle(options.input || '');
  const repository = normalizeRepository(options.repository);
  const githubBaseUrl = (options.githubBaseUrl || DEFAULT_GITHUB_BASE_URL).replace(/\/+$/g, '');
  const baseUrl = `${githubBaseUrl}/${repository}/issues/new`;
  const params = new URLSearchParams({
    title,
    body: report,
  });
  const issueLabels = normalizeIssueLabels(options.issueLabels ?? DEFAULT_ISSUE_LABELS);

  if (issueLabels) {
    params.set('labels', issueLabels);
  }

  return `${baseUrl}?${params.toString()}`;
}

/**
 * Generate a GitHub issue URL with prefilled content.
 * @param optionsOrState - Reusable issue URL options or calculator page state
 * @param t - Translation function for legacy calculator page-state calls
 */
export function generateIssueUrl(
  optionsOrState: GenerateIssueUrlOptions | PageState,
  t?: IssueReportTranslator
): string {
  if ('repository' in optionsOrState) {
    return generateIssueUrlFromOptions(optionsOrState);
  }

  const labels = t ? getLabelsFromI18n(t) : getDefaultLabels();
  return generateIssueUrlFromOptions({
    ...pageStateToReportOptions(optionsOrState, labels),
    repository: CALCULATOR_REPOSITORY,
    title: generateIssueTitle(optionsOrState.expression, t),
    issueLabels: DEFAULT_ISSUE_LABELS,
  });
}
