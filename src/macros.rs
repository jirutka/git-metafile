macro_rules! die {
    ( $( $arg:tt )* ) => ({
        err!($( $arg )*);
        ::std::process::exit(1);
    });
}

macro_rules! err {
    ( $fmt:expr ) => ({
        println_err!(concat!(env!("CARGO_PKG_NAME"), ": ", $fmt));
    });
    ( $fmt:expr, $( $arg:tt )* ) => ({
        println_err!(concat!(env!("CARGO_PKG_NAME"), ": ", $fmt), $( $arg )*);
    });
}

macro_rules! info {
    ( $( $arg:tt )* ) => ({
        if ! $crate::QUIET_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            println!($( $arg )*);
        }
    })
}

#[doc(hidden)]
macro_rules! println_err {
    ( $( $arg:tt )* ) => ({
        writeln!(&mut ::std::io::stderr(), $( $arg )*)
            .expect("Failed to write to stderr!");
    });
}
