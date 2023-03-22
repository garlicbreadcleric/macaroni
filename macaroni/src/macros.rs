/// Macro alternative for [`Option::or_else`].
macro_rules! or_else {
  ($e1: expr) => {
    $e1
  };
  ($e1: expr, $($e2: expr),*) => {
    match $e1 {
      Some(r) => Some(r),
      None => or_else!($($e2),*)
    }
  };
}

// macro_rules! debug_println {
//   ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
// }
