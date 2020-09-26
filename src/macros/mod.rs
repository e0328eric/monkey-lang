// Macro Definitions
// Used in Parser
#[macro_export]
macro_rules! expect_peek {
    ($e: expr => $e1: expr) => {
        if $e.take_token().1.is_same_type(&$e1) {
            $e.next_token();
        } else {
            return Err(Error::ParseTokDiffErr {
                expected: $e1,
                got: $e.take_token().1.clone(),
            });
        }
    };
}

#[macro_export]
macro_rules! check_position {
    ($ident: ident := $self: expr, $num: expr) => {
        let $ident = if $self.cur_position < $self.l.len() - $num {
            &$self.l[$self.cur_position + $num]
        } else {
            &Token::EOF
        };
    };
}
// Parser End

// Used in Builtin
#[macro_export]
macro_rules! check_arg_len {
    ($args: expr, $expected: expr) => {
        if $args.len() != $expected {
            return Err(MonkeyErr::EvalParamNumErr {
                expected: $expected,
                got: $args.len(),
            });
        }
    };
}
// Builtin End

// Used in Error
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
// Error End
