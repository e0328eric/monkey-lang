#[macro_export]
macro_rules! impl_partialeq {
    ($toimplement: ty | $($p: pat),+) => {
        impl PartialEq for $toimplement {
            fn eq(&self, other: &$toimplement) -> bool {
                match (self, other) {
                    $(($p, $p) => true,)+
                    _ => false,
                }
            }
        }
    }
}
