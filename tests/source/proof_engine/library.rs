//! A small in-process library of classical theorems and their textbook proofs.
//!
//! Used by the universal proof engine when a prompt names one of the
//! well-known results (Pythagorean theorem, infinitude of primes,
//! irrationality of √2, Fermat's little theorem, Gödel's first
//! incompleteness theorem, and a Newtonian / Laplacian-determinism
//! reduction).
//!
//! Every entry stores the proof in four languages (en/ru/hi/zh) so the
//! presenter never has to fall back to English for the localized chat
//! surface. The proofs are deliberately short but real: they reproduce the
//! standard deductive structure mathematicians have been using for these
//! results, not a stub.

use crate::proof_engine::types::{Proof, ProofMethod, ProofStep, StepKind};

/// Hand-rolled multilingual proof bodies for the theorems the engine
/// recognizes by keyword.
pub(super) struct KnownTheoremEntry {
    pub id: &'static str,
    pub method: ProofMethod,
    pub match_keywords: &'static [&'static str],
    pub statement_en: &'static str,
    pub statement_ru: &'static str,
    pub statement_hi: &'static str,
    pub statement_zh: &'static str,
    pub steps: &'static [LocalizedStep],
    pub conclusion_en: &'static str,
    pub conclusion_ru: &'static str,
    pub conclusion_hi: &'static str,
    pub conclusion_zh: &'static str,
}

/// One step rendered in every supported language.
pub(super) struct LocalizedStep {
    pub kind: StepKind,
    pub en: &'static str,
    pub ru: &'static str,
    pub hi: &'static str,
    pub zh: &'static str,
}

impl KnownTheoremEntry {
    pub fn matches(&self, normalized: &str) -> bool {
        self.match_keywords.iter().any(|kw| normalized.contains(kw))
    }

    pub fn build_proof(&self, language: &str) -> Proof {
        let statement = match language {
            "ru" => self.statement_ru,
            "hi" => self.statement_hi,
            "zh" => self.statement_zh,
            _ => self.statement_en,
        };
        let conclusion = match language {
            "ru" => self.conclusion_ru,
            "hi" => self.conclusion_hi,
            "zh" => self.conclusion_zh,
            _ => self.conclusion_en,
        };
        let steps = self
            .steps
            .iter()
            .map(|step| ProofStep {
                kind: step.kind,
                text: String::from(match language {
                    "ru" => step.ru,
                    "hi" => step.hi,
                    "zh" => step.zh,
                    _ => step.en,
                }),
            })
            .collect();
        Proof {
            statement: statement.to_owned(),
            steps,
            conclusion: conclusion.to_owned(),
            method: self.method,
        }
    }
}

/// All theorems the engine can discharge by direct lookup. The first
/// entry whose `matches` returns true wins.
pub(super) const REGISTRY: &[KnownTheoremEntry] = &[
    PYTHAGOREAN,
    EUCLID_INFINITUDE_OF_PRIMES,
    SQRT_TWO_IRRATIONAL,
    FERMAT_LITTLE,
    GODEL_FIRST_INCOMPLETENESS,
    LAPLACIAN_DETERMINISM,
];

const PYTHAGOREAN: KnownTheoremEntry = KnownTheoremEntry {
    id: "pythagorean_theorem",
    method: ProofMethod::KnownTheorem,
    match_keywords: &[
        "pythagor",
        "пифагор",
        "毕达哥拉斯",
        "勾股",
        "पाइथागोरस",
        "पाइथागोरियन",
    ],
    statement_en:
        "In any right triangle with legs a, b and hypotenuse c we have a² + b² = c².",
    statement_ru:
        "В любом прямоугольном треугольнике с катетами a, b и гипотенузой c выполняется a² + b² = c².",
    statement_hi:
        "किसी भी समकोण त्रिभुज जिसके लंब a, b और कर्ण c है, के लिए a² + b² = c² होता है।",
    statement_zh: "在任意直角三角形中,若两直角边为 a、b,斜边为 c,则 a² + b² = c²。",
    steps: &[
        LocalizedStep {
            kind: StepKind::Hypothesis,
            en: "Consider a right triangle with legs of length a and b and hypotenuse of length c. \
                 The right angle is opposite the hypotenuse.",
            ru: "Рассмотрим прямоугольный треугольник с катетами длины a и b и гипотенузой c. \
                 Прямой угол лежит напротив гипотенузы.",
            hi: "मान लीजिए कि एक समकोण त्रिभुज है जिसके लंब a, b और कर्ण c है। \
                 समकोण कर्ण के सामने है।",
            zh: "考虑一个直角三角形,两直角边长分别为 a 与 b,斜边长为 c,直角与斜边相对。",
        },
        LocalizedStep {
            kind: StepKind::Definition,
            en: "Drop the altitude from the right angle onto the hypotenuse. It splits the \
                 hypotenuse into two segments of lengths p (adjacent to a) and q (adjacent to b), \
                 with p + q = c.",
            ru: "Опустим высоту из прямого угла на гипотенузу. Она делит гипотенузу на \
                 отрезки длин p (примыкающий к a) и q (примыкающий к b), причём p + q = c.",
            hi: "समकोण से कर्ण पर लम्ब डालिए। यह कर्ण को दो खंडों p (a के निकट) तथा q \
                 (b के निकट) में बाँट देता है, p + q = c।",
            zh: "从直角顶点向斜边作高,它把斜边分成长度为 p(靠近 a)与 q(靠近 b)的两段,\
                 且 p + q = c。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "The two smaller triangles created by the altitude are similar to the original \
                 (AA-similarity: each shares the right angle and one acute angle with the \
                 original). Hence a/c = p/a and b/c = q/b, i.e. a² = c·p and b² = c·q.",
            ru: "Два малых треугольника, образованных высотой, подобны исходному (по двум углам: \
                 у каждого общий прямой угол и общий острый угол с исходным). Поэтому a/c = p/a \
                 и b/c = q/b, то есть a² = c·p и b² = c·q.",
            hi: "उच्चता द्वारा बने दोनों छोटे त्रिभुज मूल त्रिभुज के समरूप होते हैं (AA समरूपता: \
                 हर एक में समकोण और एक न्यूनकोण मूल त्रिभुज से समान है)। अतः a/c = p/a तथा \
                 b/c = q/b, यानी a² = c·p और b² = c·q।",
            zh: "高线分出的两个小三角形与原三角形相似(AA 相似:各有原三角形的直角和一个锐角)。\
                 因此 a/c = p/a 与 b/c = q/b,即 a² = c·p,b² = c·q。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Add the two equalities: a² + b² = c·p + c·q = c·(p + q) = c·c = c².",
            ru: "Сложим эти равенства: a² + b² = c·p + c·q = c·(p + q) = c·c = c².",
            hi: "दोनों समीकरणों को जोड़िए: a² + b² = c·p + c·q = c·(p + q) = c·c = c²।",
            zh: "把两个等式相加:a² + b² = c·p + c·q = c·(p + q) = c·c = c²。",
        },
    ],
    conclusion_en: "Therefore a² + b² = c² for every right triangle. ∎",
    conclusion_ru: "Следовательно, a² + b² = c² для любого прямоугольного треугольника. ∎",
    conclusion_hi: "अतः प्रत्येक समकोण त्रिभुज के लिए a² + b² = c² सिद्ध होता है। ∎",
    conclusion_zh: "故对一切直角三角形都有 a² + b² = c²。∎",
};

