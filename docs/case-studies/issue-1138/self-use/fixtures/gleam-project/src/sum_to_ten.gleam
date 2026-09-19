import gleam/int
import gleam/io
import gleam/list

pub fn sum_to_ten() -> Int {
  list.range(1, 10)
  |> list.fold(0, fn(total, each) { total + each })
}

pub fn main() {
  io.println(int.to_string(sum_to_ten()))
}
