#[macro_export]
macro_rules! debug {
    ($($tokens:tt)*) => {
        #[cfg(debug_assertions)]
        {
            print!("[Debug] : ");
            $($tokens)*
            print!("\n");
        }
        #[cfg(release_assertions)]
        #[allow_unused_variables]
        {}
    };
}
