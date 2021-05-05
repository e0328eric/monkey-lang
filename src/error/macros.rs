#[macro_export]
macro_rules! impl_partialeq {
    ($toimplement: ident : $($p: pat),+) => {
        impl PartialEq for $toimplement {
            fn eq(&self, other: &$toimplement) -> bool {
                use $toimplement::*;
                match (self, other) {
                    $(($p, $p) => true,)+
                    _ => false,
                }
            }
        }
    }
}

#[macro_export]
macro_rules! handle_error {
    ($handle: expr => $result: stmt) => {{
        if let Err(error) = $handle {
            if error.is_critical_err() {
                panic!("{}", error);
            } else {
                eprintln!("{}", error);
            }
        } else {
            $result
        }
    }};
}