const EUCLID_INFINITUDE_OF_PRIMES: KnownTheoremEntry = KnownTheoremEntry {
    id: "euclid_infinitude_of_primes",
    method: ProofMethod::Contradiction,
    match_keywords: &[
        "infinitely many primes",
        "infinitely many prime numbers",
        "infinitude of primes",
        "prime numbers are infinite",
        "euclid",
        "бесконечно много прост",
        "простых бесконечно",
        "простых чисел бесконечно",
        "простые бесконечны",
        "бесконечны простые",
        "бесконечности прост",
        "евклид",
        "अनंत अभाज्य",
        "अनन्त अभाज्य",
        "अभाज्य संख्याएँ अनंत",
        "अभाज्य संख्याएं अनंत",
        "अभाज्य संख्याएँ अनन्त",
        "अभाज्य संख्याएं अनन्त",
        "无穷多素数",
        "无穷多个素数",
        "素数有无穷",
        "素数无穷",
        "無窮多素數",
        "素數有無窮",
        "素數無窮",
        "欧几里得",
    ],
    statement_en: "There are infinitely many prime numbers.",
    statement_ru: "Простых чисел бесконечно много.",
    statement_hi: "अभाज्य संख्याएँ अनंत हैं।",
    statement_zh: "素数有无穷多个。",
    steps: &[
        LocalizedStep {
            kind: StepKind::Definition,
            en: "Work in elementary number theory, formalizable in Peano arithmetic (PA): \
                 a prime is an integer greater than 1 whose only positive divisors are 1 and \
                 itself. The proof uses the PA theorem that every integer greater than 1 has a \
                 prime divisor; this is the formal context for the relative-meta-logic \
                 contradiction tactic.",
            ru: "Работаем в элементарной теории чисел, формализуемой в арифметике Пеано (PA): \
                 простое число — это целое число больше 1, положительные делители которого \
                 только 1 и оно само. В доказательстве используется теорема PA: у каждого \
                 целого числа больше 1 есть простой делитель; это формальный контекст для \
                 тактики от противного в relative-meta-logic.",
            hi: "हम प्राथमिक संख्या-सिद्धांत में काम करते हैं, जिसे Peano arithmetic (PA) में \
                 औपचारिक किया जा सकता है: अभाज्य वह पूर्णांक है जो 1 से बड़ा है और जिसके \
                 धनात्मक भाजक केवल 1 और वही संख्या हैं। प्रमाण PA के इस प्रमेय का उपयोग \
                 करता है कि 1 से बड़े हर पूर्णांक का कोई अभाज्य भाजक होता है; यही \
                 relative-meta-logic की contradiction युक्ति का औपचारिक संदर्भ है।",
            zh: "在可由 Peano arithmetic (PA) 形式化的初等数论中工作:素数是大于 1 的整数,\
                 其正因数只有 1 和自身。证明使用 PA 中的定理:每个大于 1 的整数都有素因数;\
                 这就是 relative-meta-logic 反证策略的形式上下文。",
        },
        LocalizedStep {
            kind: StepKind::Hypothesis,
            en: "Assume for contradiction that only finitely many primes exist; call them \
                 p₁, p₂, …, pₙ.",
            ru: "Предположим противное: простых чисел конечное число; обозначим их \
                 p₁, p₂, …, pₙ.",
            hi: "विरोधाभास हेतु मान लीजिए कि अभाज्य संख्याएँ केवल सीमित संख्या में हैं: \
                 p₁, p₂, …, pₙ।",
            zh: "假设素数仅有有限多个,记为 p₁、p₂、…、pₙ。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Form the number N = p₁·p₂·…·pₙ + 1. Then N is an integer greater than 1.",
            ru: "Рассмотрим число N = p₁·p₂·…·pₙ + 1. Это целое число, большее единицы.",
            hi: "संख्या N = p₁·p₂·…·pₙ + 1 लीजिए। N एक से बड़ा पूर्णांक है।",
            zh: "构造数 N = p₁·p₂·…·pₙ + 1。N 是大于 1 的整数。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "By the fundamental theorem of arithmetic, N has a prime divisor q. \
                 If q = pᵢ for some i, then pᵢ divides both p₁·p₂·…·pₙ and N, so pᵢ divides their \
                 difference, which is 1 — impossible.",
            ru: "По основной теореме арифметики у N есть простой делитель q. Если q = pᵢ для \
                 некоторого i, то pᵢ делит и p₁·p₂·…·pₙ, и N, а значит делит их разность, равную 1 \
                 — противоречие.",
            hi: "अंकगणित की मूल प्रमेय से N का कोई अभाज्य भाजक q है। यदि किसी i के लिए q = pᵢ \
                 हो, तो pᵢ संख्या p₁·p₂·…·pₙ और N दोनों को विभाजित करेगा, अर्थात उनका अंतर 1 \
                 भी विभाजित करेगा — असंभव।",
            zh: "由算术基本定理,N 有一个素因数 q。若 q = pᵢ,则 pᵢ 同时整除 p₁·p₂·…·pₙ 与 N,\
                 因而整除二者之差 1,矛盾。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Hence q is a prime not in the list p₁, …, pₙ, contradicting the assumption \
                 that the list was complete.",
            ru: "Значит, q — простое число, не входящее в список p₁, …, pₙ, что противоречит \
                 предположению о полноте списка.",
            hi: "अतः q एक ऐसा अभाज्य है जो सूची p₁, …, pₙ में नहीं है, जो सूची के पूर्ण होने की \
                 परिकल्पना का खंडन करता है।",
            zh: "因此 q 是不在 p₁, …, pₙ 中的素数,与假设矛盾。",
        },
    ],
    conclusion_en: "The assumption fails, so there are infinitely many primes. ∎",
    conclusion_ru: "Предположение несостоятельно, следовательно простых чисел бесконечно много. ∎",
    conclusion_hi: "अतः परिकल्पना असत्य है और अभाज्य संख्याएँ अनंत हैं। ∎",
    conclusion_zh: "假设不成立,故素数有无穷多个。∎",
};

