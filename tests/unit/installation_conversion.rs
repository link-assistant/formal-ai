use formal_ai::FormalAiEngine;

#[test]
fn readme_install_guide_converts_to_bash_and_powershell() {
    let prompt = r"Convert this README.md installation guide into both sh and PowerShell scripts:

```markdown
## Installation

1. Clone the project.
   `git clone https://github.com/example/widget.git`
2. Enter the directory.
   `cd widget`
3. Install dependencies.
   `npm install`
4. Build the project.
   `npm run build`
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("installation_conversion_request"));
    assert!(response.answer.contains("source_format markdown"));
    assert!(response.answer.contains("target_format shell_script"));
    assert!(response.answer.contains("target_format powershell_script"));
    assert!(response.answer.contains("```bash"));
    assert!(response.answer.contains("```powershell"));
    assert!(response
        .answer
        .contains("git clone https://github.com/example/widget.git"));
    assert!(response.answer.contains("npm run build"));
}

#[test]
fn wrapped_readme_with_nested_shell_fences_converts_to_scripts() {
    let prompt = r"Convert this README.md installation guide for react/react into both sh and PowerShell scripts:

```markdown
## Installation

1. Clone the repository.

   ```sh
   git clone https://github.com/react/react.git
   cd react
   ```

2. Install and verify.

   ```sh
   yarn install
   yarn test
   ```
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("target_format shell_script"));
    assert!(response.answer.contains("target_format powershell_script"));
    assert!(response
        .answer
        .contains("git clone https://github.com/react/react.git"));
    assert!(response.answer.contains("cd react"));
    assert!(response.answer.contains("yarn install"));
    assert!(response.answer.contains("yarn test"));
}

#[test]
fn unwrapped_readme_with_shell_fences_stays_markdown_source() {
    let prompt = r"Convert this README.md installation guide for example/widget into a sh script:

## Installation

Clone the repository:

```sh
git clone https://github.com/example/widget.git
cd widget
```

Install and verify:

```sh
npm install
npm test
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("source_format markdown"));
    assert!(response.answer.contains("target_format shell_script"));
    assert!(response
        .answer
        .contains("git clone https://github.com/example/widget.git"));
    assert!(response.answer.contains("cd widget"));
    assert!(response.answer.contains("npm install"));
    assert!(response.answer.contains("npm test"));
}

