//! Templates for the wider-range coding tasks added in issue #330 — `FizzBuzz`,
//! the factorial of 5, string reversal, and the sum from 1 to 10 — in every
//! supported language. Every output here was compiled and run locally
//! (`experiments/issue-330-coding-tasks`). Split from [`super::templates_core`]
//! only to keep each file well under the repository's per-file line limit.

use super::types::CompiledTemplate;

pub(super) const TEMPLATES_EXTENDED: &[CompiledTemplate] = &[
    // Issue #330: FizzBuzz for 1..=15. Every template prints "Fizz" for multiples
    // of 3, "Buzz" for multiples of 5, "FizzBuzz" for multiples of 15, and the
    // number otherwise. Outputs were compiled and run locally
    // (experiments/issue-330-coding-tasks).
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "rust",
        code: r#"fn main() {
    for number in {{start}}..={{end}} {
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "python",
        code: r#"for number in range({{start}}, {{end_exclusive}}):
    if number % 15 == 0:
        print("FizzBuzz")
    elif number % 3 == 0:
        print("Fizz")
    elif number % 5 == 0:
        print("Buzz")
    else:
        print(number)"#,
    },
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "javascript",
        code: r#"for (let number = {{start}}; number <= {{end}}; number += 1) {
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "typescript",
        code: r#"for (let number = {{start}}; number <= {{end}}; number += 1) {
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    for number := {{start}}; number <= {{end}}; number++ {
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "c",
        code: r#"#include <stdio.h>

int main(void) {
    for (int number = {{start}}; number <= {{end}}; number++) {
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "cpp",
        code: r#"#include <iostream>

int main() {
    for (int number = {{start}}; number <= {{end}}; number++) {
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "java",
        code: r#"public class Main {
    public static void main(String[] args) {
        for (int number = {{start}}; number <= {{end}}; number++) {
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "csharp",
        code: r#"using System;

class Program {
    static void Main() {
        for (int number = {{start}}; number <= {{end}}; number++) {
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "ruby",
        code: r#"({{start}}..{{end}}).each do |number|
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
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "scala",
        code: r#"object Main {
  def main(args: Array[String]): Unit = {
    for (number <- {{start}} to {{end}}) {
      if (number % 15 == 0) {
        println("FizzBuzz")
      } else if (number % 3 == 0) {
        println("Fizz")
      } else if (number % 5 == 0) {
        println("Buzz")
      } else {
        println(number)
      }
    }
  }
}"#,
    },
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "kotlin",
        code: r#"fun main() {
    for (number in {{start}}..{{end}}) {
        when {
            number % 15 == 0 -> println("FizzBuzz")
            number % 3 == 0 -> println("Fizz")
            number % 5 == 0 -> println("Buzz")
            else -> println(number)
        }
    }
}"#,
    },
    // Issue #330: factorial of 5 (5! = 120).
    CompiledTemplate {
        task_slug: "fizzbuzz",
        language_slug: "php",
        code: r#"<?php

foreach (range({{start}}, {{end}}) as $number) {
    if ($number % 15 === 0) {
        echo "FizzBuzz", PHP_EOL;
    } elseif ($number % 3 === 0) {
        echo "Fizz", PHP_EOL;
    } elseif ($number % 5 === 0) {
        echo "Buzz", PHP_EOL;
    } else {
        echo $number, PHP_EOL;
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "rust",
        code: r#"fn main() {
    let mut result: u64 = 1;
    for number in 1..={{n}} {
        result *= number;
    }
    println!("{result}");
}"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "python",
        code: r#"result = 1
for number in range(1, {{end_exclusive}}):
    result *= number
print(result)"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "javascript",
        code: r#"let result = 1;
for (let number = 1; number <= {{n}}; number += 1) {
  result *= number;
}
console.log(result);"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "typescript",
        code: r#"let result = 1;
for (let number = 1; number <= {{n}}; number += 1) {
  result *= number;
}
console.log(result);"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    result := int64(1)
    for number := 1; number <= {{n}}; number++ {
        result *= int64(number)
    }
    fmt.Println(result)
}"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "c",
        code: r#"#include <stdio.h>

int main(void) {
    unsigned long long result = 1;
    for (int number = 1; number <= {{n}}; number++) {
        result *= number;
    }
    printf("%llu\n", result);
    return 0;
}"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "cpp",
        code: r#"#include <iostream>

int main() {
    unsigned long long result = 1;
    for (int number = 1; number <= {{n}}; number++) {
        result *= number;
    }
    std::cout << result << '\n';
}"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "java",
        code: r#"public class Main {
    public static void main(String[] args) {
        long result = 1;
        for (int number = 1; number <= {{n}}; number++) {
            result *= number;
        }
        System.out.println(result);
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "csharp",
        code: r#"using System;

class Program {
    static void Main() {
        long result = 1;
        for (int number = 1; number <= {{n}}; number++) {
            result *= number;
        }
        Console.WriteLine(result);
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "ruby",
        code: r#"result = (1..{{n}}).reduce(1, :*)
puts result"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "scala",
        code: r#"object Main {
  def main(args: Array[String]): Unit = {
    var result: Long = 1
    for (number <- 1 to {{n}}) {
      result *= number
    }
    println(result)
  }
}"#,
    },
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "kotlin",
        code: r#"fun main() {
    var result = 1L
    for (number in 1..{{n}}) {
        result *= number
    }
    println(result)
}"#,
    },
    // Issue #330: reverse the literal string "hello" -> "olleh".
    CompiledTemplate {
        task_slug: "factorial",
        language_slug: "php",
        code: r#"<?php

$result = ({{n}} === 0) ? 1 : array_product(range(1, {{n}}));
echo $result, PHP_EOL;"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "rust",
        code: r#"fn main() {
    let text = {{text}};
    let reversed: String = text.chars().rev().collect();
    println!("{reversed}");
}"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "python",
        code: r#"text = {{text}}
print(text[::-1])"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "javascript",
        code: r#"const text = {{text}};
console.log(Array.from(text).reverse().join(""));"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "typescript",
        code: r#"const text: string = {{text}};
console.log(Array.from(text).reverse().join(""));"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    text := {{text}}
    runes := []rune(text)
    for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
        runes[i], runes[j] = runes[j], runes[i]
    }
    fmt.Println(string(runes))
}"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "c",
        code: r#"#include <stdio.h>
#include <string.h>

int main(void) {
    char text[] = {{text}};
    for (size_t end = strlen(text); end > 0;) {
        size_t start = end - 1;
        while (start > 0 && ((unsigned char)text[start] & 0xC0) == 0x80) {
            start--;
        }
        fwrite(text + start, 1, end - start, stdout);
        end = start;
    }
    putchar('\n');
    return 0;
}"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "cpp",
        code: r#"#include <iostream>
#include <string>

int main() {
    std::string text = {{text}};
    for (std::size_t end = text.size(); end > 0;) {
        std::size_t start = end - 1;
        while (start > 0 && (static_cast<unsigned char>(text[start]) & 0xC0) == 0x80) {
            --start;
        }
        std::cout.write(text.data() + start, end - start);
        end = start;
    }
    std::cout << '\n';
}"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "java",
        code: r#"public class Main {
    public static void main(String[] args) {
        String text = {{text}};
        System.out.println(new StringBuilder(text).reverse().toString());
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "csharp",
        code: r#"using System;
using System.Collections.Generic;
using System.Text;

class Program {
    static void Main() {
        string text = {{text}};
        var scalars = new List<Rune>();
        foreach (Rune scalar in text.EnumerateRunes()) scalars.Add(scalar);
        scalars.Reverse();
        var reversed = new StringBuilder();
        foreach (Rune scalar in scalars) reversed.Append(scalar.ToString());
        Console.WriteLine(reversed.ToString());
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "ruby",
        code: r#"text = {{text}}
puts text.reverse"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "scala",
        code: r#"object Main {
  def main(args: Array[String]): Unit = {
    val text = {{text}}
    val reversed = new java.lang.StringBuilder()
    text.codePoints().toArray().reverse.foreach(point => reversed.appendCodePoint(point))
    println(reversed.toString)
  }
}"#,
    },
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "kotlin",
        code: r#"fun main() {
    val text = {{text}}
    val reversed = StringBuilder()
    for (point in text.codePoints().toArray().reversed()) reversed.appendCodePoint(point)
    println(reversed)
}"#,
    },
    // Issue #330: sum of the integers 1..=10 (= 55).
    CompiledTemplate {
        task_slug: "reverse_string",
        language_slug: "php",
        code: r#"<?php

$text = {{text}};
echo implode("", array_reverse(preg_split('//u', $text, -1, PREG_SPLIT_NO_EMPTY))), PHP_EOL;"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "rust",
        code: r#"fn main() {
    let total: i32 = ({{start}}..={{end}}).sum();
    println!("{total}");
}"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "python",
        code: r#"total = sum(range({{start}}, {{end_exclusive}}))
print(total)"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "javascript",
        code: r#"let total = 0;
for (let number = {{start}}; number <= {{end}}; number += 1) {
  total += number;
}
console.log(total);"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "typescript",
        code: r#"let total = 0;
for (let number = {{start}}; number <= {{end}}; number += 1) {
  total += number;
}
console.log(total);"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "go",
        code: r#"package main

import "fmt"

func main() {
    total := 0
    for number := {{start}}; number <= {{end}}; number++ {
        total += number
    }
    fmt.Println(total)
}"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "c",
        code: r#"#include <stdio.h>

int main(void) {
    int total = 0;
    for (int number = {{start}}; number <= {{end}}; number++) {
        total += number;
    }
    printf("%d\n", total);
    return 0;
}"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "cpp",
        code: r#"#include <iostream>

int main() {
    int total = 0;
    for (int number = {{start}}; number <= {{end}}; number++) {
        total += number;
    }
    std::cout << total << '\n';
}"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "java",
        code: r#"public class Main {
    public static void main(String[] args) {
        int total = 0;
        for (int number = {{start}}; number <= {{end}}; number++) {
            total += number;
        }
        System.out.println(total);
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "csharp",
        code: r#"using System;

class Program {
    static void Main() {
        int total = 0;
        for (int number = {{start}}; number <= {{end}}; number++) {
            total += number;
        }
        Console.WriteLine(total);
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "ruby",
        code: r#"total = ({{start}}..{{end}}).sum
puts total"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "scala",
        code: r#"object Main {
  def main(args: Array[String]): Unit = {
    val total = ({{start}} to {{end}}).sum
    println(total)
  }
}"#,
    },
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "kotlin",
        code: r#"fun main() {
    val total = ({{start}}..{{end}}).sum()
    println(total)
}"#,
    },
    // Issue #334: a recursive `fibonacci` function evaluated at the 10th term.
    // With F(1)=F(2)=1 the recurrence gives F(10)=55. Every template defines the
    // recursive function and prints `fibonacci(10)`. Outputs were compiled and
    // run locally (experiments/issue-334).
    CompiledTemplate {
        task_slug: "sum_to_ten",
        language_slug: "php",
        code: r#"<?php

$total = array_sum(range({{start}}, {{end}}));
echo $total, PHP_EOL;"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "rust",
        code: r#"fn fibonacci(n: u64) -> u64 {
    fn advance(remaining: u64, previous: u64, current: u64) -> u64 {
        if remaining == 0 {
            previous
        } else {
            advance(remaining - 1, current, previous + current)
        }
    }
    advance(n, 0, 1)
}

fn main() {
    println!("{}", fibonacci({{index}}));
}"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "python",
        code: r#"def fibonacci(n):
    def advance(remaining, previous, current):
        if remaining <= 0:
            return previous
        return advance(remaining - 1, current, previous + current)

    return advance(n, 0, 1)


print(fibonacci({{index}}))"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "javascript",
        code: r#"function fibonacci(n) {
  function advance(remaining, previous, current) {
    if (remaining <= 0) return previous;
    return advance(remaining - 1, current, previous + current);
  }
  return advance(n, 0, 1);
}

console.log(fibonacci({{index}}));"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "typescript",
        code: r#"function fibonacci(n: number): number {
  function advance(remaining: number, previous: number, current: number): number {
    if (remaining <= 0) return previous;
    return advance(remaining - 1, current, previous + current);
  }
  return advance(n, 0, 1);
}

console.log(fibonacci({{index}}));"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "go",
        code: r#"package main

{{go_fmt}}

func advance(remaining int, previous, current int64) int64 {
    if remaining <= 0 {
        return previous
    }
    return advance(remaining-1, current, previous+current)
}

func fibonacci(n int) int64 {
    return advance(n, 0, 1)
}

func main() {
    fmt.Println(fibonacci({{index}}))
}"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "c",
        code: r#"#include <stdio.h>

static unsigned long long advance(int remaining, unsigned long long previous,
                                  unsigned long long current) {
    if (remaining <= 0) return previous;
    return advance(remaining - 1, current, previous + current);
}

unsigned long long fibonacci(int n) {
    return advance(n, 0, 1);
}

int main(void) {
    printf("%llu\n", fibonacci({{index}}));
    return 0;
}"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "cpp",
        code: r#"#include <iostream>

static unsigned long long advance(int remaining, unsigned long long previous,
                                  unsigned long long current) {
    if (remaining <= 0) return previous;
    return advance(remaining - 1, current, previous + current);
}

unsigned long long fibonacci(int n) {
    return advance(n, 0, 1);
}

int main() {
    std::cout << fibonacci({{index}}) << '\n';
}"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "java",
        code: r#"public class Main {
    static long fibonacci(int n) {
        return advance(n, 0, 1);
    }

    private static long advance(int remaining, long previous, long current) {
        if (remaining <= 0) return previous;
        return advance(remaining - 1, current, previous + current);
    }

    public static void main(String[] args) {
        System.out.println(fibonacci({{index}}));
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "csharp",
        code: r#"using System;

class Program {
    static long Fibonacci(int n) {
        return Advance(n, 0, 1);
    }

    static long Advance(int remaining, long previous, long current) {
        if (remaining <= 0) return previous;
        return Advance(remaining - 1, current, previous + current);
    }

    static void Main() {
        Console.WriteLine(Fibonacci({{index}}));
    }
}"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "ruby",
        code: r#"def fibonacci(n)
  advance(n, 0, 1)
end

def advance(remaining, previous, current)
  return previous if remaining <= 0

  advance(remaining - 1, current, previous + current)
end

puts fibonacci({{index}})"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "scala",
        code: r#"object Main {
  def fibonacci(n: Int): Long = {
    @scala.annotation.tailrec
    def advance(remaining: Int, previous: Long, current: Long): Long =
      if (remaining <= 0) previous else advance(remaining - 1, current, previous + current)
    advance(n, 0, 1)
  }

  def main(args: Array[String]): Unit = {
    println(fibonacci({{index}}))
  }
}"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "kotlin",
        code: r#"fun fibonacci(n: Int): Long {
    tailrec fun advance(remaining: Int, previous: Long, current: Long): Long =
        if (remaining <= 0) previous else advance(remaining - 1, current, previous + current)
    return advance(n, 0, 1)
}

fun main() {
    println(fibonacci({{index}}))
}"#,
    },
    CompiledTemplate {
        task_slug: "fibonacci",
        language_slug: "php",
        code: r#"<?php

function fibonacci(int $n): int
{
    return advance($n, 0, 1);
}

function advance(int $remaining, int $previous, int $current): int
{
    if ($remaining <= 0) {
        return $previous;
    }
    return advance($remaining - 1, $current, $previous + $current);
}

echo fibonacci({{index}}), PHP_EOL;"#,
    },
];
