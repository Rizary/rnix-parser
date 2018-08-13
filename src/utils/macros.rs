#[macro_export]
macro_rules! nix_inner {
    (set (rec: $recursive:expr) {}) => {{ AST::EmptySet }};
    (set (rec: $recursive:expr) {
        $($ident:ident = ($($val:tt)*);)+
    }) => {{
        AST::Set {
            recursive: $recursive,
            values: vec![
                $(SetEntry(vec![String::from(stringify!($ident))], nix_inner!(parse $($val)*))),*
            ]
        }
    }};
    (pat_default) => {{ None }};
    (pat_default $($stuff:tt)+) => {{ Some(nix_inner!(parse $($stuff)+)) }};
    (pat_bind) => {{ None }};
    (pat_bind $name:ident) => {{ Some(String::from(stringify!($name))) }};
    (pat_exact) => {{ true }};
    (pat_exact $value:expr) => {{ $value }};
    (parse { $($token:tt)* }) => {{ nix_inner!(set (rec: false) { $($token)* }) }};
    (parse rec { $($token:tt)* }) => {{ nix_inner!(set (rec: true) { $($token)* }) }};
    (parse let {
        $($ident:ident = ($($val:tt)*);)*
    } in $($remaining:tt)*) => {{
        AST::LetIn(
            vec![$(SetEntry(vec![String::from(stringify!($ident))], nix_inner!(parse $($val)*))),*],
            Box::new(nix_inner!(parse $($remaining)*))
        )
    }};
    (parse let {
        $($ident:ident = ($($val:tt)*);)*
    }) => {{
        AST::Let(vec![$(SetEntry(vec![String::from(stringify!($ident))], nix_inner!(parse $($val)*))),*])
    }};
    (parse with ($($namespace:tt)*); $($remaining:tt)*) => {{
        AST::With(Box::new((
            nix_inner!(parse $($namespace)*),
            nix_inner!(parse $($remaining)*)
        )))
    }};
    (parse import ($($path:tt)*)) => {{
        AST::Import(Box::new(nix_inner!(parse $($path)*)))
    }};
    (parse $fn:ident: $($body:tt)*) => {{
        AST::Lambda(FnArg::Ident(String::from(stringify!($fn))), Box::new(nix_inner!(parse $($body)*)))
    }};
    (parse $($bind:ident @)* {
        $(( exact = $optional:expr ))*
        // TODO: Use optional macro args if they become a thing
        $($arg:ident $(? ($($default:tt)*))*),*
    }: $($body:tt)*) => {{
        AST::Lambda(
            FnArg::Pattern {
                args: vec![
                    $(PatEntry(String::from(stringify!($arg)), nix_inner!(pat_default $($($default)*)*))),*
                ],
                bind: nix_inner!(pat_bind $($bind)*),
                exact: nix_inner!(pat_exact $($optional)*)
            },
            Box::new(nix_inner!(parse $($body)*))
        )
    }};
    (parse ($($val1:tt)*) + ($($val2:tt)*)) => {{
        AST::Add(Box::new((nix_inner!(parse $($val1)*), nix_inner!(parse $($val2)*))))
    }};
    (parse ($($val1:tt)*) - ($($val2:tt)*)) => {{
        AST::Sub(Box::new((nix_inner!(parse $($val1)*), nix_inner!(parse $($val2)*))))
    }};
    (parse ($($val1:tt)*) * ($($val2:tt)*)) => {{
        AST::Mul(Box::new((nix_inner!(parse $($val1)*), nix_inner!(parse $($val2)*))))
    }};
    (parse ($($val1:tt)*) / ($($val2:tt)*)) => {{
        AST::Div(Box::new((nix_inner!(parse $($val1)*), nix_inner!(parse $($val2)*))))
    }};
    (parse ($($val1:tt)*) ++ ($($val2:tt)*)) => {{
        AST::Concat(Box::new((nix_inner!(parse $($val1)*), nix_inner!(parse $($val2)*))))
    }};
    (parse ($($fn:tt)*) ($($arg:tt)*)) => {{
        AST::Apply(Box::new((nix_inner!(parse $($fn)*), nix_inner!(parse $($arg)*))))
    }};
    (parse ($($set:tt)*).$field:ident) => {{
        AST::IndexSet(Box::new(nix_inner!(parse $($set)*)), String::from(stringify!($field)))
    }};
    (parse [$(($($item:tt)*))*]) => {{
        AST::List(vec![$(nix_inner!(parse $($item)*)),*])
    }};
    (parse true) => {{ AST::Value(Value::Bool(true)) }};
    (parse false) => {{ AST::Value(Value::Bool(false)) }};
    (parse null) => {{ AST::Value(Value::Bool(null)) }};
    (parse $val:ident) => {{
        AST::Var(String::from(stringify!($val)))
    }};
    (parse ./$val:expr) => {{
        AST::Value(Value::Path(Anchor::Relative, String::from(concat!("./", $val))))
    }};
    (parse raw $ast:expr) => {{ $ast }};
    (parse $val:expr) => {{ AST::Value(Value::from($val)) }};
}
#[macro_export]
macro_rules! nix {
    ($($tokens:tt)*) => {{
        #[allow(unused_imports)]
        use crate::{
            nometa::*,
            value::{Anchor, Value}
        };
        nix_inner!(parse $($tokens)*)
    }}
}

#[cfg(test)]
#[test]
fn test_macro() {
    use crate::nometa::*;
    assert_eq!(
        nix!({
            string = ("Hello World");
            number = ((3) * ((4) + (2)));
        }),
        AST::Set {
            recursive: false,
            values: vec![
                SetEntry(vec!["string".into()], AST::Value("Hello World".into())),
                SetEntry(vec!["number".into()], AST::Mul(Box::new((
                    AST::Value(3.into()),
                    AST::Add(Box::new((
                        AST::Value(4.into()),
                        AST::Value(2.into()),
                    )))
                )))),
            ]
        }
    );
}