# Issue #423 online research

Captured on 2026-06-12 for
<https://github.com/link-assistant/formal-ai/issues/423>.

## GitHub issue and PR inputs

Commands used:

```sh
gh issue view https://github.com/link-assistant/formal-ai/issues/423 --json number,title,state,body,comments
gh api repos/link-assistant/formal-ai/issues/423/comments --paginate
gh pr view 424 --repo link-assistant/formal-ai --json number,title,state,isDraft,headRefName,baseRefName,url
gh api repos/link-assistant/formal-ai/pulls/424/comments --paginate
gh api repos/link-assistant/formal-ai/issues/424/comments --paginate
gh api repos/link-assistant/formal-ai/pulls/424/reviews --paginate
```

Findings:

- Issue #423 was open and requested README.md installation/deployment guide
  conversion to `sh`/PowerShell scripts and back.
- No issue comments were present.
- PR #424 already existed as a draft on branch `issue-423-85026fdc955a`.
- No PR conversation comments, review comments, or reviews were present.

The raw JSON outputs are committed beside this file.

## Popular GitHub project corpus

The issue asked for at least 50 tests covering popular GitHub projects. The
repository snapshot was captured with:

```sh
gh api 'search/repositories?q=stars:%3E1000&sort=stars&order=desc&per_page=50'
```

The raw result is stored in `github-top-50-repositories.json`. The 50 names and
star counts at capture time were:

| # | Repository | Stars |
|---|------------|-------|
| 1 | `codecrafters-io/build-your-own-x` | 514545 |
| 2 | `sindresorhus/awesome` | 474997 |
| 3 | `freeCodeCamp/freeCodeCamp` | 446651 |
| 4 | `public-apis/public-apis` | 440939 |
| 5 | `EbookFoundation/free-programming-books` | 390115 |
| 6 | `openclaw/openclaw` | 378280 |
| 7 | `nilbuild/developer-roadmap` | 356821 |
| 8 | `donnemartin/system-design-primer` | 352714 |
| 9 | `jwasham/coding-interview-university` | 351180 |
| 10 | `vinta/awesome-python` | 302481 |
| 11 | `awesome-selfhosted/awesome-selfhosted` | 298648 |
| 12 | `996icu/996.ICU` | 276276 |
| 13 | `practical-tutorials/project-based-learning` | 268685 |
| 14 | `react/react` | 245780 |
| 15 | `torvalds/linux` | 236163 |
| 16 | `trimstray/the-book-of-secret-knowledge` | 227902 |
| 17 | `obra/superpowers` | 225329 |
| 18 | `TheAlgorithms/Python` | 221866 |
| 19 | `affaan-m/ECC` | 213794 |
| 20 | `vuejs/vue` | 209848 |
| 21 | `ossu/computer-science` | 204829 |
| 22 | `trekhleb/javascript-algorithms` | 196069 |
| 23 | `tensorflow/tensorflow` | 195606 |
| 24 | `ultraworkers/claw-code` | 193666 |
| 25 | `n8n-io/n8n` | 192134 |
| 26 | `NousResearch/hermes-agent` | 191350 |
| 27 | `ohmyzsh/ohmyzsh` | 187941 |
| 28 | `microsoft/vscode` | 186187 |
| 29 | `Significant-Gravitas/AutoGPT` | 184899 |
| 30 | `CyC2018/CS-Notes` | 184527 |
| 31 | `getify/You-Dont-Know-JS` | 184518 |
| 32 | `jackfrued/Python-100-Days` | 183292 |
| 33 | `massgravel/Microsoft-Activation-Scripts` | 178120 |
| 34 | `flutter/flutter` | 176889 |
| 35 | `DigitalPlatDev/FreeDomain` | 176853 |
| 36 | `avelino/awesome-go` | 175217 |
| 37 | `github/gitignore` | 174408 |
| 38 | `twbs/bootstrap` | 174309 |
| 39 | `ollama/ollama` | 173923 |
| 40 | `multica-ai/andrej-karpathy-skills` | 173801 |
| 41 | `anomalyco/opencode` | 173445 |
| 42 | `yt-dlp/yt-dlp` | 169976 |
| 43 | `AUTOMATIC1111/stable-diffusion-webui` | 163633 |
| 44 | `f/prompts.chat` | 163604 |
| 45 | `huggingface/transformers` | 161524 |
| 46 | `jlevy/the-art-of-command-line` | 161286 |
| 47 | `521xueweihan/HelloGitHub` | 161005 |
| 48 | `Snailclimb/JavaGuide` | 156328 |
| 49 | `microsoft/markitdown` | 151550 |
| 50 | `anthropics/skills` | 149696 |

## Test corpus derived from the snapshot

Each repository feeds one README-to-`sh` conversion prompt in
`tests/unit/installation_conversion.rs`. The commands are intentionally simple
installation or verification commands so the test checks conversion routing and
command preservation rather than live project installation.

