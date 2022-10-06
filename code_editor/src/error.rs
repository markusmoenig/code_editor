

#[derive(Eq, PartialEq, Debug, Clone)]
struct CodeError {
    pub message                 : String,
    pub line                    : Option<u32>
}