#[test]
fn install_script_converts_back_to_readme_guide() {
    let prompt = r"Convert this shell installation script back to a README.md installation guide:

```bash
#!/usr/bin/env bash
set -euo pipefail
git clone https://github.com/ollama/ollama.git
cd ollama
curl -fsSL https://ollama.com/install.sh | sh
ollama serve
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("installation_conversion_request"));
    assert!(response.answer.contains("source_format shell_script"));
    assert!(response.answer.contains("target_format markdown"));
    assert!(response.answer.contains("README.md installation guide"));
    assert!(response
        .answer
        .contains("git clone https://github.com/ollama/ollama.git"));
    assert!(response.answer.contains("ollama serve"));
}

#[test]
fn install_conversion_prompts_route_across_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
        command: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            prompt: "Convert this README.md installation guide into a sh script:\n\
                     ## Installation\n\
                     1. Run `npm install`.\n",
            command: "npm install",
        },
        Case {
            language: "ru",
            prompt: "Преобразуй это README.md руководство по установке в sh скрипт:\n\
                     ## Установка\n\
                     1. Выполни `npm install`.\n",
            command: "npm install",
        },
        Case {
            language: "hi",
            prompt: "इस README.md स्थापना guide को sh script में बदलें:\n\
                     ## स्थापना\n\
                     1. चलाएं `npm install`.\n",
            command: "npm install",
        },
        Case {
            language: "zh",
            prompt: "请把这个 README.md 安装指南转换为 sh 脚本:\n\
                     ## 安装\n\
                     1. 运行 `npm install`.\n",
            command: "npm install",
        },
    ];

    for case in cases {
        let response = FormalAiEngine.answer(case.prompt);

        assert_eq!(
            response.intent, "installation_conversion",
            "language: {}, answer was: {}",
            case.language, response.answer
        );
        assert!(response.answer.contains("source_format markdown"));
        assert!(response.answer.contains("target_format shell_script"));
        assert!(response.answer.contains(case.command));
    }
}

#[test]
fn popular_github_projects_route_through_install_conversion() {
    let cases = [
        ("codecrafters-io/build-your-own-x", "git clone https://github.com/codecrafters-io/build-your-own-x.git", "make test"),
        ("sindresorhus/awesome", "git clone https://github.com/sindresorhus/awesome.git", "npm test"),
        ("freeCodeCamp/freeCodeCamp", "pnpm install", "pnpm test"),
        ("public-apis/public-apis", "git clone https://github.com/public-apis/public-apis.git", "npx awesome-lint"),
        ("EbookFoundation/free-programming-books", "git clone https://github.com/EbookFoundation/free-programming-books.git", "npm test"),
        ("openclaw/openclaw", "cmake -S . -B build", "cmake --build build"),
        ("nilbuild/developer-roadmap", "pnpm install", "pnpm build"),
        ("donnemartin/system-design-primer", "git clone https://github.com/donnemartin/system-design-primer.git", "python -m pytest"),
        ("jwasham/coding-interview-university", "git clone https://github.com/jwasham/coding-interview-university.git", "npm test"),
        ("vinta/awesome-python", "git clone https://github.com/vinta/awesome-python.git", "python -m pytest"),
        ("awesome-selfhosted/awesome-selfhosted", "git clone https://github.com/awesome-selfhosted/awesome-selfhosted.git", "npx awesome-lint"),
        ("996icu/996.ICU", "git clone https://github.com/996icu/996.ICU.git", "npm test"),
        ("practical-tutorials/project-based-learning", "git clone https://github.com/practical-tutorials/project-based-learning.git", "npm test"),
        ("react/react", "yarn install", "yarn test"),
        ("torvalds/linux", "make defconfig", "make"),
        ("trimstray/the-book-of-secret-knowledge", "git clone https://github.com/trimstray/the-book-of-secret-knowledge.git", "npm test"),
        ("obra/superpowers", "git clone https://github.com/obra/superpowers.git", "npm test"),
        ("TheAlgorithms/Python", "python -m pip install -r requirements.txt", "python -m pytest"),
        ("affaan-m/ECC", "git clone https://github.com/affaan-m/ECC.git", "python -m pytest"),
        ("vuejs/vue", "pnpm install", "pnpm test"),
        ("ossu/computer-science", "git clone https://github.com/ossu/computer-science.git", "npm test"),
        ("trekhleb/javascript-algorithms", "npm install", "npm test"),
        ("tensorflow/tensorflow", "python -m pip install tensorflow", "python -c \"import tensorflow as tf; print(tf.__version__)\""),
        ("ultraworkers/claw-code", "npm install", "npm test"),
        ("n8n-io/n8n", "pnpm install", "pnpm test"),
        ("NousResearch/hermes-agent", "python -m pip install -r requirements.txt", "python -m pytest"),
        ("ohmyzsh/ohmyzsh", "sh -c \"$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\"", "zsh --version"),
        ("microsoft/vscode", "yarn install", "yarn compile"),
        ("Significant-Gravitas/AutoGPT", "python -m pip install -r requirements.txt", "python -m pytest"),
        ("CyC2018/CS-Notes", "git clone https://github.com/CyC2018/CS-Notes.git", "npm test"),
        ("getify/You-Dont-Know-JS", "git clone https://github.com/getify/You-Dont-Know-JS.git", "npm test"),
        ("jackfrued/Python-100-Days", "python -m pip install -r requirements.txt", "python -m pytest"),
        ("massgravel/Microsoft-Activation-Scripts", "irm https://get.activated.win | iex", "powershell -NoProfile -Command \"$PSVersionTable.PSVersion\""),
        ("flutter/flutter", "git clone https://github.com/flutter/flutter.git", "flutter doctor"),
        ("DigitalPlatDev/FreeDomain", "git clone https://github.com/DigitalPlatDev/FreeDomain.git", "npm test"),
        ("avelino/awesome-go", "git clone https://github.com/avelino/awesome-go.git", "go test ./..."),
        ("github/gitignore", "git clone https://github.com/github/gitignore.git", "npm test"),
        ("twbs/bootstrap", "npm install", "npm test"),
        ("ollama/ollama", "curl -fsSL https://ollama.com/install.sh | sh", "ollama --version"),
        ("multica-ai/andrej-karpathy-skills", "git clone https://github.com/multica-ai/andrej-karpathy-skills.git", "npm test"),
        ("anomalyco/opencode", "npm install -g opencode-ai", "opencode --version"),
        ("yt-dlp/yt-dlp", "python -m pip install -U yt-dlp", "yt-dlp --version"),
        ("AUTOMATIC1111/stable-diffusion-webui", "./webui.sh", "python launch.py --help"),
        ("f/prompts.chat", "git clone https://github.com/f/prompts.chat.git", "npm test"),
        ("huggingface/transformers", "python -m pip install transformers", "python -c \"import transformers; print(transformers.__version__)\""),
        ("jlevy/the-art-of-command-line", "git clone https://github.com/jlevy/the-art-of-command-line.git", "npm test"),
        ("521xueweihan/HelloGitHub", "python -m pip install -r requirements.txt", "python -m pytest"),
        ("Snailclimb/JavaGuide", "git clone https://github.com/Snailclimb/JavaGuide.git", "mvn test"),
        ("microsoft/markitdown", "python -m pip install markitdown", "markitdown --help"),
        ("anthropics/skills", "git clone https://github.com/anthropics/skills.git", "npm test"),
    ];

    assert_eq!(
        cases.len(),
        50,
        "issue #423 requires at least 50 popular-project conversion cases"
    );

    for (repo, install_command, verify_command) in cases {
        let prompt = format!(
            "Convert this README.md installation guide for {repo} into a sh script:\n\
             ## Installation\n\
             1. Run `{install_command}`.\n\
             2. Verify with `{verify_command}`.\n"
        );

        let response = FormalAiEngine.answer(&prompt);

        assert_eq!(
            response.intent, "installation_conversion",
            "repo {repo} returned {}: {}",
            response.intent, response.answer
        );
        assert!(response.answer.contains("source_format markdown"));
        assert!(response.answer.contains("target_format shell_script"));
        assert!(response.answer.contains(install_command));
        assert!(response.answer.contains(verify_command));
    }
}