const SQRT_TWO_IRRATIONAL: KnownTheoremEntry = KnownTheoremEntry {
    id: "sqrt_two_irrational",
    method: ProofMethod::Contradiction,
    match_keywords: &[
        "square root of two",
        "square root of 2",
        "sqrt(2)",
        "√2",
        "корень из двух",
        "корень из 2",
        "иррацион",
        "irrational",
        "दो का वर्गमूल",
        "मूल दो",
        "अपरिमेय",
        "根号二",
        "根号 2",
        "无理",
    ],
    statement_en: "√2 is irrational: there are no integers p, q with q ≠ 0 such that p/q = √2.",
    statement_ru: "√2 иррационально: не существует целых p, q (q ≠ 0), для которых p/q = √2.",
    statement_hi: "√2 अपरिमेय है: कोई पूर्णांक p, q (q ≠ 0) ऐसे नहीं हैं जिनके लिए p/q = √2 हो।",
    statement_zh: "√2 是无理数:不存在整数 p、q(q ≠ 0)使得 p/q = √2。",
    steps: &[
        LocalizedStep {
            kind: StepKind::Hypothesis,
            en: "Assume for contradiction that √2 = p/q with integers p, q ≠ 0 and gcd(p, q) = 1 \
                 (we can always cancel common factors).",
            ru: "Предположим противное: √2 = p/q при целых p, q ≠ 0 и НОД(p, q) = 1 (общие \
                 множители всегда можно сократить).",
            hi: "विरोधाभास हेतु मान लीजिए कि √2 = p/q, जहाँ p, q पूर्णांक हैं, q ≠ 0 तथा \
                 gcd(p, q) = 1 (सामान्य गुणनखंडों को रद्द किया जा सकता है)।",
            zh: "假设 √2 = p/q,其中 p、q 为整数,q ≠ 0 且 gcd(p, q) = 1(可消去公因数)。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Square both sides: 2 = p²/q², so p² = 2·q². Hence p² is even, and therefore p \
                 is even (the square of an odd integer is odd). Write p = 2·k.",
            ru: "Возведём в квадрат: 2 = p²/q², откуда p² = 2·q². Значит p² чётно, и потому p \
                 чётно (квадрат нечётного нечётен). Запишем p = 2·k.",
            hi: "दोनों ओर का वर्ग लीजिए: 2 = p²/q², अतः p² = 2·q²। इसलिए p² सम है और \
                 परिणामतः p भी सम है (विषम पूर्णांक का वर्ग विषम होता है)। p = 2·k लिखिए।",
            zh: "两边平方:2 = p²/q²,即 p² = 2·q²。故 p² 为偶数,从而 p 为偶数\
                 (奇数的平方为奇数),写作 p = 2·k。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Substitute: (2·k)² = 2·q², i.e. 4·k² = 2·q², i.e. q² = 2·k². So q² is even and q \
                 is even.",
            ru: "Подставим: (2·k)² = 2·q², то есть 4·k² = 2·q², откуда q² = 2·k². Значит q² \
                 чётно, и q чётно.",
            hi: "प्रतिस्थापन: (2·k)² = 2·q², यानी 4·k² = 2·q², अर्थात q² = 2·k²। अतः q² \
                 सम है और q भी सम है।",
            zh: "代入:(2·k)² = 2·q²,即 4·k² = 2·q²,故 q² = 2·k²。所以 q² 为偶数,q 也为偶数。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Both p and q are even, contradicting gcd(p, q) = 1.",
            ru: "Получили, что p и q оба чётны — противоречие с НОД(p, q) = 1.",
            hi: "p और q दोनों सम हैं, जो gcd(p, q) = 1 का खंडन करता है।",
            zh: "p 与 q 均为偶数,与 gcd(p, q) = 1 矛盾。",
        },
    ],
    conclusion_en: "The assumption fails, so √2 is irrational. ∎",
    conclusion_ru: "Предположение несостоятельно, поэтому √2 иррационально. ∎",
    conclusion_hi: "अतः परिकल्पना असत्य है और √2 अपरिमेय है। ∎",
    conclusion_zh: "假设不成立,故 √2 是无理数。∎",
};

