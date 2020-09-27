use crate::error::{self, MonkeyErr};
use crate::object::{Object, NULL};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Builtin {
    ConvertErr, // This is only for error handle
    Len,
    First,
    Last,
    Rest,
    Push,
    Puts, // stdout
    Gets, // stdin
}

impl From<String> for Builtin {
    fn from(value: String) -> Self {
        match value.as_str() {
            "len" => Builtin::Len,
            "first" => Builtin::First,
            "last" => Builtin::Last,
            "rest" => Builtin::Rest,
            "push" => Builtin::Push,
            "puts" => Builtin::Puts,
            "gets" => Builtin::Gets,
            _ => Builtin::ConvertErr,
        }
    }
}

impl From<&String> for Builtin {
    fn from(value: &String) -> Self {
        value.to_string().into()
    }
}

// Take an equality between Bultin and String
impl PartialEq<String> for Builtin {
    fn eq(&self, other: &String) -> bool {
        self == &Builtin::from(other)
    }
}

// This implementation needed because of proving the symmetric part
impl PartialEq<Builtin> for String {
    fn eq(&self, other: &Builtin) -> bool {
        other == self
    }
}

impl Builtin {
    pub fn apply(&self, args: Vec<Object>) -> error::Result<Object> {
        match self {
            Builtin::Len => {
                check_arg_len!(args, 1);
                let arg = &args[0];
                match arg {
                    Object::String(s) => Ok(Object::Integer(s.chars().count() as i64)),
                    Object::Array(lst) => Ok(Object::Integer(lst.len() as i64)),
                    _ => Err(MonkeyErr::EvalArgErr {
                        fnt_name: "len".to_string(),
                        got: arg.clone(),
                    }),
                }
            }
            Builtin::First => {
                check_arg_len!(args, 1);
                let arg = &args[0];
                match arg {
                    Object::Array(lst) => {
                        if !lst.is_empty() {
                            Ok(lst.first().cloned().unwrap())
                        } else {
                            Ok(NULL)
                        }
                    }
                    _ => Err(MonkeyErr::EvalArgErr {
                        fnt_name: "first".to_string(),
                        got: arg.clone(),
                    }),
                }
            }
            Builtin::Last => {
                check_arg_len!(args, 1);
                let arg = &args[0];
                match arg {
                    Object::Array(lst) => {
                        if !lst.is_empty() {
                            Ok(lst.last().cloned().unwrap())
                        } else {
                            Ok(NULL)
                        }
                    }
                    _ => Err(MonkeyErr::EvalArgErr {
                        fnt_name: "last".to_string(),
                        got: arg.clone(),
                    }),
                }
            }
            Builtin::Rest => {
                check_arg_len!(args, 1);
                let arg = &args[0];
                match arg {
                    Object::Array(lst) => {
                        if let Some((_, rest)) = lst.split_first() {
                            Ok(Object::Array(rest.to_vec()))
                        } else {
                            Ok(NULL)
                        }
                    }
                    _ => Err(MonkeyErr::EvalArgErr {
                        fnt_name: "rest".to_string(),
                        got: arg.clone(),
                    }),
                }
            }
            Builtin::Push => {
                check_arg_len!(args, 2);
                let arg = &args[0];
                match arg {
                    Object::Array(lst) => {
                        let mut lst = lst.clone();
                        lst.push(args[1].clone());
                        Ok(Object::Array(lst))
                    }
                    _ => Err(MonkeyErr::EvalArgErr {
                        fnt_name: "push".to_string(),
                        got: arg.clone(),
                    }),
                }
            }
            Builtin::Puts => {
                check_arg_len!(args, 1);
                println!("{}", &args[0]);
                Ok(NULL)
            }
            Builtin::Gets => {
                check_arg_len!(args, 0);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.pop(); // Remove \n token
                Ok(Object::String(input))
            }
            _ => Err(MonkeyErr::EvalBuiltinErr),
        }
    }
}
