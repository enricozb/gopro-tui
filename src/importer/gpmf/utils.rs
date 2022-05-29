pub fn align(count: usize) -> usize {
  let rem = count % 4;
  if rem == 0 {
    count
  } else {
    count + 4 - rem
  }
}
