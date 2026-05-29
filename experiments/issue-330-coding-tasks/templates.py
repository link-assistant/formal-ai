"""Authoritative source for the new coding-task templates (issue #330).

Each (task, language) entry is written to a file and compiled/run by verify.sh so
the deterministic output baked into src/coding/catalog.rs is proven, not assumed.
Keep this file byte-identical to the catalog templates.
"""

EXPECTED = {
    "fizzbuzz": "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz",
    "factorial": "120",
    "reverse_string": "olleh",
    "sum_to_ten": "55",
}

TEMPLATES = {
    "fizzbuzz": {
        "rust": '''fn main() {
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
}''',
        "python": '''for number in range(1, 16):
    if number % 15 == 0:
        print("FizzBuzz")
    elif number % 3 == 0:
        print("Fizz")
    elif number % 5 == 0:
        print("Buzz")
    else:
        print(number)''',
        "javascript": '''for (let number = 1; number <= 15; number += 1) {
  if (number % 15 === 0) {
    console.log("FizzBuzz");
  } else if (number % 3 === 0) {
    console.log("Fizz");
  } else if (number % 5 === 0) {
    console.log("Buzz");
  } else {
    console.log(number);
  }
}''',
        "typescript": '''for (let number = 1; number <= 15; number += 1) {
  if (number % 15 === 0) {
    console.log("FizzBuzz");
  } else if (number % 3 === 0) {
    console.log("Fizz");
  } else if (number % 5 === 0) {
    console.log("Buzz");
  } else {
    console.log(number);
  }
}''',
        "go": '''package main

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
}''',
        "c": '''#include <stdio.h>

int main(void) {
    for (int number = 1; number <= 15; number++) {
        if (number % 15 == 0) {
            puts("FizzBuzz");
        } else if (number % 3 == 0) {
            puts("Fizz");
        } else if (number % 5 == 0) {
            puts("Buzz");
        } else {
            printf("%d\\n", number);
        }
    }
    return 0;
}''',
        "cpp": '''#include <iostream>

int main() {
    for (int number = 1; number <= 15; number++) {
        if (number % 15 == 0) {
            std::cout << "FizzBuzz\\n";
        } else if (number % 3 == 0) {
            std::cout << "Fizz\\n";
        } else if (number % 5 == 0) {
            std::cout << "Buzz\\n";
        } else {
            std::cout << number << '\\n';
        }
    }
}''',
        "java": '''public class Main {
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
}''',
        "csharp": '''using System;

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
}''',
        "ruby": '''(1..15).each do |number|
  if (number % 15).zero?
    puts "FizzBuzz"
  elsif (number % 3).zero?
    puts "Fizz"
  elsif (number % 5).zero?
    puts "Buzz"
  else
    puts number
  end
end''',
    },
    "factorial": {
        "rust": '''fn main() {
    let mut result: u64 = 1;
    for number in 1..=5 {
        result *= number;
    }
    println!("{result}");
}''',
        "python": '''result = 1
for number in range(1, 6):
    result *= number
print(result)''',
        "javascript": '''let result = 1;
for (let number = 1; number <= 5; number += 1) {
  result *= number;
}
console.log(result);''',
        "typescript": '''let result = 1;
for (let number = 1; number <= 5; number += 1) {
  result *= number;
}
console.log(result);''',
        "go": '''package main

import "fmt"

func main() {
    result := 1
    for number := 1; number <= 5; number++ {
        result *= number
    }
    fmt.Println(result)
}''',
        "c": '''#include <stdio.h>

int main(void) {
    unsigned long long result = 1;
    for (int number = 1; number <= 5; number++) {
        result *= number;
    }
    printf("%llu\\n", result);
    return 0;
}''',
        "cpp": '''#include <iostream>

int main() {
    unsigned long long result = 1;
    for (int number = 1; number <= 5; number++) {
        result *= number;
    }
    std::cout << result << '\\n';
}''',
        "java": '''public class Main {
    public static void main(String[] args) {
        long result = 1;
        for (int number = 1; number <= 5; number++) {
            result *= number;
        }
        System.out.println(result);
    }
}''',
        "csharp": '''using System;

class Program {
    static void Main() {
        long result = 1;
        for (int number = 1; number <= 5; number++) {
            result *= number;
        }
        Console.WriteLine(result);
    }
}''',
        "ruby": '''result = (1..5).reduce(1, :*)
puts result''',
    },
    "reverse_string": {
        "rust": '''fn main() {
    let text = "hello";
    let reversed: String = text.chars().rev().collect();
    println!("{reversed}");
}''',
        "python": '''text = "hello"
print(text[::-1])''',
        "javascript": '''const text = "hello";
console.log(text.split("").reverse().join(""));''',
        "typescript": '''const text: string = "hello";
console.log(text.split("").reverse().join(""));''',
        "go": '''package main

import "fmt"

func main() {
    text := "hello"
    runes := []rune(text)
    for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
        runes[i], runes[j] = runes[j], runes[i]
    }
    fmt.Println(string(runes))
}''',
        "c": '''#include <stdio.h>
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
}''',
        "cpp": '''#include <algorithm>
#include <iostream>
#include <string>

int main() {
    std::string text = "hello";
    std::reverse(text.begin(), text.end());
    std::cout << text << '\\n';
}''',
        "java": '''public class Main {
    public static void main(String[] args) {
        String text = "hello";
        System.out.println(new StringBuilder(text).reverse().toString());
    }
}''',
        "csharp": '''using System;

class Program {
    static void Main() {
        var text = "hello".ToCharArray();
        Array.Reverse(text);
        Console.WriteLine(new string(text));
    }
}''',
        "ruby": '''text = "hello"
puts text.reverse''',
    },
    "sum_to_ten": {
        "rust": '''fn main() {
    let total: u32 = (1..=10).sum();
    println!("{total}");
}''',
        "python": '''total = sum(range(1, 11))
print(total)''',
        "javascript": '''let total = 0;
for (let number = 1; number <= 10; number += 1) {
  total += number;
}
console.log(total);''',
        "typescript": '''let total = 0;
for (let number = 1; number <= 10; number += 1) {
  total += number;
}
console.log(total);''',
        "go": '''package main

import "fmt"

func main() {
    total := 0
    for number := 1; number <= 10; number++ {
        total += number
    }
    fmt.Println(total)
}''',
        "c": '''#include <stdio.h>

int main(void) {
    int total = 0;
    for (int number = 1; number <= 10; number++) {
        total += number;
    }
    printf("%d\\n", total);
    return 0;
}''',
        "cpp": '''#include <iostream>

int main() {
    int total = 0;
    for (int number = 1; number <= 10; number++) {
        total += number;
    }
    std::cout << total << '\\n';
}''',
        "java": '''public class Main {
    public static void main(String[] args) {
        int total = 0;
        for (int number = 1; number <= 10; number++) {
            total += number;
        }
        System.out.println(total);
    }
}''',
        "csharp": '''using System;

class Program {
    static void Main() {
        int total = 0;
        for (int number = 1; number <= 10; number++) {
            total += number;
        }
        Console.WriteLine(total);
    }
}''',
        "ruby": '''total = (1..10).sum
puts total''',
    },
}

FILENAMES = {
    "rust": "main.rs",
    "python": "main.py",
    "javascript": "main.js",
    "typescript": "main.ts",
    "go": "main.go",
    "c": "main.c",
    "cpp": "main.cpp",
    "java": "Main.java",
    "csharp": "Program.cs",
    "ruby": "main.rb",
}

if __name__ == "__main__":
    import json
    import os
    import sys

    out = sys.argv[1]
    for task, langs in TEMPLATES.items():
        for lang, code in langs.items():
            d = os.path.join(out, task, lang)
            os.makedirs(d, exist_ok=True)
            with open(os.path.join(d, FILENAMES[lang]), "w") as f:
                f.write(code + "\n")
    with open(os.path.join(out, "expected.json"), "w") as f:
        json.dump(EXPECTED, f, indent=2)
    print("wrote templates for", len(TEMPLATES), "tasks")
