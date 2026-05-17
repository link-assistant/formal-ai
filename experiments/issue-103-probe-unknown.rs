use formal_ai::FormalAiEngine;
fn main() {
    let e = FormalAiEngine;
    let r = e.answer("Some unseen request");
    println!("intent: {}", r.intent);
    println!("answer: {}", r.answer);
}
