#[macro_export]
macro_rules! debug {
    ($($tokens:tt)*) => {
        #[cfg(debug_assertions)]
        {
            print!("[Debug]: ");
            $($tokens)*
            print!("\n");
        }
    };
}