const FERMAT_LITTLE: KnownTheoremEntry = KnownTheoremEntry {
    id: "fermat_little_theorem",
    method: ProofMethod::Induction,
    match_keywords: &[
        "fermat's little",
        "fermat little",
        "малая теорема ферма",
        "малой теоремы ферма",
        "फर्मा की लघु",
        "फर्मा लघु",
        "费马小定理",
        "費馬小定理",
    ],
    statement_en:
        "For every prime p and every integer a, aᵖ ≡ a (mod p). Equivalently, if gcd(a, p) = 1, \
         then aᵖ⁻¹ ≡ 1 (mod p).",
    statement_ru:
        "Для любого простого p и любого целого a выполнено aᵖ ≡ a (mod p). Эквивалентно, при \
         НОД(a, p) = 1 имеем aᵖ⁻¹ ≡ 1 (mod p).",
    statement_hi: "हर अभाज्य p तथा हर पूर्णांक a के लिए aᵖ ≡ a (mod p)। यदि gcd(a, p) = 1 हो, \
         तो aᵖ⁻¹ ≡ 1 (mod p)।",
    statement_zh: "对任意素数 p 与任意整数 a,有 aᵖ ≡ a (mod p)。\
                   等价地,若 gcd(a, p) = 1,则 aᵖ⁻¹ ≡ 1 (mod p)。",
    steps: &[
        LocalizedStep {
            kind: StepKind::Hypothesis,
            en: "Fix a prime p. We prove aᵖ ≡ a (mod p) for every non-negative integer a by \
                 induction on a; the result for negative a follows by parity / sign.",
            ru: "Зафиксируем простое p. Докажем aᵖ ≡ a (mod p) для каждого неотрицательного \
                 целого a индукцией по a; для отрицательных случай сводится знаком.",
            hi: "एक अभाज्य p तय कीजिए। हम हर अऋणात्मक पूर्णांक a के लिए aᵖ ≡ a (mod p) को \
                 a पर आगमन द्वारा सिद्ध करते हैं; ऋणात्मक a का मामला चिह्न से अनुसरण करता है।",
            zh: "固定素数 p。对非负整数 a 用数学归纳法证 aᵖ ≡ a (mod p);负整数情形按符号化归。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Base case a = 0: 0ᵖ = 0 ≡ 0 (mod p). The base case holds.",
            ru: "База a = 0: 0ᵖ = 0 ≡ 0 (mod p). База выполнена.",
            hi: "आधार स्थिति a = 0: 0ᵖ = 0 ≡ 0 (mod p)। आधार सिद्ध है।",
            zh: "归纳基础 a = 0:0ᵖ = 0 ≡ 0 (mod p),成立。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Inductive step: assume aᵖ ≡ a (mod p). Use the binomial expansion \
                 (a + 1)ᵖ = Σ C(p, k)·aᵏ. For 1 ≤ k ≤ p − 1, the binomial coefficient C(p, k) is \
                 divisible by p (because p is prime and appears in the numerator but not in the \
                 denominator). Hence (a + 1)ᵖ ≡ aᵖ + 1 (mod p).",
            ru: "Шаг индукции: пусть aᵖ ≡ a (mod p). Воспользуемся биномом Ньютона \
                 (a + 1)ᵖ = Σ C(p, k)·aᵏ. При 1 ≤ k ≤ p − 1 биномиальный коэффициент C(p, k) \
                 делится на p (p — простое, входит в числитель, но не в знаменатель). Поэтому \
                 (a + 1)ᵖ ≡ aᵖ + 1 (mod p).",
            hi: "आगमन-चरण: मान लीजिए aᵖ ≡ a (mod p)। द्विपद विस्तार \
                 (a + 1)ᵖ = Σ C(p, k)·aᵏ का उपयोग करें। 1 ≤ k ≤ p − 1 के लिए C(p, k), p से \
                 विभाज्य है (p अभाज्य अंश में आता है पर हर में नहीं)। अतः \
                 (a + 1)ᵖ ≡ aᵖ + 1 (mod p)।",
            zh: "归纳步:设 aᵖ ≡ a (mod p)。由二项式展开 (a + 1)ᵖ = Σ C(p, k)·aᵏ,\
                 当 1 ≤ k ≤ p − 1 时 C(p, k) 被 p 整除(p 为素数,出现在分子而不在分母)。\
                 故 (a + 1)ᵖ ≡ aᵖ + 1 (mod p)。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Combining with the inductive hypothesis: (a + 1)ᵖ ≡ aᵖ + 1 ≡ a + 1 (mod p). \
                 The inductive step is established.",
            ru: "Совмещая с предположением: (a + 1)ᵖ ≡ aᵖ + 1 ≡ a + 1 (mod p). Шаг индукции \
                 доказан.",
            hi: "आगमन-परिकल्पना के साथ संयोजन: (a + 1)ᵖ ≡ aᵖ + 1 ≡ a + 1 (mod p)। \
                 आगमन-चरण सिद्ध हुआ।",
            zh: "结合归纳假设:(a + 1)ᵖ ≡ aᵖ + 1 ≡ a + 1 (mod p)。归纳步完成。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en:
                "By induction, aᵖ ≡ a (mod p) for every non-negative integer a. When gcd(a, p) = 1 \
                 we may cancel one factor of a (it is invertible modulo p) to get aᵖ⁻¹ ≡ 1 (mod p).",
            ru: "По индукции aᵖ ≡ a (mod p) для всех a ≥ 0. Если НОД(a, p) = 1, можно сократить \
                 один множитель a (он обратим по модулю p), получив aᵖ⁻¹ ≡ 1 (mod p).",
            hi: "आगमन से, हर अऋणात्मक a के लिए aᵖ ≡ a (mod p)। gcd(a, p) = 1 पर a का एक \
                 गुणनखंड रद्द कर सकते हैं (वह mod p में व्युत्क्रमणीय है), जिससे \
                 aᵖ⁻¹ ≡ 1 (mod p) प्राप्त होता है।",
            zh: "由归纳,对所有非负整数 a 都有 aᵖ ≡ a (mod p)。当 gcd(a, p) = 1 时,a 在 mod p \
                 下可逆,可消去得到 aᵖ⁻¹ ≡ 1 (mod p)。",
        },
    ],
    conclusion_en: "Therefore aᵖ ≡ a (mod p) for every integer a and every prime p. ∎",
    conclusion_ru: "Следовательно, aᵖ ≡ a (mod p) для всякого целого a и всякого простого p. ∎",
    conclusion_hi: "अतः हर पूर्णांक a और हर अभाज्य p के लिए aᵖ ≡ a (mod p)। ∎",
    conclusion_zh: "故对一切整数 a 与素数 p,皆有 aᵖ ≡ a (mod p)。∎",
};

