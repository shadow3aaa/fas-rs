#[macro_export]
macro_rules! debug {
    ($($tokens:tt)*) => {
        #[cfg(debug_assertions)]
        {
            println!("###Debug###");
            $($tokens)*
            println!("###Debug###");
        }
    };
}
