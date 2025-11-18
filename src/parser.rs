enum Cmd {
    Simple { program: String, args: Vec<String>},
    Pipleline(Vec<Cmd>),
    Sequence(Vec<Cmd>),
    Background(Box<Cmd>),
}