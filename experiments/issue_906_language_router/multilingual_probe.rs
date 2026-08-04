//! Probe: what does the router resolve for hi / zh / es request surfaces?
use formal_ai::implementation_language::requested_in;

fn main() {
    let prompts = [
        "Python में hello world प्रोग्राम लिखो",
        "हैलो वर्ल्ड प्रोग्राम Python में लिखो",
        "मौजूदा डायरेक्टरी में hello.txt नाम की फ़ाइल बनाओ",
        "用 Python 写一个 hello world 程序",
        "在当前目录下创建一个名为 hello.txt 的文件",
        "escribe un programa hola mundo en Python",
        "escribe un programa hello world en JavaScript",
        "crea un archivo llamado hello.txt en el directorio actual",
        "escribe un programa",
    ];
    for prompt in prompts {
        println!("{prompt:?} -> {:?}", requested_in(prompt));
    }
}