const GODEL_FIRST_INCOMPLETENESS: KnownTheoremEntry = KnownTheoremEntry {
    id: "godel_first_incompleteness",
    method: ProofMethod::AxiomReduction,
    match_keywords: &[
        "godel",
        "gödel",
        "godels",
        "gödels",
        "godel's",
        "gödel's",
        "incompleteness",
        "теорем гёдел",
        "теорема гёдел",
        "теоремы гёдел",
        "теоремы гедел",
        "теорема гедел",
        "теорем гедел",
        "गोडेल",
        "अपूर्णता",
        "哥德尔",
        "不完备",
        "不完备性",
    ],
    statement_en:
        "Any consistent, recursively axiomatised formal system F that interprets Peano arithmetic \
         admits a true arithmetical sentence G_F that F neither proves nor refutes (Gödel's first \
         incompleteness theorem).",
    statement_ru:
        "Всякая непротиворечивая, рекурсивно аксиоматизированная формальная система F, \
         интерпретирующая арифметику Пеано, имеет истинное арифметическое утверждение G_F, \
         которое F не доказывает и не опровергает (первая теорема Гёделя о неполноте).",
    statement_hi:
        "हर सुसंगत, पुनरावर्ती रूप से अभिगृहीत औपचारिक तंत्र F, जो Peano अंकगणित को निरूपित \
         करता है, में एक सत्य अंकगणितीय कथन G_F मौजूद है जिसे F न तो सिद्ध करता है न खंडित \
         (गोडेल का प्रथम अपूर्णता प्रमेय)।",
    statement_zh: "任意一个一致、可递归公理化且可表达 Peano 算术的形式系统 F,\
                   存在一个真的算术语句 G_F,F 既不能证明也不能否证(哥德尔第一不完备性定理)。",
    steps: &[
        LocalizedStep {
            kind: StepKind::Axiom,
            en: "Fix a formal system F that satisfies (i) consistency, (ii) a decidable axiom \
                 set, and (iii) enough arithmetic to define primitive recursive functions \
                 (e.g. F ⊇ Peano arithmetic).",
            ru: "Зафиксируем формальную систему F, удовлетворяющую: (i) непротиворечивость, \
                 (ii) разрешимое множество аксиом, (iii) достаточно арифметики для определения \
                 примитивно-рекурсивных функций (например, F ⊇ арифметика Пеано).",
            hi: "एक औपचारिक तंत्र F लीजिए जो (i) सुसंगत हो, (ii) निर्णायक अभिगृहीत-समुच्चय रखे, \
                 (iii) आदिम पुनरावर्ती फलन परिभाषित करने योग्य अंकगणित (उदा. F ⊇ Peano अंकगणित) \
                 रखे।",
            zh: "取一个形式系统 F:(i) 一致,(ii) 公理集可判定,(iii) 表达原始递归函数所需的算术\
                 (例如 F ⊇ Peano 算术)。",
        },
        LocalizedStep {
            kind: StepKind::Definition,
            en: "Gödel-numbering: assign each formula φ a unique natural number ⌜φ⌝. Provability \
                 in F becomes a primitive-recursive predicate Prov_F(x, y) meaning \"x codes a \
                 proof in F of the formula coded by y\".",
            ru: "Гёделева нумерация: каждой формуле φ ставится в соответствие уникальное \
                 натуральное число ⌜φ⌝. Доказуемость в F представляется примитивно-рекурсивным \
                 предикатом Prov_F(x, y), означающим: «x кодирует доказательство в F формулы, \
                 кодируемой y».",
            hi: "गोडेल-संख्यांकन: हर सूत्र φ को एक अद्वितीय प्राकृत संख्या ⌜φ⌝ दीजिए। F में \
                 साध्यता एक आदिम-पुनरावर्ती विधेय Prov_F(x, y) बन जाती है, जिसका अर्थ है \
                 \"x द्वारा कोडित प्रमाण F में y द्वारा कोडित सूत्र को सिद्ध करता है\"।",
            zh: "哥德尔编号:为每个公式 φ 指派唯一自然数 ⌜φ⌝。F 中的可证性化为原始递归谓词 \
                 Prov_F(x, y),意为 \"x 编码一个 F 中的证明,证明的目标公式编号为 y\"。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Diagonal lemma: for the formula ψ(y) ≔ ¬∃x Prov_F(x, y) (\"y is not provable in \
                 F\") there exists a sentence G_F with ⌜G_F⌝ = n such that F ⊢ G_F ↔ ψ(n). \
                 Intuitively G_F says \"I am not provable in F\".",
            ru: "Диагональная лемма: для формулы ψ(y) ≔ ¬∃x Prov_F(x, y) («y недоказуемо в F») \
                 существует утверждение G_F с ⌜G_F⌝ = n, для которого F ⊢ G_F ↔ ψ(n). \
                 По существу G_F утверждает: «Я недоказуемо в F».",
            hi: "विकर्ण लेम्मा: सूत्र ψ(y) ≔ ¬∃x Prov_F(x, y) (\"y, F में साध्य नहीं है\") के \
                 लिए एक कथन G_F है, ⌜G_F⌝ = n, ऐसा कि F ⊢ G_F ↔ ψ(n)। तात्पर्य: G_F कहता है \
                 \"मैं F में साध्य नहीं हूँ\"।",
            zh: "对角引理:对公式 ψ(y) ≔ ¬∃x Prov_F(x, y)(\"y 在 F 中不可证\"),\
                 存在句子 G_F 使得 ⌜G_F⌝ = n 且 F ⊢ G_F ↔ ψ(n)。直观上 G_F 表示 \"我在 F 中不可证\"。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Suppose F ⊢ G_F. Then there exists a proof code m with Prov_F(m, n), so by the \
                 equivalence F ⊢ ¬G_F too, contradicting the consistency of F. Hence F ⊬ G_F.",
            ru: "Предположим F ⊢ G_F. Тогда найдётся код доказательства m с Prov_F(m, n), и по \
                 эквивалентности F ⊢ ¬G_F, что противоречит непротиворечивости F. Значит \
                 F ⊬ G_F.",
            hi: "मान लीजिए F ⊢ G_F। तब किसी प्रमाण कोड m के लिए Prov_F(m, n) होगा, और \
                 तुल्यता से F ⊢ ¬G_F भी होगा, जो F की सुसंगति का विरोध करता है। अतः F ⊬ G_F।",
            zh: "若 F ⊢ G_F,则存在证明编码 m 使 Prov_F(m, n) 成立,根据等价 F ⊢ ¬G_F,\
                 与 F 的一致性矛盾。故 F ⊬ G_F。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Suppose F ⊢ ¬G_F. Then F ⊢ ∃x Prov_F(x, n), yet by the previous step no actual \
                 proof of G_F exists in F. A standard ω-consistency (or 1-consistency, in \
                 Rosser's strengthening) argument turns this into another contradiction; hence \
                 F ⊬ ¬G_F.",
            ru: "Предположим F ⊢ ¬G_F. Тогда F ⊢ ∃x Prov_F(x, n), однако из предыдущего шага \
                 настоящего доказательства G_F в F нет. Стандартный аргумент ω-непротиворечивости \
                 (или 1-непротиворечивости в усилении Россера) превращает это во второе \
                 противоречие; следовательно F ⊬ ¬G_F.",
            hi: "मान लीजिए F ⊢ ¬G_F। तब F ⊢ ∃x Prov_F(x, n) होगा, परन्तु पिछले चरण के अनुसार \
                 F में G_F का वास्तविक प्रमाण नहीं है। मानक ω-संगति (या रोसर-वर्धित \
                 1-संगति) तर्क इसे एक दूसरे विरोधाभास में बदलता है; अतः F ⊬ ¬G_F।",
            zh: "若 F ⊢ ¬G_F,则 F ⊢ ∃x Prov_F(x, n),然而由上一步,在 F 中并不存在 G_F 的实际证明。\
                 标准 ω-一致性(或 Rosser 加强的 1-一致性)论证将其转为又一矛盾,故 F ⊬ ¬G_F。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "G_F is therefore independent of F. Since the metatheory sees that G_F holds \
                 (because every actual proof in F would be visible to the metatheory and none \
                 exists), G_F is a true arithmetical sentence that F cannot decide.",
            ru: "Значит, G_F независимо от F. Метатеория видит, что G_F истинно (любое реальное \
                 доказательство в F было бы заметно метатеории, а его нет), поэтому G_F — \
                 истинное арифметическое утверждение, не разрешаемое в F.",
            hi: "अतः G_F, F से स्वतंत्र है। मेटा-सिद्धांत में दिखता है कि G_F सत्य है \
                 (F का कोई भी वास्तविक प्रमाण मेटा-सिद्धांत में दिखाई देगा, परंतु ऐसा कोई नहीं है), \
                 इसलिए G_F एक सत्य अंकगणितीय कथन है जिसे F निर्णीत नहीं कर सकता।",
            zh: "故 G_F 独立于 F。元理论可见 G_F 为真(F 内任何真实证明均会显现于元理论中,\
                 但并不存在),所以 G_F 是 F 无法判定的真算术语句。",
        },
    ],
    conclusion_en: "F is incomplete: a true sentence about natural numbers escapes its proof \
                    system. ∎",
    conclusion_ru: "F неполна: истинное утверждение о натуральных числах ускользает от её системы \
                    доказательств. ∎",
    conclusion_hi: "F अपूर्ण है: प्राकृतिक संख्याओं के बारे में एक सत्य कथन उसकी प्रमाण-व्यवस्था से \
                    बच निकलता है। ∎",
    conclusion_zh: "故 F 不完备:存在关于自然数的真语句逸出其证明系统。∎",
};

