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
fn powershell_install_script_converts_back_to_readme_guide() {
    let prompt = r#"Convert this PowerShell installation script back to a README.md installation guide:

```powershell
$ErrorActionPreference = 'Stop'
irm https://get.activated.win | iex
powershell -NoProfile -Command "$PSVersionTable.PSVersion"
```
"#;

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(response.answer.contains("source_format powershell_script"));
    assert!(response.answer.contains("target_format markdown"));
    assert!(response.answer.contains("README.md installation guide"));
    assert!(response
        .answer
        .contains("irm https://get.activated.win | iex"));
    assert!(response
        .answer
        .contains("powershell -NoProfile -Command \"$PSVersionTable.PSVersion\""));
}

#[test]
fn conversion_answer_exposes_algorithm_construction_trace() {
    let prompt = r"Convert this README.md installation guide into a sh script and show the meta algorithm:

```markdown
## Installation

1. Install dependencies.
   `python -m pip install -r requirements.txt`
2. Verify the package.
   `python -m pytest`
```
";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("Meta algorithm for constructing conversion algorithms"),
        "answer should expose the algorithm-construction layer, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("shared intermediate representation")
            && response.answer.contains("verification fixture"),
        "meta algorithm should name IR construction and verification, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("program_blueprint")
            && response.answer.contains("rule_synthesis")
            && response.answer.contains("numeric_list"),
        "meta algorithm should connect existing coding surfaces, got: {}",
        response.answer
    );
    assert!(
        response
            .links_notation
            .contains("algorithm_construction:stage"),
        "trace should record algorithm construction stages, got: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("coding_surface")
            && response.links_notation.contains("program_blueprint"),
        "formal meaning should record producible coding surfaces, got: {}",
        response.links_notation
    );
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
        ("langflow-ai/langflow", "python -m pip install langflow", "python -m langflow run --help"),
        ("airbnb/javascript", "git clone https://github.com/airbnb/javascript.git", "npm test"),
        ("langgenius/dify", "docker compose up -d", "docker ps"),
        ("Genymobile/scrcpy", "sudo apt install scrcpy", "bash -lc \"scrcpy --version\""),
        ("open-webui/open-webui", "docker run -d -p 3000:8080 ghcr.io/open-webui/open-webui:main", "docker ps"),
        ("ytdl-org/youtube-dl", "python -m pip install -U youtube_dl", "python -m youtube_dl --version"),
        ("yangshun/tech-interview-handbook", "pnpm install", "pnpm test"),
        ("x1xhlol/system-prompts-and-models-of-ai-tools", "git clone https://github.com/x1xhlol/system-prompts-and-models-of-ai-tools.git", "npm test"),
        ("vercel/next.js", "pnpm install", "pnpm test"),
        ("langchain-ai/langchain", "python -m pip install langchain", "python -c \"import langchain; print(langchain.__version__)\""),
        ("golang/go", "git clone https://github.com/golang/go.git", "bash ./src/all.bash"),
        ("microsoft/PowerToys", "git clone https://github.com/microsoft/PowerToys.git", "powershell -NoProfile -Command \"Write-Output PowerToys\""),
        ("labuladong/fucking-algorithm", "git clone https://github.com/labuladong/fucking-algorithm.git", "npm test"),
        ("anthropics/claude-code", "npm install -g @anthropic-ai/claude-code", "npm view @anthropic-ai/claude-code version"),
        ("firecrawl/firecrawl", "pnpm install", "pnpm test"),
        ("Chalarangelo/30-seconds-of-code", "npm install", "npm test"),
        ("krahets/hello-algo", "git clone https://github.com/krahets/hello-algo.git", "npm test"),
        ("mattpocock/skills", "git clone https://github.com/mattpocock/skills.git", "bash --version"),
        ("react/react-native", "yarn install", "yarn test"),
        ("excalidraw/excalidraw", "yarn install", "yarn test"),
        ("clash-verge-rev/clash-verge-rev", "pnpm install", "pnpm test"),
        ("ripienaar/free-for-dev", "git clone https://github.com/ripienaar/free-for-dev.git", "npm test"),
        ("kubernetes/kubernetes", "git clone https://github.com/kubernetes/kubernetes.git", "make test"),
        ("electron/electron", "npm install", "npm test"),
        ("iptv-org/iptv", "npm install", "npm test"),
        ("nodejs/node", "python configure.py", "make test"),
        ("justjavac/free-programming-books-zh_CN", "git clone https://github.com/justjavac/free-programming-books-zh_CN.git", "npm test"),
        ("Comfy-Org/ComfyUI", "python -m pip install -r requirements.txt", "python main.py --help"),
        ("shadcn-ui/ui", "pnpm install", "pnpm test"),
        ("ggml-org/llama.cpp", "cmake -B build", "cmake --build build"),
        ("rustdesk/rustdesk", "cargo build", "cargo test"),
        ("Shubhamsaboo/awesome-llm-apps", "python -m pip install -r requirements.txt", "python -m pytest"),
        ("Hack-with-Github/Awesome-Hacking", "git clone https://github.com/Hack-with-Github/Awesome-Hacking.git", "npm test"),
        ("rust-lang/rust", "git clone https://github.com/rust-lang/rust.git", "python x.py test library/std"),
        ("d3/d3", "npm install", "npm test"),
        ("mrdoob/three.js", "npm install", "npm test"),
        ("godotengine/godot", "python -m pip install scons", "python -c \"import SCons; print(SCons.__version__)\""),
        ("msitarzewski/agency-agents", "git clone https://github.com/msitarzewski/agency-agents.git", "bash --version"),
        ("microsoft/generative-ai-for-beginners", "git clone https://github.com/microsoft/generative-ai-for-beginners.git", "python -m pytest"),
        ("github/spec-kit", "python -m pip install -e .", "python -m pytest"),
        ("garrytan/gstack", "npm install", "npm test"),
        ("microsoft/TypeScript", "npm install", "npm test"),
        ("axios/axios", "npm install", "npm test"),
        ("2dust/v2rayN", "git clone https://github.com/2dust/v2rayN.git", "powershell -NoProfile -Command \"Write-Output v2rayN\""),
        ("GrowingGit/GitHub-Chinese-Top-Charts", "git clone https://github.com/GrowingGit/GitHub-Chinese-Top-Charts.git", "mvn test"),
        ("tauri-apps/tauri", "cargo build", "cargo test"),
        ("fatedier/frp", "go test ./...", "go test ./cmd/..."),
        ("denoland/deno", "cargo build", "cargo test"),
        ("papers-we-love/papers-we-love", "git clone https://github.com/papers-we-love/papers-we-love.git", "npm test"),
        ("jaywcjlove/awesome-mac", "git clone https://github.com/jaywcjlove/awesome-mac.git", "npm test"),
    ];

    assert_eq!(
        cases.len(),
        100,
        "issue #423 follow-up requires doubling the popular-project conversion cases"
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

#[test]
fn unlisted_tools_still_route_through_install_conversion() {
    // Issue #433: the command recognizer no longer leans on an enumerated tool
    // whitelist. `bun`, `deno`, and `uv` never appeared in the old `PREFIXES`
    // table, yet their commands are recognized purely from structure/provenance.
    let cases = [
        ("acme/widget", "bun install", "bun test"),
        ("acme/server", "deno task setup", "deno test"),
        (
            "acme/tool",
            "uv pip install -r requirements.txt",
            "uv run pytest",
        ),
        ("acme/native", "zig build", "zig build test"),
    ];

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
        assert!(
            response.answer.contains(install_command),
            "missing install command for {repo}: {}",
            response.answer
        );
        assert!(
            response.answer.contains(verify_command),
            "missing verify command for {repo}: {}",
            response.answer
        );
    }
}