| Repository | Install command | Verify command |
|------------|-----------------|----------------|
| `codecrafters-io/build-your-own-x` | `git clone https://github.com/codecrafters-io/build-your-own-x.git` | `make test` |
| `sindresorhus/awesome` | `git clone https://github.com/sindresorhus/awesome.git` | `npm test` |
| `freeCodeCamp/freeCodeCamp` | `pnpm install` | `pnpm test` |
| `public-apis/public-apis` | `git clone https://github.com/public-apis/public-apis.git` | `npx awesome-lint` |
| `EbookFoundation/free-programming-books` | `git clone https://github.com/EbookFoundation/free-programming-books.git` | `npm test` |
| `openclaw/openclaw` | `cmake -S . -B build` | `cmake --build build` |
| `nilbuild/developer-roadmap` | `pnpm install` | `pnpm build` |
| `donnemartin/system-design-primer` | `git clone https://github.com/donnemartin/system-design-primer.git` | `python -m pytest` |
| `jwasham/coding-interview-university` | `git clone https://github.com/jwasham/coding-interview-university.git` | `npm test` |
| `vinta/awesome-python` | `git clone https://github.com/vinta/awesome-python.git` | `python -m pytest` |
| `awesome-selfhosted/awesome-selfhosted` | `git clone https://github.com/awesome-selfhosted/awesome-selfhosted.git` | `npx awesome-lint` |
| `996icu/996.ICU` | `git clone https://github.com/996icu/996.ICU.git` | `npm test` |
| `practical-tutorials/project-based-learning` | `git clone https://github.com/practical-tutorials/project-based-learning.git` | `npm test` |
| `react/react` | `yarn install` | `yarn test` |
| `torvalds/linux` | `make defconfig` | `make` |
| `trimstray/the-book-of-secret-knowledge` | `git clone https://github.com/trimstray/the-book-of-secret-knowledge.git` | `npm test` |
| `obra/superpowers` | `git clone https://github.com/obra/superpowers.git` | `npm test` |
| `TheAlgorithms/Python` | `python -m pip install -r requirements.txt` | `python -m pytest` |
| `affaan-m/ECC` | `git clone https://github.com/affaan-m/ECC.git` | `python -m pytest` |
| `vuejs/vue` | `pnpm install` | `pnpm test` |
| `ossu/computer-science` | `git clone https://github.com/ossu/computer-science.git` | `npm test` |
| `trekhleb/javascript-algorithms` | `npm install` | `npm test` |
| `tensorflow/tensorflow` | `python -m pip install tensorflow` | `python -c "import tensorflow as tf; print(tf.__version__)"` |
| `ultraworkers/claw-code` | `npm install` | `npm test` |
| `n8n-io/n8n` | `pnpm install` | `pnpm test` |
| `NousResearch/hermes-agent` | `python -m pip install -r requirements.txt` | `python -m pytest` |
| `ohmyzsh/ohmyzsh` | `sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"` | `zsh --version` |
| `microsoft/vscode` | `yarn install` | `yarn compile` |
| `Significant-Gravitas/AutoGPT` | `python -m pip install -r requirements.txt` | `python -m pytest` |
| `CyC2018/CS-Notes` | `git clone https://github.com/CyC2018/CS-Notes.git` | `npm test` |
| `getify/You-Dont-Know-JS` | `git clone https://github.com/getify/You-Dont-Know-JS.git` | `npm test` |
| `jackfrued/Python-100-Days` | `python -m pip install -r requirements.txt` | `python -m pytest` |
| `massgravel/Microsoft-Activation-Scripts` | `irm https://get.activated.win \| iex` | `powershell -NoProfile -Command "$PSVersionTable.PSVersion"` |
| `flutter/flutter` | `git clone https://github.com/flutter/flutter.git` | `flutter doctor` |
| `DigitalPlatDev/FreeDomain` | `git clone https://github.com/DigitalPlatDev/FreeDomain.git` | `npm test` |
| `avelino/awesome-go` | `git clone https://github.com/avelino/awesome-go.git` | `go test ./...` |
| `github/gitignore` | `git clone https://github.com/github/gitignore.git` | `npm test` |
| `twbs/bootstrap` | `npm install` | `npm test` |
| `ollama/ollama` | `curl -fsSL https://ollama.com/install.sh \| sh` | `ollama --version` |
| `multica-ai/andrej-karpathy-skills` | `git clone https://github.com/multica-ai/andrej-karpathy-skills.git` | `npm test` |
| `anomalyco/opencode` | `npm install -g opencode-ai` | `opencode --version` |
| `yt-dlp/yt-dlp` | `python -m pip install -U yt-dlp` | `yt-dlp --version` |
| `AUTOMATIC1111/stable-diffusion-webui` | `./webui.sh` | `python launch.py --help` |
| `f/prompts.chat` | `git clone https://github.com/f/prompts.chat.git` | `npm test` |
| `huggingface/transformers` | `python -m pip install transformers` | `python -c "import transformers; print(transformers.__version__)"` |
| `jlevy/the-art-of-command-line` | `git clone https://github.com/jlevy/the-art-of-command-line.git` | `npm test` |
| `521xueweihan/HelloGitHub` | `python -m pip install -r requirements.txt` | `python -m pytest` |
| `Snailclimb/JavaGuide` | `git clone https://github.com/Snailclimb/JavaGuide.git` | `mvn test` |
| `microsoft/markitdown` | `python -m pip install markitdown` | `markitdown --help` |
| `anthropics/skills` | `git clone https://github.com/anthropics/skills.git` | `npm test` |

## Research conclusion

The corpus spans common README installation command families: clone-and-enter,
Node package managers (`npm`, `pnpm`, `yarn`), Python package/test commands,
CMake, Make, Go, Maven, Flutter, curl pipes, PowerShell one-liners, and project
provided shell scripts. A single ordered command IR is enough for these cases:
the conversion does not need repository-specific templates, only accurate
surface detection, command extraction, and target rendering.
