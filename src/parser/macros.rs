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
