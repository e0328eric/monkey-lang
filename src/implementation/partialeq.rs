#[macro_export]
macro_rules! impl_partialeq {
    ($toimplement: ident =>> $($p: pat),+) => {
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
