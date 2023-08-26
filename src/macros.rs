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

macro_rules! tap {
    (
        $subj:expr;
        $(
            $(
                .$meth:ident($( $arg:expr ),*)
            )+
        ),+
        $(,)*  // permit trailing commas
    ) => ({
        let mut subj = $subj;
        $(
            subj$( .$meth($($arg),*) )+;
        )+
        subj
    });
}

#[cfg(test)]
mod tests {

    #[test]
    fn tap() {
        let actual = tap!(vec![1, 2]; .push(4), .push(8));

        assert_eq!(actual, vec![1, 2, 4, 8])
    }

    #[test]
    fn tap_trailing_comma() {
        let actual = tap!(vec![1, 2]; .push(4), .push(8),);

        assert_eq!(actual, vec![1, 2, 4, 8])
    }

    #[test]
    fn tap_method_calls() {
        let actual = tap! { vec!["hi".to_string()];
            .push("there".into()),
            .get_mut(1).map(|v| v.push('?')),
        };
        assert_eq!(actual, vec!["hi".to_string(), "there?".to_string()])
    }
}