const LAPLACIAN_DETERMINISM: KnownTheoremEntry = KnownTheoremEntry {
    id: "laplacian_determinism",
    method: ProofMethod::AxiomReduction,
    match_keywords: &[
        "determinism",
        "deterministic",
        "детерминизм",
        "детерминирован",
        "决定论",
        "निर्धारणवाद",
    ],
    statement_en:
        "Within the Newtonian axiom set N = (Euclidean space, smooth time, Newton's second law \
         F = m·ẍ, Lipschitz-bounded forces), the future trajectory of every closed mechanical \
         system is uniquely determined by its present state (Laplacian determinism).",
    statement_ru:
        "В аксиоматике Ньютона N = (евклидово пространство, гладкое время, второй закон Ньютона \
         F = m·ẍ, силы с ограничением Липшица) будущая траектория каждой замкнутой механической \
         системы однозначно определена её текущим состоянием (лапласовский детерминизм).",
    statement_hi:
        "न्यूटनीय अभिगृहीत समुच्चय N = (यूक्लिडीय आकाश, सतत काल, न्यूटन का द्वितीय नियम \
         F = m·ẍ, लिप्शिट्ज़-सीमित बल) के अंतर्गत हर बंद यांत्रिक तंत्र की भविष्य प्रक्षेपवक्र \
         उसकी वर्तमान अवस्था से अद्वितीय रूप से निर्धारित होती है (Laplace का निर्धारणवाद)।",
    statement_zh: "在牛顿公理集 N =(欧氏空间、光滑时间、牛顿第二定律 F = m·ẍ、\
                   Lipschitz 受限力场)下,任意闭合力学系统的未来轨道由其当前状态唯一决定\
                   (拉普拉斯式决定论)。",
    steps: &[
        LocalizedStep {
            kind: StepKind::Axiom,
            en: "Take the axiom set N: configuration space ℝᵈ with the standard Euclidean \
                 structure, time ∈ ℝ, mass m > 0, and a force field F: ℝᵈ × ℝᵈ × ℝ → ℝᵈ that is \
                 Lipschitz-continuous in position and velocity.",
            ru: "Возьмём аксиоматику N: конфигурационное пространство ℝᵈ с евклидовой \
                 структурой, время ∈ ℝ, масса m > 0 и силовое поле \
                 F: ℝᵈ × ℝᵈ × ℝ → ℝᵈ, липшицевое по координатам и скоростям.",
            hi: "अभिगृहीत समुच्चय N: कॉन्फ़िगरेशन स्पेस ℝᵈ यूक्लिडीय संरचना के साथ, समय ∈ ℝ, \
                 द्रव्यमान m > 0, और बल क्षेत्र F: ℝᵈ × ℝᵈ × ℝ → ℝᵈ जो स्थिति और वेग में \
                 लिप्शिट्ज़-संतत है।",
            zh: "取公理集 N:位形空间 ℝᵈ 配欧氏结构、时间 ∈ ℝ、质量 m > 0,以及在位置与速度上\
                 Lipschitz 连续的力场 F: ℝᵈ × ℝᵈ × ℝ → ℝᵈ。",
        },
        LocalizedStep {
            kind: StepKind::Definition,
            en: "A state of the system at time t₀ is the pair s₀ = (x(t₀), ẋ(t₀)). Newton's \
                 second law turns the dynamics into the first-order ODE \
                 (ẋ, v̇) = (v, F(x, v, t)/m).",
            ru: "Состояние системы в момент t₀ — это пара s₀ = (x(t₀), ẋ(t₀)). Второй закон \
                 Ньютона превращает динамику в систему ОДУ первого порядка \
                 (ẋ, v̇) = (v, F(x, v, t)/m).",
            hi: "समय t₀ पर तंत्र की अवस्था युग्म s₀ = (x(t₀), ẋ(t₀)) है। न्यूटन का द्वितीय \
                 नियम गतिकी को प्रथम-कोटि ODE (ẋ, v̇) = (v, F(x, v, t)/m) में बदलता है।",
            zh: "系统在 t₀ 时刻的状态为 s₀ = (x(t₀), ẋ(t₀))。牛顿第二定律将动力学化为一阶常微分方程组 \
                 (ẋ, v̇) = (v, F(x, v, t)/m)。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Apply the Picard–Lindelöf theorem: a Lipschitz right-hand side on an open \
                 neighbourhood of (s₀, t₀) gives a unique maximal solution s(t) through that \
                 initial condition.",
            ru: "Применим теорему Пикара–Линделёфа: липшицева правая часть в окрестности \
                 (s₀, t₀) даёт единственное максимальное решение s(t), проходящее через это \
                 начальное условие.",
            hi: "Picard–Lindelöf प्रमेय लागू करें: (s₀, t₀) के एक खुले प्रांत में लिप्शिट्ज़ \
                 दायाँ पक्ष उस आरंभिक शर्त से होकर गुजरने वाला एक अद्वितीय अधिकतम हल s(t) देता है।",
            zh: "应用 Picard–Lindelöf 定理:在 (s₀, t₀) 的开邻域上 Lipschitz 的右端项确保过该初值\
                 存在唯一的极大解 s(t)。",
        },
        LocalizedStep {
            kind: StepKind::Inference,
            en: "Hence two trajectories that agree at t₀ agree on the entire common domain of \
                 existence; equivalently, the time-t evolution Φ_t : (s₀, t₀) ↦ s(t₀ + t) is a \
                 well-defined function on the state space.",
            ru: "Следовательно, две траектории, совпадающие в t₀, совпадают на всей общей \
                 области существования; иначе говоря, эволюция за время t \
                 Φ_t : (s₀, t₀) ↦ s(t₀ + t) — корректно определённая функция на пространстве \
                 состояний.",
            hi: "अतः जो दो प्रक्षेपवक्र t₀ पर मेल खाते हैं, वे संपूर्ण सामान्य अस्तित्व-प्रांत पर \
                 मेल खाते हैं; तुल्य रूप से, समय-t विकास Φ_t : (s₀, t₀) ↦ s(t₀ + t) \
                 अवस्था-स्थान पर सुपरिभाषित फलन है।",
            zh: "故凡在 t₀ 时刻重合的两条轨道在共同存在域上完全一致;\
                 等价地,时间 t 演化 Φ_t : (s₀, t₀) ↦ s(t₀ + t) 在状态空间上是良定义的函数。",
        },
        LocalizedStep {
            kind: StepKind::SubProof,
            en: "Note (Gödel-style limit): the proof above is reductive — it transfers \
                 determinism to existence and uniqueness of solutions of an ODE. The statement \
                 \"physical reality is deterministic\" is not formally decidable inside N; it \
                 becomes decidable only after fixing both the axioms of physics and the \
                 metalanguage. Inside a richer axiom set (e.g. one that interprets PA), Gödel's \
                 first incompleteness theorem forbids a complete internal certificate of \
                 determinism.",
            ru: "Замечание (граница в духе Гёделя): доказательство сведено — детерминизм \
                 переведён в вопрос существования и единственности решений ОДУ. Утверждение \
                 «физическая реальность детерминирована» формально не разрешимо внутри N; оно \
                 становится разрешимым только после фиксации аксиом физики и металогики. В более \
                 богатой системе (например, интерпретирующей арифметику Пеано) первая теорема \
                 Гёделя запрещает полный внутренний сертификат детерминизма.",
            hi: "टिप्पणी (गोडेल-शैली सीमा): उपरोक्त प्रमाण निरूपण-मूलक है — निर्धारणवाद को ODE \
                 के समाधान के अस्तित्व और अद्वितीयता पर स्थानांतरित किया गया है। \"भौतिक यथार्थ \
                 निर्धारित है\" कथन N के भीतर औपचारिक रूप से निर्णीत नहीं है; यह केवल भौतिकी \
                 के अभिगृहीत और मेटा-भाषा तय कर लेने पर निर्णीत होता है। अधिक समृद्ध तंत्र (जो \
                 PA का अर्थ करे) में गोडेल का प्रथम अपूर्णता प्रमेय निर्धारणवाद के पूर्ण आंतरिक \
                 प्रमाणपत्र को रोक देता है।",
            zh: "注(哥德尔式界限):以上证明是化归式的——把决定论化归为常微分方程解的存在唯一性。\
                 \"物理现实是决定论的\" 这一断言在 N 内部并非形式可判定,只有在固定物理公理与元语言之后\
                 才能讨论。在更丰富的公理系统(可表达 PA)中,哥德尔第一不完备定理禁止给出关于\
                 决定论的完全内部证书。",
        },
    ],
    conclusion_en: "Under the axiom set N, Laplacian determinism holds for every closed \
                    mechanical system; outside N (or inside any system interpreting PA), the \
                    statement is not formally complete in the sense of Gödel. ∎",
    conclusion_ru: "В аксиоматике N лапласовский детерминизм выполнен для любой замкнутой \
                    механической системы; вне N (или в системе, интерпретирующей PA) утверждение \
                    формально неполно по Гёделю. ∎",
    conclusion_hi: "अभिगृहीत समुच्चय N के अंतर्गत हर बंद यांत्रिक तंत्र के लिए Laplace का \
                    निर्धारणवाद सिद्ध है; N के बाहर (या PA का अर्थ करने वाले किसी तंत्र में) \
                    यह कथन गोडेल के अर्थ में औपचारिक रूप से पूर्ण नहीं है। ∎",
    conclusion_zh: "在公理集 N 下,任意闭合力学系统的拉普拉斯式决定论成立;在 N 之外\
                    (或任一可表达 PA 的系统中),该断言按哥德尔意义并非形式完备。∎",
};

#[path = "../source_tests/proof_engine/library/tests.rs"]
mod tests;
