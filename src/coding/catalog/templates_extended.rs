//! Templates for the wider-range coding tasks added in issue #330 — `FizzBuzz`,
//! the factorial of 5, string reversal, and the sum from 1 to 10 — in every
//! supported language. Every output here was compiled and run locally
//! (`experiments/issue-330-coding-tasks`). Split from [`super::templates_core`]
//! only to keep each file well under the repository's per-file line limit.

use super::types::ProgramTemplate;

pub(super) const TEMPLATES_EXTENDED: &[ProgramTemplate] = &[
    // Issue #330: FizzBuzz for 1..=15. Every template prints "Fizz" for multiples
    // of 3, "Buzz" for multiples of 5, "FizzBuzz" for multiples of 15, and the
    // number otherwise. Outputs were compiled and run locally
    // (experiments/issue-330-coding-tasks).
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "rust",
        code: r#"fn main() {
    for number in 1..=15 {
        if number % 15 == 0 {
            println!("FizzBuzz");
        } else if number % 3 == 0 {
            println!("Fizz");
        } else if number % 5 == 0 {
            println!("Buzz");
        } else {
            println!("{number}");
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "python",
        code: r#"for number in range(1, 16):
    if number % 15 == 0:
        print("FizzBuzz")
    elif number % 3 == 0:
        print("Fizz")
    elif number % 5 == 0:
        print("Buzz")
    else:
        print(number)"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "javascript",
        code: r#"for (let number = 1; number <= 15; number += 1) {
  if (number % 15 === 0) {
    console.log("FizzBuzz");
  } else if (number % 3 === 0) {
    console.log("Fizz");
  } else if (number % 5 === 0) {
    console.log("Buzz");
  } else {
    console.log(number);
  }
}"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "typescript",
        code: r#"for (let number = 1; number <= 15; number += 1) {
  if (number % 15 === 0) {
    console.log("FizzBuzz");
  } else if (number % 3 === 0) {
    console.log("Fizz");
  } else if (number % 5 === 0) {
    console.log("Buzz");
  } else {
    console.log(number);
  }
}"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    for number := 1; number <= 15; number++ {
        switch {
        case number%15 == 0:
            fmt.Println("FizzBuzz")
        case number%3 == 0:
            fmt.Println("Fizz")
        case number%5 == 0:
            fmt.Println("Buzz")
        default:
            fmt.Println(number)
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "c",
        code: r#"#include <stdio.h>

int main(void) {
    for (int number = 1; number <= 15; number++) {
        if (number % 15 == 0) {
            puts("FizzBuzz");
        } else if (number % 3 == 0) {
            puts("Fizz");
        } else if (number % 5 == 0) {
            puts("Buzz");
        } else {
            printf("%d\n", number);
        }
    }
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "cpp",
        code: r#"#include <iostream>

int main() {
    for (int number = 1; number <= 15; number++) {
        if (number % 15 == 0) {
            std::cout << "FizzBuzz\n";
        } else if (number % 3 == 0) {
            std::cout << "Fizz\n";
        } else if (number % 5 == 0) {
            std::cout << "Buzz\n";
        } else {
            std::cout << number << '\n';
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "java",
        code: r#"public class Main {
    public static void main(String[] args) {
        for (int number = 1; number <= 15; number++) {
            if (number % 15 == 0) {
                System.out.println("FizzBuzz");
            } else if (number % 3 == 0) {
                System.out.println("Fizz");
            } else if (number % 5 == 0) {
                System.out.println("Buzz");
            } else {
                System.out.println(number);
            }
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "csharp",
        code: r#"using System;

class Program {
    static void Main() {
        for (int number = 1; number <= 15; number++) {
            if (number % 15 == 0) {
                Console.WriteLine("FizzBuzz");
            } else if (number % 3 == 0) {
                Console.WriteLine("Fizz");
            } else if (number % 5 == 0) {
                Console.WriteLine("Buzz");
            } else {
                Console.WriteLine(number);
            }
        }
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "fizzbuzz",
        language_slug: "ruby",
        code: r#"(1..15).each do |number|
  if (number % 15).zero?
    puts "FizzBuzz"
  elsif (number % 3).zero?
    puts "Fizz"
  elsif (number % 5).zero?
    puts "Buzz"
  else
    puts number
  end
end"#,
    },
    // Issue #330: factorial of 5 (5! = 120).
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "rust",
        code: r#"fn main() {
    let mut result: u64 = 1;
    for number in 1..=5 {
        result *= number;
    }
    println!("{result}");
}"#,
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "python",
        code: r"result = 1
for number in range(1, 6):
    result *= number
print(result)",
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "javascript",
        code: r"let result = 1;
for (let number = 1; number <= 5; number += 1) {
  result *= number;
}
console.log(result);",
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "typescript",
        code: r"let result = 1;
for (let number = 1; number <= 5; number += 1) {
  result *= number;
}
console.log(result);",
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    result := 1
    for number := 1; number <= 5; number++ {
        result *= number
    }
    fmt.Println(result)
}"#,
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "c",
        code: r#"#include <stdio.h>

int main(void) {
    unsigned long long result = 1;
    for (int number = 1; number <= 5; number++) {
        result *= number;
    }
    printf("%llu\n", result);
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "cpp",
        code: r"#include <iostream>

int main() {
    unsigned long long result = 1;
    for (int number = 1; number <= 5; number++) {
        result *= number;
    }
    std::cout << result << '\n';
}",
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "java",
        code: r"public class Main {
    public static void main(String[] args) {
        long result = 1;
        for (int number = 1; number <= 5; number++) {
            result *= number;
        }
        System.out.println(result);
    }
}",
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "csharp",
        code: r"using System;

class Program {
    static void Main() {
        long result = 1;
        for (int number = 1; number <= 5; number++) {
            result *= number;
        }
        Console.WriteLine(result);
    }
}",
    },
    ProgramTemplate {
        task_slug: "factorial",
        language_slug: "ruby",
        code: r"result = (1..5).reduce(1, :*)
puts result",
    },
    // Issue #330: reverse the literal string "hello" -> "olleh".
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "rust",
        code: r#"fn main() {
    let text = "hello";
    let reversed: String = text.chars().rev().collect();
    println!("{reversed}");
}"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "python",
        code: r#"text = "hello"
print(text[::-1])"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "javascript",
        code: r#"const text = "hello";
console.log(text.split("").reverse().join(""));"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "typescript",
        code: r#"const text: string = "hello";
console.log(text.split("").reverse().join(""));"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    text := "hello"
    runes := []rune(text)
    for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
        runes[i], runes[j] = runes[j], runes[i]
    }
    fmt.Println(string(runes))
}"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "c",
        code: r#"#include <stdio.h>
#include <string.h>

int main(void) {
    char text[] = "hello";
    size_t length = strlen(text);
    for (size_t i = 0; i < length / 2; i++) {
        char temp = text[i];
        text[i] = text[length - 1 - i];
        text[length - 1 - i] = temp;
    }
    puts(text);
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "cpp",
        code: r#"#include <algorithm>
#include <iostream>
#include <string>

int main() {
    std::string text = "hello";
    std::reverse(text.begin(), text.end());
    std::cout << text << '\n';
}"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "java",
        code: r#"public class Main {
    public static void main(String[] args) {
        String text = "hello";
        System.out.println(new StringBuilder(text).reverse().toString());
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "csharp",
        code: r#"using System;

class Program {
    static void Main() {
        var text = "hello".ToCharArray();
        Array.Reverse(text);
        Console.WriteLine(new string(text));
    }
}"#,
    },
    ProgramTemplate {
        task_slug: "reverse_string",
        language_slug: "ruby",
        code: r#"text = "hello"
puts text.reverse"#,
    },
    // Issue #330: sum of the integers 1..=10 (= 55).
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "rust",
        code: r#"fn main() {
    let total: u32 = (1..=10).sum();
    println!("{total}");
}"#,
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "python",
        code: r"total = sum(range(1, 11))
print(total)",
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "javascript",
        code: r"let total = 0;
for (let number = 1; number <= 10; number += 1) {
  total += number;
}
console.log(total);",
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "typescript",
        code: r"let total = 0;
for (let number = 1; number <= 10; number += 1) {
  total += number;
}
console.log(total);",
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    total := 0
    for number := 1; number <= 10; number++ {
        total += number
    }
    fmt.Println(total)
}"#,
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "c",
        code: r#"#include <stdio.h>

int main(void) {
    int total = 0;
    for (int number = 1; number <= 10; number++) {
        total += number;
    }
    printf("%d\n", total);
    return 0;
}"#,
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "cpp",
        code: r"#include <iostream>

int main() {
    int total = 0;
    for (int number = 1; number <= 10; number++) {
        total += number;
    }
    std::cout << total << '\n';
}",
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "java",
        code: r"public class Main {
    public static void main(String[] args) {
        int total = 0;
        for (int number = 1; number <= 10; number++) {
            total += number;
        }
        System.out.println(total);
    }
}",
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "csharp",
        code: r"using System;

class Program {
    static void Main() {
        int total = 0;
        for (int number = 1; number <= 10; number++) {
            total += number;
        }
        Console.WriteLine(total);
    }
}",
    },
    ProgramTemplate {
        task_slug: "sum_to_ten",
        language_slug: "ruby",
        code: r"total = (1..10).sum
puts total",
    },
];
