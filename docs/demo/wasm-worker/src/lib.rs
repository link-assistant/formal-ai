#![no_std]

const GREETING: u32 = 1;
const RUST_HELLO_WORLD: u32 = 2;
const UNKNOWN: u32 = 0;
const INPUT_CAPACITY: usize = 4096;

static mut INPUT: [u8; INPUT_CAPACITY] = [0; INPUT_CAPACITY];

#[no_mangle]
pub extern "C" fn input_ptr() -> *mut u8 {
    core::ptr::addr_of_mut!(INPUT).cast::<u8>()
}

#[no_mangle]
pub extern "C" fn classify(length: usize) -> u32 {
    let length = min(length, INPUT_CAPACITY);
    let input =
        unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(INPUT).cast::<u8>(), length) };

    if is_exact_greeting(input) {
        GREETING
    } else if contains_word(input, b"rust")
        && contains_word(input, b"hello")
        && contains_word(input, b"world")
    {
        RUST_HELLO_WORLD
    } else {
        UNKNOWN
    }
}

fn is_exact_greeting(input: &[u8]) -> bool {
    let trimmed = trim_ascii(input);
    ascii_eq(trimmed, b"hi") || ascii_eq(trimmed, b"hello") || ascii_eq(trimmed, b"hey")
}

fn contains_word(input: &[u8], word: &[u8]) -> bool {
    let mut index = 0;
    while index < input.len() {
        while index < input.len() && !is_ascii_alphanumeric(input[index]) {
            index += 1;
        }

        let start = index;
        while index < input.len() && is_ascii_alphanumeric(input[index]) {
            index += 1;
        }

        if start < index && ascii_eq(&input[start..index], word) {
            return true;
        }
    }

    false
}

fn trim_ascii(input: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = input.len();

    while start < end && !is_ascii_alphanumeric(input[start]) {
        start += 1;
    }
    while end > start && !is_ascii_alphanumeric(input[end - 1]) {
        end -= 1;
    }

    &input[start..end]
}

fn ascii_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut index = 0;
    while index < left.len() {
        if to_ascii_lower(left[index]) != right[index] {
            return false;
        }
        index += 1;
    }

    true
}

const fn is_ascii_alphanumeric(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

const fn to_ascii_lower(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte + 32
    } else {
        byte
    }
}

const fn min(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
