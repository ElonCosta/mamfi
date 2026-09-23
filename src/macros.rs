#[macro_export]
macro_rules! trait_alias {

 ($name:ident: $($types:tt)+ {
     $($impl:tt)*
 }) => {
    trait $name: $($types)* {
    }
    impl<T: $($types)*> $name for T {}
 };
 ($name:ident: $($types:tt)+) => {
    trait $name: $($types)* {}
    impl<T: $($types)*> $name for T {}
 };
}