#[test]
fn prose_bullets_do_not_leak_into_generated_scripts() {
    // Issue #433: adversarial prose surrounding a single real command. Only the
    // back-ticked command should survive into the rendered script; the prose
    // sentences (even ones that name tools) must be rejected.
    let prompt = "Convert this README.md installation guide for acme/widget into a sh script:\n\
                  ## Installation\n\
                  First, make sure you have the toolchain installed and configured.\n\
                  1. Install the project with `npm install`.\n\
                  Then build everything and run the whole pipeline manually.\n";

    let response = FormalAiEngine.answer(prompt);

    assert_eq!(
        response.intent, "installation_conversion",
        "answer: {}",
        response.answer
    );
    assert!(
        response.answer.contains("npm install"),
        "real command dropped: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("make sure you have"),
        "prose leaked into script: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Then build everything"),
        "prose leaked into script: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("First, make sure"),
        "prose leaked into script: {}",
        response.answer
    );
}

#[test]
fn prose_rejection_holds_across_supported_languages() {
    // Issue #433: the structural recognizer rejects prose and keeps real commands
    // independently of the surrounding natural language. Exercise the same
    // adversarial shape (prose sentences wrapping one back-ticked command) in
    // every supported language so the generalization is not English-only.
    struct Case {
        language: &'static str,
        prompt: &'static str,
        prose_fragment: &'static str,
    }

    let cases = [
        Case {
            // english
            language: "en",
            prompt: "Convert this README.md installation guide into a sh script:\n\
                     ## Installation\n\
                     First, make sure your environment is ready before continuing.\n\
                     1. Install the project with `npm install`.\n",
            prose_fragment: "make sure your environment",
        },
        Case {
            // russian / русский
            language: "ru",
            prompt: "Преобразуй это README.md руководство по установке в sh скрипт:\n\
                     ## Установка\n\
                     Сначала убедитесь, что окружение готово к работе.\n\
                     1. Установите проект командой `npm install`.\n",
            prose_fragment: "убедитесь, что окружение",
        },
        Case {
            // hindi / हिन्दी
            language: "hi",
            prompt: "इस README.md स्थापना guide को sh script में बदलें:\n\
                     ## स्थापना\n\
                     पहले सुनिश्चित करें कि आपका वातावरण तैयार है।\n\
                     1. परियोजना को `npm install` से स्थापित करें।\n",
            prose_fragment: "सुनिश्चित करें कि आपका",
        },
        Case {
            // chinese / 中文
            language: "zh",
            prompt: "请把这个 README.md 安装指南转换为 sh 脚本:\n\
                     ## 安装\n\
                     首先请确认你的环境已经准备就绪。\n\
                     1. 使用 `npm install` 安装项目。\n",
            prose_fragment: "首先请确认你的环境",
        },
    ];

    for case in cases {
        let response = FormalAiEngine.answer(case.prompt);

        assert_eq!(
            response.intent, "installation_conversion",
            "language {} returned {}: {}",
            case.language, response.intent, response.answer
        );
        assert!(
            response.answer.contains("npm install"),
            "language {}: real command dropped: {}",
            case.language,
            response.answer
        );
        assert!(
            !response.answer.contains(case.prose_fragment),
            "language {}: prose leaked into script: {}",
            case.language,
            response.answer
        );
    }
}
